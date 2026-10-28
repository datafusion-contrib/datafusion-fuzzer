// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: See the NOTICE file distributed with this work for additional information regarding copyright ownership
// SPDX-FileContributor: Licensed to the Apache Software Foundation (ASF) under one or more contributor license agreements

//! Structs related to JOIN used in SQL statement generation.
use std::sync::Arc;

use datafusion::prelude::Expr;
use rand::{Rng, rngs::StdRng};

use crate::common::{LogicalTable, Result};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum JoinType {
    /// Join and InnerJoin are equivalent, but they will be using different keyword
    /// in the generated SQL query
    Join,
    InnerJoin,
    LeftJoin,
    RightJoin,
    FullJoin,
    LeftAntiJoin,
    LeftSemiJoin,
    RightAntiJoin,
    RightSemiJoin,
    CrossJoin,
    NaturalJoin,
}

impl JoinType {
    /// Returns a random JoinType using the provided random number generator.
    pub fn get_random(rng: &mut StdRng) -> Self {
        // Update this if you add/remove variants!
        // Exclude NaturalJoin if you want to avoid it for now.
        // Here, all variants are included.
        match rng.random_range(0..=10) {
            0 => JoinType::Join,
            1 => JoinType::InnerJoin,
            2 => JoinType::LeftJoin,
            3 => JoinType::RightJoin,
            4 => JoinType::FullJoin,
            5 => JoinType::LeftAntiJoin,
            6 => JoinType::LeftSemiJoin,
            7 => JoinType::RightAntiJoin,
            8 => JoinType::RightSemiJoin,
            9 => JoinType::CrossJoin,
            10 => JoinType::NaturalJoin,
            _ => unreachable!(),
        }
    }
}

impl std::fmt::Display for JoinType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let keyword = match self {
            JoinType::Join => "JOIN",
            JoinType::InnerJoin => "INNER JOIN",
            JoinType::LeftJoin => "LEFT JOIN",
            JoinType::RightJoin => "RIGHT JOIN",
            JoinType::FullJoin => "FULL JOIN",
            JoinType::LeftAntiJoin => "LEFT ANTI JOIN",
            JoinType::LeftSemiJoin => "LEFT SEMI JOIN",
            JoinType::RightAntiJoin => "RIGHT ANTI JOIN",
            JoinType::RightSemiJoin => "RIGHT SEMI JOIN",
            JoinType::CrossJoin => "CROSS JOIN",
            JoinType::NaturalJoin => "NATURAL JOIN",
        };
        write!(f, "{}", keyword)
    }
}

/// Generates JOIN clause, its intermediate representation can be converted to
/// valid SQL string that appears in the query through `to_sql_string()`
///
/// Note:
/// - CrossJoin type does not support ON expression, but the caller is free
///   to maybe add one, to generate invalid expressions to strengthen the test.
/// - TODO(low-priority): Now every table has different column names, NaturalJoin type is very
///   likely to fail. It's possible to support this in the future.
/// - All other join types support join expression
pub(crate) struct JoinClause {
    pub(crate) join_table: Arc<LogicalTable>,
    pub(crate) join_type: JoinType,
    pub(crate) join_on_expr: Option<Arc<Expr>>,
}

impl JoinClause {
    /// Generate SQL strings like
    /// `JOIN t1 ON t0.v1 = t1.v1`
    /// If `join_on_expr` is None, omit the ON clause.
    pub fn to_sql_string(&self) -> Result<String> {
        let base = format!("{} {}", self.join_type, self.join_table.name);
        if let Some(expr) = &self.join_on_expr {
            Ok(format!(
                "{} ON {}",
                base,
                crate::common::util::to_sql_string(expr)?
            ))
        } else {
            Ok(base)
        }
    }
}
