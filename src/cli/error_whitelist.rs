use regex::Regex;
use std::sync::LazyLock;

/// Error pattern matching strategies
#[derive(Debug, Clone)]
pub enum ErrorPattern {
    /// Exact string match - checks if the error message contains this exact substring
    Contains(&'static str),
    /// Regex pattern match - checks if the error message matches this regex pattern
    RegexMatch(&'static str),
    /// Combined condition: query SQL contains a substring AND error contains a substring
    QueryAndErrorContains {
        query_sub: &'static str,
        error_sub: &'static str,
    },
}

/// Configuration for error whitelist patterns
///
/// This module provides flexible error pattern matching for the fuzzer.
/// You can whitelist errors using either exact string matching or regex patterns.
///
/// # Examples
///
/// ## Adding Exact String Patterns
/// ```rust
/// # use datafusion_fuzzer::cli::error_whitelist::ErrorPattern;
/// // This will match any error containing "Arrow error: Divide by zero error"
/// ErrorPattern::Contains("Arrow error: Divide by zero error");
/// ```
///
/// ## Adding Regex Patterns
/// ```rust
/// # use datafusion_fuzzer::cli::error_whitelist::ErrorPattern;
/// // This will match any division by zero error with flexible formatting
/// ErrorPattern::RegexMatch(r"(?i)divide\s*by\s*zero");
///
/// // This will match any Arrow error with specific error codes
/// ErrorPattern::RegexMatch(r"Arrow error: (Divide by zero|Invalid argument)");
///
/// // This will match memory-related errors
/// ErrorPattern::RegexMatch(r"(?i)(out of memory|memory.*exhausted|allocation.*failed)");
/// ```
///
/// ## Regex Pattern Tips
/// - Use `(?i)` at the start for case-insensitive matching
/// - Use `\s*` or `\s+` for flexible whitespace matching
/// - Use `.*` for wildcard matching
/// - Use `(option1|option2)` for alternative matching
/// - Use `\b` for word boundaries to avoid partial matches
///
/// # Performance Note
/// Regex patterns are compiled once and cached for performance.
/// Exact string patterns use simple substring matching and are faster.
static ERROR_PATTERNS: LazyLock<Vec<ErrorPattern>> = LazyLock::new(|| {
    vec![
        // =========================
        // False Positives
        // =========================

        // select 1 / 0;
        ErrorPattern::Contains("Arrow error: Divide by zero error"),
        // select Null * Null;
        ErrorPattern::RegexMatch(
            r"Error during planning: Cannot coerce arithmetic expression (.+) to valid types",
        ),
        // TODO: check if expected
        // This is a type coersion error: DuckDB also fails but I'm not sure if this
        // should be expected.
        // CREATE TABLE t3 (col_t3_5_uint64 UBIGINT);
        // INSERT INTO t3 VALUES (52);
        // SELECT (86 / ((t3.col_t3_5_uint64 - 117) % t3.col_t3_5_uint64)) FROM t3;
        ErrorPattern::RegexMatch(
            r"(?i)Query execution failed: Arrow error: Cast error: value of (.+) is out of range uint(.+)",
        ),
        // timestamp * timestamp
        ErrorPattern::Contains("Invalid timestamp arithmetic operation"),
        // Query timeout
        ErrorPattern::Contains("Query execution timed out"),
        // Create view might fail
        ErrorPattern::Contains("Failed to create view"),
        // Null - Null
        ErrorPattern::Contains("Cannot get result type for null arithmetic Null - Null"),
        // Only whitelist regex parse errors when query uses regexp-related function
        ErrorPattern::QueryAndErrorContains {
            query_sub: "regexp_replace(",
            error_sub: "regex parse error",
        },
        // Invalid JOIN ON expression like '... t1 natural join t2 on true'
        // Newer sqlparser versions append the location, e.g.
        // `found: ON at Line: 3, Column: 17`
        ErrorPattern::RegexMatch(
            r#"SQL error: ParserError\("Expected: end of statement, found: ON( at Line: \d+, Column: \d+)?"\)"#,
        ),
        // For anti joins, the fuzzer might generate join predicates that referencing
        // eliminated columns from anti joins, example (note t0.flag is a valid column
        // from t0, but it's eliminated by the first RIGHT ANTI JOIN):
        // SELECT *
        // FROM t0
        // RIGHT ANTI JOIN t1 ON TRUE
        // RIGHT ANTI JOIN t2 ON t0.flag;
        ErrorPattern::QueryAndErrorContains {
            query_sub: "ANTI JOIN",
            error_sub: "Schema error: No field named",
        },
        ErrorPattern::QueryAndErrorContains {
            query_sub: "to_date(",
            error_sub: "Casting from",
        },
        ErrorPattern::QueryAndErrorContains {
            query_sub: "to_date(",
            error_sub: "Error parsing timestamp from",
        },
        ErrorPattern::QueryAndErrorContains {
            query_sub: "to_char(",
            error_sub: "Cannot cast",
        },
        ErrorPattern::Contains("Regular expression did not compile"),
        ErrorPattern::Contains("to_unixtime function unsupported data type"),
        ErrorPattern::QueryAndErrorContains {
            query_sub: "to_unixtime(",
            error_sub: "Error parsing timestamp from",
        },
        ErrorPattern::QueryAndErrorContains {
            query_sub: "to_timestamp",
            error_sub: "Error parsing timestamp from",
        },
        // =========================
        // Known Issues
        // =========================

        // https://github.com/apache/datafusion/issues/13558
        ErrorPattern::Contains("Projections require unique expression names"),
        // `Operator::IsDistinctFrom` and `Operator::IsNotDistinctFrom` can not
        // be unparsed in `expr_to_sql` function
        ErrorPattern::Contains("unsupported operation: IsNotDistinctFrom"),
        ErrorPattern::Contains("unsupported operation: IsDistinctFrom"),
        // More works to be done to generate valid `to_char()` function
        ErrorPattern::Contains("to_char"),
        // Adding numeric type with time time might not be supported
        ErrorPattern::Contains("Cannot infer common argument type for comparison operation"),
        // https://github.com/apache/datafusion/issues/17387
        ErrorPattern::Contains("Invalid arithmetic operation: Null % Null"),
        // https://github.com/apache/datafusion/issues/17390
        ErrorPattern::Contains("Schema error: No field named"),
        // https://github.com/apache/datafusion/issues/17472
        ErrorPattern::Contains("to_local_time"),
        // =========================
        // Investigate Later
        // =========================
        ErrorPattern::Contains("Cast error: Format error"),
        ErrorPattern::Contains("to_date"),
        // This is function taking a invalid regex, but triggered a confusing optimizer
        // error -- I think the best thing to do is provide better error message
        ErrorPattern::Contains("Optimizer rule 'simplify_expressions' failed"),
        ErrorPattern::Contains("to_timestamp"),
    ]
});

/// Compiled regex patterns cache
static COMPILED_REGEXES: LazyLock<Vec<Option<Regex>>> = LazyLock::new(|| {
    ERROR_PATTERNS
        .iter()
        .map(|pattern| match pattern {
            ErrorPattern::Contains(_) => None, // No regex compilation needed for exact matches
            ErrorPattern::RegexMatch(regex_str) => match Regex::new(regex_str) {
                Ok(regex) => Some(regex),
                Err(e) => {
                    eprintln!("Warning: Invalid regex pattern '{}': {}", regex_str, e);
                    None
                }
            },
            ErrorPattern::QueryAndErrorContains { .. } => None,
        })
        .collect()
});

/// Check if an error message matches any pattern in the whitelist
///
/// This function checks both exact string patterns and regex patterns.
/// It returns true if the error message matches any whitelisted pattern.
///
/// # Arguments
/// * `error_msg` - The error message to check
/// * `query_sql` - The SQL text for the query that produced the error, if available
///
/// # Returns
/// * `true` if the error message matches any whitelisted pattern
/// * `false` if no patterns match
///
/// # Examples
/// ```rust
/// use datafusion_fuzzer::cli::error_whitelist::is_error_whitelisted;
///
/// // These should match if the patterns are configured
/// assert!(is_error_whitelisted("Query failed: Arrow error: Divide by zero error", None));
/// assert!(is_error_whitelisted("Some context: Arrow error: Divide by zero error here", None));
///
/// // This should not match
/// assert!(!is_error_whitelisted("Unexpected segmentation fault", None));
/// ```
pub fn is_error_whitelisted(error_msg: &str, query_sql: Option<&str>) -> bool {
    for (i, pattern) in ERROR_PATTERNS.iter().enumerate() {
        match pattern {
            ErrorPattern::Contains(exact_str) => {
                if error_msg.contains(exact_str) {
                    return true;
                }
            }
            ErrorPattern::RegexMatch(_) => {
                if let Some(Some(regex)) = COMPILED_REGEXES.get(i) {
                    if regex.is_match(error_msg) {
                        return true;
                    }
                }
            }
            ErrorPattern::QueryAndErrorContains {
                query_sub,
                error_sub,
            } => {
                if let Some(sql) = query_sql {
                    if sql.contains(query_sub) && error_msg.contains(error_sub) {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Get a list of all configured error patterns for debugging/logging
pub fn get_configured_patterns() -> Vec<String> {
    ERROR_PATTERNS
        .iter()
        .map(|pattern| match pattern {
            ErrorPattern::Contains(s) => format!("Exact: {}", s),
            ErrorPattern::RegexMatch(s) => format!("Regex: {}", s),
            ErrorPattern::QueryAndErrorContains {
                query_sub,
                error_sub,
            } => {
                format!(
                    "QueryAndError: query contains '{}' AND error contains '{}'",
                    query_sub, error_sub
                )
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::is_error_whitelisted;

    #[test]
    fn whitelists_timestamp_parse_errors_for_to_timestamp_queries() {
        let error = "Query execution failed: Execution error: Error parsing timestamp from 'abc' using format 'fmt': input contains invalid characters";
        let query = "SELECT to_timestamp_seconds(66, 'fmt')";

        assert!(is_error_whitelisted(error, Some(query)));
    }

    #[test]
    fn does_not_whitelist_timestamp_parse_errors_without_to_timestamp_query() {
        let error = "Query execution failed: Execution error: Error parsing timestamp from 'abc' using format 'fmt': input contains invalid characters";
        let query = "SELECT 1";

        assert!(!is_error_whitelisted(error, Some(query)));
    }

    #[test]
    fn whitelists_timestamp_parse_errors_for_to_date_queries() {
        let error = "Query execution failed: Execution error: Error parsing timestamp from 'abc' using format 'fmt': input contains invalid characters";
        let query = "SELECT to_date('abc', 'fmt')";

        assert!(is_error_whitelisted(error, Some(query)));
    }

    #[test]
    fn whitelists_join_on_parser_errors_with_and_without_location() {
        let with_location = "Query planning failed: SQL error: ParserError(\"Expected: end of statement, found: ON at Line: 3, Column: 17\")";
        let without_location = "Query planning failed: SQL error: ParserError(\"Expected: end of statement, found: ON\")";

        assert!(is_error_whitelisted(with_location, None));
        assert!(is_error_whitelisted(without_location, None));
    }

    #[test]
    fn does_not_whitelist_other_end_of_statement_parser_errors() {
        let error = "Query planning failed: SQL error: ParserError(\"Expected: end of statement, found: ONLY at Line: 1, Column: 5\")";

        assert!(!is_error_whitelisted(error, None));
    }
}
