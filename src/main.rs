// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: See the NOTICE file distributed with this work for additional information regarding copyright ownership
// SPDX-FileContributor: Licensed to the Apache Software Foundation (ASF) under one or more contributor license agreements

use clap::Parser;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{EnvFilter, filter::LevelFilter, fmt, prelude::*};

use datafusion_fuzzer::{
    cli::{Cli, TuiApp, init, restore, run_fuzzer},
    common::{Result, init_available_data_types},
    fuzz_context::{GlobalContext, RunnerConfig, RuntimeContext},
    fuzz_runner::{FuzzerStats, create_fuzzer_stats_with_timeout, get_tui_stats},
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize available data types early
    init_available_data_types();

    let cli = Cli::parse();
    let runner_config = RunnerConfig::from_cli(&cli)?;
    let _log_guards = setup_logging(&runner_config)?;

    // Create global context with all state
    let fuzzer_stats =
        create_fuzzer_stats_with_timeout(runner_config.rounds, runner_config.timeout_seconds);
    let global_context = Arc::new(GlobalContext::new(
        runner_config.clone(),
        RuntimeContext::default(),
        fuzzer_stats,
    ));

    // Spawn TUI in a separate thread (if enabled)
    if runner_config.enable_tui {
        let tui_context = Arc::clone(&global_context);
        tokio::spawn(async move {
            let mut terminal = init();
            let _ = TuiApp::new(Arc::clone(&tui_context.fuzzer_stats)).run(&mut terminal);
            restore();
        });
    }

    // Run the fuzzer
    run_fuzzer(global_context.clone()).await?;

    print_final_stats(&global_context.fuzzer_stats);

    Ok(())
}

/// RAII logging workers
struct LogGuards {
    _trace_guard: Option<tracing_appender::non_blocking::WorkerGuard>,
    _error_guard: Option<tracing_appender::non_blocking::WorkerGuard>,
}

/// Logging guide:
/// - For general logs use `info!` macro
/// - For system under test (DataFusion)'s found bugs, use `error!` macro
///
/// When configured with a log directory (e.g., `logs/`), the system creates two log files:
/// - `logs/trace.log`: Contains all logs generated using the `info!` macro
/// - `logs/error.log`: Contains logs specifically related to system under test
/// bugs using the `error!` macro
fn setup_logging(config: &RunnerConfig) -> Result<LogGuards> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let format = fmt::format()
        .with_level(true)
        .with_target(false)
        .with_thread_ids(false)
        .with_line_number(false)
        .compact();

    let mut log_guards = LogGuards {
        _trace_guard: None,
        _error_guard: None,
    };

    if let Some(log_dir) = &config.log_path {
        // Create log directory if it doesn't exist
        if !log_dir.exists() {
            std::fs::create_dir_all(log_dir)?;
        }

        // Create the log files
        let trace_path = log_dir.join("trace.log");
        let error_path = log_dir.join("error.log");

        // Create appenders for the log files
        let trace_file = tracing_appender::rolling::never(
            trace_path.parent().unwrap(),
            trace_path.file_name().unwrap(),
        );
        let error_file = tracing_appender::rolling::never(
            error_path.parent().unwrap(),
            error_path.file_name().unwrap(),
        );

        // Create non-blocking writers
        let (trace_writer, trace_guard) = tracing_appender::non_blocking(trace_file);
        let (error_writer, error_guard) = tracing_appender::non_blocking(error_file);

        // Store the guards
        log_guards._trace_guard = Some(trace_guard);
        log_guards._error_guard = Some(error_guard);

        // Set up the subscriber
        let subscriber = tracing_subscriber::registry().with(env_filter);

        // Add stdout layer if display_logs is enabled
        if config.display_logs {
            let stdout_layer = fmt::Layer::default()
                .with_writer(std::io::stdout)
                .event_format(format.clone());

            // Add error-only layer
            let error_layer = fmt::Layer::default()
                .with_writer(error_writer)
                .event_format(format.clone())
                .with_filter(LevelFilter::ERROR);

            // Add all-logs layer
            let trace_layer = fmt::Layer::default()
                .with_writer(trace_writer)
                .event_format(format.clone());

            subscriber
                .with(stdout_layer)
                .with(error_layer)
                .with(trace_layer)
                .init();
        } else {
            // Add error-only layer
            let error_layer = fmt::Layer::default()
                .with_writer(error_writer)
                .event_format(format.clone())
                .with_filter(LevelFilter::ERROR);

            // Add all-logs layer
            let trace_layer = fmt::Layer::default()
                .with_writer(trace_writer)
                .event_format(format.clone());

            subscriber.with(error_layer).with(trace_layer).init();
        }

        info!("Logging initialized to directory: {:?}", log_dir);
        info!("All logs go to trace.log, errors go to error.log");
    } else {
        // Just log to stdout
        if config.display_logs {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(
                    fmt::Layer::default()
                        .with_writer(std::io::stdout)
                        .event_format(format),
                )
                .init();

            info!("Logging initialized to stdout only");
        }
    }

    Ok(log_guards)
}

/// Print final statistics when the fuzzer completes
fn print_final_stats(fuzzer_stats: &Arc<std::sync::Mutex<FuzzerStats>>) {
    let stats = get_tui_stats(fuzzer_stats);

    println!("\n{}", "=".repeat(60));
    println!("🎯 DataFusion Fuzzer - Final Statistics");
    println!("{}", "=".repeat(60));

    println!("📊 Execution Summary:");
    println!("  • Rounds Completed: {}", stats.rounds_completed);
    println!("  • Queries Executed: {}", stats.queries_executed);
    println!("  • Query Success Rate: {:.2}%", stats.success_rate);
    println!("  • Queries Per Second: {:.2}", stats.queries_per_second);
    println!(
        "  • Slow Queries (>=90% of timeout): {} ({:.2}%)",
        stats.queries_slow,
        if stats.queries_executed > 0 {
            (stats.queries_slow as f64 / stats.queries_executed as f64) * 100.0
        } else {
            0.0
        }
    );

    let total_secs = stats.running_time_secs;
    let hours = (total_secs / 3600.0) as u64;
    let minutes = ((total_secs % 3600.0) / 60.0) as u64;
    let seconds = total_secs % 60.0;

    if hours > 0 {
        println!("  • Total Runtime: {}h {}m {:.2}s", hours, minutes, seconds);
    } else if minutes > 0 {
        println!("  • Total Runtime: {}m {:.2}s", minutes, seconds);
    } else {
        println!("  • Total Runtime: {:.2}s", seconds);
    }

    // Display query runtime statistics if available
    if let Some(ref runtime_stats) = stats.query_runtime_stats {
        println!("\n⏱️  Query Runtime Statistics:");
        println!("  • Average: {:.2}ms", runtime_stats.avg_ms);
        println!("  • Fastest: {:.2}ms", runtime_stats.fastest_ms);
        println!("  • Slowest: {:.2}ms", runtime_stats.slowest_ms);
        println!("  • 90th percentile: {:.2}ms", runtime_stats.p90_ms);
        println!("  • 99th percentile: {:.2}ms", runtime_stats.p99_ms);

        // Display the slowest query
        println!("\n🐌 Slowest Query ({:.2}ms):", runtime_stats.slowest_ms);
        println!("{}", "-".repeat(40));
        for line in runtime_stats.slowest_query.lines() {
            println!("  {}", line);
        }
        println!("{}", "-".repeat(40));
    }

    if !stats.recent_query.is_empty() {
        println!("\n🔍 Most Recent Query:");
        println!("{}", "-".repeat(40));
        for line in stats.recent_query.lines() {
            println!("  {}", line);
        }
        println!("{}", "-".repeat(40));
    }

    println!("{}", "=".repeat(60));
    println!("✅ Fuzzing completed successfully!");
    println!(
        "⚠️  The program might be stuck if there are timed out queries. Please press Ctrl+C to exit."
    );
}
