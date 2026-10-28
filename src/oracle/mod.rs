// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: See the NOTICE file distributed with this work for additional information regarding copyright ownership
// SPDX-FileContributor: Licensed to the Apache Software Foundation (ASF) under one or more contributor license agreements

// Oracle module - provides testing oracles for query consistency and correctness

pub(crate) mod oracle_common;
pub mod oracle_impl_nested_queries;
pub mod oracle_impl_no_crash;
pub mod oracle_impl_tlp_having;
pub mod oracle_impl_tlp_where;
pub mod oracle_trait;
#[cfg(test)]
pub(crate) mod test_helpers;

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::fuzz_context::GlobalContext;

// Re-export main types and traits
pub use oracle_impl_nested_queries::NestedQueriesOracle;
pub use oracle_impl_no_crash::NoCrashOracle;
pub use oracle_impl_tlp_having::TlpHavingOracle;
pub use oracle_impl_tlp_where::TlpWhereOracle;
pub use oracle_trait::{Oracle, QueryContext, QueryExecutionResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfiguredOracle {
    #[serde(rename = "NoCrash", alias = "NoCrashOracle")]
    NoCrash,
    #[serde(rename = "NestedQueries", alias = "NestedQueriesOracle")]
    NestedQueries,
    #[serde(rename = "TlpWhere", alias = "TlpWhereOracle")]
    TlpWhere,
    #[serde(rename = "TlpHaving", alias = "TlpHavingOracle")]
    TlpHaving,
}

impl ConfiguredOracle {
    pub fn build(self, seed: u64, ctx: Arc<GlobalContext>) -> Box<dyn Oracle + Send> {
        match self {
            Self::NoCrash => Box::new(NoCrashOracle::new(seed, ctx)),
            Self::NestedQueries => Box::new(NestedQueriesOracle::new(seed, ctx)),
            Self::TlpWhere => Box::new(TlpWhereOracle::new(seed, ctx)),
            Self::TlpHaving => Box::new(TlpHavingOracle::new(seed, ctx)),
        }
    }
}
