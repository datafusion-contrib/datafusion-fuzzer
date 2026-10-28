<!--
SPDX-License-Identifier: Apache-2.0
SPDX-FileCopyrightText: See the NOTICE file distributed with this work for additional information regarding copyright ownership
SPDX-FileContributor: Licensed to the Apache Software Foundation (ASF) under one or more contributor license agreements
-->

# Oracle System

The Oracle system provides a framework for testing query consistency and correctness in DataFusion. It supports both traditional query equivalence testing and configuration consistency testing.

## Overview

The Oracle trait defines a contract for implementing test oracles that can:

1. The same query should return identical results under different DataFusion configurations
2. Different equivalent queries should return consistent results under the same configuration
3. Complex consistency checks involving both query variations and configuration variations

## Key Types

### `QueryContext`

Represents a SQL query paired with a specific DataFusion session context:

```rust
pub struct QueryContext {
    pub query: String,
    pub context: Arc<SessionContext>,
    pub context_description: Option<String>,
}
```

The Oracle trait now works with `Vec<QueryContext>` directly, keeping the API simple and straightforward.

## Usage Examples

### 1. Configuration Consistency Testing

Test the same query across different DataFusion configurations:

```rust
use std::sync::Arc;
use datafusion::prelude::SessionContext;
use datafusion::config::ConfigOptions;
use crate::oracle::QueryContext;

// Create different contexts
let default_ctx = Arc::new(SessionContext::new());
let mut config = ConfigOptions::new();
config.execution.batch_size = 1024;
let optimized_ctx = Arc::new(SessionContext::new_with_config(config.into()));

// Create query-context group
let query = "SELECT COUNT(*) FROM test_table WHERE value > 100".to_string();
let contexts = vec![
    (default_ctx, Some("Default Config".to_string())),
    (optimized_ctx, Some("Optimized Config".to_string())),
];

let group = QueryContext::from_single_query_multiple_contexts(query, contexts);
```

### 2. Backward Compatibility

Convert from the old Vec<String> API:

```rust
let queries = vec![
    "SELECT * FROM t1 WHERE x > 0".to_string(),
    "SELECT * FROM t1 WHERE x > 0 AND TRUE".to_string(),
];
let context = Arc::new(SessionContext::new());
let group = QueryContext::from_queries(queries, context);
```

### 3. Manual Construction

Build query-context pairs manually:

```rust
let mut group = Vec::new();

// Add first query with default context
group.push(QueryContext::with_description(
    "SELECT sum(a) FROM t1".to_string(),
    default_ctx,
    "Sum with default config".to_string(),
));

// Add equivalent query with optimized context
group.push(QueryContext::with_description(
    "SELECT sum(a) FROM t1 GROUP BY ()".to_string(),
    optimized_ctx,
    "Sum with optimized config".to_string(),
));
```

## Implementing an Oracle

```rust
use crate::oracle::{Oracle, QueryContext};
use crate::common::Result;

pub struct MyOracle {
    // Your oracle state
}

#[async_trait::async_trait]
impl Oracle for MyOracle {
    fn name(&self) -> &'static str {
        "MyOracle"
    }
    
    fn generate_query_group(&mut self) -> Result<Vec<QueryContext>> {
        let mut group = Vec::new();
        
        // Generate your query-context pairs
        // This could involve:
        // - Same query with different configs
        // - Different equivalent queries
        // - Complex combinations
        
        Ok(group)
    }
    
    async fn validate_consistency(&self, query_group: &[QueryContext]) -> Result<()> {
        // Execute each query with its specific context
        for entry in query_group {
            let results = entry.context.sql(&entry.query).await?;
            // Compare results...
        }
        
        Ok(())
    }
    
    fn create_error_report(&self, query_group: &[QueryContext]) -> Result<String> {
        // Generate detailed error report
        let mut report = String::new();
        for (i, entry) in query_group.iter().enumerate() {
            report.push_str(&format!("{}. {}\n", i + 1, entry.display_description()));
        }
        Ok(report)
    }
}
```

## Helper Functions

The `QueryContext` type provides several helpful static methods:

```rust
// Convert from old Vec<String> API
let queries = vec!["SELECT 1".to_string(), "SELECT 2".to_string()];
let context = Arc::new(SessionContext::new());
let group = QueryContext::from_queries(queries, context);

// Test same query with multiple contexts
let query = "SELECT COUNT(*)".to_string();
let contexts = vec![
    (ctx1, Some("Config 1".to_string())),
    (ctx2, Some("Config 2".to_string())),
];
let group = QueryContext::from_single_query_multiple_contexts(query, contexts);

// Extract queries for backward compatibility
let queries = QueryContext::get_queries(&group);
```

## Migration Guide

If you have existing oracle implementations using `Vec<String>`, you can migrate them easily:

### Before (Old API)
```rust
fn generate_query_group(&mut self) -> Result<Vec<String>> {
    Ok(vec!["SELECT 1".to_string(), "SELECT 1+0".to_string()])
}

fn validate_consistency(&self, query_group: &Vec<String>) -> Result<()> {
    // validation logic
}
```

### After (New API)
```rust
#[async_trait::async_trait]
impl Oracle for MyOracle {
    fn generate_query_group(&mut self) -> Result<Vec<QueryContext>> {
        let queries = vec!["SELECT 1".to_string(), "SELECT 1+0".to_string()];
        let context = Arc::new(SessionContext::new());
        Ok(QueryContext::from_queries(queries, context))
    }

    async fn validate_consistency(&self, query_group: &[QueryContext]) -> Result<()> {
        // Now you have access to both queries and their contexts
        for entry in query_group {
            let query = &entry.query;
            let context = &entry.context;
            // validation logic
        }
        Ok(())
    }
}
```

## Benefits

1. **Simplified API**: Direct use of `Vec<QueryContext>` removes unnecessary abstraction
2. **Configuration Testing**: Test that query results are consistent across different DataFusion configurations
3. **Better Debugging**: Context descriptions help identify which configuration caused issues
4. **Flexibility**: Support both traditional query equivalence testing and configuration consistency testing
5. **Backward Compatibility**: Existing oracles can be migrated with minimal changes
6. **Rich Error Reports**: More detailed error reports with context information 