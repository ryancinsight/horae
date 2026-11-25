use horae_civil_time::{CivilTimeError, UtcTimestamp};

#[test]
fn offsets_fractions_and_leap_seconds_have_exact_utc_outputs() {
    let _trace = super::trace();
    let cases = tracing::info_span!("arrange").in_scope(|| {
        [
            ("1970-01-01T00:45:00+01:00", "1969-12-31T23:45:00Z", -900, 0),
            (
                "1999-12-31T23:30:00-00:30",
                "2000-01-01T00:00:00Z",
                946_684_800,
                0,
            ),
            (
                "1970-01-01T00:00:00.1Z",
                "1970-01-01T00:00:00.100Z",
                0,
                100_000_000,
            ),
            (
                "1970-01-01T00:00:00.000348Z",
                "1970-01-01T00:00:00.000348Z",
                0,
                348_000,
            ),
            (
                "1970-01-01T00:00:00.123456789Z",
                "1970-01-01T00:00:00.123456789Z",
                0,
                123_456_789,
            ),
            (
                "2017-01-01T00:59:60.125+01:00",
                "2016-12-31T23:59:60.125Z",
                1_483_228_799,
                125_000_000,
            ),
        ]
    });
    let actual = tracing::info_span!("act").in_scope(|| {
        cases.map(|(input, _, _, _)| input.parse::<UtcTimestamp>().expect("valid timestamp"))
    });
    let _assert = tracing::info_span!("assert").entered();
    for (time, (_, expected, seconds, nanos)) in actual.into_iter().zip(cases) {
        assert_eq!(time.unix_seconds(), seconds);
        assert_eq!(time.subsec_nanoseconds(), nanos);
        assert_eq!(time.to_string(), expected);
        assert_eq!(expected.parse::<UtcTimestamp>(), Ok(time));
    }
    let leap = actual[5];
    assert!(leap.is_leap_second());
    assert!(leap < UtcTimestamp::from_unix_parts(1_483_228_800, 0).expect("next epoch second"));
    assert_eq!(format!("{leap:#}"), "2016-12-31 23:59:60 UTC");
}

#[test]
fn timestamp_validation_rejects_impossible_fields_and_unrepresentable_precision() {
    let _trace = super::trace();
    let cases = tracing::info_span!("arrange").in_scope(|| {
        [
            "1900-02-29T00:00:00Z",
            "2000-04-31T00:00:00Z",
            "2000-02-29T24:00:00Z",
            "1970-01-01T00:00:61Z",
            "1970-01-01T00:00:00",
            "1970-01-01T00:00:00.+00:00",
            "1970-01-01T00:00:00.0000000001Z",
            "1970-01-01T00:00:00+24:00",
            "0000-01-01T00:00:00+00:01",
            "9999-12-31T23:59:59-00:01",
            "1970-01-01T00:00:00.１２Z",
        ]
    });
    let actual = tracing::info_span!("act").in_scope(|| cases.map(str::parse::<UtcTimestamp>));
    let _assert = tracing::info_span!("assert").entered();
    // Each refusal names its own reason, so a case that starts failing for a
    // different reason than the one it is cited for fails here.
    let expected = [
        CivilTimeError::InvalidDate,
        CivilTimeError::InvalidDate,
        CivilTimeError::InvalidTimestamp,
        CivilTimeError::InvalidTimestamp,
        CivilTimeError::InvalidTimestamp,
        CivilTimeError::InvalidFraction,
        CivilTimeError::InvalidFraction,
        CivilTimeError::InvalidTimestamp,
        CivilTimeError::OutOfRange,
        CivilTimeError::OutOfRange,
        CivilTimeError::InvalidFraction,
    ];
    for ((result, input), expected) in actual.into_iter().zip(cases).zip(expected) {
        assert_eq!(result, Err(expected), "{input}");
    }
    assert_eq!(
        UtcTimestamp::from_unix_parts(0, 1_000_000_000),
        Err(CivilTimeError::InvalidFraction)
    );
}

#[cfg(feature = "std")]
#[test]
fn system_time_round_trips_before_and_after_the_epoch() {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    let _trace = super::trace();
    let clocks = tracing::info_span!("arrange").in_scope(|| {
        [
            UNIX_EPOCH - Duration::new(1, 250_000_000),
            UNIX_EPOCH,
            UNIX_EPOCH + Duration::new(1, 250_000_000),
        ]
    });
    let timestamps = tracing::info_span!("act").in_scope(|| {
        clocks.map(|clock| UtcTimestamp::try_from(clock).expect("representable clock"))
    });
    let _assert = tracing::info_span!("assert").entered();
    assert_eq!(
        timestamps[0],
        UtcTimestamp::from_unix_parts(-2, 750_000_000).expect("known time")
    );
    assert_eq!(
        timestamps[1],
        UtcTimestamp::from_unix_parts(0, 0).expect("epoch")
    );
    assert_eq!(
        timestamps[2],
        UtcTimestamp::from_unix_parts(1, 250_000_000).expect("known time")
    );
    assert_eq!(timestamps.map(SystemTime::try_from), clocks.map(Ok));
    let leap = "2016-12-31T23:59:60Z"
        .parse::<UtcTimestamp>()
        .expect("known leap second");
    assert_eq!(
        SystemTime::try_from(leap),
        Err(CivilTimeError::LeapSecondOnSystemClock)
    );
}

#[cfg(feature = "serde")]
#[test]
fn older_timestamp_reports_round_trip_to_the_same_json_strings() {
    let _trace = super::trace();
    let cases = tracing::info_span!("arrange").in_scope(|| {
        [
            (r#""2000-01-01T00:00:00Z""#, r#""2000-01-01T00:00:00Z""#),
            (
                r#""2000-01-01T01:00:00.001136500+01:00""#,
                r#""2000-01-01T00:00:00.001136500Z""#,
            ),
            (
                r#""2016-12-31T23:59:60.125Z""#,
                r#""2016-12-31T23:59:60.125Z""#,
            ),
        ]
    });
    let actual = tracing::info_span!("act").in_scope(|| {
        cases.map(|(record, _)| {
            let timestamp: UtcTimestamp =
                serde_json::from_str(record).expect("existing RFC 3339 record");
            serde_json::to_string(&timestamp).expect("timestamp serializes")
        })
    });
    let _assert = tracing::info_span!("assert").entered();
    for (record, (_, expected)) in actual.into_iter().zip(cases) {
        assert_eq!(record, expected);
    }
}
