// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: See the NOTICE file distributed with this work for additional information regarding copyright ownership
// SPDX-FileContributor: Licensed to the Apache Software Foundation (ASF) under one or more contributor license agreements

pub mod ctx_observability;
mod runner_config;

use std::sync::{
    Arc, Mutex, RwLock,
    atomic::{AtomicU32, Ordering},
};

use datafusion::{common::HashMap, prelude::SessionContext};

use crate::common::LogicalTable;
use crate::common::value_generator::ValueGenerationConfig;
use crate::fuzz_runner::FuzzerStats;

pub use runner_config::RunnerConfig;

/// Create a default DataFusion SessionContext with standard configuration
/// This ensures consistency between initial creation and reset operations
fn default_df_session_context() -> Arc<SessionContext> {
    Arc::new(SessionContext::new())
}

pub struct GlobalContext {
    pub runner_config: RunnerConfig,
    pub runtime_context: RuntimeContext,
    pub fuzzer_stats: Arc<Mutex<FuzzerStats>>,
}

impl GlobalContext {
    pub fn new(
        runner_config: RunnerConfig,
        runtime_context: RuntimeContext,
        fuzzer_stats: Arc<Mutex<FuzzerStats>>,
    ) -> Self {
        Self {
            runner_config,
            runtime_context,
            fuzzer_stats,
        }
    }

    pub fn default() -> Self {
        let default_config = RunnerConfig::default();
        let fuzzer_stats = Arc::new(Mutex::new(FuzzerStats::new(default_config.rounds)));

        Self {
            runner_config: default_config,
            runtime_context: RuntimeContext::default(),
            fuzzer_stats,
        }
    }

    /// Reset the DataFusion context to drop all registered tables
    /// This creates a fresh SessionContext and clears all table registrations
    pub fn reset_datafusion_context(&self) {
        // Create a new SessionContext to completely reset the DataFusion state
        let new_session_context = default_df_session_context();

        // Replace the existing SessionContext with a new one
        {
            let mut df_ctx = self.runtime_context.df_ctx.write().unwrap();
            *df_ctx = new_session_context;
        }

        // Clear the fuzzer's table registry
        {
            let mut tables = self.runtime_context.registered_tables.write().unwrap();
            tables.clear();
        }

        // Reset the table counter
        self.runtime_context
            .current_table_idx
            .store(0, Ordering::Relaxed);
    }
}

pub struct RuntimeContext {
    pub df_ctx: Arc<RwLock<Arc<SessionContext>>>,
    pub registered_tables: Arc<RwLock<HashMap<String, Arc<LogicalTable>>>>,
    current_table_idx: AtomicU32,
    // Cached value generation config for performance (nullable by default)
    pub value_generation_config: ValueGenerationConfig,
}

impl RuntimeContext {
    pub fn new(df_ctx: Arc<SessionContext>) -> Self {
        Self {
            df_ctx: Arc::new(RwLock::new(df_ctx)),
            registered_tables: Arc::new(RwLock::new(HashMap::new())),
            current_table_idx: AtomicU32::new(0),
            value_generation_config: ValueGenerationConfig::default(), // Nullable by default
        }
    }

    pub fn default() -> Self {
        Self {
            df_ctx: Arc::new(RwLock::new(default_df_session_context())),
            registered_tables: Arc::new(RwLock::new(HashMap::new())),
            current_table_idx: AtomicU32::new(0),
            value_generation_config: ValueGenerationConfig::default(), // Nullable by default
        }
    }

    pub fn next_table_name(&self) -> String {
        format!(
            "t{}",
            self.current_table_idx.fetch_add(1, Ordering::Relaxed)
        )
    }

    /// Reset the table counter to 0 for deterministic naming
    pub fn reset_table_counter(&self) {
        self.current_table_idx.store(0, Ordering::Relaxed);
    }

    /// Get a clone of the current DataFusion SessionContext
    pub fn get_session_context(&self) -> Arc<SessionContext> {
        let df_ctx = self.df_ctx.read().unwrap();
        Arc::clone(&*df_ctx)
    }
}
