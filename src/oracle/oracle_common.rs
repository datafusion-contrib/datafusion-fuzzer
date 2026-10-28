// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: See the NOTICE file distributed with this work for additional information regarding copyright ownership
// SPDX-FileContributor: Licensed to the Apache Software Foundation (ASF) under one or more contributor license agreements

use crate::common::{Result, fuzzer_err, util};
use crate::oracle::QueryExecutionResult;

pub(crate) fn validate_binary_tlp_consistency(
    results: &[QueryExecutionResult],
    oracle_name: &str,
) -> Result<()> {
    let result_count = results.len();
    if result_count != 2 {
        return Err(fuzzer_err(&format!(
            "{oracle_name} expects 2 query results, got {result_count}"
        )));
    }

    let num_ok = results.iter().filter(|r| r.result.is_ok()).count();
    let num_err = result_count - num_ok;

    match num_ok {
        2 => validate_value_equivalence(results, 0, 1, oracle_name),
        0 => Ok(()),
        _ => Err(fuzzer_err(&format!(
            "{oracle_name} consistency requires all queries to either succeed or fail; got mixed outcomes (ok={num_ok}, err={num_err})"
        ))),
    }
}

pub(crate) fn validate_value_equivalence(
    results: &[QueryExecutionResult],
    left_idx: usize,
    right_idx: usize,
    oracle_name: &str,
) -> Result<()> {
    let left_result = results
        .get(left_idx)
        .ok_or_else(|| fuzzer_err(&format!("Missing result at index {}", left_idx)))?;
    let right_result = results
        .get(right_idx)
        .ok_or_else(|| fuzzer_err(&format!("Missing result at index {}", right_idx)))?;

    let left_batches = left_result
        .result
        .as_ref()
        .map_err(|e| fuzzer_err(&e.to_string()))?;
    let right_batches = right_result
        .result
        .as_ref()
        .map_err(|e| fuzzer_err(&e.to_string()))?;

    util::validate_batches_value_equivalence(left_batches, right_batches, oracle_name)
}

pub(crate) fn append_labeled_query_results(
    report: &mut String,
    results: &[QueryExecutionResult],
    labels: &[&str],
) {
    for (idx, result) in results.iter().enumerate() {
        let label = labels.get(idx).copied().unwrap_or("unknown");
        report.push_str(&format!(
            "Q{} [{}]:\n{}\n",
            idx + 1,
            label,
            result.query_context.query
        ));

        match &result.result {
            Ok(batches) => report.push_str(&format!(
                "  status: ok, rows={}\n\n",
                util::count_total_rows(batches)
            )),
            Err(e) => report.push_str(&format!("  status: error, details={}\n\n", e)),
        }
    }
}

pub(crate) fn append_binary_value_equivalence_report(
    report: &mut String,
    results: &[QueryExecutionResult],
) -> Result<()> {
    if results.len() != 2 || results.iter().any(|r| r.result.is_err()) {
        return Ok(());
    }

    let q_all_batches = results[0]
        .result
        .as_ref()
        .map_err(|e| fuzzer_err(&e.to_string()))?;
    let q_union_batches = results[1]
        .result
        .as_ref()
        .map_err(|e| fuzzer_err(&e.to_string()))?;

    report.push_str(&format!(
        "Row counts: all={}, partition_union={}\n",
        util::count_total_rows(q_all_batches),
        util::count_total_rows(q_union_batches)
    ));

    let all_multiset = util::batches_to_row_multiset(q_all_batches)?;
    let partition_multiset = util::batches_to_row_multiset(q_union_batches)?;

    if all_multiset != partition_multiset {
        report.push_str("\nTop multiset differences:\n");
        report.push_str(&util::format_row_multiset_diff(
            &all_multiset,
            &partition_multiset,
        ));
        report.push('\n');
    } else {
        report.push_str("Multiset equivalence: true\n");
    }

    Ok(())
}
