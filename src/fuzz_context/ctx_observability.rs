// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: See the NOTICE file distributed with this work for additional information regarding copyright ownership
// SPDX-FileContributor: Licensed to the Apache Software Foundation (ASF) under one or more contributor license agreements

use std::sync::Arc;

use datafusion::{arrow::util::pretty::pretty_format_batches, error::Result};
use tracing::info;

use super::GlobalContext;

/// Display the contents of all registered tables, showing up to 3 rows each
pub async fn display_all_tables(ctx: Arc<GlobalContext>) -> Result<()> {
    let tables = ctx.runtime_context.registered_tables.read().unwrap();

    for (table_name, _) in tables.iter() {
        let sql = format!("SELECT * FROM {} LIMIT 3", table_name);
        let df_ctx = ctx.runtime_context.get_session_context();

        match df_ctx.sql(&sql).await {
            Ok(df) => match df.collect().await {
                Ok(batches) => match pretty_format_batches(&batches) {
                    Ok(formatted) => info!("\n=== Table: {} ===\n{}", table_name, formatted),
                    Err(e) => info!("Error formatting results: {}", e),
                },
                Err(e) => info!("Error collecting results: {}", e),
            },
            Err(e) => info!("Error executing query: {}", e),
        }
    }

    Ok(())
}
