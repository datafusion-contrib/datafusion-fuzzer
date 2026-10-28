// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: See the NOTICE file distributed with this work for additional information regarding copyright ownership
// SPDX-FileContributor: Licensed to the Apache Software Foundation (ASF) under one or more contributor license agreements

use crate::common::{InclusionConfig, Result};
use crate::oracle::{Oracle, QueryContext, QueryExecutionResult};
use crate::query_generator::stmt_select_def::SelectStatementBuilder;
use std::sync::Arc;

/// An oracle that generates random queries and ensures they don't crash or produce non-whitelisted errors.
/// This oracle works in conjunction with an error message whitelist system - whitelisted errors
/// (such as "divide by zero") are considered acceptable, while non-whitelisted errors indicate
/// potential stability issues in the query engine.
pub struct NoCrashOracle {
    /// Random seed for query generation
    seed: u64,
    /// Global context containing table information and configuration
    ctx: Arc<crate::fuzz_context::GlobalContext>,
}

impl NoCrashOracle {
    /// Create a new NoCrashOracle with the specified seed and context
    pub fn new(seed: u64, ctx: Arc<crate::fuzz_context::GlobalContext>) -> Self {
        Self { seed, ctx }
    }
}

// TODO: QPS drops significantly when using this oracle interface, instead of letting
// random queries be executed directly. Need to investigate why.
#[async_trait::async_trait]
impl Oracle for NoCrashOracle {
    fn name(&self) -> &'static str {
        "NoCrashOracle"
    }

    fn generate_query_group(&mut self) -> Result<Vec<QueryContext>> {
        // Generate a single random query using the existing query generator
        let mut stmt_builder = SelectStatementBuilder::new(
            self.seed,
            Arc::clone(&self.ctx),
            InclusionConfig::Maybe(0.9),
            InclusionConfig::Maybe(0.5),
        )
        // Views/subqueries are tested by other oracles
        .with_allow_derived_tables(false);
        let stmt = stmt_builder.generate_stmt()?;
        let sql = stmt.to_sql_string()?;

        // Use the existing DataFusion context from the global context
        // This context already has all the tables registered from the dataset generator
        let session_context = self.ctx.runtime_context.get_session_context();

        // Create a single QueryContext for this query
        let query_context = QueryContext::with_description(
            sql,
            session_context,
            "Random Query No-Crash Test".to_string(),
        );

        Ok(vec![query_context])
    }

    async fn validate_consistency(&self, results: &[QueryExecutionResult]) -> Result<()> {
        if results.is_empty() {
            return Err(crate::common::fuzzer_err("No query results to validate"));
        }

        // For the no-crash oracle, since error message whitelist check is now done
        // outside in the runner, we always pass validation. The oracle only fails
        // for truly unexpected crashes, not for whitelisted errors.
        // The actual error checking (whitelist validation) is handled in execute_single_query.
        Ok(())
    }

    fn create_error_report(&self, results: &[QueryExecutionResult]) -> Result<String> {
        let mut report = String::new();
        report.push_str("No-Crash Oracle Test Failed\n");
        report.push_str("============================\n\n");

        if !results.is_empty() {
            let query_result = &results[0];
            let query_context = &*query_result.query_context;

            report.push_str(&format!(
                "Query that caused non-whitelisted error/crash:\n{}\n\n",
                query_context.query
            ));
            report.push_str(&format!(
                "Context: {}\n\n",
                query_context.display_description()
            ));

            // Include the specific error if available
            if let Err(e) = &query_result.result {
                report.push_str(&format!("Error details: {}\n\n", e));
            }
        }

        report.push_str(
            "Expected: Query should execute without crashing or return a whitelisted error\n",
        );
        report.push_str("Actual: Query crashed or returned a non-whitelisted error\n\n");

        report.push_str("This indicates a potential stability issue in the query engine.\n");
        report.push_str(
            "The query should either return valid results or a graceful whitelisted error message.\n",
        );
        report.push_str("Whitelisted errors are acceptable and expected (e.g., divide by zero).\n");

        Ok(report)
    }
}
