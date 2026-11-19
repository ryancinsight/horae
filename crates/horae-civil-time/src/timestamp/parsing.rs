use super::{NANOS_PER_SECOND, SECONDS_PER_DAY, UtcTimestamp};
use crate::{CivilDate, CivilTimeError};
use core::str::FromStr;

impl FromStr for UtcTimestamp {
    type Err = CivilTimeError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let Some((prefix, tail)) = input.as_bytes().split_at_checked(19) else {
            return Err(CivilTimeError::InvalidTimestamp);
        };
        if prefix[4] != b'-'
            || prefix[7] != b'-'
            || !matches!(prefix[10], b'T' | b't' | b' ')
            || prefix[13] != b':'
            || prefix[16] != b':'
        {
            return Err(CivilTimeError::InvalidTimestamp);
        }

        let date = CivilDate::new(
            u16::try_from(digits(&prefix[..4])?).map_err(|_| CivilTimeError::InvalidDate)?,
            u8::try_from(digits(&prefix[5..7])?).map_err(|_| CivilTimeError::InvalidDate)?,
            u8::try_from(digits(&prefix[8..10])?).map_err(|_| CivilTimeError::InvalidDate)?,
        )?;
        let hours = digits(&prefix[11..13])?;
        let minutes = digits(&prefix[14..16])?;
        let seconds = digits(&prefix[17..19])?;
        if hours > 23 || minutes > 59 || seconds > 60 {
            return Err(CivilTimeError::InvalidTimestamp);
        }
        let (nanos, tail) = parse_fraction(tail)?;
        let utc_offset = parse_offset(tail)?;

        // The four-digit date and one-day-wide offset keep this sum inside
        // i64. Range-check again after applying the offset across midnight.
        let day_seconds = hours * 3600 + minutes * 60 + seconds.min(59);
        let unix_seconds = date.unix_days() * SECONDS_PER_DAY + i64::from(day_seconds) - utc_offset;
        let mut instant = Self::from_unix_parts(unix_seconds, nanos)?;
        if seconds == 60 {
            instant.nanos += NANOS_PER_SECOND;
        }
        Ok(instant)
    }
}

fn digits(bytes: &[u8]) -> Result<u32, CivilTimeError> {
    bytes.iter().try_fold(0, |value, &byte| {
        if byte.is_ascii_digit() {
            Ok(value * 10 + u32::from(byte - b'0'))
        } else {
            Err(CivilTimeError::InvalidTimestamp)
        }
    })
}

fn parse_fraction(tail: &[u8]) -> Result<(u32, &[u8]), CivilTimeError> {
    let Some(fraction) = tail.strip_prefix(b".") else {
        return Ok((0, tail));
    };
    let length = fraction
        .iter()
        .take_while(|digit| digit.is_ascii_digit())
        .count();
    if !(1..=9).contains(&length) {
        return Err(CivilTimeError::InvalidFraction);
    }
    let (number, offset) = fraction.split_at(length);
    let length = u32::try_from(length).map_err(|_| CivilTimeError::InvalidFraction)?;
    Ok((digits(number)? * 10_u32.pow(9 - length), offset))
}

fn parse_offset(offset: &[u8]) -> Result<i64, CivilTimeError> {
    if matches!(offset, [b'Z' | b'z']) {
        return Ok(0);
    }
    let [sign @ (b'+' | b'-'), _, _, b':', _, _] = offset else {
        return Err(CivilTimeError::InvalidTimestamp);
    };
    let hours = digits(&offset[1..3])?;
    let minutes = digits(&offset[4..6])?;
    if hours > 23 || minutes > 59 {
        return Err(CivilTimeError::InvalidTimestamp);
    }
    let seconds = i64::from(hours * 3600 + minutes * 60);
    Ok(if *sign == b'-' { -seconds } else { seconds })
}
