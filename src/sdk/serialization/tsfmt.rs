//! Byte-compatible chrono→jiff RFC 3339 rendering (FIND-652).
//!
//! Persisted surfaces (postcard payloads of `Value`/`FieldValue`, payload-index
//! keys `d:`/`ld:`, export/import JSONL lines) were written by chrono 0.4.x and
//! must round-trip byte-identically after the migration to `jiff::Timestamp`:
//!
//! - `chrono::DateTime<Utc>::serialize` is an unconditional `collect_str` of
//!   `write_rfc3339(AutoSi, use_z=true)` — a string, in binary formats too.
//!   `jiff::Timestamp::serialize` is also `collect_str`, but of jiff's
//!   minimal-precision `Display` — the emitted bytes differ (e.g. chrono emits
//!   `.001500` for 1.5ms, jiff `.0015`). The `serialize_with` helpers below pin
//!   the chrono text exactly.
//! - The read path needs no wrapper: chrono's deserializer is a plain
//!   `deserialize_str` + RFC 3339 parse, which jiff's `Timestamp` `Deserialize`
//!   accepts (any offset form, `Z` included).
//!
//! Chrono `SecondsFormat::AutoSi` semantics reproduced here (chrono-0.4.45
//! `src/format/formatting.rs::write_rfc3339`): no fraction when `nano == 0`,
//! 3 digits when `nano % 1_000_000 == 0`, 6 digits when `nano % 1_000 == 0`,
//! else 9 digits. `Millis`/`Micros`/`Secs` are fixed-width truncations.
//! All values here are UTC instants, so the offset renders as `Z`
//! (`use_z = true`) or `+00:00` (`use_z = false`, chrono `to_rfc3339()`).

use core::fmt::Write as _;
use jiff::civil::DateTime as CivilDateTime;
use jiff::tz::TimeZone;
use jiff::Timestamp;

fn civil_utc(ts: Timestamp) -> CivilDateTime {
    ts.to_zoned(TimeZone::UTC).datetime()
}

/// `YYYY-MM-DDTHH:MM:SS` — chrono writes the year as two `write_hundreds`
/// pairs inside `0..=9999` and ISO-8601 signed `{year:+05}` outside it.
fn write_date_time(out: &mut String, dt: CivilDateTime) {
    let year = dt.year();
    if (0..=9999).contains(&year) {
        let _ = write!(out, "{year:04}");
    } else {
        let _ = write!(out, "{year:+05}");
    }
    let _ = write!(
        out,
        "-{:02}-{:02}T{:02}:{:02}:{:02}",
        dt.month(),
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second()
    );
}

/// Fractional part per chrono `SecondsFormat::AutoSi` (0/3/6/9 digits).
fn write_autosi_fraction(out: &mut String, nano: i32) {
    if nano == 0 {
    } else if nano % 1_000_000 == 0 {
        let _ = write!(out, ".{:03}", nano / 1_000_000);
    } else if nano % 1_000 == 0 {
        let _ = write!(out, ".{:06}", nano / 1_000);
    } else {
        let _ = write!(out, ".{nano:09}");
    }
}

/// chrono `DateTime::to_rfc3339()` — AutoSi fraction, `+00:00` offset.
pub fn to_rfc3339(ts: Timestamp) -> String {
    let dt = civil_utc(ts);
    let mut out = String::with_capacity(32);
    write_date_time(&mut out, dt);
    write_autosi_fraction(&mut out, dt.subsec_nanosecond());
    out.push_str("+00:00");
    out
}

/// chrono `to_rfc3339_opts(SecondsFormat::AutoSi, true)` — serde write form.
pub fn rfc3339_autosi_z(ts: Timestamp) -> String {
    let dt = civil_utc(ts);
    let mut out = String::with_capacity(32);
    write_date_time(&mut out, dt);
    write_autosi_fraction(&mut out, dt.subsec_nanosecond());
    out.push('Z');
    out
}

/// chrono `to_rfc3339_opts(SecondsFormat::Secs, true)` — no fraction, `Z`.
pub fn rfc3339_secs_z(ts: Timestamp) -> String {
    let mut out = String::with_capacity(20);
    write_date_time(&mut out, civil_utc(ts));
    out.push('Z');
    out
}

/// chrono `to_rfc3339_opts(SecondsFormat::Millis, true)` — `.sssZ` (truncated).
pub fn rfc3339_millis_z(ts: Timestamp) -> String {
    let dt = civil_utc(ts);
    let mut out = String::with_capacity(24);
    write_date_time(&mut out, dt);
    let _ = write!(out, ".{:03}Z", dt.subsec_nanosecond() / 1_000_000);
    out
}

/// chrono `to_rfc3339_opts(SecondsFormat::Micros, true)` — `.ssssssZ` (truncated).
/// Used by the persisted payload-index keys (`d:`/`ld:`) and canonical reprs.
pub fn rfc3339_micros_z(ts: Timestamp) -> String {
    let dt = civil_utc(ts);
    let mut out = String::with_capacity(27);
    write_date_time(&mut out, dt);
    let _ = write!(out, ".{:06}Z", dt.subsec_nanosecond() / 1_000);
    out
}

/// `serialize_with` for `Timestamp` fields/variants: emits the chrono
/// `Serialize` bytes exactly (`collect_str` of RFC 3339 AutoSi + `Z`).
/// The matching `deserialize` is jiff's own `Timestamp` impl (RFC 3339 parse).
pub fn ser_ts<S: serde::Serializer>(ts: &Timestamp, s: S) -> Result<S::Ok, S::Error> {
    s.collect_str(&rfc3339_autosi_z(*ts))
}

/// `serialize_with` for `Vec<Timestamp>` fields/variants: seq of the same
/// chrono-formatted strings — byte-identical to a derived `Vec<DateTime<Utc>>`.
pub fn ser_ts_vec<S: serde::Serializer>(vts: &[Timestamp], s: S) -> Result<S::Ok, S::Error> {
    use serde::Serialize;
    vts.iter()
        .map(|ts| rfc3339_autosi_z(*ts))
        .collect::<Vec<String>>()
        .serialize(s)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    fn ts(secs: i64, nanos: i32) -> Timestamp {
        Timestamp::new(secs, nanos).expect("in-range timestamp")
    }

    /// Golden strings produced by chrono-0.4.45 `Serialize`/`to_rfc3339*` —
    /// the bytes persisted stores already contain. AutoSi: 0/3/6/9 digits.
    #[test]
    fn autosi_z_matches_chrono_goldens() {
        let cases = [
            // Exact second — no fractional part at all.
            (0, 0, "1970-01-01T00:00:00Z"),
            (1_700_000_000, 0, "2023-11-14T22:13:20Z"),
            // Millisecond-aligned → 3 digits.
            (1_700_000_000, 123_000_000, "2023-11-14T22:13:20.123Z"),
            (1_700_000_000, 500_000_000, "2023-11-14T22:13:20.500Z"),
            // Microsecond-aligned → 6 digits (jiff Display would emit ".0015").
            (1_700_000_000, 1_500_000, "2023-11-14T22:13:20.001500Z"),
            (1_700_000_000, 123_456_000, "2023-11-14T22:13:20.123456Z"),
            // Arbitrary nanos → 9 digits.
            (1_700_000_000, 123_456_789, "2023-11-14T22:13:20.123456789Z"),
            (1_700_000_000, 1, "2023-11-14T22:13:20.000000001Z"),
            (1_700_000_000, 100, "2023-11-14T22:13:20.000000100Z"),
            (1_700_000_000, 10_000, "2023-11-14T22:13:20.000010Z"),
            // Pre-epoch instant (floor decomposition, positive subsec).
            (-1, 500_000_000, "1969-12-31T23:59:59.500Z"),
            (-62_135_596_800, 0, "0001-01-01T00:00:00Z"),
        ];
        for (secs, nanos, expected) in cases {
            assert_eq!(
                rfc3339_autosi_z(ts(secs, nanos)),
                expected,
                "{secs}/{nanos}"
            );
        }
    }

    #[test]
    fn offset_form_matches_chrono_to_rfc3339() {
        // `to_rfc3339()` uses `+00:00` (not `Z`) for UTC.
        assert_eq!(
            to_rfc3339(ts(1_700_000_000, 123_456_789)),
            "2023-11-14T22:13:20.123456789+00:00"
        );
        assert_eq!(to_rfc3339(ts(0, 0)), "1970-01-01T00:00:00+00:00");
    }

    /// Postcard write-path contract: `Value::DateTime` must emit the exact
    /// bytes chrono's `Serialize` produced — variant index `4` (u32 varint),
    /// then `str` = varint len + RFC 3339 AutoSi + `Z` payload. Reading is
    /// covered by `compat_v090` fixtures; this pins the writer.
    #[test]
    fn postcard_datetime_bytes_match_chrono_encoding() {
        let dt: Timestamp = "2024-05-01T12:00:00Z".parse().unwrap();
        let bytes = postcard::to_stdvec(&crate::sdk::Value::DateTime(dt)).unwrap();
        let text = "2024-05-01T12:00:00Z";
        let mut expected = vec![0x04u8, text.len() as u8];
        expected.extend_from_slice(text.as_bytes());
        assert_eq!(bytes, expected);

        // ListDateTime: variant index 9, then vec len + per-element strings.
        let dt2: Timestamp = "2024-05-01T12:00:00.500Z".parse().unwrap();
        let bytes = postcard::to_stdvec(&crate::sdk::Value::ListDateTime(vec![dt, dt2])).unwrap();
        let mut expected = vec![0x09u8, 0x02, text.len() as u8];
        expected.extend_from_slice(text.as_bytes());
        expected.push("2024-05-01T12:00:00.500Z".len() as u8);
        expected.extend_from_slice(b"2024-05-01T12:00:00.500Z");
        assert_eq!(bytes, expected);

        // Same for the node-level FieldValue (variant index 4 / 9 there too).
        let bytes = postcard::to_stdvec(&crate::node::FieldValue::DateTime(dt)).unwrap();
        let mut expected = vec![0x04u8, text.len() as u8];
        expected.extend_from_slice(text.as_bytes());
        assert_eq!(bytes, expected);
    }

    #[test]
    fn fixed_width_variants_truncate_like_chrono() {
        let t = ts(1_700_000_000, 123_456_789);
        // Millis/Micros truncate sub-second digits — never round.
        assert_eq!(rfc3339_millis_z(t), "2023-11-14T22:13:20.123Z");
        assert_eq!(rfc3339_micros_z(t), "2023-11-14T22:13:20.123456Z");
        assert_eq!(rfc3339_secs_z(t), "2023-11-14T22:13:20Z");
        // Micros always pads to 6 even for whole seconds.
        assert_eq!(rfc3339_micros_z(ts(0, 0)), "1970-01-01T00:00:00.000000Z");
        assert_eq!(rfc3339_millis_z(ts(0, 0)), "1970-01-01T00:00:00.000Z");
    }
}
