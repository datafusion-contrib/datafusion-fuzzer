// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: See the NOTICE file distributed with this work for additional information regarding copyright ownership
// SPDX-FileContributor: Licensed to the Apache Software Foundation (ASF) under one or more contributor license agreements

use datafusion::scalar::ScalarValue;
use rand::rngs::StdRng;
use std::sync::Arc;

use crate::common::FuzzerDataType;
use crate::common::value_generator::generate_value;
use crate::fuzz_context::GlobalContext;

// TODO(coverage): now only small numbers are geenrated to avoid overflows. Change to
// large edge cases (e.g. max/min values) in the future.
pub fn generate_scalar_literal(
    ctx: &Arc<GlobalContext>,
    rng: &mut StdRng,
    target_type: &FuzzerDataType,
) -> ScalarValue {
    let value = generate_value(
        rng,
        target_type,
        &ctx.runtime_context.value_generation_config,
    );
    value.to_scalar_value()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::rng::rng_from_seed;

    #[test]
    fn test_interval_scalar_literal_generation() {
        // Test that INTERVAL scalar literals can be generated
        let ctx = Arc::new(GlobalContext::default());
        let mut rng = rng_from_seed(42);
        let fuzzer_type = FuzzerDataType::IntervalMonthDayNano;

        let scalar_value = generate_scalar_literal(&ctx, &mut rng, &fuzzer_type);

        // Should generate a valid IntervalMonthDayNano ScalarValue
        assert!(
            matches!(scalar_value, ScalarValue::IntervalMonthDayNano(Some(_))),
            "Should generate IntervalMonthDayNano ScalarValue"
        );

        // Test that it can be converted to a string representation
        let string_repr = format!("{:?}", scalar_value);
        assert!(
            string_repr.contains("IntervalMonthDayNano"),
            "String representation should contain IntervalMonthDayNano"
        );
    }
}
