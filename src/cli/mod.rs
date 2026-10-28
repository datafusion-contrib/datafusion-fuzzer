// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: See the NOTICE file distributed with this work for additional information regarding copyright ownership
// SPDX-FileContributor: Licensed to the Apache Software Foundation (ASF) under one or more contributor license agreements

pub mod error_whitelist;
mod runner;
mod tui;

use clap::Parser;
pub use runner::run_fuzzer;
use std::path::PathBuf;
pub use tui::{TuiApp, init, restore};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to config file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Random seed
    #[arg(short, long, default_value_t = 310104)]
    pub seed: u64,

    /// Number of rounds to run
    #[arg(short, long)]
    pub rounds: Option<u32>,

    /// Number of queries per round
    #[arg(short, long)]
    pub queries_per_round: Option<u32>,

    /// Query timeout in seconds
    #[arg(short, long)]
    pub timeout: Option<u64>,

    /// Path to log file
    #[arg(short, long)]
    pub log_path: Option<PathBuf>,

    /// Display logs
    #[arg(short, long)]
    pub display_logs: bool,

    /// Enable TUI display
    #[arg(long)]
    pub enable_tui: bool,
}
