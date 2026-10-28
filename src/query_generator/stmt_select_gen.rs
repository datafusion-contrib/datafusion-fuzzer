// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: See the NOTICE file distributed with this work for additional information regarding copyright ownership
// SPDX-FileContributor: Licensed to the Apache Software Foundation (ASF) under one or more contributor license agreements

use std::result::Result;
use std::sync::Arc;

use rand::rngs::StdRng;

use crate::{common::rng::rng_from_seed, fuzz_context::GlobalContext};

pub struct StatementGenerator {
    rng: StdRng,
    ctx: Arc<GlobalContext>,
}

impl StatementGenerator {
    pub fn new(seed: u64, context: Arc<GlobalContext>) -> Self {
        Self {
            rng: rng_from_seed(seed),
            ctx: context,
        }
    }

    pub fn generate_query(&mut self) -> Result<(), String> {
        todo!()
    }
}
