// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: See the NOTICE file distributed with this work for additional information regarding copyright ownership
// SPDX-FileContributor: Licensed to the Apache Software Foundation (ASF) under one or more contributor license agreements

use crate::common::fuzzer_err;
use crate::oracle::{QueryContext, QueryExecutionResult};
use datafusion::arrow::array::{Array, Int64Array, RecordBatch};
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::prelude::SessionContext;
use std::sync::Arc;

pub(crate) fn make_query_context(label: &str) -> Arc<QueryContext> {
    Arc::new(QueryContext::new(
        format!("SELECT {}", label),
        Arc::new(SessionContext::new()),
    ))
}

pub(crate) fn make_success_result(
    label: &str,
    column_name: &str,
    values: Vec<i64>,
) -> QueryExecutionResult {
    let schema = Arc::new(Schema::new(vec![Field::new(
        column_name,
        DataType::Int64,
        false,
    )]));
    let array = Arc::new(Int64Array::from(values)) as Arc<dyn Array>;
    let batch = RecordBatch::try_new(schema, vec![array]).unwrap();

    QueryExecutionResult {
        query_context: make_query_context(label),
        result: Ok(vec![batch]),
    }
}

pub(crate) fn make_error_result(label: &str) -> QueryExecutionResult {
    QueryExecutionResult {
        query_context: make_query_context(label),
        result: Err(fuzzer_err("expected execution error in test")),
    }
}

pub(crate) fn make_one_col_batch(values: Vec<i64>) -> RecordBatch {
    let schema = Arc::new(Schema::new(vec![Field::new("c1", DataType::Int64, false)]));
    RecordBatch::try_new(
        schema,
        vec![Arc::new(Int64Array::from(values)) as Arc<dyn Array>],
    )
    .unwrap()
}

pub(crate) fn make_two_col_batch(col1: Vec<i64>, col2: Vec<i64>) -> RecordBatch {
    let schema = Arc::new(Schema::new(vec![
        Field::new("c1", DataType::Int64, false),
        Field::new("c2", DataType::Int64, false),
    ]));
    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int64Array::from(col1)) as Arc<dyn Array>,
            Arc::new(Int64Array::from(col2)) as Arc<dyn Array>,
        ],
    )
    .unwrap()
}
