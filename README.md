<!--
SPDX-License-Identifier: Apache-2.0
SPDX-FileCopyrightText: See the NOTICE file distributed with this work for additional information regarding copyright ownership
SPDX-FileContributor: Licensed to the Apache Software Foundation (ASF) under one or more contributor license agreements
-->

# DataFusion Fuzzer

A fuzzing tool for Apache DataFusion that tests SQL query execution and helps find potential bugs, crashes, and inconsistencies in query results.

## Overview
This fuzzer primarily:
1. Generates random tables and SQL queries.
2. Runs them on DataFusion and checks whether the results satisfy an oracle-defined consistency rule.

### Example
```text
Oracle: TLP (Ternary Logic Partitioning)

Random query (Q1):
    SELECT * FROM t1;

Mutated query (Q2):
    SELECT * FROM t1 WHERE v1 > 0
    UNION ALL
    SELECT * FROM t1 WHERE NOT (v1 > 0)
    UNION ALL
    SELECT * FROM t1 WHERE (v1 > 0) IS NULL;

Consistency check:
    Q1 and Q2 should return the same multiset of rows.
```

This project is inspired by [SQLancer](https://github.com/sqlancer/sqlancer).

For an introduction to database fuzzing techniques, see this talk by the author of SQLancer: https://youtu.be/Np46NQ6lqP8?si=lSVAU7Jy3H-QtrWV

## Quick Start

To run the fuzzer with the default sample configuration:

```bash
cargo run --release -- --config fuzzer-default.toml
```

This runs the fuzzer against the DataFusion version specified in `Cargo.toml`.

The config file controls options such as round count, timeout, and log directory.

If a bug is found, use the CLI output and generated log files to reproduce it.

To override values from the configuration file by using CLI arguments:
```bash
cargo run --release -- --config fuzzer-default.toml --rounds 5 --queries-per-round 20
```

See `fuzzer-default.toml` for supported options.

### Command Line Options

```
Options:
  -c, --config <FILE>                    Path to config file
  -s, --seed <SEED>                      Random seed [default: 42]
  -r, --rounds <ROUNDS>                  Number of rounds to run
  -q, --queries-per-round <QUERIES>      Number of queries per round
  -t, --timeout <TIMEOUT>                Query timeout in seconds
  -l, --log-path <LOG_PATH>              Path to log file
  -d, --display-logs                     Display logs
      --enable-tui                       Enable TUI display
  -h, --help                             Print help
  -V, --version                          Print version
```

## Roadmap

### Implemented Oracles
The runner currently chooses one oracle at random for each test case:

- [x] `NoCrashOracle`: checks for non-whitelisted crashes and errors.
- [x] `TlpWhereOracle`: validates TLP partitioning over `WHERE` (`p`, `NOT p`, `p IS NULL`) using value-level multiset comparison.
- [x] `TlpHavingOracle`: validates TLP partitioning over `HAVING` (`p`, `NOT p`, `p IS NULL`) using value-level multiset comparison.
- [ ] `NoREC` (planned): [paper](https://www.manuelrigger.at/preprints/NoREC.pdf)

### SQL Features
- [x] WHERE
- [ ] SORT + LIMIT/OFFSET
- [ ] AGGREGATE
- [x] HAVING
- [ ] JOIN
- [ ] UNION/UNION ALL/INTERSECT/EXCEPT

### SQL Subqueries
- [ ] Views
- [ ] Scalar subquery
- [ ] `Relation-like` subquery

### Expressions
- [ ] Operators
- [ ] Scalar functions
- [ ] Aggregate Functions
- [ ] Window Functions

### Types
- [ ] Complete primitive type coverage
- [ ] Time-related types
- [ ] Array types
- [ ] Struct/JSON

### Infrastructure
- [x] CLI
- [x] Oracle interface

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
