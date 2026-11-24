use horae_civil_time::{CivilDate, CivilTimeError};

#[test]
fn every_day_of_a_gregorian_cycle_round_trips() {
    let _trace = super::trace();
    let (first, last) = tracing::info_span!("arrange").in_scope(|| {
        (
            CivilDate::new(1600, 1, 1).expect("valid date").unix_days(),
            CivilDate::new(2000, 1, 1).expect("valid date").unix_days(),
        )
    });
    let converted = tracing::info_span!("act").in_scope(|| {
        (first..last)
            .map(|day| CivilDate::from_unix_days(day).expect("date within representation"))
            .collect::<Vec<_>>()
    });
    let _assert = tracing::info_span!("assert").entered();
    assert_eq!(last - first, 146_097);
    for (day, date) in (first..last).zip(converted) {
        assert_eq!(date.unix_days(), day);
    }
}

#[test]
fn calendar_boundaries_and_epoch_match_known_dates() {
    let _trace = super::trace();
    let cases = tracing::info_span!("arrange").in_scope(|| {
        [
            ((1969, 12, 31), -1),
            ((1970, 1, 1), 0),
            ((2000, 1, 1), 10_957),
            ((2000, 2, 29), 11_016),
        ]
    });
    let (actual, invalid) = tracing::info_span!("act").in_scope(|| {
        (
            cases.map(|((year, month, day), _)| {
                CivilDate::new(year, month, day).expect("known Gregorian date")
            }),
            [
                (1900, 2, 29),
                (2000, 0, 1),
                (2000, 4, 31),
                (2000, 4, 0),
                (10_000, 1, 1),
            ]
            .map(|(year, month, day)| CivilDate::new(year, month, day)),
        )
    });
    let _assert = tracing::info_span!("assert").entered();
    for (date, (_, days)) in actual.into_iter().zip(cases) {
        assert_eq!(date.unix_days(), days);
        assert_eq!(CivilDate::from_unix_days(days), Ok(date));
    }
    assert_eq!(invalid, [Err(CivilTimeError::InvalidDate); 5]);
    assert_eq!(
        CivilDate::from_unix_days(i64::MAX),
        Err(CivilTimeError::OutOfRange)
    );
    assert_eq!(
        CivilDate::from_unix_days(-719_528)
            .expect("earliest date")
            .to_string(),
        "0000-01-01"
    );
}
