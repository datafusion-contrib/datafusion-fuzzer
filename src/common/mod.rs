use core::fmt;
use std::io;
use std::sync::OnceLock;

use datafusion::{arrow::datatypes::DataType, error::DataFusionError};

pub mod rng;
pub mod util;
pub mod value_generator;

// How to add a new data type:
// 1. Add enum variant and update all match statements
//    --> src/common/mod.rs
// 2. Add SQL value generation
//    --> src/datasource_generator/dataset_generator.rs
// 3. Add scalar literal generation
//    --> src/query_generator/expr_literal_gen.rs
// 4. Add new operators (if needed)
//    --> src/query_generator/expr_impl.rs
// 5. Add operator enum variants (if needed)
//    --> src/query_generator/expr_def.rs
// 6. Add expression building (if needed)
//    --> src/query_generator/expr_gen.rs
// 7. Add error patterns (if needed)
//    --> src/cli/error_whitelist.rs

// TODO(coverage): Support `Duration` time, which is not a standard SQL type,
// but supported in Arrow.

/// FuzzerDataType is a logical type, it won't include detail value like the
/// timezone string inside `Timestamp` type. Those details will be specified
/// inside `GeneartedValue` type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FuzzerDataType {
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float32,
    Float64,
    Boolean,
    // When precision is [1, 38], the physical type in DF is Decimal128.
    // When precision is [39, 76], the physical type in DF is Decimal256.
    Decimal,
    Date32,
    // Time64 with nanosecond precision, following DataFusion specification
    Time64Nanosecond,
    // Timestamp with nanosecond precision
    Timestamp,
    // Interval with month, day, and nanosecond components
    IntervalMonthDayNano,
    // String type for text data
    String,
    Int8Array,
    // List of Int32 values, the first (simplest) array type
    Int32Array,
    Int64Array,
}

impl FuzzerDataType {
    /// Convert fuzzer data type to DataFusion data type
    pub fn to_datafusion_type(&self) -> DataType {
        match self {
            FuzzerDataType::Int8 => DataType::Int8,
            FuzzerDataType::Int16 => DataType::Int16,
            FuzzerDataType::Int32 => DataType::Int32,
            FuzzerDataType::Int64 => DataType::Int64,
            FuzzerDataType::UInt8 => DataType::UInt8,
            FuzzerDataType::UInt16 => DataType::UInt16,
            FuzzerDataType::UInt32 => DataType::UInt32,
            FuzzerDataType::UInt64 => DataType::UInt64,
            FuzzerDataType::Float32 => DataType::Float32,
            FuzzerDataType::Float64 => DataType::Float64,
            FuzzerDataType::Boolean => DataType::Boolean,
            FuzzerDataType::Decimal => {
                // DataFusion automatically chooses the best internal representation:
                // - Decimal128 for precision 1-38
                // - Decimal256 for precision 39-76
                // Since we don't have precision/scale in the type definition,
                // we'll use a default Decimal type for schema definition
                // The actual precision/scale will be determined when generating values
                DataType::Decimal128(10, 2) // Default precision and scale
            }
            FuzzerDataType::Date32 => DataType::Date32,
            FuzzerDataType::Time64Nanosecond => {
                DataType::Time64(datafusion::arrow::datatypes::TimeUnit::Nanosecond)
            }
            FuzzerDataType::Timestamp => {
                DataType::Timestamp(datafusion::arrow::datatypes::TimeUnit::Nanosecond, None)
            }
            FuzzerDataType::IntervalMonthDayNano => {
                DataType::Interval(datafusion::arrow::datatypes::IntervalUnit::MonthDayNano)
            }
            FuzzerDataType::String => DataType::Utf8,
            FuzzerDataType::Int8Array => DataType::new_list(DataType::Int8, true),
            FuzzerDataType::Int32Array => DataType::new_list(DataType::Int32, true),
            FuzzerDataType::Int64Array => DataType::new_list(DataType::Int64, true),
        }
    }

    /// Convert DataFusion data type to fuzzer data type (if supported)
    pub fn from_datafusion_type(data_type: &DataType) -> Option<Self> {
        match data_type {
            DataType::Int8 => Some(FuzzerDataType::Int8),
            DataType::Int16 => Some(FuzzerDataType::Int16),
            DataType::Int32 => Some(FuzzerDataType::Int32),
            DataType::Int64 => Some(FuzzerDataType::Int64),
            DataType::UInt8 => Some(FuzzerDataType::UInt8),
            DataType::UInt16 => Some(FuzzerDataType::UInt16),
            DataType::UInt32 => Some(FuzzerDataType::UInt32),
            DataType::UInt64 => Some(FuzzerDataType::UInt64),
            DataType::Float32 => Some(FuzzerDataType::Float32),
            DataType::Float64 => Some(FuzzerDataType::Float64),
            DataType::Boolean => Some(FuzzerDataType::Boolean),
            // Handle both Decimal128 and Decimal256 as the same fuzzer type
            // DataFusion automatically chooses the appropriate internal representation
            DataType::Decimal128(_precision, _scale) => Some(FuzzerDataType::Decimal),
            DataType::Decimal256(_precision, _scale) => Some(FuzzerDataType::Decimal),
            DataType::Date32 => Some(FuzzerDataType::Date32),
            DataType::Time64(datafusion::arrow::datatypes::TimeUnit::Nanosecond) => {
                Some(FuzzerDataType::Time64Nanosecond)
            }
            DataType::Timestamp(datafusion::arrow::datatypes::TimeUnit::Nanosecond, _tz) => {
                Some(FuzzerDataType::Timestamp)
            }
            DataType::Interval(datafusion::arrow::datatypes::IntervalUnit::MonthDayNano) => {
                Some(FuzzerDataType::IntervalMonthDayNano)
            }
            DataType::Utf8 => Some(FuzzerDataType::String),
            DataType::List(field) if *field.data_type() == DataType::Int8 => {
                Some(FuzzerDataType::Int8Array)
            }
            DataType::List(field) if *field.data_type() == DataType::Int32 => {
                Some(FuzzerDataType::Int32Array)
            }
            DataType::List(field) if *field.data_type() == DataType::Int64 => {
                Some(FuzzerDataType::Int64Array)
            }
            _ => None,
        }
    }

    /// Get the display name for column naming
    pub fn display_name(&self) -> &'static str {
        match self {
            FuzzerDataType::Int8 => "int8",
            FuzzerDataType::Int16 => "int16",
            FuzzerDataType::Int32 => "int32",
            FuzzerDataType::Int64 => "int64",
            FuzzerDataType::UInt8 => "uint8",
            FuzzerDataType::UInt16 => "uint16",
            FuzzerDataType::UInt32 => "uint32",
            FuzzerDataType::UInt64 => "uint64",
            FuzzerDataType::Float32 => "float32",
            FuzzerDataType::Float64 => "float64",
            FuzzerDataType::Boolean => "boolean",
            FuzzerDataType::Decimal => "decimal128",
            FuzzerDataType::Date32 => "date32",
            FuzzerDataType::Time64Nanosecond => "time64_nanosecond",
            FuzzerDataType::Timestamp => "timestamp",
            FuzzerDataType::IntervalMonthDayNano => "interval_month_day_nano",
            FuzzerDataType::String => "string",
            FuzzerDataType::Int8Array => "int8_array",
            FuzzerDataType::Int32Array => "int32_array",
            FuzzerDataType::Int64Array => "int64_array",
        }
    }

    pub fn is_numeric(&self) -> bool {
        match self {
            FuzzerDataType::Int8
            | FuzzerDataType::Int16
            | FuzzerDataType::Int32
            | FuzzerDataType::Int64
            | FuzzerDataType::UInt8
            | FuzzerDataType::UInt16
            | FuzzerDataType::UInt32
            | FuzzerDataType::UInt64
            | FuzzerDataType::Float32
            | FuzzerDataType::Float64
            | FuzzerDataType::Decimal => true,
            FuzzerDataType::Boolean
            | FuzzerDataType::Date32
            | FuzzerDataType::Time64Nanosecond
            | FuzzerDataType::Timestamp
            | FuzzerDataType::IntervalMonthDayNano
            | FuzzerDataType::String
            | FuzzerDataType::Int8Array
            | FuzzerDataType::Int32Array
            | FuzzerDataType::Int64Array => false,
        }
    }

    pub fn is_time(&self) -> bool {
        match self {
            FuzzerDataType::Date32
            | FuzzerDataType::Time64Nanosecond
            | FuzzerDataType::Timestamp
            | FuzzerDataType::IntervalMonthDayNano => true,
            FuzzerDataType::Int8
            | FuzzerDataType::Int16
            | FuzzerDataType::Int32
            | FuzzerDataType::Int64
            | FuzzerDataType::UInt8
            | FuzzerDataType::UInt16
            | FuzzerDataType::UInt32
            | FuzzerDataType::UInt64
            | FuzzerDataType::Float32
            | FuzzerDataType::Float64
            | FuzzerDataType::Boolean
            | FuzzerDataType::Decimal
            | FuzzerDataType::String
            | FuzzerDataType::Int8Array
            | FuzzerDataType::Int32Array
            | FuzzerDataType::Int64Array => false,
        }
    }

    /// Create a random Decimal type
    /// Note: Precision and scale are now generated when creating values, not when defining the type
    pub fn random_decimal<R: rand::Rng>(_rng: &mut R) -> Self {
        FuzzerDataType::Decimal
    }

    /// Convert to SQL type string for CREATE TABLE statements
    pub fn to_sql_type(&self) -> &'static str {
        match self {
            FuzzerDataType::Int8 => "TINYINT",
            FuzzerDataType::Int16 => "SMALLINT",
            FuzzerDataType::Int32 => "INT",
            FuzzerDataType::Int64 => "BIGINT",
            FuzzerDataType::UInt8 => "TINYINT UNSIGNED",
            FuzzerDataType::UInt16 => "SMALLINT UNSIGNED",
            FuzzerDataType::UInt32 => "INT UNSIGNED",
            FuzzerDataType::UInt64 => "BIGINT UNSIGNED",
            FuzzerDataType::Float32 => "FLOAT",
            FuzzerDataType::Float64 => "DOUBLE",
            FuzzerDataType::Boolean => "BOOLEAN",
            FuzzerDataType::Decimal => {
                // Note: This is a simplified approach. In a real implementation,
                // you might want to cache these strings or use a more sophisticated approach
                // For now, we'll use a default DECIMAL type
                // The actual precision and scale will be determined when generating values
                "DECIMAL"
            }
            FuzzerDataType::Date32 => "DATE",
            FuzzerDataType::Time64Nanosecond => "TIME",
            FuzzerDataType::Timestamp => "TIMESTAMP",
            FuzzerDataType::IntervalMonthDayNano => "INTERVAL",
            FuzzerDataType::String => "VARCHAR",
            FuzzerDataType::Int8Array => "TINYINT[]",
            FuzzerDataType::Int32Array => "INT[]",
            FuzzerDataType::Int64Array => "BIGINT[]",
        }
    }
}

/// All available data types for the fuzzer
static AVAILABLE_DATA_TYPES: OnceLock<Vec<FuzzerDataType>> = OnceLock::new();

/// Initialize the available data types (called once)
// TODO(known-bug): Generate Decimal 256 after the upstream issue addressed
// https://github.com/apache/datafusion/issues/16689
// TODO(valid-rate): Since Decimal and TimestampTz type has many variants, they
// are more likely to be choosen. I want make them less likely to improve the
// probability of generating a valid query.
pub fn init_available_data_types() {
    AVAILABLE_DATA_TYPES.get_or_init(|| {
        vec![
            FuzzerDataType::Int8,
            FuzzerDataType::Int16,
            FuzzerDataType::Int32,
            FuzzerDataType::Int64,
            FuzzerDataType::UInt8,
            FuzzerDataType::UInt16,
            FuzzerDataType::UInt32,
            FuzzerDataType::UInt64,
            FuzzerDataType::Float32,
            FuzzerDataType::Float64,
            FuzzerDataType::Boolean,
            // Add decimal type for testing
            // Precision and scale will be generated when creating values
            FuzzerDataType::Decimal,
            // Note: Decimal256 types (precision > 38) currently cause casting issues in DataFusion
            // They will be re-enabled once the upstream casting bugs are fixed
            FuzzerDataType::Date32,
            FuzzerDataType::Time64Nanosecond,
            FuzzerDataType::Timestamp,
            FuzzerDataType::IntervalMonthDayNano,
            FuzzerDataType::String,
            FuzzerDataType::Int8Array,
            FuzzerDataType::Int32Array,
            FuzzerDataType::Int64Array,
        ]
    });
}

/// Get all available data types
pub fn get_available_data_types() -> &'static Vec<FuzzerDataType> {
    AVAILABLE_DATA_TYPES
        .get()
        .expect("Available data types not initialized. Call init_available_data_types() first.")
}

/// Get all numeric data types (excludes Boolean)
pub fn get_numeric_data_types() -> Vec<FuzzerDataType> {
    get_available_data_types()
        .iter()
        .filter(|data_type| data_type.is_numeric())
        .cloned()
        .collect()
}

/// Get all time data types (Date32, and future time types)
pub fn get_time_data_types() -> Vec<FuzzerDataType> {
    get_available_data_types()
        .iter()
        .filter(|data_type| data_type.is_time())
        .cloned()
        .collect()
}

#[derive(Debug, Clone)]
pub struct LogicalTable {
    pub name: String,
    pub columns: Vec<LogicalColumn>,
}

#[derive(Debug, Clone)]
pub struct LogicalColumn {
    pub name: String,
    pub data_type: FuzzerDataType,
}

#[derive(Debug, Clone)]
pub enum LogicalTableType {
    Table,
    View,
    Subquery(String),
}

impl LogicalTable {
    pub fn new(name: String) -> Self {
        Self {
            name,
            columns: Vec::new(),
        }
    }

    pub fn with_columns(name: String, columns: Vec<LogicalColumn>) -> Self {
        Self { name, columns }
    }
}

pub type Result<T = ()> = std::result::Result<T, FuzzerError>;

// ====
// Fuzzer Errors
// ====
#[derive(Debug)]
pub enum FuzzerError {
    FuzzerError(String),
    DataFusionError(DataFusionError),
    IoError(io::Error),
    // Add other error types as needed
}

impl std::error::Error for FuzzerError {}

impl fmt::Display for FuzzerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FuzzerError::FuzzerError(msg) => write!(f, "{}", msg),
            FuzzerError::DataFusionError(e) => write!(f, "DataFusion error: {}", e),
            FuzzerError::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

// From conversion to allow ? operator with DataFusionError
impl From<DataFusionError> for FuzzerError {
    fn from(error: DataFusionError) -> Self {
        FuzzerError::DataFusionError(error)
    }
}

// From conversion to allow ? operator with io::Error
impl From<io::Error> for FuzzerError {
    fn from(error: io::Error) -> Self {
        FuzzerError::IoError(error)
    }
}

// Helper functions to create FuzzerError easily
pub fn fuzzer_err(msg: &str) -> FuzzerError {
    FuzzerError::FuzzerError(msg.to_string())
}

// ====
// Configuration related
// ====

/// Controls whether a feature is always included, always excluded, or included with a certain probability.
///
/// - `Always(true)` means 100% chance to include the feature.
/// - `Maybe(0.2)` means 20% chance to include the feature.
///
/// Use [`should_enable`] to decide if the feature should be included.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InclusionConfig {
    Always(bool),
    Maybe(f64),
}

impl InclusionConfig {
    /// Decide if the feature should be included.
    /// - If `Always(true)`, returns true.
    /// - If `Always(false)`, returns false.
    /// - If `Probability(p)`, returns true with probability `p` using the provided RNG.
    ///
    /// # Panics
    /// Panics if `Probability` variant is used and `rng` is `None`.
    pub fn should_enable<R: rand::Rng>(&self, rng: Option<&mut R>) -> bool {
        match self {
            InclusionConfig::Always(val) => *val,
            InclusionConfig::Maybe(prob) => {
                let rng = rng.expect("RNG must be provided for probabilistic InclusionConfig");
                rng.random_bool(*prob)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::rng::rng_from_seed;

    #[test]
    fn test_int8_type_properties() {
        let int8_type = FuzzerDataType::Int8;

        assert_eq!(int8_type.to_datafusion_type(), DataType::Int8);
        assert_eq!(
            FuzzerDataType::from_datafusion_type(&DataType::Int8),
            Some(FuzzerDataType::Int8)
        );
        assert_eq!(int8_type.display_name(), "int8");
        assert_eq!(int8_type.to_sql_type(), "TINYINT");
        assert!(int8_type.is_numeric());
        assert!(!int8_type.is_time());
    }

    #[test]
    fn test_array_type_properties() {
        let cases = [
            (
                FuzzerDataType::Int8Array,
                DataType::Int8,
                "int8_array",
                "TINYINT[]",
            ),
            (
                FuzzerDataType::Int32Array,
                DataType::Int32,
                "int32_array",
                "INT[]",
            ),
            (
                FuzzerDataType::Int64Array,
                DataType::Int64,
                "int64_array",
                "BIGINT[]",
            ),
        ];

        for (array_type, element_type, display_name, sql_type) in cases {
            let list_type = DataType::new_list(element_type, true);

            assert_eq!(array_type.to_datafusion_type(), list_type);
            assert_eq!(
                FuzzerDataType::from_datafusion_type(&list_type),
                Some(array_type.clone())
            );
            assert_eq!(array_type.display_name(), display_name);
            assert_eq!(array_type.to_sql_type(), sql_type);
            assert!(!array_type.is_numeric());
            assert!(!array_type.is_time());
        }
    }

    /// End-to-end example: the exact SQL shapes the fuzzer generates for
    /// array types must be accepted by DataFusion (DDL, INSERT and SELECT).
    #[tokio::test]
    async fn test_array_types_end_to_end() {
        use datafusion::prelude::SessionContext;

        let array_types = [
            FuzzerDataType::Int8Array,
            FuzzerDataType::Int32Array,
            FuzzerDataType::Int64Array,
        ];

        for array_type in array_types {
            let ctx = SessionContext::new();
            ctx.sql(&format!(
                "CREATE TABLE arr_example (id INT, vals {})",
                array_type.to_sql_type()
            ))
            .await
            .unwrap()
            .collect()
            .await
            .unwrap();
            ctx.sql("INSERT INTO arr_example VALUES (1, [1, 2, 3]), (2, [-5]), (3, NULL)")
                .await
                .unwrap()
                .collect()
                .await
                .unwrap();

            let batches = ctx
                .sql("SELECT vals FROM arr_example")
                .await
                .unwrap()
                .collect()
                .await
                .unwrap();
            let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
            assert_eq!(total_rows, 3);
            assert_eq!(
                batches[0].schema().field(0).data_type(),
                &array_type.to_datafusion_type()
            );
        }
    }

    #[test]
    fn test_simplified_decimal_type() {
        // Test that the simplified Decimal type works correctly
        let decimal_type = FuzzerDataType::Decimal;

        // Test display name
        assert_eq!(decimal_type.display_name(), "decimal128");

        // Test is_numeric
        assert!(decimal_type.is_numeric());

        // Test is_time
        assert!(!decimal_type.is_time());

        // Test SQL type
        assert_eq!(decimal_type.to_sql_type(), "DECIMAL");

        // Test DataFusion type conversion
        let df_type = decimal_type.to_datafusion_type();
        match df_type {
            DataType::Decimal128(precision, scale) => {
                assert_eq!(precision, 10);
                assert_eq!(scale, 2);
            }
            _ => panic!("Expected Decimal128 type"),
        }
    }

    #[test]
    fn test_decimal_value_generation() {
        // Test that decimal values are generated with random precision and scale
        use crate::common::value_generator::{
            ValueGenerationConfig, generate_value, safe_power_of_10,
        };

        let mut rng = rng_from_seed(42);
        let mut config = ValueGenerationConfig::default();
        config.nullable = false;

        for _ in 0..10 {
            let value = generate_value(&mut rng, &FuzzerDataType::Decimal, &config);

            match value {
                crate::common::value_generator::GeneratedValue::Decimal {
                    value,
                    precision,
                    scale,
                } => {
                    // Check that precision and scale are within valid ranges
                    assert!(precision >= 1 && precision <= 76);
                    assert!(scale >= 0 && scale <= precision as i8);
                    assert!(
                        value >= -99999 * safe_power_of_10(scale)
                            && value <= 99999 * safe_power_of_10(scale)
                    );
                }
                other => panic!("Expected Decimal value, got: {:?}", other),
            }
        }
    }
    #[test]
    fn test_remaining_integer_type_properties() {
        let cases = [
            (FuzzerDataType::Int16, DataType::Int16, "int16", "SMALLINT"),
            (
                FuzzerDataType::UInt8,
                DataType::UInt8,
                "uint8",
                "TINYINT UNSIGNED",
            ),
            (
                FuzzerDataType::UInt16,
                DataType::UInt16,
                "uint16",
                "SMALLINT UNSIGNED",
            ),
        ];

        for (fuzzer_type, datafusion_type, display_name, sql_type) in cases {
            assert_eq!(fuzzer_type.to_datafusion_type(), datafusion_type);
            assert_eq!(
                FuzzerDataType::from_datafusion_type(&datafusion_type),
                Some(fuzzer_type.clone())
            );
            assert_eq!(fuzzer_type.display_name(), display_name);
            assert_eq!(fuzzer_type.to_sql_type(), sql_type);
            assert!(fuzzer_type.is_numeric());
            assert!(!fuzzer_type.is_time());
        }
    }
}
