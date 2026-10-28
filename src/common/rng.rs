// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: See the NOTICE file distributed with this work for additional information regarding copyright ownership
// SPDX-FileContributor: Licensed to the Apache Software Foundation (ASF) under one or more contributor license agreements

use rand::{SeedableRng, rngs::StdRng};

pub fn rng_from_seed(seed: u64) -> StdRng {
    StdRng::seed_from_u64(seed)
}
