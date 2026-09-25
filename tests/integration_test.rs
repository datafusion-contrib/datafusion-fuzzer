use datafusion_fuzzer::fuzz_context::RunnerConfig;
use datafusion_fuzzer::oracle::ConfiguredOracle;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Runs the fuzzer end-to-end with a fixed seed.
///
/// After fuzzer feature changes, update the snapshot and review the SQL manually to
/// make sure it still looks reasonable. This also ensures the fuzzer runs are
/// deterministic.
///
/// To update the snapshot after changes, run:
/// `cargo insta test --accept --test integration_test`
#[test]
fn full_run_logs_expected_queries_and_stats_for_no_crash_oracle() -> Result<(), Box<dyn Error>> {
    let log_dir = make_temp_log_dir("integration")?;
    let config_path = generate_default_config_with_oracles(&log_dir, &[ConfiguredOracle::NoCrash])?;
    let run_output = run_fuzzer_once(&config_path)?;

    insta::assert_snapshot!(run_output.query_log, @"
    === round=1 query=1 oracle=NoCrashOracle query_seed=310304 ===
    --- statement=1 context=Random Query No-Crash Test ---
    SELECT (31905.000000000000 + 42185.000000000000000000000), NULL, ((39.372757 * NULL) % -37.15892)
    FROM t0, t2, t1
    WHERE true

    === round=1 query=2 oracle=NoCrashOracle query_seed=310305 ===
    --- statement=1 context=Random Query No-Crash Test ---
    SELECT ((current_time() + current_time()) < -55), (65.706448177695 * ((NULL + -0.06514454367550115) + -85.93427693961893))
    FROM t0, t2, t1
    WHERE true

    === round=1 query=3 oracle=NoCrashOracle query_seed=310306 ===
    --- statement=1 context=Random Query No-Crash Test ---
    SELECT NULL
    FROM t2
    WHERE false

    === round=1 query=4 oracle=NoCrashOracle query_seed=310307 ===
    --- statement=1 context=Random Query No-Crash Test ---
    SELECT (85 + 132), ((-34 * -82) % -74)
    FROM t0
    WHERE false

    === round=1 query=5 oracle=NoCrashOracle query_seed=310308 ===
    --- statement=1 context=Random Query No-Crash Test ---
    SELECT (to_char((CAST('14:29:14.387490541' AS TIME) + t0.col_t0_3_time64_nanosecond), to_char(CAST('2058-07-27' AS DATE), '?598KgX')) ~~* to_char(to_timestamp_seconds(114, '<DsT[M'), 'ly5GZ9::]%%=yiyl')), (((195 * 195) % 109) % (131 / (185 % 148))), (81 / t0.col_t0_2_uint64)
    FROM t2, t0, t1
    WHERE true

    === round=2 query=1 oracle=NoCrashOracle query_seed=311304 ===
    --- statement=1 context=Random Query No-Crash Test ---
    SELECT current_date(), 27, (((63 * -4) * (2 + 33)) - ((76 + 61) * -91))
    FROM t1, t0
    WHERE ((to_char(CAST('2000-06-23T18:11:00.994418145+09:00' AS TIMESTAMP), t0.col_t0_5_string) ~~ to_char(CAST('2018-01-08' AS DATE), '^tvmf:B,s')) AND ('zAj?tsovnki' ~~ to_char(CAST('2051-03-08 22:26:00.614618371' AS TIMESTAMP), 'bR>g0Rc_-|bD`5[3>eiypC!B=V}- oa>y1j.5BVhcu(w+Cw')))

    === round=2 query=2 oracle=NoCrashOracle query_seed=311305 ===
    --- statement=1 context=Random Query No-Crash Test ---
    SELECT INTERVAL '9 MONS 12 DAYS -0.376941645 SECS', 80
    FROM t2, t0
    RIGHT SEMI JOIN t1 ON ('L/fQTV' !~ to_char((t0.col_t0_2_time64_nanosecond + CAST('00:47:51.915143361' AS TIME)), to_char(INTERVAL '4 MONS -23 DAYS -0.512025870 SECS', NULL)))
    WHERE true

    === round=2 query=3 oracle=NoCrashOracle query_seed=311306 ===
    --- statement=1 context=Random Query No-Crash Test ---
    SELECT (118 % 25), -6, NULL
    FROM t2
    WHERE false

    === round=2 query=4 oracle=NoCrashOracle query_seed=311307 ===
    --- statement=1 context=Random Query No-Crash Test ---
    SELECT [-86, -7, -92, 31], (((-12 % -76) + (70 + -96)) - -46), current_date()
    FROM t0, t1
    WHERE (to_char(t0.col_t0_2_time64_nanosecond, 'ofK{JwVT8u7{CI{{ ]w=K=/e6jJ:>') ~~ to_char(t0.col_t0_2_time64_nanosecond, '&^!k$A^76Nypu59M5g hP!l268DvM}?!qsf`z~d3!8c%'))

    === round=2 query=5 oracle=NoCrashOracle query_seed=311308 ===
    --- statement=1 context=Random Query No-Crash Test ---
    SELECT (-24 / (94 - NULL))
    FROM t2
    WHERE true
    ");

    insta::assert_snapshot!(run_output.stats_summary, @r"
    ============================================================
    🎯 DataFusion Fuzzer - Final Statistics
    ============================================================
    📊 Execution Summary:
      • Rounds Completed: 2
      • Queries Executed: 10
      • Query Success Rate: 60.00%
    ");

    fs::remove_dir_all(&log_dir)?;

    Ok(())
}

#[test]
fn full_run_logs_expected_queries_for_tlp_where_oracle() -> Result<(), Box<dyn Error>> {
    let log_dir = make_temp_log_dir("integration-tlp-where")?;
    let config_path =
        generate_default_config_with_oracles(&log_dir, &[ConfiguredOracle::TlpWhere])?;
    let run_output = run_fuzzer_once(&config_path)?;

    insta::assert_snapshot!(run_output.query_log, @r#"
    === round=1 query=1 oracle=TlpWhereOracle query_seed=310304 ===
    --- statement=1 context=TLP-WHERE all ---
    SELECT *
    FROM t0

    --- statement=2 context=TLP-WHERE p UNION ALL NOT p UNION ALL p IS NULL ---
    SELECT *
    FROM t0
    WHERE (false)
    UNION ALL
    SELECT *
    FROM t0
    WHERE NOT (false)
    UNION ALL
    SELECT *
    FROM t0
    WHERE (false) IS NULL

    === round=1 query=2 oracle=TlpWhereOracle query_seed=310305 ===
    --- statement=1 context=TLP-WHERE all ---
    SELECT *
    FROM t1

    --- statement=2 context=TLP-WHERE p UNION ALL NOT p UNION ALL p IS NULL ---
    SELECT *
    FROM t1
    WHERE (false)
    UNION ALL
    SELECT *
    FROM t1
    WHERE NOT (false)
    UNION ALL
    SELECT *
    FROM t1
    WHERE (false) IS NULL

    === round=1 query=3 oracle=TlpWhereOracle query_seed=310306 ===
    --- statement=1 context=TLP-WHERE all ---
    SELECT *
    FROM t2

    --- statement=2 context=TLP-WHERE p UNION ALL NOT p UNION ALL p IS NULL ---
    SELECT *
    FROM t2
    WHERE ((49 < 146))
    UNION ALL
    SELECT *
    FROM t2
    WHERE NOT ((49 < 146))
    UNION ALL
    SELECT *
    FROM t2
    WHERE ((49 < 146)) IS NULL

    === round=1 query=4 oracle=TlpWhereOracle query_seed=310307 ===
    --- statement=1 context=TLP-WHERE all ---
    SELECT *
    FROM t0

    --- statement=2 context=TLP-WHERE p UNION ALL NOT p UNION ALL p IS NULL ---
    SELECT *
    FROM t0
    WHERE ((NULL IS NOT DISTINCT FROM to_timestamp_millis((25 / 181), '|Z>XgVn5}e~aHs#pFd~N')))
    UNION ALL
    SELECT *
    FROM t0
    WHERE NOT ((NULL IS NOT DISTINCT FROM to_timestamp_millis((25 / 181), '|Z>XgVn5}e~aHs#pFd~N')))
    UNION ALL
    SELECT *
    FROM t0
    WHERE ((NULL IS NOT DISTINCT FROM to_timestamp_millis((25 / 181), '|Z>XgVn5}e~aHs#pFd~N'))) IS NULL

    === round=1 query=5 oracle=TlpWhereOracle query_seed=310308 ===
    --- statement=1 context=TLP-WHERE all ---
    SELECT *
    FROM t0

    --- statement=2 context=TLP-WHERE p UNION ALL NOT p UNION ALL p IS NULL ---
    SELECT *
    FROM t0
    WHERE ((to_char(current_time(), '4l6.-bG6dPLWk-7 ~9azH0^V') ~~ '#|%@?MyX"Sv3Vm^G~t[z2I2cRt*'))
    UNION ALL
    SELECT *
    FROM t0
    WHERE NOT ((to_char(current_time(), '4l6.-bG6dPLWk-7 ~9azH0^V') ~~ '#|%@?MyX"Sv3Vm^G~t[z2I2cRt*'))
    UNION ALL
    SELECT *
    FROM t0
    WHERE ((to_char(current_time(), '4l6.-bG6dPLWk-7 ~9azH0^V') ~~ '#|%@?MyX"Sv3Vm^G~t[z2I2cRt*')) IS NULL

    === round=2 query=1 oracle=TlpWhereOracle query_seed=311304 ===
    --- statement=1 context=TLP-WHERE all ---
    SELECT *
    FROM t0

    --- statement=2 context=TLP-WHERE p UNION ALL NOT p UNION ALL p IS NULL ---
    SELECT *
    FROM t0
    WHERE (false)
    UNION ALL
    SELECT *
    FROM t0
    WHERE NOT (false)
    UNION ALL
    SELECT *
    FROM t0
    WHERE (false) IS NULL

    === round=2 query=2 oracle=TlpWhereOracle query_seed=311305 ===
    --- statement=1 context=TLP-WHERE all ---
    SELECT *
    FROM t0

    --- statement=2 context=TLP-WHERE p UNION ALL NOT p UNION ALL p IS NULL ---
    SELECT *
    FROM t0
    WHERE (t0.col_t0_3_boolean)
    UNION ALL
    SELECT *
    FROM t0
    WHERE NOT (t0.col_t0_3_boolean)
    UNION ALL
    SELECT *
    FROM t0
    WHERE (t0.col_t0_3_boolean) IS NULL

    === round=2 query=3 oracle=TlpWhereOracle query_seed=311306 ===
    --- statement=1 context=TLP-WHERE all ---
    SELECT *
    FROM t2

    --- statement=2 context=TLP-WHERE p UNION ALL NOT p UNION ALL p IS NULL ---
    SELECT *
    FROM t2
    WHERE ((71.975845 > 59))
    UNION ALL
    SELECT *
    FROM t2
    WHERE NOT ((71.975845 > 59))
    UNION ALL
    SELECT *
    FROM t2
    WHERE ((71.975845 > 59)) IS NULL

    === round=2 query=4 oracle=TlpWhereOracle query_seed=311307 ===
    --- statement=1 context=TLP-WHERE all ---
    SELECT *
    FROM t1

    --- statement=2 context=TLP-WHERE p UNION ALL NOT p UNION ALL p IS NULL ---
    SELECT *
    FROM t1
    WHERE (('gWBxNx-(,<szw+N+q&ccjnB(c9kF$Kf5' !~~* 'x<'))
    UNION ALL
    SELECT *
    FROM t1
    WHERE NOT (('gWBxNx-(,<szw+N+q&ccjnB(c9kF$Kf5' !~~* 'x<'))
    UNION ALL
    SELECT *
    FROM t1
    WHERE (('gWBxNx-(,<szw+N+q&ccjnB(c9kF$Kf5' !~~* 'x<')) IS NULL

    === round=2 query=5 oracle=TlpWhereOracle query_seed=311308 ===
    --- statement=1 context=TLP-WHERE all ---
    SELECT *
    FROM t2

    --- statement=2 context=TLP-WHERE p UNION ALL NOT p UNION ALL p IS NULL ---
    SELECT *
    FROM t2
    WHERE ((((-27 % -48) IS NOT DISTINCT FROM (NULL + -13)) IS DISTINCT FROM 16))
    UNION ALL
    SELECT *
    FROM t2
    WHERE NOT ((((-27 % -48) IS NOT DISTINCT FROM (NULL + -13)) IS DISTINCT FROM 16))
    UNION ALL
    SELECT *
    FROM t2
    WHERE ((((-27 % -48) IS NOT DISTINCT FROM (NULL + -13)) IS DISTINCT FROM 16)) IS NULL
    "#);
    insta::assert_snapshot!(run_output.stats_summary, @"
    ============================================================
    🎯 DataFusion Fuzzer - Final Statistics
    ============================================================
    📊 Execution Summary:
      • Rounds Completed: 2
      • Queries Executed: 20
      • Query Success Rate: 95.00%
    ");

    fs::remove_dir_all(&log_dir)?;

    Ok(())
}

#[test]
fn full_run_logs_expected_queries_for_tlp_having_oracle() -> Result<(), Box<dyn Error>> {
    let log_dir = make_temp_log_dir("integration-tlp-having")?;
    let config_path =
        generate_default_config_with_oracles(&log_dir, &[ConfiguredOracle::TlpHaving])?;
    let run_output = run_fuzzer_once(&config_path)?;

    insta::assert_snapshot!(run_output.query_log, @r#"
    === round=1 query=1 oracle=TlpHavingOracle query_seed=310304 ===
    --- statement=1 context=TLP-HAVING all groups ---
    SELECT t0.col_t0_2_uint64, t0.col_t0_1_decimal128
    FROM t0
    GROUP BY t0.col_t0_2_uint64, t0.col_t0_1_decimal128

    --- statement=2 context=TLP-HAVING p UNION ALL NOT p UNION ALL p IS NULL ---
    SELECT t0.col_t0_2_uint64, t0.col_t0_1_decimal128
    FROM t0
    GROUP BY t0.col_t0_2_uint64, t0.col_t0_1_decimal128
    HAVING (false)
    UNION ALL
    SELECT t0.col_t0_2_uint64, t0.col_t0_1_decimal128
    FROM t0
    GROUP BY t0.col_t0_2_uint64, t0.col_t0_1_decimal128
    HAVING NOT (false)
    UNION ALL
    SELECT t0.col_t0_2_uint64, t0.col_t0_1_decimal128
    FROM t0
    GROUP BY t0.col_t0_2_uint64, t0.col_t0_1_decimal128
    HAVING (false) IS NULL

    === round=1 query=2 oracle=TlpHavingOracle query_seed=310305 ===
    --- statement=1 context=TLP-HAVING all groups ---
    SELECT t1.col_t1_1_int32, t1.col_t1_3_date32, t1.col_t1_2_int32
    FROM t1
    GROUP BY t1.col_t1_1_int32, t1.col_t1_3_date32, t1.col_t1_2_int32

    --- statement=2 context=TLP-HAVING p UNION ALL NOT p UNION ALL p IS NULL ---
    SELECT t1.col_t1_1_int32, t1.col_t1_3_date32, t1.col_t1_2_int32
    FROM t1
    GROUP BY t1.col_t1_1_int32, t1.col_t1_3_date32, t1.col_t1_2_int32
    HAVING (true)
    UNION ALL
    SELECT t1.col_t1_1_int32, t1.col_t1_3_date32, t1.col_t1_2_int32
    FROM t1
    GROUP BY t1.col_t1_1_int32, t1.col_t1_3_date32, t1.col_t1_2_int32
    HAVING NOT (true)
    UNION ALL
    SELECT t1.col_t1_1_int32, t1.col_t1_3_date32, t1.col_t1_2_int32
    FROM t1
    GROUP BY t1.col_t1_1_int32, t1.col_t1_3_date32, t1.col_t1_2_int32
    HAVING (true) IS NULL

    === round=1 query=3 oracle=TlpHavingOracle query_seed=310306 ===
    --- statement=1 context=TLP-HAVING all groups ---
    SELECT t2.col_t2_1_int64_array
    FROM t2
    WHERE (49 < 146)
    GROUP BY t2.col_t2_1_int64_array

    --- statement=2 context=TLP-HAVING p UNION ALL NOT p UNION ALL p IS NULL ---
    SELECT t2.col_t2_1_int64_array
    FROM t2
    WHERE (49 < 146)
    GROUP BY t2.col_t2_1_int64_array
    HAVING (true)
    UNION ALL
    SELECT t2.col_t2_1_int64_array
    FROM t2
    WHERE (49 < 146)
    GROUP BY t2.col_t2_1_int64_array
    HAVING NOT (true)
    UNION ALL
    SELECT t2.col_t2_1_int64_array
    FROM t2
    WHERE (49 < 146)
    GROUP BY t2.col_t2_1_int64_array
    HAVING (true) IS NULL

    === round=1 query=4 oracle=TlpHavingOracle query_seed=310307 ===
    --- statement=1 context=TLP-HAVING all groups ---
    SELECT t0.col_t0_3_time64_nanosecond
    FROM t0
    WHERE (NULL IS NOT DISTINCT FROM to_timestamp_millis((25 / 181), '|Z>XgVn5}e~aHs#pFd~N'))
    GROUP BY t0.col_t0_3_time64_nanosecond

    --- statement=2 context=TLP-HAVING p UNION ALL NOT p UNION ALL p IS NULL ---
    SELECT t0.col_t0_3_time64_nanosecond
    FROM t0
    WHERE (NULL IS NOT DISTINCT FROM to_timestamp_millis((25 / 181), '|Z>XgVn5}e~aHs#pFd~N'))
    GROUP BY t0.col_t0_3_time64_nanosecond
    HAVING (false)
    UNION ALL
    SELECT t0.col_t0_3_time64_nanosecond
    FROM t0
    WHERE (NULL IS NOT DISTINCT FROM to_timestamp_millis((25 / 181), '|Z>XgVn5}e~aHs#pFd~N'))
    GROUP BY t0.col_t0_3_time64_nanosecond
    HAVING NOT (false)
    UNION ALL
    SELECT t0.col_t0_3_time64_nanosecond
    FROM t0
    WHERE (NULL IS NOT DISTINCT FROM to_timestamp_millis((25 / 181), '|Z>XgVn5}e~aHs#pFd~N'))
    GROUP BY t0.col_t0_3_time64_nanosecond
    HAVING (false) IS NULL

    === round=1 query=5 oracle=TlpHavingOracle query_seed=310308 ===
    --- statement=1 context=TLP-HAVING all groups ---
    SELECT t0.col_t0_2_uint64, t0.col_t0_3_time64_nanosecond, t0.col_t0_1_decimal128
    FROM t0
    WHERE (to_char(current_time(), '4l6.-bG6dPLWk-7 ~9azH0^V') ~~ '#|%@?MyX"Sv3Vm^G~t[z2I2cRt*')
    GROUP BY t0.col_t0_2_uint64, t0.col_t0_3_time64_nanosecond, t0.col_t0_1_decimal128

    --- statement=2 context=TLP-HAVING p UNION ALL NOT p UNION ALL p IS NULL ---
    SELECT t0.col_t0_2_uint64, t0.col_t0_3_time64_nanosecond, t0.col_t0_1_decimal128
    FROM t0
    WHERE (to_char(current_time(), '4l6.-bG6dPLWk-7 ~9azH0^V') ~~ '#|%@?MyX"Sv3Vm^G~t[z2I2cRt*')
    GROUP BY t0.col_t0_2_uint64, t0.col_t0_3_time64_nanosecond, t0.col_t0_1_decimal128
    HAVING ((to_char(INTERVAL '-7 MONS 29 DAYS -0.000000001 SECS', '%X `B') !~* '0SsYa@-p]yc`qTL8PvF #c;Tei9))DXs:^wgv['))
    UNION ALL
    SELECT t0.col_t0_2_uint64, t0.col_t0_3_time64_nanosecond, t0.col_t0_1_decimal128
    FROM t0
    WHERE (to_char(current_time(), '4l6.-bG6dPLWk-7 ~9azH0^V') ~~ '#|%@?MyX"Sv3Vm^G~t[z2I2cRt*')
    GROUP BY t0.col_t0_2_uint64, t0.col_t0_3_time64_nanosecond, t0.col_t0_1_decimal128
    HAVING NOT ((to_char(INTERVAL '-7 MONS 29 DAYS -0.000000001 SECS', '%X `B') !~* '0SsYa@-p]yc`qTL8PvF #c;Tei9))DXs:^wgv['))
    UNION ALL
    SELECT t0.col_t0_2_uint64, t0.col_t0_3_time64_nanosecond, t0.col_t0_1_decimal128
    FROM t0
    WHERE (to_char(current_time(), '4l6.-bG6dPLWk-7 ~9azH0^V') ~~ '#|%@?MyX"Sv3Vm^G~t[z2I2cRt*')
    GROUP BY t0.col_t0_2_uint64, t0.col_t0_3_time64_nanosecond, t0.col_t0_1_decimal128
    HAVING ((to_char(INTERVAL '-7 MONS 29 DAYS -0.000000001 SECS', '%X `B') !~* '0SsYa@-p]yc`qTL8PvF #c;Tei9))DXs:^wgv[')) IS NULL

    === round=2 query=1 oracle=TlpHavingOracle query_seed=311304 ===
    --- statement=1 context=TLP-HAVING all groups ---
    SELECT t0.col_t0_2_time64_nanosecond, t0.col_t0_5_string, t0.col_t0_4_int32_array
    FROM t0
    WHERE false
    GROUP BY t0.col_t0_2_time64_nanosecond, t0.col_t0_5_string, t0.col_t0_4_int32_array

    --- statement=2 context=TLP-HAVING p UNION ALL NOT p UNION ALL p IS NULL ---
    SELECT t0.col_t0_2_time64_nanosecond, t0.col_t0_5_string, t0.col_t0_4_int32_array
    FROM t0
    WHERE false
    GROUP BY t0.col_t0_2_time64_nanosecond, t0.col_t0_5_string, t0.col_t0_4_int32_array
    HAVING (false)
    UNION ALL
    SELECT t0.col_t0_2_time64_nanosecond, t0.col_t0_5_string, t0.col_t0_4_int32_array
    FROM t0
    WHERE false
    GROUP BY t0.col_t0_2_time64_nanosecond, t0.col_t0_5_string, t0.col_t0_4_int32_array
    HAVING NOT (false)
    UNION ALL
    SELECT t0.col_t0_2_time64_nanosecond, t0.col_t0_5_string, t0.col_t0_4_int32_array
    FROM t0
    WHERE false
    GROUP BY t0.col_t0_2_time64_nanosecond, t0.col_t0_5_string, t0.col_t0_4_int32_array
    HAVING (false) IS NULL

    === round=2 query=2 oracle=TlpHavingOracle query_seed=311305 ===
    --- statement=1 context=TLP-HAVING all groups ---
    SELECT t0.col_t0_3_boolean, t0.col_t0_1_float64
    FROM t0
    WHERE t0.col_t0_3_boolean
    GROUP BY t0.col_t0_3_boolean, t0.col_t0_1_float64

    --- statement=2 context=TLP-HAVING p UNION ALL NOT p UNION ALL p IS NULL ---
    SELECT t0.col_t0_3_boolean, t0.col_t0_1_float64
    FROM t0
    WHERE t0.col_t0_3_boolean
    GROUP BY t0.col_t0_3_boolean, t0.col_t0_1_float64
    HAVING (((CAST('04:34:53.736298388' AS TIME) - CAST('13:39:07.159400901' AS TIME)) IS DISTINCT FROM current_date()))
    UNION ALL
    SELECT t0.col_t0_3_boolean, t0.col_t0_1_float64
    FROM t0
    WHERE t0.col_t0_3_boolean
    GROUP BY t0.col_t0_3_boolean, t0.col_t0_1_float64
    HAVING NOT (((CAST('04:34:53.736298388' AS TIME) - CAST('13:39:07.159400901' AS TIME)) IS DISTINCT FROM current_date()))
    UNION ALL
    SELECT t0.col_t0_3_boolean, t0.col_t0_1_float64
    FROM t0
    WHERE t0.col_t0_3_boolean
    GROUP BY t0.col_t0_3_boolean, t0.col_t0_1_float64
    HAVING (((CAST('04:34:53.736298388' AS TIME) - CAST('13:39:07.159400901' AS TIME)) IS DISTINCT FROM current_date())) IS NULL

    === round=2 query=3 oracle=TlpHavingOracle query_seed=311306 ===
    --- statement=1 context=TLP-HAVING all groups ---
    SELECT t2.col_t2_1_uint32
    FROM t2
    WHERE (71.975845 > 59)
    GROUP BY t2.col_t2_1_uint32

    --- statement=2 context=TLP-HAVING p UNION ALL NOT p UNION ALL p IS NULL ---
    SELECT t2.col_t2_1_uint32
    FROM t2
    WHERE (71.975845 > 59)
    GROUP BY t2.col_t2_1_uint32
    HAVING (false)
    UNION ALL
    SELECT t2.col_t2_1_uint32
    FROM t2
    WHERE (71.975845 > 59)
    GROUP BY t2.col_t2_1_uint32
    HAVING NOT (false)
    UNION ALL
    SELECT t2.col_t2_1_uint32
    FROM t2
    WHERE (71.975845 > 59)
    GROUP BY t2.col_t2_1_uint32
    HAVING (false) IS NULL

    === round=2 query=4 oracle=TlpHavingOracle query_seed=311307 ===
    --- statement=1 context=TLP-HAVING all groups ---
    SELECT t1.col_t1_4_date32
    FROM t1
    GROUP BY t1.col_t1_4_date32

    --- statement=2 context=TLP-HAVING p UNION ALL NOT p UNION ALL p IS NULL ---
    SELECT t1.col_t1_4_date32
    FROM t1
    GROUP BY t1.col_t1_4_date32
    HAVING (true)
    UNION ALL
    SELECT t1.col_t1_4_date32
    FROM t1
    GROUP BY t1.col_t1_4_date32
    HAVING NOT (true)
    UNION ALL
    SELECT t1.col_t1_4_date32
    FROM t1
    GROUP BY t1.col_t1_4_date32
    HAVING (true) IS NULL

    === round=2 query=5 oracle=TlpHavingOracle query_seed=311308 ===
    --- statement=1 context=TLP-HAVING all groups ---
    SELECT t2.col_t2_1_uint32
    FROM t2
    GROUP BY t2.col_t2_1_uint32

    --- statement=2 context=TLP-HAVING p UNION ALL NOT p UNION ALL p IS NULL ---
    SELECT t2.col_t2_1_uint32
    FROM t2
    GROUP BY t2.col_t2_1_uint32
    HAVING (((-54.097916 + 82.92108) < to_timestamp_seconds(-33, to_char(CAST('2012-07-21T16:46:18.355033437-09:00' AS TIMESTAMP), 'clQaw=v(!#dUGtlPSU4GzDeJ{KF$%HvHn~'))))
    UNION ALL
    SELECT t2.col_t2_1_uint32
    FROM t2
    GROUP BY t2.col_t2_1_uint32
    HAVING NOT (((-54.097916 + 82.92108) < to_timestamp_seconds(-33, to_char(CAST('2012-07-21T16:46:18.355033437-09:00' AS TIMESTAMP), 'clQaw=v(!#dUGtlPSU4GzDeJ{KF$%HvHn~'))))
    UNION ALL
    SELECT t2.col_t2_1_uint32
    FROM t2
    GROUP BY t2.col_t2_1_uint32
    HAVING (((-54.097916 + 82.92108) < to_timestamp_seconds(-33, to_char(CAST('2012-07-21T16:46:18.355033437-09:00' AS TIMESTAMP), 'clQaw=v(!#dUGtlPSU4GzDeJ{KF$%HvHn~')))) IS NULL
    "#);
    insta::assert_snapshot!(run_output.stats_summary, @"
    ============================================================
    🎯 DataFusion Fuzzer - Final Statistics
    ============================================================
    📊 Execution Summary:
      • Rounds Completed: 2
      • Queries Executed: 20
      • Query Success Rate: 85.00%
    ");

    fs::remove_dir_all(&log_dir)?;

    Ok(())
}

struct RunOutput {
    query_log: String,
    stats_summary: String,
}

fn run_fuzzer_once(config_path: &Path) -> Result<RunOutput, Box<dyn Error>> {
    let config = RunnerConfig::from_file(config_path)?;
    let log_dir = config
        .log_path
        .ok_or("expected test config to include a log_path")?;

    let output = Command::new(env!("CARGO_BIN_EXE_datafusion-fuzzer"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["--config"])
        .arg(config_path)
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "fuzzer run failed with status {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    let query_log_path = log_dir.join("queries.log");
    if !query_log_path.exists() {
        return Err(format!(
            "expected query log at '{}', but it was not created\nstdout:\n{}\nstderr:\n{}",
            query_log_path.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    let stdout = String::from_utf8(output.stdout)?;

    Ok(RunOutput {
        query_log: fs::read_to_string(query_log_path)?,
        stats_summary: extract_stats_summary(&stdout)?,
    })
}

fn generate_default_config_with_oracles(
    log_dir: &Path,
    oracles: &[ConfiguredOracle],
) -> Result<PathBuf, Box<dyn Error>> {
    let config_path = log_dir.join("integration.toml");
    let config = RunnerConfig {
        rounds: 2,
        queries_per_round: 5,
        log_path: Some(log_dir.to_path_buf()),
        enable_tui: false,
        oracles: oracles.to_vec(),
        ..RunnerConfig::default()
    };

    // Generate an integration-test config from the default config shape while
    // letting each test pin its oracle set and deterministic runtime knobs.
    fs::write(&config_path, toml::to_string(&config)?)?;

    Ok(config_path)
}

fn extract_stats_summary(stdout: &str) -> Result<String, Box<dyn Error>> {
    let lines: Vec<&str> = stdout.lines().collect();
    let start = lines
        .iter()
        .position(|line| *line == "============================================================")
        .ok_or("failed to find statistics header in stdout")?;

    let end = start + 6;
    if end >= lines.len() {
        return Err("stdout did not contain the expected statistics summary lines".into());
    }

    Ok(lines[start..=end].join("\n"))
}

fn make_temp_log_dir(label: &str) -> Result<PathBuf, Box<dyn Error>> {
    let unique_id = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let path = std::env::temp_dir().join(format!(
        "datafusion-fuzzer-e2e-{label}-{}-{unique_id}",
        std::process::id()
    ));
    fs::create_dir_all(&path)?;
    Ok(path)
}
