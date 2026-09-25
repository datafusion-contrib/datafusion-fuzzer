use crate::common::FuzzerDataType;
use rand::Rng;
use rand::rngs::StdRng;
use std::sync::Arc;

/// Raw value representation for generated data
#[derive(Debug, Clone)]
pub enum GeneratedValue {
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    Float32(f32),
    Float64(f64),
    Boolean(bool),
    Decimal {
        value: i128,
        precision: u8,
        scale: i8,
    },
    Date32(i32),                    // Days since Unix epoch (1970-01-01)
    Time64Nanosecond(i64),          // Nanoseconds since midnight
    Timestamp(i64, Option<String>), // Nanoseconds since Unix epoch (1970-01-01 00:00:00 UTC) with optional timezone
    IntervalMonthDayNano(i128),     // MonthDayNano interval as i128 (months, days, nanoseconds)
    String(String),                 // String value
    Int8Array(Vec<i8>),
    Int32Array(Vec<i32>), // List of Int32 values
    Int64Array(Vec<i64>),
    Null,
}

/// Configuration for value generation
#[derive(Debug, Clone, PartialEq)]
pub struct ValueGenerationConfig {
    pub nullable: bool,
    pub null_probability: f64,
    pub int_range: (i32, i32),
    pub uint_range: (u32, u32),
    pub float_range: (f64, f64),
}

impl Default for ValueGenerationConfig {
    fn default() -> Self {
        Self {
            nullable: true,
            null_probability: 0.1,
            int_range: (-100, 100),
            uint_range: (0, 200),
            float_range: (-100.0, 100.0),
        }
    }
}

/// Core value generation logic shared by both functions
pub fn generate_value(
    rng: &mut StdRng,
    fuzzer_type: &FuzzerDataType,
    config: &ValueGenerationConfig,
) -> GeneratedValue {
    // Handle null generation
    if config.nullable && rng.random_bool(config.null_probability) {
        return GeneratedValue::Null;
    }

    match fuzzer_type {
        FuzzerDataType::Int8 => {
            let (start, end) = clamp_signed_range(config.int_range, i8::MIN as i32, i8::MAX as i32);
            let value = rng.random_range(start..=end) as i8;
            GeneratedValue::Int8(value)
        }
        FuzzerDataType::Int16 => {
            let (start, end) =
                clamp_signed_range(config.int_range, i16::MIN as i32, i16::MAX as i32);
            let value = rng.random_range(start..=end) as i16;
            GeneratedValue::Int16(value)
        }
        FuzzerDataType::Int32 => {
            let value = rng.random_range(config.int_range.0..=config.int_range.1);
            GeneratedValue::Int32(value)
        }
        FuzzerDataType::Int64 => {
            let value = rng.random_range(config.int_range.0 as i64..=config.int_range.1 as i64);
            GeneratedValue::Int64(value)
        }
        FuzzerDataType::UInt8 => {
            let (start, end) = clamp_unsigned_range(config.uint_range, u8::MAX as u32);

            let value =
                u8::try_from(rng.random_range(start..=end)).expect("clamped value must fit in u8");

            GeneratedValue::UInt8(value)
        }
        FuzzerDataType::UInt16 => {
            let (start, end) = clamp_unsigned_range(config.uint_range, u16::MAX as u32);

            let value = u16::try_from(rng.random_range(start..=end))
                .expect("clamped value must fit in u16");

            GeneratedValue::UInt16(value)
        }
        FuzzerDataType::UInt32 => {
            let value = rng.random_range(config.uint_range.0..=config.uint_range.1);
            GeneratedValue::UInt32(value)
        }
        FuzzerDataType::UInt64 => {
            let value = rng.random_range(config.uint_range.0 as u64..=config.uint_range.1 as u64);
            GeneratedValue::UInt64(value)
        }
        FuzzerDataType::Float32 => {
            let value = rng.random_range(config.float_range.0 as f32..=config.float_range.1 as f32);
            GeneratedValue::Float32(value)
        }
        FuzzerDataType::Float64 => {
            let value = rng.random_range(config.float_range.0..=config.float_range.1);
            GeneratedValue::Float64(value)
        }
        FuzzerDataType::Boolean => {
            let value = rng.random_bool(0.5);
            GeneratedValue::Boolean(value)
        }
        FuzzerDataType::Decimal => {
            // Generate random precision and scale for decimal values
            // Precision: 1-76 (maximum for Decimal256, DataFusion auto-chooses implementation)
            let precision = rng.random_range(1..=76);
            // Scale: 0 to precision (can't exceed precision)
            let scale = rng.random_range(0..=precision as i8);

            // Use the existing safe decimal generation logic
            let simple_value = rng.random_range(-99999..=99999);
            let scale_factor = safe_power_of_10(scale);
            let decimal_value = simple_value * scale_factor;

            GeneratedValue::Decimal {
                value: decimal_value,
                precision,
                scale,
            }
        }
        FuzzerDataType::Int8Array => {
            let (start, end) = clamp_signed_range(config.int_range, i8::MIN as i32, i8::MAX as i32);
            let length = rng.random_range(1..=5);
            let values = (0..length)
                .map(|_| rng.random_range(start..=end) as i8)
                .collect();
            GeneratedValue::Int8Array(values)
        }
        FuzzerDataType::Int32Array => {
            // Short non-empty arrays: empty literals like `[]` have an untyped
            // element type and fail to cast, so always generate 1..=5 elements
            let length = rng.random_range(1..=5);
            let values = (0..length)
                .map(|_| rng.random_range(config.int_range.0..=config.int_range.1))
                .collect();
            GeneratedValue::Int32Array(values)
        }
        FuzzerDataType::Int64Array => {
            let length = rng.random_range(1..=5);
            let values = (0..length)
                .map(|_| rng.random_range(config.int_range.0 as i64..=config.int_range.1 as i64))
                .collect();
            GeneratedValue::Int64Array(values)
        }
        FuzzerDataType::Date32 => {
            // Generate a reasonable range of dates:
            // - Days 0-36500 covers approximately 100 years from 1970-01-01
            // - This gives us dates from 1970-01-01 to roughly 2070
            let days_since_epoch = rng.random_range(0..=36500);
            GeneratedValue::Date32(days_since_epoch)
        }
        FuzzerDataType::Time64Nanosecond => {
            // Generate a reasonable range of times in nanoseconds since midnight:
            // - 0 to 86,399,999,999,999 nanoseconds (24 hours in nanoseconds)
            // - This covers the full range of a day from 00:00:00 to 23:59:59.999999999
            let nanoseconds_per_day = 24 * 60 * 60 * 1_000_000_000i64; // 24 hours in nanoseconds
            let nanoseconds_since_midnight = rng.random_range(0..nanoseconds_per_day);
            GeneratedValue::Time64Nanosecond(nanoseconds_since_midnight)
        }
        FuzzerDataType::Timestamp => {
            // Generate a reasonable range of timestamps in nanoseconds since Unix epoch:
            // - Start: 0 (1970-01-01 00:00:00 UTC)
            // - End: approximately 100 years of nanoseconds from epoch
            // - This gives us timestamps from 1970-01-01 to roughly 2070
            let nanoseconds_per_day = 24 * 60 * 60 * 1_000_000_000i64;
            let days_in_100_years = 36500i64; // Approximate
            let max_nanoseconds = nanoseconds_per_day * days_in_100_years;
            let nanoseconds_since_epoch = rng.random_range(0..=max_nanoseconds);

            // Randomly decide whether to add a timezone
            let timezones = vec![
                None,
                Some("UTC".to_string()),
                Some("+09:00".to_string()),
                Some("-09".to_string()),
                Some("+0930".to_string()),
                Some("America/New_York".to_string()),
            ];
            let timezone_index = rng.random_range(0..timezones.len());
            let final_tz = timezones[timezone_index].clone();

            GeneratedValue::Timestamp(nanoseconds_since_epoch, final_tz)
        }
        FuzzerDataType::IntervalMonthDayNano => {
            // Generate a reasonable interval with month, day, and nanosecond components
            // MonthDayNano interval is stored as i128 with:
            // - bits 0-31: days
            // - bits 32-63: months
            // - bits 64-127: nanoseconds
            let months = rng.random_range(-12..=12); // Reasonable month range
            let days = rng.random_range(-30..=30); // Reasonable day range
            let nanoseconds = rng.random_range(-1_000_000_000..=1_000_000_000); // Reasonable nanosecond range

            // Pack into i128: (months << 32) | (days & 0xFFFFFFFF)
            let interval_value = ((months as i128) << 32)
                | ((days as i128) & 0xFFFFFFFF)
                | ((nanoseconds as i128) << 64);
            GeneratedValue::IntervalMonthDayNano(interval_value)
        }
        FuzzerDataType::String => {
            // Generate a random string with reasonable length
            let length = rng.random_range(1..=50); // 1 to 50 characters
            let chars: Vec<char> = (0..length)
                .map(|_| {
                    // Generate printable ASCII characters, excluding problematic SQL characters
                    // Exclude: single quote (39), backslash (92), and other SQL-unsafe characters
                    let safe_chars = [
                        32, 33, 34, 35, 36, 37, 38, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51,
                        52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70,
                        71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89,
                        90, 91, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107,
                        108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122,
                        123, 124, 125, 126,
                    ];
                    let char_code = safe_chars[rng.random_range(0..safe_chars.len())];
                    char::from_u32(char_code).unwrap_or(' ')
                })
                .collect();
            let string_value = chars.into_iter().collect::<String>();
            GeneratedValue::String(string_value)
        }
    }
}

impl GeneratedValue {
    /// Convert to SQL string representation
    pub fn to_sql_string(&self) -> String {
        match self {
            GeneratedValue::Int8(v) => v.to_string(),
            GeneratedValue::Int16(v) => v.to_string(),
            GeneratedValue::Int32(v) => v.to_string(),
            GeneratedValue::Int64(v) => v.to_string(),
            GeneratedValue::UInt8(v) => v.to_string(),
            GeneratedValue::UInt16(v) => v.to_string(),
            GeneratedValue::UInt32(v) => v.to_string(),
            GeneratedValue::UInt64(v) => v.to_string(),
            GeneratedValue::Float32(v) => v.to_string(),
            GeneratedValue::Float64(v) => v.to_string(),
            GeneratedValue::Boolean(v) => if *v { "TRUE" } else { "FALSE" }.to_string(),
            GeneratedValue::Decimal {
                value,
                precision: _,
                scale,
            } => {
                // Format decimal with proper scale
                if *scale > 0 {
                    let scale_factor = safe_power_of_10(*scale);
                    let integer_part = value / scale_factor;
                    let fractional_part = (value % scale_factor).abs();
                    format!(
                        "{}.{:0width$}",
                        integer_part,
                        fractional_part,
                        width = *scale as usize
                    )
                } else {
                    value.to_string()
                }
            }
            GeneratedValue::Date32(days_since_epoch) => {
                // Convert days since Unix epoch to SQL date format
                // Unix epoch is 1970-01-01
                days_to_date_string(*days_since_epoch)
            }
            GeneratedValue::Time64Nanosecond(nanoseconds) => {
                // Convert nanoseconds since midnight to SQL time format (HH:MM:SS.nnnnnnnnn)
                let ns = *nanoseconds;

                // Calculate hours, minutes, seconds, and nanoseconds
                let hours = ns / (60 * 60 * 1_000_000_000);
                let remaining_ns = ns % (60 * 60 * 1_000_000_000);
                let minutes = remaining_ns / (60 * 1_000_000_000);
                let remaining_ns = remaining_ns % (60 * 1_000_000_000);
                let seconds = remaining_ns / 1_000_000_000;
                let nanoseconds = remaining_ns % 1_000_000_000;

                // Format as SQL time literal with nanosecond precision
                format!(
                    "'{:02}:{:02}:{:02}.{:09}'",
                    hours, minutes, seconds, nanoseconds
                )
            }
            GeneratedValue::Timestamp(nanoseconds_since_epoch, tz) => {
                // Convert nanoseconds since Unix epoch to SQL timestamp format
                let timestamp_str = nanoseconds_to_timestamp_string(*nanoseconds_since_epoch);
                if let Some(tz) = tz {
                    // Remove the closing quote and add timezone, then add closing quote
                    let timestamp_without_quote = &timestamp_str[..timestamp_str.len() - 1];
                    format!("{}{}'", timestamp_without_quote, tz)
                } else {
                    timestamp_str
                }
            }
            GeneratedValue::IntervalMonthDayNano(interval_value) => {
                // Convert i128 interval value to SQL INTERVAL format
                // Extract components: months, days, nanoseconds
                let months = ((*interval_value >> 32) & 0xFFFFFFFF) as i32;
                let days = (*interval_value & 0xFFFFFFFF) as i32;
                let nanoseconds = ((*interval_value >> 64) & 0xFFFFFFFFFFFFFFFF) as i64;
                // Format as SQL INTERVAL literal
                // Example: INTERVAL '1 year 2 months 3 days 4 hours 5 minutes 6 seconds'
                let mut parts = Vec::new();

                if months != 0 {
                    let abs_months = months.abs();
                    let years = abs_months / 12;
                    let remaining_months = abs_months % 12;

                    if years > 0 {
                        parts.push(format!(
                            "{} year{}",
                            years,
                            if years == 1 { "" } else { "s" }
                        ));
                    }
                    if remaining_months > 0 {
                        parts.push(format!(
                            "{} month{}",
                            remaining_months,
                            if remaining_months == 1 { "" } else { "s" }
                        ));
                    }
                }

                if days != 0 {
                    parts.push(format!(
                        "{} day{}",
                        days.abs(),
                        if days.abs() == 1 { "" } else { "s" }
                    ));
                }

                if nanoseconds != 0 {
                    let abs_ns = nanoseconds.abs();
                    let hours = abs_ns / (60 * 60 * 1_000_000_000);
                    let remaining_ns = abs_ns % (60 * 60 * 1_000_000_000);
                    let minutes = remaining_ns / (60 * 1_000_000_000);
                    let remaining_ns = remaining_ns % (60 * 1_000_000_000);
                    let seconds = remaining_ns / 1_000_000_000;

                    if hours > 0 {
                        parts.push(format!(
                            "{} hour{}",
                            hours,
                            if hours == 1 { "" } else { "s" }
                        ));
                    }
                    if minutes > 0 {
                        parts.push(format!(
                            "{} minute{}",
                            minutes,
                            if minutes == 1 { "" } else { "s" }
                        ));
                    }
                    if seconds > 0 {
                        parts.push(format!(
                            "{} second{}",
                            seconds,
                            if seconds == 1 { "" } else { "s" }
                        ));
                    }
                }

                if parts.is_empty() {
                    "INTERVAL '0'".to_string()
                } else {
                    format!("INTERVAL '{}'", parts.join(" "))
                }
            }
            GeneratedValue::String(s) => {
                // Escape single quotes by doubling them (SQL standard)
                let escaped = s.replace("'", "''");
                format!("'{}'", escaped)
            }
            GeneratedValue::Int8Array(values) => {
                let elements: Vec<String> = values.iter().map(|v| v.to_string()).collect();
                format!("[{}]", elements.join(", "))
            }
            GeneratedValue::Int32Array(values) => {
                let elements: Vec<String> = values.iter().map(|v| v.to_string()).collect();
                format!("[{}]", elements.join(", "))
            }
            GeneratedValue::Int64Array(values) => {
                let elements: Vec<String> = values.iter().map(|v| v.to_string()).collect();
                format!("[{}]", elements.join(", "))
            }
            GeneratedValue::Null => "NULL".to_string(),
        }
    }

    /// Convert to DataFusion ScalarValue
    pub fn to_scalar_value(&self) -> datafusion::scalar::ScalarValue {
        use datafusion::scalar::ScalarValue;

        match self {
            GeneratedValue::Int8(v) => ScalarValue::Int8(Some(*v)),
            GeneratedValue::Int16(v) => ScalarValue::Int16(Some(*v)),
            GeneratedValue::Int32(v) => ScalarValue::Int32(Some(*v)),
            GeneratedValue::Int64(v) => ScalarValue::Int64(Some(*v)),
            GeneratedValue::UInt8(v) => ScalarValue::UInt8(Some(*v)),
            GeneratedValue::UInt16(v) => ScalarValue::UInt16(Some(*v)),
            GeneratedValue::UInt32(v) => ScalarValue::UInt32(Some(*v)),
            GeneratedValue::UInt64(v) => ScalarValue::UInt64(Some(*v)),
            GeneratedValue::Float32(v) => ScalarValue::Float32(Some(*v)),
            GeneratedValue::Float64(v) => ScalarValue::Float64(Some(*v)),
            GeneratedValue::Boolean(v) => ScalarValue::Boolean(Some(*v)),
            GeneratedValue::Decimal {
                value,
                precision,
                scale,
            } => {
                if *precision <= 38 {
                    ScalarValue::Decimal128(Some(*value), *precision, *scale)
                } else {
                    use datafusion::arrow::datatypes::i256;
                    let decimal_value_256 = i256::from_i128(*value);
                    ScalarValue::Decimal256(Some(decimal_value_256), *precision, *scale)
                }
            }
            GeneratedValue::Date32(v) => ScalarValue::Date32(Some(*v)),
            GeneratedValue::Time64Nanosecond(v) => ScalarValue::Time64Nanosecond(Some(*v)),
            GeneratedValue::Timestamp(v, tz) => {
                ScalarValue::TimestampNanosecond(Some(*v), tz.as_deref().map(Arc::from))
            }
            GeneratedValue::IntervalMonthDayNano(v) => {
                use datafusion::arrow::datatypes::IntervalMonthDayNano;
                // Extract components from i128: months, days, nanoseconds
                let months = ((*v >> 32) & 0xFFFFFFFF) as i32;
                let days = (*v & 0xFFFFFFFF) as i32;
                let nanoseconds = ((*v >> 64) & 0xFFFFFFFFFFFFFFFF) as i64;

                // Create IntervalMonthDayNano value
                let interval_value = IntervalMonthDayNano::new(months, days, nanoseconds);
                ScalarValue::IntervalMonthDayNano(Some(interval_value))
            }
            GeneratedValue::String(s) => ScalarValue::Utf8(Some(s.clone())),
            GeneratedValue::Int8Array(values) => {
                use datafusion::arrow::array::ListArray;
                use datafusion::arrow::datatypes::Int8Type;
                let list = ListArray::from_iter_primitive::<Int8Type, _, _>(std::iter::once(Some(
                    values.iter().map(|v| Some(*v)).collect::<Vec<_>>(),
                )));
                ScalarValue::List(Arc::new(list))
            }
            GeneratedValue::Int32Array(values) => {
                use datafusion::arrow::array::ListArray;
                use datafusion::arrow::datatypes::Int32Type;
                let list = ListArray::from_iter_primitive::<Int32Type, _, _>(std::iter::once(
                    Some(values.iter().map(|v| Some(*v)).collect::<Vec<_>>()),
                ));
                ScalarValue::List(Arc::new(list))
            }
            GeneratedValue::Int64Array(values) => {
                use datafusion::arrow::array::ListArray;
                use datafusion::arrow::datatypes::Int64Type;
                let list = ListArray::from_iter_primitive::<Int64Type, _, _>(std::iter::once(
                    Some(values.iter().map(|v| Some(*v)).collect::<Vec<_>>()),
                ));
                ScalarValue::List(Arc::new(list))
            }
            GeneratedValue::Null => ScalarValue::Null,
        }
    }
}

// =================
// Utility functions
// =================

/// Clamps a configured signed integer range to the target type's valid range.
///
/// If the configured range does not overlap with the target range, both
/// bounds are clamped to the nearest target boundary.
fn clamp_signed_range(configured: (i32, i32), type_min: i32, type_max: i32) -> (i32, i32) {
    assert!(
        configured.0 <= configured.1,
        "configured integer range start must not exceed its end"
    );

    (
        configured.0.clamp(type_min, type_max),
        configured.1.clamp(type_min, type_max),
    )
}
/// Clamps a configured unsigned integer range to the target type's maximum.
///
/// The lower bound does not need explicit zero clamping because the configured
/// range uses `u32`. If the whole range exceeds `type_max`, both bounds become
/// `type_max`.
fn clamp_unsigned_range(configured: (u32, u32), type_max: u32) -> (u32, u32) {
    assert!(
        configured.0 <= configured.1,
        "configured unsigned integer range start must not exceed its end"
    );

    (configured.0.min(type_max), configured.1.min(type_max))
}
/// Safely calculate 10^scale, preventing overflow
pub fn safe_power_of_10(scale: i8) -> i128 {
    // The maximum power of 10 that fits in i128 is approximately 10^38
    // For safety, we limit to 10^30 to avoid overflow in calculations
    let safe_scale = std::cmp::min(scale as u32, 30);
    match safe_scale {
        0 => 1,
        1..=30 => 10_i128.pow(safe_scale),
        _ => 10_i128.pow(30), // Fallback to 10^30 for any edge cases
    }
}

/// Convert days since Unix epoch (1970-01-01) to a proper date string
/// This function properly handles leap years and varying month lengths
fn days_to_date_string(days_since_epoch: i32) -> String {
    // Days per month (non-leap year)
    const DAYS_IN_MONTH: [i32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    let mut days = days_since_epoch;
    let mut year = 1970;

    // Calculate year
    while days >= 365 {
        // Check if current year is a leap year
        let is_leap = (year % 4 == 0) && (year % 100 != 0 || year % 400 == 0);
        let days_in_year = if is_leap { 366 } else { 365 };

        if days >= days_in_year {
            days -= days_in_year;
            year += 1;
        } else {
            break;
        }
    }

    // Calculate month and day
    let is_leap = (year % 4 == 0) && (year % 100 != 0 || year % 400 == 0);
    let mut month = 1;
    let mut day = days + 1; // +1 because days are 0-indexed

    for (i, &days_in_month) in DAYS_IN_MONTH.iter().enumerate() {
        let adjusted_days = if i == 1 && is_leap {
            days_in_month + 1
        } else {
            days_in_month
        };

        if day <= adjusted_days {
            break;
        }
        day -= adjusted_days;
        month += 1;
    }

    format!("'{:04}-{:02}-{:02}'", year, month, day)
}

/// Convert nanoseconds since Unix epoch to a proper timestamp string
/// This function properly handles leap years and varying month lengths
fn nanoseconds_to_timestamp_string(nanoseconds_since_epoch: i64) -> String {
    // Days per month (non-leap year)
    const DAYS_IN_MONTH: [i32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    let nanoseconds_per_day = 24 * 60 * 60 * 1_000_000_000i64;
    let days_since_epoch = nanoseconds_since_epoch / nanoseconds_per_day;
    let remaining_ns = nanoseconds_since_epoch % nanoseconds_per_day;

    let mut days = days_since_epoch as i32;
    let mut year = 1970;

    // Calculate year
    while days >= 365 {
        // Check if current year is a leap year
        let is_leap = (year % 4 == 0) && (year % 100 != 0 || year % 400 == 0);
        let days_in_year = if is_leap { 366 } else { 365 };

        if days >= days_in_year {
            days -= days_in_year;
            year += 1;
        } else {
            break;
        }
    }

    // Calculate month and day
    let is_leap = (year % 4 == 0) && (year % 100 != 0 || year % 400 == 0);
    let mut month = 1;
    let mut day = days + 1; // +1 because days are 0-indexed

    for (i, &days_in_month) in DAYS_IN_MONTH.iter().enumerate() {
        let adjusted_days = if i == 1 && is_leap {
            days_in_month + 1
        } else {
            days_in_month
        };

        if day <= adjusted_days {
            break;
        }
        day -= adjusted_days;
        month += 1;
    }

    // Calculate time components
    let hours = remaining_ns / (60 * 60 * 1_000_000_000);
    let remaining_ns = remaining_ns % (60 * 60 * 1_000_000_000);
    let minutes = remaining_ns / (60 * 1_000_000_000);
    let remaining_ns = remaining_ns % (60 * 1_000_000_000);
    let seconds = remaining_ns / 1_000_000_000;
    let nanoseconds = remaining_ns % 1_000_000_000;

    format!(
        "'{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:09}'",
        year, month, day, hours, minutes, seconds, nanoseconds
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::rng::rng_from_seed;

    #[test]
    fn test_integer_value_generation_and_conversions() {
        use datafusion::scalar::ScalarValue;

        let mut rng = rng_from_seed(42);
        let config = ValueGenerationConfig {
            nullable: false,
            ..ValueGenerationConfig::default()
        };

        for _ in 0..100 {
            // Int8
            let int8 = generate_value(&mut rng, &FuzzerDataType::Int8, &config);
            match int8 {
                GeneratedValue::Int8(v) => {
                    assert!((-100..=100).contains(&v));
                    assert_eq!(int8.to_sql_string(), v.to_string());
                    assert_eq!(int8.to_scalar_value(), ScalarValue::Int8(Some(v)));
                }
                other => panic!("Expected Int8 value, got: {other:?}"),
            }

            // Int16
            let int16 = generate_value(&mut rng, &FuzzerDataType::Int16, &config);
            match int16 {
                GeneratedValue::Int16(v) => {
                    assert!((-100..=100).contains(&v));
                    assert_eq!(int16.to_sql_string(), v.to_string());
                    assert_eq!(int16.to_scalar_value(), ScalarValue::Int16(Some(v)));
                }
                other => panic!("Expected Int16 value, got: {other:?}"),
            }

            // UInt8
            let uint8 = generate_value(&mut rng, &FuzzerDataType::UInt8, &config);
            match uint8 {
                GeneratedValue::UInt8(v) => {
                    assert!((0..=200).contains(&v));
                    assert_eq!(uint8.to_sql_string(), v.to_string());
                    assert_eq!(uint8.to_scalar_value(), ScalarValue::UInt8(Some(v)));
                }
                other => panic!("Expected UInt8 value, got: {other:?}"),
            }

            // UInt16
            let uint16 = generate_value(&mut rng, &FuzzerDataType::UInt16, &config);
            match uint16 {
                GeneratedValue::UInt16(v) => {
                    assert!((0..=200).contains(&v));
                    assert_eq!(uint16.to_sql_string(), v.to_string());
                    assert_eq!(uint16.to_scalar_value(), ScalarValue::UInt16(Some(v)));
                }
                other => panic!("Expected UInt16 value, got: {other:?}"),
            }
        }
    }

    #[test]
    fn test_array_value_generation_and_conversions() {
        use datafusion::arrow::array::Array;
        use datafusion::arrow::datatypes::DataType;
        use datafusion::scalar::ScalarValue;

        let mut rng = rng_from_seed(42);
        let config = ValueGenerationConfig {
            nullable: false,
            ..ValueGenerationConfig::default()
        };
        let int_range = config.int_range.0 as i64..=config.int_range.1 as i64;

        let cases = [
            (FuzzerDataType::Int8Array, DataType::Int8),
            (FuzzerDataType::Int32Array, DataType::Int32),
            (FuzzerDataType::Int64Array, DataType::Int64),
        ];

        for (array_type, element_type) in cases {
            for _ in 0..100 {
                let value = generate_value(&mut rng, &array_type, &config);

                let elements: Vec<i64> = match &value {
                    GeneratedValue::Int8Array(values) => values.iter().map(|v| *v as i64).collect(),
                    GeneratedValue::Int32Array(values) => {
                        values.iter().map(|v| *v as i64).collect()
                    }
                    GeneratedValue::Int64Array(values) => values.clone(),
                    other => panic!("Expected array value, got: {other:?}"),
                };

                assert!((1..=5).contains(&elements.len()));
                assert!(elements.iter().all(|v| int_range.contains(v)));

                let expected_sql = format!(
                    "[{}]",
                    elements
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                assert_eq!(value.to_sql_string(), expected_sql);

                match value.to_scalar_value() {
                    ScalarValue::List(list) => {
                        assert_eq!(list.len(), 1);
                        assert_eq!(list.value(0).len(), elements.len());
                        assert_eq!(list.value(0).data_type(), &element_type);
                    }
                    other => panic!("Expected List scalar, got: {other:?}"),
                }
            }
        }
    }

    #[test]
    fn test_integer_ranges_outside_target_type_clamp_to_nearest_boundary() {
        assert_eq!(
            clamp_signed_range((200, 300), i8::MIN as i32, i8::MAX as i32),
            (i8::MAX as i32, i8::MAX as i32)
        );
        assert_eq!(
            clamp_signed_range((-300, -200), i8::MIN as i32, i8::MAX as i32),
            (i8::MIN as i32, i8::MIN as i32)
        );
        assert_eq!(
            clamp_unsigned_range((300, 400), u8::MAX as u32),
            (u8::MAX as u32, u8::MAX as u32)
        );
        assert_eq!(
            clamp_unsigned_range((70_000, 80_000), u16::MAX as u32),
            (u16::MAX as u32, u16::MAX as u32)
        );
    }

    #[test]
    #[should_panic(expected = "configured integer range start must not exceed its end")]
    fn test_clamp_signed_range_rejects_reversed_range() {
        clamp_signed_range((1, -1), i8::MIN as i32, i8::MAX as i32);
    }

    #[test]
    #[should_panic(expected = "configured unsigned integer range start must not exceed its end")]
    fn test_clamp_unsigned_range_rejects_reversed_range() {
        clamp_unsigned_range((1, 0), u8::MAX as u32);
    }

    #[test]
    fn test_cached_config_in_runtime_context() {
        // Test that RuntimeContext has the expected cached config
        use crate::fuzz_context::RuntimeContext;

        let runtime_ctx = RuntimeContext::default();
        let expected_config = ValueGenerationConfig::default(); // Nullable by default

        assert_eq!(runtime_ctx.value_generation_config, expected_config);
        assert!(runtime_ctx.value_generation_config.nullable); // Should be nullable
    }

    #[test]
    fn test_generate_value_with_cached_config() {
        // Test that generate_value works with cached configs from RuntimeContext
        use crate::fuzz_context::RuntimeContext;

        let mut rng = rng_from_seed(42);
        let fuzzer_type = FuzzerDataType::Int32;
        let runtime_ctx = RuntimeContext::default();

        let value = generate_value(&mut rng, &fuzzer_type, &runtime_ctx.value_generation_config);

        // Should generate a non-null Int32 value (since config is non-nullable by default)
        match value {
            GeneratedValue::Int32(v) => {
                assert!(v >= -100 && v <= 100, "Value should be in expected range");
            }
            _ => panic!("Expected Int32 value, got: {:?}", value),
        }
    }

    #[test]
    fn test_timestamp_generation() {
        // Test that Timestamp generation works correctly
        use crate::fuzz_context::RuntimeContext;

        let mut rng = rng_from_seed(42);
        let fuzzer_type = FuzzerDataType::Timestamp; // Use the unified Timestamp variant
        let runtime_ctx = RuntimeContext::default();

        let value = generate_value(&mut rng, &fuzzer_type, &runtime_ctx.value_generation_config);

        // Should generate a Timestamp value
        match value {
            GeneratedValue::Timestamp(v, _) => {
                assert!(v >= 0, "Timestamp should be non-negative");
                // Check that it's a reasonable timestamp (not too far in the future)
                let max_ns = 24 * 60 * 60 * 1_000_000_000i64 * 36500; // ~100 years
                assert!(v <= max_ns, "Timestamp should be within reasonable range");
            }
            _ => panic!("Expected Timestamp value, got: {:?}", value),
        }

        // Test SQL string generation
        let sql_string = value.to_sql_string();
        assert!(
            sql_string.starts_with("'"),
            "SQL string should start with quote"
        );
        assert!(
            sql_string.ends_with("'"),
            "SQL string should end with quote"
        );
        assert!(
            sql_string.contains("-"),
            "SQL string should contain date separators"
        );
        assert!(
            sql_string.contains(":"),
            "SQL string should contain time separators"
        );

        // Test DataFusion ScalarValue conversion
        let scalar_value = value.to_scalar_value();
        assert!(
            matches!(
                scalar_value,
                datafusion::scalar::ScalarValue::TimestampNanosecond(Some(_), _)
            ),
            "Should convert to Timestamp ScalarValue (timezone may or may not be present)"
        );
    }

    #[test]
    fn test_timestamp_type_conversions() {
        // Test that Timestamp type conversions work correctly
        let fuzzer_type = FuzzerDataType::Timestamp; // Use the unified Timestamp variant

        // Test conversion to DataFusion type
        let df_type = fuzzer_type.to_datafusion_type();
        assert!(
            matches!(
                df_type,
                datafusion::arrow::datatypes::DataType::Timestamp(
                    datafusion::arrow::datatypes::TimeUnit::Nanosecond,
                    None
                )
            ),
            "Should convert to Timestamp(Nanosecond, None)"
        );

        // Test round-trip conversion
        let back_to_fuzzer = FuzzerDataType::from_datafusion_type(&df_type);
        assert_eq!(
            back_to_fuzzer,
            Some(fuzzer_type.clone()),
            "Round-trip conversion should work"
        );

        // Test properties
        assert_eq!(fuzzer_type.display_name(), "timestamp");
        assert_eq!(fuzzer_type.to_sql_type(), "TIMESTAMP");
        assert!(fuzzer_type.is_time());
        assert!(!fuzzer_type.is_numeric());
    }

    #[test]
    fn test_timestamp_tz_generation() {
        // Test that TimestampTz generation works correctly
        use crate::fuzz_context::RuntimeContext;

        let mut rng = rng_from_seed(42);
        let fuzzer_type = FuzzerDataType::Timestamp;
        let runtime_ctx = RuntimeContext::default();

        let value = generate_value(&mut rng, &fuzzer_type, &runtime_ctx.value_generation_config);

        // Should generate a TimestampTz value
        match value {
            GeneratedValue::Timestamp(v, _) => {
                assert!(v >= 0, "Timestamp should be non-negative");
                // Check that it's a reasonable timestamp (not too far in the future)
                let max_ns = 24 * 60 * 60 * 1_000_000_000i64 * 36500; // ~100 years
                assert!(v <= max_ns, "Timestamp should be within reasonable range");
            }
            _ => panic!("Expected TimestampTz value, got: {:?}", value),
        }

        // Test SQL string generation
        let sql_string = value.to_sql_string();
        assert!(
            sql_string.starts_with("'"),
            "SQL string should start with quote"
        );
        assert!(
            sql_string.ends_with("'"),
            "SQL string should end with a single quote (timezone present)"
        );
        // Check that the timezone is present in the SQL string
        let timezones = [
            "UTC",
            "+08:00",
            "-05:00",
            "+02:00",
            "America/New_York",
            "Europe/London",
            "Asia/Tokyo",
            "Australia/Sydney",
        ];
        assert!(
            timezones.iter().any(|tz| sql_string.contains(tz)),
            "SQL string should contain a known timezone"
        );
        assert!(
            sql_string.contains("-"),
            "SQL string should contain date separators"
        );
        assert!(
            sql_string.contains(":"),
            "SQL string should contain time separators"
        );

        // Test DataFusion ScalarValue conversion
        let scalar_value = value.to_scalar_value();
        assert!(
            matches!(
                scalar_value,
                datafusion::scalar::ScalarValue::TimestampNanosecond(Some(_), _)
            ),
            "Should convert to Timestamp ScalarValue (timezone may or may not be present)"
        );
    }

    #[test]
    fn test_timestamp_tz_type_conversions() {
        // Test that TimestampTz type conversions work correctly
        let fuzzer_type = FuzzerDataType::Timestamp;

        // Test conversion to DataFusion type
        let df_type = fuzzer_type.to_datafusion_type();
        assert!(
            matches!(
                df_type,
                datafusion::arrow::datatypes::DataType::Timestamp(
                    datafusion::arrow::datatypes::TimeUnit::Nanosecond,
                    None
                )
            ),
            "Should convert to Timestamp(Nanosecond, None)"
        );

        // Test round-trip conversion
        let back_to_fuzzer = FuzzerDataType::from_datafusion_type(&df_type);
        assert_eq!(
            back_to_fuzzer,
            Some(fuzzer_type.clone()),
            "Round-trip conversion should work"
        );

        // Test properties
        assert_eq!(fuzzer_type.display_name(), "timestamp");
        assert_eq!(fuzzer_type.to_sql_type(), "TIMESTAMP");
        assert!(fuzzer_type.is_time());
        assert!(!fuzzer_type.is_numeric());
    }

    #[test]
    fn test_date_generation_validity() {
        // Test that date generation produces valid dates

        let mut rng = rng_from_seed(42);
        let fuzzer_type = FuzzerDataType::Date32;

        // Use non-nullable configuration for testing
        let config = ValueGenerationConfig {
            nullable: false,
            null_probability: 0.0,
            int_range: (-100, 100),
            uint_range: (0, 200),
            float_range: (-100.0, 100.0),
        };

        // Generate multiple dates and verify they are valid
        for _ in 0..100 {
            let value = generate_value(&mut rng, &fuzzer_type, &config);

            match value {
                GeneratedValue::Date32(_) => {
                    let sql_string = value.to_sql_string();
                    // Verify the format is correct: 'YYYY-MM-DD'
                    assert!(sql_string.starts_with("'"), "Date should start with quote");
                    assert!(sql_string.ends_with("'"), "Date should end with quote");

                    // Extract the date part (remove quotes)
                    let date_part = &sql_string[1..sql_string.len() - 1];
                    let parts: Vec<&str> = date_part.split('-').collect();
                    assert_eq!(parts.len(), 3, "Date should have 3 parts: year-month-day");

                    let year: i32 = parts[0].parse().expect("Year should be parseable");
                    let month: i32 = parts[1].parse().expect("Month should be parseable");
                    let day: i32 = parts[2].parse().expect("Day should be parseable");

                    // Verify valid ranges
                    assert!(year >= 1970, "Year should be >= 1970");
                    assert!(year <= 2070, "Year should be <= 2070");
                    assert!(month >= 1, "Month should be >= 1");
                    assert!(month <= 12, "Month should be <= 12");
                    assert!(day >= 1, "Day should be >= 1");
                    assert!(day <= 31, "Day should be <= 31");

                    // Verify month-specific day limits
                    let max_days = match month {
                        2 => 29, // Allow leap year February
                        4 | 6 | 9 | 11 => 30,
                        _ => 31,
                    };
                    assert!(
                        day <= max_days,
                        "Day {} is invalid for month {}",
                        day,
                        month
                    );
                }
                _ => panic!("Expected Date32 value, got: {:?}", value),
            }
        }
    }

    #[test]
    fn test_timestamp_generation_validity() {
        // Test that timestamp generation produces valid timestamps

        let mut rng = rng_from_seed(42);
        let fuzzer_type = FuzzerDataType::Timestamp; // Use the unified Timestamp variant

        // Use non-nullable configuration for testing
        let config = ValueGenerationConfig {
            nullable: false,
            null_probability: 0.0,
            int_range: (-100, 100),
            uint_range: (0, 200),
            float_range: (-100.0, 100.0),
        };

        // Generate multiple timestamps and verify they are valid
        for _ in 0..100 {
            let value = generate_value(&mut rng, &fuzzer_type, &config);

            match value {
                GeneratedValue::Timestamp(v, _) => {
                    let sql_string = value.to_sql_string();
                    // Verify the format is correct: 'YYYY-MM-DD HH:MM:SS.nnnnnnnnn'
                    assert!(
                        sql_string.starts_with("'"),
                        "Timestamp should start with quote"
                    );
                    assert!(sql_string.ends_with("'"), "Timestamp should end with quote");

                    // Extract the date-time part (remove quotes)
                    let datetime_part = &sql_string[1..sql_string.len() - 1];
                    let space_parts: Vec<&str> = datetime_part.split(' ').collect();
                    assert!(
                        space_parts.len() >= 2,
                        "Timestamp should have date and time parts"
                    );

                    let date_part = space_parts[0];
                    let time_part_with_tz = space_parts[1..].join(" "); // Join remaining parts in case timezone is present

                    // Extract time part without timezone
                    let time_part =
                        if time_part_with_tz.contains('+') || time_part_with_tz.contains('-') {
                            // Find the timezone separator (usually a space before timezone)
                            if let Some(tz_sep_pos) = time_part_with_tz.rfind(' ') {
                                &time_part_with_tz[..tz_sep_pos]
                            } else {
                                // If no space, timezone might be directly attached (e.g., +08:00)
                                // Find the first + or - that's not part of the time
                                let mut tz_start = None;
                                for (i, ch) in time_part_with_tz.chars().enumerate() {
                                    if (ch == '+' || ch == '-') && i > 8 {
                                        // After HH:MM:SS.nnnnnnnnn
                                        tz_start = Some(i);
                                        break;
                                    }
                                }
                                if let Some(start) = tz_start {
                                    &time_part_with_tz[..start]
                                } else {
                                    &time_part_with_tz
                                }
                            }
                        } else if time_part_with_tz.contains('/') {
                            // Handle timezone names like "Europe/London"
                            if let Some(tz_sep_pos) = time_part_with_tz.find(' ') {
                                &time_part_with_tz[..tz_sep_pos]
                            } else {
                                // Find the first / that's not part of the time
                                let mut tz_start = None;
                                for (i, ch) in time_part_with_tz.chars().enumerate() {
                                    if ch == '/' && i > 8 {
                                        // After HH:MM:SS.nnnnnnnnn
                                        tz_start = Some(i);
                                        break;
                                    }
                                }
                                if let Some(start) = tz_start {
                                    &time_part_with_tz[..start]
                                } else {
                                    &time_part_with_tz
                                }
                            }
                        } else {
                            &time_part_with_tz
                        };

                    // Parse date part
                    let date_parts: Vec<&str> = date_part.split('-').collect();
                    assert_eq!(
                        date_parts.len(),
                        3,
                        "Date should have 3 parts: year-month-day"
                    );

                    let year: i32 = date_parts[0].parse().expect("Year should be parseable");
                    let month: i32 = date_parts[1].parse().expect("Month should be parseable");
                    let day: i32 = date_parts[2].parse().expect("Day should be parseable");

                    // Verify valid date ranges
                    assert!(year >= 1970, "Year should be >= 1970");
                    assert!(year <= 2070, "Year should be <= 2070");
                    assert!(month >= 1, "Month should be >= 1");
                    assert!(month <= 12, "Month should be <= 12");
                    assert!(day >= 1, "Day should be >= 1");
                    assert!(day <= 31, "Day should be <= 31");

                    // Verify month-specific day limits
                    let max_days = match month {
                        2 => 29, // Allow leap year February
                        4 | 6 | 9 | 11 => 30,
                        _ => 31,
                    };
                    assert!(
                        day <= max_days,
                        "Day {} is invalid for month {}",
                        day,
                        month
                    );

                    // Parse time part
                    let time_parts: Vec<&str> = time_part.split(':').collect();
                    assert_eq!(
                        time_parts.len(),
                        3,
                        "Time should have 3 parts: hour:minute:second.nanosecond"
                    );

                    let hour: i32 = time_parts[0].parse().expect("Hour should be parseable");
                    let minute: i32 = time_parts[1].parse().expect("Minute should be parseable");
                    let second_part = time_parts[2];

                    // Verify valid time ranges
                    assert!(hour >= 0, "Hour should be >= 0");
                    assert!(hour <= 23, "Hour should be <= 23");
                    assert!(minute >= 0, "Minute should be >= 0");
                    assert!(minute <= 59, "Minute should be <= 59");

                    // Parse seconds and nanoseconds
                    let second_parts: Vec<&str> = second_part.split('.').collect();
                    assert_eq!(
                        second_parts.len(),
                        2,
                        "Second part should have seconds and nanoseconds"
                    );

                    let second: i32 = second_parts[0].parse().expect("Second should be parseable");
                    // Strip any trailing non-digit characters (timezone) from nanosecond part
                    let ns_digits: String = second_parts[1]
                        .chars()
                        .take_while(|c| c.is_ascii_digit())
                        .collect();
                    let nanosecond: i32 =
                        ns_digits.parse().expect("Nanosecond should be parseable");

                    assert!(second >= 0, "Second should be >= 0");
                    assert!(second <= 59, "Second should be <= 59");
                    assert!(nanosecond >= 0, "Nanosecond should be >= 0");
                    assert!(
                        nanosecond <= 999_999_999,
                        "Nanosecond should be <= 999999999"
                    );
                }
                _ => panic!("Expected Timestamp value, got: {:?}", value),
            }
        }
    }

    #[test]
    fn test_interval_month_day_nano_generation() {
        // Test that IntervalMonthDayNano generation works correctly
        use crate::fuzz_context::RuntimeContext;

        let mut rng = rng_from_seed(42);
        let fuzzer_type = FuzzerDataType::IntervalMonthDayNano;
        let runtime_ctx = RuntimeContext::default();

        let value = generate_value(&mut rng, &fuzzer_type, &runtime_ctx.value_generation_config);

        // Should generate an IntervalMonthDayNano value
        match value {
            GeneratedValue::IntervalMonthDayNano(v) => {
                // Extract components to verify they are reasonable
                let months = ((v >> 32) & 0xFFFFFFFF) as i32;
                let days = (v & 0xFFFFFFFF) as i32;
                let nanoseconds = ((v >> 64) & 0xFFFFFFFFFFFFFFFF) as i64;

                // Check that values are within reasonable ranges
                assert!(
                    months >= -12 && months <= 12,
                    "Months should be in reasonable range"
                );
                assert!(
                    days >= -30 && days <= 30,
                    "Days should be in reasonable range"
                );
                assert!(
                    nanoseconds >= -1_000_000_000 && nanoseconds <= 1_000_000_000,
                    "Nanoseconds should be in reasonable range"
                );
            }
            _ => panic!("Expected IntervalMonthDayNano value, got: {:?}", value),
        }

        // Test SQL string generation
        let sql_string = value.to_sql_string();
        assert!(
            sql_string.starts_with("INTERVAL '"),
            "SQL string should start with INTERVAL '"
        );
        assert!(
            sql_string.ends_with("'"),
            "SQL string should end with quote"
        );

        // Test DataFusion ScalarValue conversion
        let scalar_value = value.to_scalar_value();
        assert!(
            matches!(
                scalar_value,
                datafusion::scalar::ScalarValue::IntervalMonthDayNano(Some(_))
            ),
            "Should convert to IntervalMonthDayNano ScalarValue"
        );
    }

    #[test]
    fn test_interval_month_day_nano_type_conversions() {
        // Test that IntervalMonthDayNano type conversions work correctly
        let fuzzer_type = FuzzerDataType::IntervalMonthDayNano;

        // Test conversion to DataFusion type
        let df_type = fuzzer_type.to_datafusion_type();
        assert!(
            matches!(
                df_type,
                datafusion::arrow::datatypes::DataType::Interval(
                    datafusion::arrow::datatypes::IntervalUnit::MonthDayNano
                )
            ),
            "Should convert to Interval(MonthDayNano)"
        );

        // Test round-trip conversion
        let back_to_fuzzer = FuzzerDataType::from_datafusion_type(&df_type);
        assert_eq!(
            back_to_fuzzer,
            Some(fuzzer_type.clone()),
            "Round-trip conversion should work"
        );

        // Test properties
        assert_eq!(fuzzer_type.display_name(), "interval_month_day_nano");
        assert_eq!(fuzzer_type.to_sql_type(), "INTERVAL");
        assert!(fuzzer_type.is_time());
        assert!(!fuzzer_type.is_numeric());
    }

    #[test]
    fn test_string_generation_sql_safety() {
        // Test that string generation produces SQL-safe strings

        let mut rng = rng_from_seed(42);
        let fuzzer_type = FuzzerDataType::String;

        // Use non-nullable configuration for testing
        let config = ValueGenerationConfig {
            nullable: false,
            null_probability: 0.0,
            int_range: (-100, 100),
            uint_range: (0, 200),
            float_range: (-100.0, 100.0),
        };

        // Generate multiple strings and verify they are SQL-safe
        for _ in 0..100 {
            let value = generate_value(&mut rng, &fuzzer_type, &config);

            match &value {
                GeneratedValue::String(s) => {
                    // Check that the string doesn't contain problematic SQL characters
                    assert!(!s.contains('\''), "String should not contain single quotes");
                    assert!(!s.contains('\\'), "String should not contain backslashes");

                    // Test SQL string conversion
                    let sql_string = value.to_sql_string();
                    assert!(
                        sql_string.starts_with("'"),
                        "SQL string should start with quote"
                    );
                    assert!(
                        sql_string.ends_with("'"),
                        "SQL string should end with quote"
                    );

                    // Verify the SQL string is properly formatted
                    // Remove the outer quotes to get the content
                    let content = &sql_string[1..sql_string.len() - 1];
                    // The content should not contain unescaped single quotes
                    assert!(
                        !content.contains("'"),
                        "SQL string content should not contain unescaped single quotes"
                    );
                }
                _ => panic!("Expected String value, got: {:?}", value),
            }
        }
    }

    #[test]
    fn test_string_escaping() {
        // Test that strings with single quotes are properly escaped
        let test_string = "test'string";
        let value = GeneratedValue::String(test_string.to_string());
        let sql_string = value.to_sql_string();

        // Should be properly escaped: 'test''string'
        assert_eq!(sql_string, "'test''string'");

        // Test with multiple single quotes
        let test_string2 = "test'string'with'quotes";
        let value2 = GeneratedValue::String(test_string2.to_string());
        let sql_string2 = value2.to_sql_string();

        // Should be properly escaped: 'test''string''with''quotes'
        assert_eq!(sql_string2, "'test''string''with''quotes'");
    }
}
