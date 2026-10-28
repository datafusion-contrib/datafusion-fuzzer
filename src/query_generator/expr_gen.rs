// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: See the NOTICE file distributed with this work for additional information regarding copyright ownership
// SPDX-FileContributor: Licensed to the Apache Software Foundation (ASF) under one or more contributor license agreements

use std::sync::Arc;

use datafusion::{arrow::datatypes::DataType, common::Column, prelude::Expr, sql::TableReference};
use rand::{Rng, rngs::StdRng};

use crate::{
    common::{FuzzerDataType, LogicalTable, rng::rng_from_seed},
    fuzz_context::GlobalContext,
};

use super::{
    expr_def::{BaseExpr, ExprWrapper, all_available_exprs},
    expr_literal_gen::generate_scalar_literal,
};

pub struct ExprGenerator {
    rng: StdRng,
    ctx: Arc<GlobalContext>,
    max_level: u32,

    /// All possible column references that can be used in the generated expressions.
    src_columns: Arc<Vec<Column>>,
}

impl ExprGenerator {
    pub fn new(seed: u64, context: Arc<GlobalContext>) -> Self {
        let max_level = context.runner_config.max_expr_level;
        Self {
            rng: rng_from_seed(seed),
            ctx: context,
            max_level,
            src_columns: Arc::new(Vec::new()),
        }
    }

    pub fn with_src_columns(mut self, src_columns: Arc<Vec<Column>>) -> Self {
        self.src_columns = src_columns;
        self
    }

    /// Helper function to convert a vector of LogicalTable to a vector of Column references
    pub fn tables_to_columns(
        tables: &[Arc<LogicalTable>],
        _ctx: &Arc<GlobalContext>,
    ) -> Vec<Column> {
        let mut columns = Vec::new();

        for table in tables {
            let table_ref = TableReference::bare(table.name.clone());
            let table = table.as_ref();

            // Use the actual column information from the table
            for logical_column in &table.columns {
                columns.push(Column::new(
                    Some(table_ref.clone()),
                    logical_column.name.clone(),
                ));
            }
        }
        columns
    }

    fn pick_random_expr_with_return_type(
        &mut self,
        target_type: DataType,
    ) -> Option<Arc<ExprWrapper>> {
        let exprs = all_available_exprs();
        let expr_with_return_type: Vec<Arc<ExprWrapper>> = exprs
            .iter()
            .filter(|expr| expr.return_type.contains(&target_type))
            .map(|expr| Arc::clone(expr))
            .collect();

        // If no expressions match the target type, return None
        if expr_with_return_type.is_empty() {
            return None;
        }

        let expr =
            expr_with_return_type[self.rng.random_range(0..expr_with_return_type.len())].clone();

        Some(expr)
    }

    pub fn generate_random_expr(&mut self, target_type: DataType, cur_level: u32) -> Expr {
        let half_chance = self.rng.random_bool(0.5);
        if cur_level == self.max_level || half_chance {
            // Generate a leaf expression
            return self.generate_leaf_expr(target_type);
        }

        // Try to pick a random expression with the target return type
        if let Some(random_expr) = self.pick_random_expr_with_return_type(target_type.clone()) {
            let child_signature = random_expr.pick_child_signature(target_type, &mut self.rng);

            let child_exprs: Vec<Expr> = child_signature
                .iter()
                .map(|dt| self.generate_random_expr(dt.clone(), cur_level + 1))
                .collect();

            self.build_with_childs(random_expr.expr.clone(), &child_exprs)
        } else {
            // No expressions available for this type, fallback to leaf expression
            self.generate_leaf_expr(target_type)
        }
    }

    // Generate either a constant value or a column reference
    fn generate_leaf_expr(&mut self, target_type: DataType) -> Expr {
        // For certain chance: try to generate a column reference if available
        let columns = self.get_all_columns_of_type(target_type.clone());
        if !columns.is_empty() && self.rng.random_bool(0.5) {
            let column = columns[self.rng.random_range(0..columns.len())].clone();
            return Expr::Column(column);
        }

        // Otherwise, generate a constant literal
        if let Some(fuzzer_type) = FuzzerDataType::from_datafusion_type(&target_type) {
            let scalar_value = generate_scalar_literal(&self.ctx, &mut self.rng, &fuzzer_type);
            Expr::Literal(scalar_value, None)
        } else {
            // Fallback to a simple boolean literal for unsupported types
            let scalar_value =
                generate_scalar_literal(&self.ctx, &mut self.rng, &FuzzerDataType::Boolean);
            Expr::Literal(scalar_value, None)
        }
    }

    fn get_all_columns_of_type(&self, target_type: DataType) -> Vec<Column> {
        // Use the actual column information from the tables
        let mut matching_columns = Vec::new();

        for column in self.src_columns.as_ref() {
            // Try to find the table this column belongs to
            if let Some(table_ref) = &column.relation {
                let table_name = table_ref.to_string();

                // Get the table from the fuzzer context
                let tables_lock = self.ctx.runtime_context.registered_tables.read().unwrap();
                if let Some(logical_table) = tables_lock.get(&table_name) {
                    // Find the matching logical column
                    for logical_column in &logical_table.columns {
                        if logical_column.name == column.name {
                            // Check if the data type matches
                            let column_df_type = logical_column.data_type.to_datafusion_type();
                            if column_df_type == target_type {
                                matching_columns.push(column.clone());
                            }
                            break;
                        }
                    }
                }
            }
        }

        matching_columns
    }

    /// If the number of childs is not correct, it will try to fix automatically.
    /// Note this function does not guarantee to return a valid expression on purpose
    /// , because invalid expression (like `true + 1`) can provide more test coverage.
    fn build_with_childs(&self, base_expr: BaseExpr, child_exprs: &[Expr]) -> Expr {
        // TODO: validate the number of `child_exprs`
        let expr_impl = base_expr.to_impl();
        expr_impl.build_expr(child_exprs)
    }
}
