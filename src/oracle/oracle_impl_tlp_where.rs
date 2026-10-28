// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: See the NOTICE file distributed with this work for additional information regarding copyright ownership
// SPDX-FileContributor: Licensed to the Apache Software Foundation (ASF) under one or more contributor license agreements

use crate::common::{InclusionConfig, Result, fuzzer_err};
use crate::oracle::oracle_common;
use crate::oracle::{Oracle, QueryContext, QueryExecutionResult};
use crate::query_generator::stmt_select_def::SelectStatementBuilder;
use std::sync::Arc;

/// TLP-WHERE oracle.
///
/// It validates value-level multiset equivalence:
/// q_all == q_p UNION ALL q_not_p UNION ALL q_p_is_null
///
/// ### Example:
///
/// SELECT * FROM t;
///
/// should return the same multiset as
///
/// SELECT * FROM t
/// WHERE date_col = DATE '2000-01-01'
/// UNION ALL
/// SELECT * FROM t
/// WHERE NOT (date_col = DATE '2000-01-01')
/// UNION ALL
/// SELECT * FROM t
/// WHERE (date_col = DATE '2000-01-01') IS NULL;
///
pub struct TlpWhereOracle {
    seed: u64,
    ctx: Arc<crate::fuzz_context::GlobalContext>,
}

impl TlpWhereOracle {
    pub fn new(seed: u64, ctx: Arc<crate::fuzz_context::GlobalContext>) -> Self {
        Self { seed, ctx }
    }
}

#[async_trait::async_trait]
impl Oracle for TlpWhereOracle {
    fn name(&self) -> &'static str {
        "TlpWhereOracle"
    }

    fn generate_query_group(&mut self) -> Result<Vec<QueryContext>> {
        let mut stmt_builder = SelectStatementBuilder::new(
            self.seed,
            Arc::clone(&self.ctx),
            InclusionConfig::Always(true),
            InclusionConfig::Always(false),
        )
        .with_allow_derived_tables(false)
        .with_max_table_count(1);

        let stmt = stmt_builder.generate_stmt()?;
        let source_sql = stmt.to_from_join_sql()?;
        let predicate = stmt
            .where_expr()
            .ok_or_else(|| fuzzer_err("TLP-WHERE expected a generated WHERE predicate"))?;
        let predicate_sql = crate::common::util::to_sql_string(predicate)?;

        let q_all = format!("SELECT *\n{}", source_sql);
        let q_partition_union = format!(
            "SELECT *\n{}\nWHERE ({})\nUNION ALL\nSELECT *\n{}\nWHERE NOT ({})\nUNION ALL\nSELECT *\n{}\nWHERE ({}) IS NULL",
            source_sql, predicate_sql, source_sql, predicate_sql, source_sql, predicate_sql
        );

        let session_context = self.ctx.runtime_context.get_session_context();
        Ok(vec![
            QueryContext::with_description(
                q_all,
                Arc::clone(&session_context),
                "TLP-WHERE all".to_string(),
            ),
            QueryContext::with_description(
                q_partition_union,
                Arc::clone(&session_context),
                "TLP-WHERE p UNION ALL NOT p UNION ALL p IS NULL".to_string(),
            ),
        ])
    }

    async fn validate_consistency(&self, results: &[QueryExecutionResult]) -> Result<()> {
        oracle_common::validate_binary_tlp_consistency(results, "TLP-WHERE")
    }

    fn create_error_report(&self, results: &[QueryExecutionResult]) -> Result<String> {
        let mut report = String::new();
        report.push_str("TLP-WHERE Oracle Test Failed\n");
        report.push_str("===========================\n\n");

        let labels = ["all", "p UNION ALL NOT p UNION ALL p IS NULL"];
        oracle_common::append_labeled_query_results(&mut report, results, &labels);
        oracle_common::append_binary_value_equivalence_report(&mut report, results)?;

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{LogicalTable, init_available_data_types};
    use crate::oracle::test_helpers;

    #[tokio::test]
    async fn tlp_where_validate_passes_for_matching_values() {
        let oracle =
            TlpWhereOracle::new(1, Arc::new(crate::fuzz_context::GlobalContext::default()));
        let results = vec![
            test_helpers::make_success_result("all", "cnt", vec![1, 2, 3]),
            test_helpers::make_success_result("partition_union", "cnt", vec![1, 2, 3]),
        ];

        assert!(oracle.validate_consistency(&results).await.is_ok());
    }

    #[tokio::test]
    async fn tlp_where_validate_fails_for_value_mismatch() {
        let oracle =
            TlpWhereOracle::new(1, Arc::new(crate::fuzz_context::GlobalContext::default()));
        let results = vec![
            test_helpers::make_success_result("all", "cnt", vec![1, 2]),
            test_helpers::make_success_result("partition_union", "cnt", vec![1, 2, 2]),
        ];

        let err = oracle.validate_consistency(&results).await.unwrap_err();
        assert!(
            err.to_string()
                .contains("TLP-WHERE value equivalence violated")
        );
    }

    #[tokio::test]
    async fn tlp_where_validate_fails_for_mixed_query_outcomes() {
        let oracle =
            TlpWhereOracle::new(1, Arc::new(crate::fuzz_context::GlobalContext::default()));
        let results = vec![
            test_helpers::make_success_result("all", "cnt", vec![1, 2]),
            test_helpers::make_error_result("partition_union"),
        ];

        let err = oracle.validate_consistency(&results).await.unwrap_err();
        assert!(err.to_string().contains("got mixed outcomes"));
    }

    #[tokio::test]
    async fn tlp_where_validate_allows_all_query_errors() {
        let oracle =
            TlpWhereOracle::new(1, Arc::new(crate::fuzz_context::GlobalContext::default()));
        let results = vec![
            test_helpers::make_error_result("all"),
            test_helpers::make_error_result("partition_union"),
        ];

        assert!(oracle.validate_consistency(&results).await.is_ok());
    }

    #[tokio::test]
    async fn tlp_where_validate_fails_for_schema_mismatch() {
        let one_col_batch = test_helpers::make_one_col_batch(vec![1, 2]);
        let two_col_batch = test_helpers::make_two_col_batch(vec![1], vec![9]);

        let oracle =
            TlpWhereOracle::new(1, Arc::new(crate::fuzz_context::GlobalContext::default()));
        let results = vec![
            QueryExecutionResult {
                query_context: test_helpers::make_query_context("all"),
                result: Ok(vec![one_col_batch.clone()]),
            },
            QueryExecutionResult {
                query_context: test_helpers::make_query_context("partition_union"),
                result: Ok(vec![two_col_batch]),
            },
        ];

        let err = oracle.validate_consistency(&results).await.unwrap_err();
        assert!(err.to_string().contains("value equivalence violated"));
    }

    #[test]
    fn tlp_where_generates_expected_query_group_shape() {
        init_available_data_types();
        let ctx = Arc::new(crate::fuzz_context::GlobalContext::default());
        ctx.runtime_context
            .registered_tables
            .write()
            .unwrap()
            .insert(
                "t0".to_string(),
                Arc::new(LogicalTable::new("t0".to_string())),
            );

        let mut oracle = TlpWhereOracle::new(123, Arc::clone(&ctx));
        let query_group = oracle.generate_query_group().unwrap();
        let queries = QueryContext::get_queries(&query_group);

        assert_eq!(queries.len(), 2);
        assert!(queries[0].contains("SELECT *"));
        assert!(!queries[0].contains("\nWHERE "));
        assert!(queries[1].contains("UNION ALL"));
        assert!(queries[1].contains("\nWHERE ("));
        assert!(queries[1].contains("WHERE NOT ("));
        assert!(queries[1].contains(") IS NULL"));
    }
}
