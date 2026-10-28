<!--
SPDX-License-Identifier: Apache-2.0
SPDX-FileCopyrightText: See the NOTICE file distributed with this work for additional information regarding copyright ownership
SPDX-FileContributor: Licensed to the Apache Software Foundation (ASF) under one or more contributor license agreements
-->

# Error Whitelist System

The error whitelist system allows the fuzzer to distinguish between acceptable errors (like divide by zero) and unexpected crashes or stability issues.

## Overview

The system supports two types of error patterns:

1. **Exact String Patterns** - Fast substring matching for known error messages
2. **Regex Patterns** - Flexible pattern matching for variable error formats

## Configuration

Error patterns are configured in `src/cli/error_whitelist.rs` in the `ERROR_PATTERNS` static:

```rust
static ERROR_PATTERNS: LazyLock<Vec<ErrorPattern>> = LazyLock::new(|| {
    vec![
        // Exact patterns
        ErrorPattern::Exact("Arrow error: Divide by zero error"),
        
        // Regex patterns
        ErrorPattern::Regex(r"(?i)divide\s*by\s*zero"),
    ]
});
```

## Pattern Types

### Exact String Patterns

```rust
ErrorPattern::Exact("Arrow error: Divide by zero error")
```

- **Performance**: Fast substring matching
- **Case Sensitivity**: Case-sensitive
- **Use Case**: Known exact error messages
- **Example**: Matches "Query failed: Arrow error: Divide by zero error"

### Regex Patterns

```rust
ErrorPattern::Regex(r"(?i)divide\s*by\s*zero")
```

- **Performance**: Slower than exact matching but cached
- **Flexibility**: Full regex support
- **Use Case**: Variable error message formats
- **Example**: Matches "Divide By Zero", "divide by zero", "divide  by  zero"

## Adding New Patterns

Edit `src/cli/error_whitelist.rs` and add patterns to the `ERROR_PATTERNS` vector:

```rust
static ERROR_PATTERNS: LazyLock<Vec<ErrorPattern>> = LazyLock::new(|| {
    vec![
        // Your new exact pattern
        ErrorPattern::Exact("New exact error message"),
        
        // Your new regex pattern
        ErrorPattern::Regex(r"(?i)your.*regex.*pattern"),
    ]
});
```

## Regex Pattern Examples

### Case-Insensitive Matching
```rust
// Matches: "divide by zero", "DIVIDE BY ZERO", "Divide By Zero"
ErrorPattern::Regex(r"(?i)divide\s*by\s*zero")
```

### Multiple Alternatives
```rust
// Matches: "Arrow error: Divide by zero" OR "Arrow error: Invalid argument"
ErrorPattern::Regex(r"Arrow error: (Divide by zero|Invalid argument|Schema mismatch)")
```

### Flexible Whitespace
```rust
// Matches: "timeout", "time out", "timeout expired", "timed out"
ErrorPattern::Regex(r"(?i)(timeout|timed?\s*out)")
```

### Memory Errors
```rust
// Matches various memory-related errors
ErrorPattern::Regex(r"(?i)(out of memory|memory.*exhausted|allocation.*failed)")
```

### Network Errors
```rust
// Matches various network connectivity issues
ErrorPattern::Regex(r"(?i)(connection.*(refused|reset|timeout)|network.*unreachable)")
```

### Data Type Errors
```rust
// Matches type conversion errors
ErrorPattern::Regex(r"(?i)(type.*conversion|cast.*error|invalid.*format)")
```

### File System Errors
```rust
// Matches file system related errors
ErrorPattern::Regex(r"(?i)(file.*not.*found|permission.*denied|disk.*full)")
```

## Regex Tips

- **`(?i)`** - Case-insensitive matching (place at start)
- **`\s*`** - Zero or more whitespace characters
- **`\s+`** - One or more whitespace characters  
- **`.*`** - Any characters (wildcard)
- **`(option1|option2)`** - Alternative matching
- **`\b`** - Word boundary (avoid partial matches)
- **`?`** - Make preceding element optional
- **`+`** - One or more of preceding element
- **`*`** - Zero or more of preceding element

## Testing Patterns

Use the example to test your patterns:

```bash
cargo run --example error_whitelist_example
```

Or write unit tests in the `error_whitelist` module:

```rust
#[test]
fn test_my_pattern() {
    assert!(is_error_whitelisted("My error message"));
    assert!(!is_error_whitelisted("Should not match"));
}
```

## How It Works

1. When a query fails, `execute_single_query()` checks the error message
2. Each pattern in `ERROR_PATTERNS` is tested against the error message
3. Exact patterns use simple `contains()` checking
4. Regex patterns are compiled once and cached for performance
5. If any pattern matches, the error is logged as "whitelisted" (INFO level)
6. If no patterns match, the error is logged as problematic (ERROR level)
7. The NoCrashOracle now always passes validation since error checking is external

## Benefits

- **Reduced False Positives**: Expected errors don't cause test failures
- **Flexible Matching**: Both exact strings and regex patterns supported
- **Performance Optimized**: Regex patterns compiled once and cached
- **Easy to Extend**: Simple configuration file approach
- **Better Precision**: Focus testing on truly unexpected issues
- **Maintainable**: Clear separation of concerns

## Migration from Old System

The old hardcoded `ERROR_MESSAGE_WHITELIST` has been replaced with the flexible pattern system. Update your error handling code to use:

```rust
use crate::cli::error_whitelist;

// Old way
// if ERROR_MESSAGE_WHITELIST.contains(&error_msg) { ... }

// New way  
if error_whitelist::is_error_whitelisted(&error_msg) { ... }
```

## Performance Considerations

- Exact string patterns are faster than regex patterns
- Regex patterns are compiled once at startup and cached
- Use exact patterns for known error messages when possible
- Use regex patterns for variable error formats
- Complex regex patterns may impact performance - test thoroughly

## Examples in Practice

### Arithmetic Errors
```rust
ErrorPattern::Exact("Arrow error: Divide by zero error"),
ErrorPattern::Regex(r"(?i)(divide\s*by\s*zero|division\s*by\s*zero)"),
```

### Schema Validation
```rust
ErrorPattern::Regex(r"(?i)(schema.*mismatch|column.*not.*found|table.*does.*not.*exist)"),
```

### Resource Limits
```rust
ErrorPattern::Regex(r"(?i)(timeout|memory.*limit|disk.*space)"),
```

### Data Format Issues
```rust
ErrorPattern::Regex(r"(?i)(parse.*error|invalid.*format|conversion.*failed)"),
``` 