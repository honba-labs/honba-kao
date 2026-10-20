//! Calendar arithmetic shared by pairing, analytics and the app's chart plumbing.
//!
//! Pure integer civil-date maths (Howard Hinnant's algorithms) and the IST offset the
//! simulator buckets by — no `Intl`, no time-zone database. UTC+5:30 has no DST, so a fixed
//! offset is exact.

use honba_messages::UnixNanos;

/// IST is the session the Simulator charts; UTC+5:30 has no DST.
pub const IST_OFFSET_SECONDS: i64 = 5 * 3600 + 1800;

/// `ts_event` → whole UTC seconds, by exact integer division. `None` only for the epoch
/// placeholder.
#[must_use]
pub fn unix_nanos_to_seconds(ts: UnixNanos) -> Option<i64> {
    let nanos = ts.as_u64();
    if nanos == 0 {
        return None;
    }
    Some((nanos / 1_000_000_000) as i64)
}

/// Civil date (year, month 1-12, day 1-31) of the day index
/// `days since 1970-01-01` (Hinnant's `civil_from_days`).
#[must_use]
pub fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    (if m <= 2 { y + 1 } else { y }, m as u32, d as u32)
}

/// Day index `days since 1970-01-01` of a civil date (Hinnant's
/// `days_from_civil`). Month must be 1-12; the day may overflow (e.g. 32),
/// the caller normalises first.
#[must_use]
pub fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400; // [0, 399]
    let mp = if m > 2 { m - 3 } else { m + 9 } as i64; // [0, 11]
    let doy = (153 * mp + 2) / 5 + d as i64 - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146_097 + doe - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 2024-01-01 00:00 UTC.
    const JAN1_2024: i64 = 1_704_067_200;

    #[test]
    fn reads_whole_seconds_off_the_wire_timestamps() {
        // 1.7e18 ns is a u64 — the seconds are an exact integer division on
        // the Rust side (fe sliced the string to dodge Number precision).
        let ts = UnixNanos::from_u64(1_700_000_000_250_000_000);
        assert_eq!(unix_nanos_to_seconds(ts), Some(1_700_000_000));
        // The epoch placeholder reads as absent.
        assert_eq!(unix_nanos_to_seconds(UnixNanos::from_u64(0)), None);
    }

    #[test]
    fn civil_helpers_round_trip() {
        // Sanity of the civil helpers the IST bucketing leans on.
        let (y, m, d) = civil_from_days((JAN1_2024).div_euclid(86_400));
        assert_eq!((y, m, d), (2024, 1u32, 1u32));
        assert_eq!(days_from_civil(2024, 1, 1), JAN1_2024.div_euclid(86_400));
    }
}
