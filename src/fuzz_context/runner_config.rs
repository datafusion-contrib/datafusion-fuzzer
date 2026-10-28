// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: See the NOTICE file distributed with this work for additional information regarding copyright ownership
// SPDX-FileContributor: Licensed to the Apache Software Foundation (ASF) under one or more contributor license agreements

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::common::{Result, fuzzer_err};
use crate::oracle::ConfiguredOracle;

/// Unified configuration for the DataFusion fuzzer.
///
/// This configuration controls both:
/// 1. The overall fuzzing process (rounds, queries, timeout)
/// 2. The table and query generation parameters
/// 3. UI and display parameters
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RunnerConfig {
    // General fuzzing parameters
    pub seed: u64,
    pub rounds: u32,
    pub queries_per_round: u32,
    pub timeout_seconds: u64,
    pub log_path: Option<PathBuf>,

    // UI and display parameters
    pub display_logs: bool,
    pub enable_tui: bool,
    pub sample_interval_secs: u64,

    // Table and query generation parameters
    pub max_column_count: u64,
    pub max_row_count: u64,
    pub max_expr_level: u32,
    #[serde(default = "RunnerConfig::default_max_group_by_count")]
    pub max_group_by_count: u32,
    pub max_table_count: u32,
    pub max_insert_per_table: u32,
    #[serde(default = "RunnerConfig::default_oracles", alias = "oracle")]
    pub oracles: Vec<ConfiguredOracle>,
}

impl RunnerConfig {
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| fuzzer_err(&format!("Failed to read config file: {}", e)))?;

        Self::from_toml_str(&content)
    }

    pub fn from_cli(cli: &crate::cli::Cli) -> Result<Self> {
        // Start with default or config file if provided
        let mut config = if let Some(config_path) = &cli.config {
            Self::from_file(config_path)?
        } else {
            Self::default()
        };

        // Override with CLI arguments if provided
        if cli.seed != 42 {
            config.seed = cli.seed;
        }

        if let Some(rounds) = cli.rounds {
            config.rounds = rounds;
        }

        if let Some(queries) = cli.queries_per_round {
            config.queries_per_round = queries;
        }

        if let Some(timeout) = cli.timeout {
            config.timeout_seconds = timeout;
        }

        if let Some(log_path) = &cli.log_path {
            config.log_path = Some(log_path.clone());
        }

        // Set display_logs from CLI argument
        config.display_logs = cli.display_logs;

        // Set enable_tui from CLI argument
        config.enable_tui = cli.enable_tui;

        config.validate()
    }

    fn from_toml_str(content: &str) -> Result<Self> {
        let config: Self = toml::from_str(content)
            .map_err(|e| fuzzer_err(&format!("Failed to parse config file: {}", e)))?;

        config.validate()
    }

    fn validate(self) -> Result<Self> {
        if self.oracles.is_empty() {
            return Err(fuzzer_err("At least one oracle must be configured"));
        }

        Ok(self)
    }

    fn default_oracles() -> Vec<ConfiguredOracle> {
        vec![ConfiguredOracle::NoCrash]
    }

    fn default_max_group_by_count() -> u32 {
        3
    }
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            rounds: 3,
            queries_per_round: 10,
            timeout_seconds: 2,
            log_path: Some(PathBuf::from("logs")),
            display_logs: false,
            enable_tui: true,
            sample_interval_secs: 5,
            max_column_count: 5,
            max_row_count: 100,
            max_expr_level: 3,
            max_group_by_count: Self::default_max_group_by_count(),
            max_table_count: 3,
            max_insert_per_table: 20,
            oracles: Self::default_oracles(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_oracle_from_array_syntax() {
        let config = RunnerConfig::from_toml_str(
            r#"
seed = 42
rounds = 1
queries_per_round = 1
timeout_seconds = 2
log_path = "logs"
display_logs = false
enable_tui = false
sample_interval_secs = 5
max_column_count = 5
max_row_count = 100
max_expr_level = 3
max_group_by_count = 3
max_table_count = 3
max_insert_per_table = 20
oracles = ["NoCrash"]
"#,
        )
        .unwrap();

        assert_eq!(config.oracles, vec![ConfiguredOracle::NoCrash]);
    }

    #[test]
    fn parses_multiple_oracles_from_array_syntax() {
        let config = RunnerConfig::from_toml_str(
            r#"
seed = 42
rounds = 1
queries_per_round = 1
timeout_seconds = 2
log_path = "logs"
display_logs = false
enable_tui = false
sample_interval_secs = 5
max_column_count = 5
max_row_count = 100
max_expr_level = 3
max_group_by_count = 3
max_table_count = 3
max_insert_per_table = 20
oracles = ["NoCrash", "NestedQueries", "TlpWhere", "TlpHaving"]
"#,
        )
        .unwrap();

        assert_eq!(
            config.oracles,
            vec![
                ConfiguredOracle::NoCrash,
                ConfiguredOracle::NestedQueries,
                ConfiguredOracle::TlpWhere,
                ConfiguredOracle::TlpHaving
            ]
        );
    }

    #[test]
    fn rejects_empty_oracle_set() {
        let error = RunnerConfig::from_toml_str(
            r#"
seed = 42
rounds = 1
queries_per_round = 1
timeout_seconds = 2
log_path = "logs"
display_logs = false
enable_tui = false
sample_interval_secs = 5
max_column_count = 5
max_row_count = 100
max_expr_level = 3
max_group_by_count = 3
max_table_count = 3
max_insert_per_table = 20
oracles = []
"#,
        )
        .unwrap_err();

        assert_eq!(error.to_string(), "At least one oracle must be configured");
    }
}
