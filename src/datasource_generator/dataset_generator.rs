// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: See the NOTICE file distributed with this work for additional information regarding copyright ownership
// SPDX-FileContributor: Licensed to the Apache Software Foundation (ASF) under one or more contributor license agreements

use std::sync::Arc;

use datafusion::error::Result;
use rand::Rng;
use rand::rngs::StdRng;
use tracing::info;

use crate::common::value_generator::generate_value;
use crate::common::{FuzzerDataType, LogicalColumn, LogicalTable, get_available_data_types};
use crate::{common::rng::rng_from_seed, fuzz_context::GlobalContext};

pub struct DatasetGenerator {
    rng: StdRng,
    ctx: Arc<GlobalContext>,
}

impl DatasetGenerator {
    pub fn new(seed: u64, context: Arc<GlobalContext>) -> Self {
        Self {
            rng: rng_from_seed(seed),
            ctx: context,
        }
    }

    // TODO(coverage): support NULLs in data generation
    pub async fn generate_dataset(&mut self) -> Result<LogicalTable> {
        // ==== Generate schema ====
        let table_name = self.ctx.runtime_context.next_table_name(); // t1, t2, ...
        let cfg_max_col_count = self.ctx.runner_config.max_column_count;

        let num_columns = self.rng.random_range(1..=cfg_max_col_count);
        let mut column_definitions = Vec::new();
        let mut column_fuzzer_types = Vec::new();
        let available_types = get_available_data_types();

        // Generate column definitions
        for i in 0..num_columns {
            let fuzzer_column_type =
                &available_types[self.rng.random_range(0..available_types.len())];
            let column_name = format!(
                "col_{table_name}_{}_{}",
                i + 1,
                fuzzer_column_type.display_name()
            );
            let sql_type = fuzzer_column_type.to_sql_type();
            // TODO(coverage): enforce `NOT NULL` somethimes.
            column_definitions.push(format!("{} {}", column_name, sql_type));
            column_fuzzer_types.push(fuzzer_column_type.clone());
        }

        // Generate CREATE TABLE SQL
        let create_table_sql = format!(
            "CREATE TABLE {} (\n    {}\n);",
            table_name,
            column_definitions.join(",\n    ")
        );

        // Log the CREATE TABLE statement
        info!("Executing CREATE TABLE SQL: {}", create_table_sql);

        // ==== Generate data and INSERT statements ====
        let cfg_max_row_count = self.ctx.runner_config.max_row_count;
        let cfg_max_insert_per_table = self.ctx.runner_config.max_insert_per_table;
        let actual_row_count = self.rng.random_range(0..cfg_max_row_count);
        let num_insert_statements =
            std::cmp::min(actual_row_count, cfg_max_insert_per_table as u64);

        let mut insert_statements = Vec::new();
        for _ in 0..num_insert_statements {
            let mut values = Vec::new();
            for fuzzer_type in &column_fuzzer_types {
                let value = self.generate_sql_value(fuzzer_type);
                values.push(value);
            }
            let insert_sql = format!("INSERT INTO {} VALUES ({});", table_name, values.join(", "));
            insert_statements.push(insert_sql);
        }

        // Log the INSERT statements
        for insert_sql in &insert_statements {
            info!("Executing INSERT SQL: {}", insert_sql);
        }

        // ==== Execute SQL statements ====
        let df_ctx = self.ctx.runtime_context.get_session_context();

        // Execute CREATE TABLE
        let create_result = df_ctx.sql(&create_table_sql).await?.collect().await;

        if let Err(e) = create_result {
            return Err(datafusion::error::DataFusionError::External(
                format!("Failed to create table {}: {}", table_name, e).into(),
            ));
        }

        // Execute INSERT statements
        for insert_sql in &insert_statements {
            let insert_result = df_ctx.sql(insert_sql).await?.collect().await;

            if let Err(e) = insert_result {
                return Err(datafusion::error::DataFusionError::External(
                    format!("Failed to insert data into {}: {}", table_name, e).into(),
                ));
            }
        }

        // ==== Register table in fuzzer context ====
        let logical_columns: Vec<LogicalColumn> = column_fuzzer_types
            .iter()
            .enumerate()
            .map(|(i, fuzzer_type)| {
                let column_name =
                    format!("col_{table_name}_{}_{}", i + 1, fuzzer_type.display_name());
                LogicalColumn {
                    name: column_name,
                    data_type: fuzzer_type.clone(),
                }
            })
            .collect();

        let logical_table = LogicalTable::with_columns(table_name.clone(), logical_columns);
        self.ctx
            .runtime_context
            .registered_tables
            .write()
            .unwrap()
            .insert(table_name, Arc::new(logical_table.clone()));

        Ok(logical_table)
    }

    // TODO(coverage): Now we only use simple values to prevent overflow.
    // switch to full range with edge cases like min/max for more coverage.
    fn generate_sql_value(&mut self, fuzzer_type: &FuzzerDataType) -> String {
        let value = generate_value(
            &mut self.rng,
            fuzzer_type,
            &self.ctx.runtime_context.value_generation_config,
        );
        value.to_sql_string()
    }
}
