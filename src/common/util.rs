// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: See the NOTICE file distributed with this work for additional information regarding copyright ownership
// SPDX-FileContributor: Licensed to the Apache Software Foundation (ASF) under one or more contributor license agreements

use datafusion::{
    arrow::array::RecordBatch, common::utils::get_row_at_idx, prelude::Expr, scalar::ScalarValue,
    sql::unparser::expr_to_sql,
};
use std::collections::HashMap;

use super::{Result, fuzzer_err};

/// Convert a DataFusion `Expr` into a SQL string using DataFusion's unparser.
pub fn to_sql_string(expr: &Expr) -> Result<String> {
    let unparsed = expr_to_sql(expr)?;
    Ok(unparsed.to_string())
}

pub(crate) type RowMultiset = HashMap<Vec<ScalarValue>, usize>;

pub(crate) fn batches_to_row_multiset(batches: &[RecordBatch]) -> Result<RowMultiset> {
    let mut multiset: RowMultiset = HashMap::new();
    let mut expected_num_cols: Option<usize> = None;

    for batch in batches {
        if let Some(expected) = expected_num_cols {
            if batch.num_columns() != expected {
                return Err(fuzzer_err(&format!(
                    "Mismatched column count across batches: expected {}, got {}",
                    expected,
                    batch.num_columns()
                )));
            }
        } else {
            expected_num_cols = Some(batch.num_columns());
        }

        for row_idx in 0..batch.num_rows() {
            let mut row_key = get_row_at_idx(batch.columns(), row_idx)
                .map_err(|e| fuzzer_err(&format!("Failed to extract row {}: {}", row_idx, e)))?;
            row_key.iter_mut().for_each(|v| *v = v.clone().compacted());
            *multiset.entry(row_key).or_insert(0) += 1;
        }
    }

    Ok(multiset)
}

pub(crate) fn format_row_multiset_diff(left: &RowMultiset, right: &RowMultiset) -> String {
    let mut lines = Vec::new();
    for (row, left_count) in left {
        let right_count = right.get(row).copied().unwrap_or(0);
        if *left_count != right_count {
            lines.push(format!(
                "row={:?}, left_count={}, right_count={}",
                row, left_count, right_count
            ));
        }
    }
    for (row, right_count) in right {
        if !left.contains_key(row) {
            lines.push(format!(
                "row={:?}, left_count=0, right_count={}",
                row, right_count
            ));
        }
    }

    lines.sort();
    let preview = lines.into_iter().take(20).collect::<Vec<_>>();
    if preview.is_empty() {
        "no row differences".to_string()
    } else {
        preview.join("\n")
    }
}

pub(crate) fn count_total_rows(batches: &[RecordBatch]) -> usize {
    batches.iter().map(RecordBatch::num_rows).sum()
}

pub(crate) fn validate_batches_value_equivalence(
    left_batches: &[RecordBatch],
    right_batches: &[RecordBatch],
    oracle_name: &str,
) -> Result<()> {
    let left_multiset = batches_to_row_multiset(left_batches)?;
    let right_multiset = batches_to_row_multiset(right_batches)?;

    if left_multiset != right_multiset {
        return Err(fuzzer_err(&format!(
            "{} value equivalence violated:\n{}",
            oracle_name,
            format_row_multiset_diff(&left_multiset, &right_multiset)
        )));
    }

    Ok(())
}
