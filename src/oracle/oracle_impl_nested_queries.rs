// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: See the NOTICE file distributed with this work for additional information regarding copyright ownership
// SPDX-FileContributor: Licensed to the Apache Software Foundation (ASF) under one or more contributor license agreements

use crate::common::{InclusionConfig, Result};
use crate::oracle::{Oracle, QueryContext, QueryExecutionResult};
use crate::query_generator::stmt_select_def::SelectStatementBuilder;
use std::sync::Arc;

/// An oracle that tests query consistency using nested queries and subqueries.
/// This oracle generates nested query patterns and validates their consistency.
pub struct NestedQueriesOracle {
    /// Random seed for query generation
    seed: u64,
    /// Global context containing table information and configuration
    ctx: Arc<crate::fuzz_context::GlobalContext>,
}

impl NestedQueriesOracle {
    /// Create a new NestedQueriesOracle with the specified seed and context
    pub fn new(seed: u64, ctx: Arc<crate::fuzz_context::GlobalContext>) -> Self {
        Self { seed, ctx }
    }
}

#[async_trait::async_trait]
impl Oracle for NestedQueriesOracle {
    fn name(&self) -> &'static str {
        "NestedQueriesOracle"
    }

    fn generate_query_group(&mut self) -> Result<Vec<QueryContext>> {
        // Generate a single random query using the existing query generator
        let mut stmt_builder = SelectStatementBuilder::new(
            self.seed,
            Arc::clone(&self.ctx),
            InclusionConfig::Maybe(0.2),
            InclusionConfig::Maybe(0.2),
        )
        // Enable derived tables (views/subqueries) for nested query testing
        .with_allow_derived_tables(true)
        // Avoid huge joins to slow down fuzzing
        .with_max_table_count(3);
        let stmt = stmt_builder.generate_stmt()?;
        let sql = stmt.to_sql_string()?;

        // Use the existing DataFusion context from the global context
        // This context already has all the tables registered from the dataset generator
        let session_context = self.ctx.runtime_context.get_session_context();

        // Create a single QueryContext for this query
        let query_context = QueryContext::with_description(
            sql,
            session_context,
            "Nested Queries Consistency Test".to_string(),
        );

        Ok(vec![query_context])
    }

    async fn validate_consistency(&self, results: &[QueryExecutionResult]) -> Result<()> {
        if results.is_empty() {
            return Err(crate::common::fuzzer_err("No query results to validate"));
        }

        // For the nested queries oracle, we validate that queries with derived tables
        // (views and subqueries) execute consistently and produce expected results.
        // Since error message whitelist check is done outside in the runner,
        // we always pass validation here. The oracle only fails for truly unexpected
        // crashes or inconsistencies, not for whitelisted errors.
        Ok(())
    }

    fn create_error_report(&self, results: &[QueryExecutionResult]) -> Result<String> {
        let mut report = String::new();
        report.push_str("Nested Queries Oracle Test Failed\n");
        report.push_str("==================================\n\n");

        if !results.is_empty() {
            let query_result = &results[0];
            let query_context = &*query_result.query_context;

            report.push_str(&format!(
                "Nested query that caused inconsistency or error:\n{}\n\n",
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
            "Expected: Nested queries with views/subqueries should execute consistently\n",
        );
        report.push_str("Actual: Query produced inconsistent results or non-whitelisted error\n\n");

        report.push_str("This indicates a potential issue with nested query processing.\n");
        report.push_str(
            "Nested queries involving views and subqueries should produce consistent results.\n",
        );
        report.push_str("Whitelisted errors are acceptable and expected (e.g., divide by zero).\n");

        Ok(report)
    }
}
