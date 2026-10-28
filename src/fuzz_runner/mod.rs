// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: See the NOTICE file distributed with this work for additional information regarding copyright ownership
// SPDX-FileContributor: Licensed to the Apache Software Foundation (ASF) under one or more contributor license agreements

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// A query execution record containing both the query text and its execution time
#[derive(Debug, Clone)]
struct QueryExecutionRecord {
    query: String,
    execution_time: Duration,
}

/// Query runtime statistics for the 5 key metrics
#[derive(Debug, Clone)]
pub struct QueryRuntimeStats {
    pub avg_ms: f64,
    pub fastest_ms: f64,
    pub slowest_ms: f64,
    pub p90_ms: f64,
    pub p99_ms: f64,
    pub slowest_query: String,
}

impl QueryRuntimeStats {
    /// Create runtime statistics from a collection of execution records
    pub fn from_records(records: &[QueryExecutionRecord]) -> Option<Self> {
        if records.is_empty() {
            return None;
        }

        // Convert to milliseconds for easier reading and create (time, query) pairs
        let mut time_query_pairs: Vec<(f64, &String)> = records
            .iter()
            .map(|record| (record.execution_time.as_secs_f64() * 1000.0, &record.query))
            .collect();

        // Sort by execution time for percentile calculations
        time_query_pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let times_ms: Vec<f64> = time_query_pairs.iter().map(|(time, _)| *time).collect();
        let count = times_ms.len();
        let avg_ms = times_ms.iter().sum::<f64>() / count as f64;
        let fastest_ms = times_ms[0];
        let slowest_ms = times_ms[count - 1];
        let slowest_query = time_query_pairs[count - 1].1.clone();

        // Calculate percentiles using the nearest-rank method
        let p90_ms = percentile(&times_ms, 90.0);
        let p99_ms = percentile(&times_ms, 99.0);

        Some(Self {
            avg_ms,
            fastest_ms,
            slowest_ms,
            p90_ms,
            p99_ms,
            slowest_query,
        })
    }

    /// Format the runtime statistics for display
    pub fn format_display(&self) -> String {
        format!(
            "⏱️  Query Runtime Statistics:\n\
             • Average: {:.2}ms\n\
             • Fastest: {:.2}ms\n\
             • Slowest: {:.2}ms\n\
             • 90th percentile: {:.2}ms\n\
             • 99th percentile: {:.2}ms",
            self.avg_ms, self.fastest_ms, self.slowest_ms, self.p90_ms, self.p99_ms
        )
    }
}

/// Calculate percentile using the nearest-rank method
fn percentile(sorted_values: &[f64], percentile: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }

    if percentile <= 0.0 {
        return sorted_values[0];
    }

    if percentile >= 100.0 {
        return sorted_values[sorted_values.len() - 1];
    }

    // Use nearest-rank method: n = ceil(P/100 * N)
    let index = ((percentile / 100.0) * sorted_values.len() as f64).ceil() as usize - 1;
    let clamped_index = index.min(sorted_values.len() - 1);

    sorted_values[clamped_index]
}

/// Live statistics for the fuzzing process
#[derive(Debug, Clone)]
pub struct FuzzerStats {
    // Basic counters
    pub rounds_completed: u32,
    pub total_rounds: u32,
    pub queries_executed: u64,
    pub queries_succeeded: u64,
    pub queries_slow: u64,

    // Timers
    pub start_time: Instant,

    // Sample a recent query every t seconds
    pub last_sample_time: Instant,
    pub recent_query: String,

    // Query execution records for runtime statistics
    query_execution_records: Vec<QueryExecutionRecord>,

    // Slow query tracking
    pub slow_query_threshold_ms: f64,
}

// Struct to hold formatted stats for display in a TUI
#[derive(Debug, Clone)]
pub struct TuiStats {
    pub rounds_completed: u32,
    pub total_rounds: u32,
    pub queries_executed: u64,
    pub queries_succeeded: u64,
    pub queries_slow: u64,
    pub success_rate: f64,
    pub queries_per_second: f64,
    pub running_time_secs: f64,
    pub recent_query: String,
    pub query_runtime_stats: Option<QueryRuntimeStats>,
}

impl FuzzerStats {
    /// Create new FuzzerStats with the specified total rounds
    pub fn new(total_rounds: u32) -> Self {
        Self::new_with_timeout(total_rounds, 1000.0)
    }

    /// Create new FuzzerStats with the specified total rounds and slow query threshold
    pub fn new_with_timeout(total_rounds: u32, slow_query_threshold_ms: f64) -> Self {
        Self {
            rounds_completed: 0,
            total_rounds,
            queries_executed: 0,
            queries_succeeded: 0,
            queries_slow: 0,
            start_time: Instant::now(),
            last_sample_time: Instant::now(),
            recent_query: String::new(),
            query_execution_records: Vec::new(),
            slow_query_threshold_ms,
        }
    }

    /// Record a query execution for display and statistics purposes
    ///
    /// This method serves two main purposes:
    /// 1. **CLI Live Display**: Shows the most recent query being tested in the TUI/CLI interface
    /// 2. **Statistics Tracking**: Updates execution counters and success rates for reporting
    ///
    /// **When to call this method:**
    /// - Call once per generated query group, typically right after successful query generation
    /// - The `success` parameter should reflect the final validation outcome
    /// - For CLI display purposes, the query text is more important than the success status
    ///
    /// **Sampling behavior:**
    /// - Queries are sampled for display based on time intervals to avoid overwhelming the UI
    /// - All queries contribute to statistics, but only sampled queries are shown in the interface
    ///
    /// # Arguments
    /// * `query` - The SQL query string to display and track
    /// * `success` - Whether the query validation/execution succeeded
    /// * `sample_interval_secs` - The interval in seconds for sampling queries for display
    pub fn record_query(&mut self, query: &str, success: bool, sample_interval_secs: u64) {
        self.queries_executed += 1;
        self.queries_succeeded += if success { 1 } else { 0 };

        let now = Instant::now();

        // Sample the query if enough time has passed since the last sample OR if there's no existing query
        if self.recent_query.is_empty()
            || now.duration_since(self.last_sample_time).as_secs() >= sample_interval_secs
        {
            self.last_sample_time = now;
            // Store the query as is to preserve multiline formatting
            self.recent_query = query.to_string();
        }
    }

    /// Record a query execution with runtime tracking
    ///
    /// This extends the basic query recording to include execution time for performance statistics.
    ///
    /// # Arguments
    /// * `query` - The SQL query string to display and track
    /// * `success` - Whether the query validation/execution succeeded
    /// * `execution_time` - How long the query took to execute
    /// * `sample_interval_secs` - The interval in seconds for sampling queries for display
    pub fn record_query_with_time(
        &mut self,
        query: &str,
        success: bool,
        execution_time: Duration,
        sample_interval_secs: u64,
    ) {
        // Record the basic query stats
        self.record_query(query, success, sample_interval_secs);

        // Store the execution record for runtime statistics
        self.query_execution_records.push(QueryExecutionRecord {
            query: query.to_string(),
            execution_time,
        });

        // Check if this is a slow query (queries that took close to or exceed the timeout)
        let execution_time_ms = execution_time.as_secs_f64() * 1000.0;
        // Consider a query slow if it took 90% or more of the timeout threshold
        let slow_threshold = self.slow_query_threshold_ms * 0.9;
        if execution_time_ms >= slow_threshold {
            self.queries_slow += 1;
        }
    }

    /// Complete a round of fuzzing
    pub fn complete_round(&mut self) {
        self.rounds_completed += 1;
    }

    /// Get statistics formatted for display in a TUI
    pub fn get_tui_stats(&self) -> TuiStats {
        let elapsed = self.start_time.elapsed();

        let elapsed_secs = elapsed.as_secs_f64();
        let qps = if elapsed_secs > 0.0 {
            self.queries_executed as f64 / elapsed_secs
        } else {
            0.0
        };

        let success_rate = if self.queries_executed > 0 {
            (self.queries_succeeded as f64 / self.queries_executed as f64) * 100.0
        } else {
            0.0
        };

        TuiStats {
            rounds_completed: self.rounds_completed,
            total_rounds: self.total_rounds,
            queries_executed: self.queries_executed,
            queries_succeeded: self.queries_succeeded,
            queries_slow: self.queries_slow,
            success_rate,
            queries_per_second: qps,
            running_time_secs: elapsed_secs,
            recent_query: self.recent_query.clone(),
            query_runtime_stats: QueryRuntimeStats::from_records(&self.query_execution_records),
        }
    }
}

/// Helper function to create a new shared FuzzerStats instance
pub fn create_fuzzer_stats(total_rounds: u32) -> Arc<Mutex<FuzzerStats>> {
    Arc::new(Mutex::new(FuzzerStats::new(total_rounds)))
}

/// Helper function to create a new shared FuzzerStats instance with timeout configuration
pub fn create_fuzzer_stats_with_timeout(
    total_rounds: u32,
    timeout_seconds: u64,
) -> Arc<Mutex<FuzzerStats>> {
    let slow_query_threshold_ms = (timeout_seconds as f64) * 1000.0;
    Arc::new(Mutex::new(FuzzerStats::new_with_timeout(
        total_rounds,
        slow_query_threshold_ms,
    )))
}

/// Helper function to record a query execution
pub fn record_query(
    stats: &Arc<Mutex<FuzzerStats>>,
    query: &str,
    success: bool,
    sample_interval_secs: u64,
) {
    let mut stats_guard = stats.lock().unwrap();
    stats_guard.record_query(query, success, sample_interval_secs);
}

/// Helper function to record a query execution with runtime tracking
pub fn record_query_with_time(
    stats: &Arc<Mutex<FuzzerStats>>,
    query: &str,
    success: bool,
    execution_time: Duration,
    sample_interval_secs: u64,
) {
    let mut stats_guard = stats.lock().unwrap();
    stats_guard.record_query_with_time(query, success, execution_time, sample_interval_secs);
}

/// Helper function to complete a fuzzing round
pub fn update_stat_for_round_completion(stats: &Arc<Mutex<FuzzerStats>>) {
    let mut stats_guard = stats.lock().unwrap();
    stats_guard.complete_round();
}

/// Helper function to get TUI stats
pub fn get_tui_stats(stats: &Arc<Mutex<FuzzerStats>>) -> TuiStats {
    let stats_guard = stats.lock().unwrap();
    stats_guard.get_tui_stats()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percentile_calculation() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

        assert_eq!(percentile(&values, 0.0), 1.0);
        assert_eq!(percentile(&values, 50.0), 5.0);
        assert_eq!(percentile(&values, 90.0), 9.0);
        assert_eq!(percentile(&values, 100.0), 10.0);
    }

    #[test]
    fn test_runtime_stats_creation() {
        let records = vec![
            QueryExecutionRecord {
                query: "SELECT 1".to_string(),
                execution_time: Duration::from_millis(10),
            },
            QueryExecutionRecord {
                query: "SELECT 2".to_string(),
                execution_time: Duration::from_millis(20),
            },
            QueryExecutionRecord {
                query: "SELECT 3".to_string(),
                execution_time: Duration::from_millis(30),
            },
            QueryExecutionRecord {
                query: "SELECT 4".to_string(),
                execution_time: Duration::from_millis(100),
            },
            QueryExecutionRecord {
                query: "SELECT 5 -- slowest".to_string(),
                execution_time: Duration::from_millis(200),
            },
        ];

        let stats = QueryRuntimeStats::from_records(&records).unwrap();

        assert_eq!(stats.fastest_ms, 10.0);
        assert_eq!(stats.slowest_ms, 200.0);
        assert_eq!(stats.avg_ms, 72.0); // (10+20+30+100+200)/5
        assert_eq!(stats.slowest_query, "SELECT 5 -- slowest");
    }

    #[test]
    fn test_empty_query_records() {
        let stats = QueryRuntimeStats::from_records(&[]);
        assert!(stats.is_none());
    }

    #[test]
    fn test_fuzzer_stats_with_runtime() {
        let mut stats = FuzzerStats::new(1);

        // Record some queries with execution times
        stats.record_query_with_time("SELECT 1", true, Duration::from_millis(10), 5);
        stats.record_query_with_time("SELECT 2 -- slowest", true, Duration::from_millis(20), 5);

        let tui_stats = stats.get_tui_stats();
        assert_eq!(tui_stats.queries_executed, 2);
        assert!(tui_stats.query_runtime_stats.is_some());

        let runtime_stats = tui_stats.query_runtime_stats.unwrap();
        assert_eq!(runtime_stats.fastest_ms, 10.0);
        assert_eq!(runtime_stats.slowest_ms, 20.0);
        assert_eq!(runtime_stats.avg_ms, 15.0);
        assert_eq!(runtime_stats.slowest_query, "SELECT 2 -- slowest");
    }
}
