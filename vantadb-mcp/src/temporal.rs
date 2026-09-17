//! FIND-103: deterministic ES/EN temporal-expression → Unix-ms range translator.
//!
//! The agent must NEVER improvise a time range ("ayer a las 2pm" is not a vibe,
//! it is a `[from_ms, to_ms]` interval). This module is the single deterministic
//! implementation of that mapping: pure function, no I/O, no chrono dependency
//! (civil-date math is Hinnant's days↔civil algorithm —
//! https://howardhinnant.github.io/date_algorithms.html — proven by the
//! round-trip test in `tests/temporal_tests.rs`, not by authority).
//!
//! ## Contract
//!
//! - Input: free text in Spanish or English, case-insensitive, accents optional.
//! - Anchor: `now_ms` (Unix-ms, UTC). All days/weeks/months are UTC.
//! - Output: `Some((from_ms, to_ms))` with INCLUSIVE bounds on both ends — the
//!   same convention as `graph_traverse`'s `time_range` — or `None` when the
//!   expression is unresolvable or points strictly to the future.
//! - Digits only for quantities (`hace 3 días`, not `hace tres días`).
//! - Policy fallback (unresolvable → last 30 days + explicit warning) lives in
//!   `skills/vantadb-mcp/references/recall-policy.md`, not here: this function
//!   reports `None` honestly and lets the caller decide.
//!
//! ## Executable mapping (v1, verified — see FIND-103 task file §4 decisión 5)
//!
//! The range feeds `memory_list` pagination filtered client-side on
//! `created_at_ms` (`matches_advanced_filters` is metadata-only, so there is no
//! server-side timestamp filter), or `graph_traverse` `time_range` for edge
//! windows. `search_memory` filters are equality-only and cannot express this.

const DAY_MS: u64 = 86_400_000;
const HOUR_MS: u64 = 3_600_000;
const MIN_MS: u64 = 60_000;

/// Policy default for the unresolvable-expression fallback (last N days).
/// The recall policy (`recall-policy.md`) owns the behaviour; this const only
/// names the number so hosts and docs cannot drift apart.
pub const FALLBACK_LAST_DAYS: u64 = 30;

/// Translate `expr` into an INCLUSIVE `[from_ms, to_ms]` Unix-ms range anchored
/// at `now_ms` (UTC). Returns `None` when unresolvable or future-only.
///
/// ```
/// # use vantadb_mcp::parse_temporal_expression;
/// let now = 1_800_000_000_000u64; // arbitrary anchor; only structure is asserted
/// let (from, to) = parse_temporal_expression("ayer", now).unwrap();
/// assert_eq!(to - from + 1, 86_400_000); // exactly one full day
/// assert_eq!(from % 86_400_000, 0); // day-aligned (UTC)
/// assert!(to < now);
/// ```
pub fn parse_temporal_expression(expr: &str, now_ms: u64) -> Option<(u64, u64)> {
    let norm = normalize(expr);
    if norm.is_empty() {
        return None;
    }
    let words: Vec<&str> = norm.split_whitespace().collect();

    // 1. Explicit day: YYYY-MM-DD (validated by round-trip, future → None).
    if let Some((y, m, d)) = parse_ymd(&norm) {
        let days = days_from_civil(y, m, d);
        if civil_from_days(days) != (y, m, d) {
            return None; // e.g. month 13 or Feb 30
        }
        if days < 0 {
            return None;
        }
        let start = (days as u64).checked_mul(DAY_MS)?;
        let end = start.checked_add(DAY_MS - 1)?;
        if start > now_ms {
            return None; // strictly in the future
        }
        return Some((start, end.min(now_ms).max(start)));
    }

    // 2. Relative days with optional hour slot.
    if let Some(rest) = strip_any(&norm, &["anteayer", "day before yesterday"]) {
        return match parse_hour(rest) {
            Some(h) => hour_slot(offset_day_start(now_ms, 2)?, h),
            None if rest.trim().is_empty() => full_day(offset_day_start(now_ms, 2)?),
            None => None,
        };
    }
    if let Some(rest) = strip_any(&norm, &["ayer", "yesterday"]) {
        return match parse_hour(rest) {
            Some(h) => hour_slot(offset_day_start(now_ms, 1)?, h),
            None if rest.trim().is_empty() => full_day(offset_day_start(now_ms, 1)?),
            None => None,
        };
    }
    if let Some(rest) = strip_any(&norm, &["hoy", "today"]) {
        return match parse_hour(rest) {
            Some(h) => hour_slot_capped(day_start(now_ms), h, now_ms),
            None if rest.trim().is_empty() => Some((day_start(now_ms), now_ms)),
            None => None,
        };
    }

    // 3. Weeks.
    if contains_any(&norm, &["esta semana", "this week"]) {
        return Some((week_monday(now_ms), now_ms));
    }
    if contains_any(&norm, &["la semana pasada", "semana pasada", "last week"]) {
        let this_mon = week_monday(now_ms);
        let prev_mon = this_mon.checked_sub(7 * DAY_MS)?;
        return Some((prev_mon, prev_mon.checked_add(7 * DAY_MS - 1)?));
    }

    // 4. Months.
    if contains_any(&norm, &["este mes", "this month"]) {
        return Some((month_start(now_ms)?, now_ms));
    }
    if contains_any(&norm, &["el mes pasado", "mes pasado", "last month"]) {
        let (y, m, _) = civil_from_days(day_index(now_ms));
        let (py, pm) = if m == 1 { (y - 1, 12) } else { (y, m - 1) };
        if py < 1970 {
            return None;
        }
        let start = (days_from_civil(py, pm, 1) as u64).checked_mul(DAY_MS)?;
        let len = month_len_days(py, pm)?.checked_mul(DAY_MS)?;
        return Some((start, start.checked_add(len - 1)?));
    }

    // 5. "hace N <unit>" / "N <unit> ago" / "últimos N <unit>" / "last N <unit>".
    if let Some((n, unit_ms)) = parse_ago(&words) {
        let span = n.checked_mul(unit_ms)?;
        return Some((now_ms.saturating_sub(span), now_ms));
    }

    // 6. Weekday names (most recent on-or-before today; "pasado/last" → one week earlier).
    if let Some((wd, past)) = parse_weekday(&norm) {
        let base = most_recent_weekday(now_ms, wd)?;
        let start = if past {
            base.checked_sub(7 * DAY_MS)?
        } else {
            base
        };
        return full_day(start);
    }

    // 7. Bare hour ("a las 14", "at 2pm"): today when already started, else yesterday.
    if let Some(h) = parse_hour(&norm) {
        let today_slot = day_start(now_ms).checked_add(h as u64 * HOUR_MS)?;
        if today_slot <= now_ms {
            return hour_slot_capped(day_start(now_ms), h, now_ms);
        }
        return hour_slot(offset_day_start(now_ms, 1)?, h);
    }

    None
}

// ── normalization ────────────────────────────────────────────────────────────

/// Lowercase, de-accent vowels (accents optional in input), drop punctuation
/// except `:`/`-` (needed by hours and YYYY-MM-DD), collapse whitespace.
fn normalize(expr: &str) -> String {
    let mut out = String::with_capacity(expr.len());
    for c in expr.trim().to_lowercase().chars() {
        let c = match c {
            'á' | 'à' | 'ä' => 'a',
            'é' | 'è' | 'ë' => 'e',
            'í' | 'ì' | 'ï' => 'i',
            'ó' | 'ò' | 'ö' => 'o',
            'ú' | 'ù' | 'ü' => 'u',
            'ñ' => 'ñ',
            _ => c,
        };
        if c.is_alphanumeric() || c == ':' || c == '-' || c == '/' || c == '.' {
            out.push(c);
        } else {
            out.push(' ');
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn contains_any(hay: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| hay.contains(n))
}

/// If `hay` equals a needle or starts with `needle + " "` — allowing a leading
/// "el "/"the " — return the remainder after it.
fn strip_any<'a>(hay: &'a str, needles: &[&str]) -> Option<&'a str> {
    let hay = hay.strip_prefix("el ").unwrap_or(hay);
    let hay = hay.strip_prefix("the ").unwrap_or(hay);
    needles.iter().find_map(|n| {
        if hay == *n {
            Some("")
        } else {
            hay.strip_prefix(n)
                .and_then(|rest| rest.strip_prefix(' '))
                .map(str::trim_start)
        }
    })
}

// ── day arithmetic (UTC, std-only) ───────────────────────────────────────────

fn day_index(now_ms: u64) -> i64 {
    (now_ms / DAY_MS) as i64
}

fn day_start(now_ms: u64) -> u64 {
    now_ms - (now_ms % DAY_MS)
}

fn offset_day_start(now_ms: u64, days_back: u64) -> Option<u64> {
    day_start(now_ms).checked_sub(days_back.checked_mul(DAY_MS)?)
}

fn full_day(start: u64) -> Option<(u64, u64)> {
    Some((start, start.checked_add(DAY_MS - 1)?))
}

fn hour_slot(day: u64, hour: u32) -> Option<(u64, u64)> {
    let start = day.checked_add(hour as u64 * HOUR_MS)?;
    Some((start, start.checked_add(HOUR_MS - 1)?))
}

/// Hour slot capped at `now_ms`; `None` when the hour has not started yet.
fn hour_slot_capped(day: u64, hour: u32, now_ms: u64) -> Option<(u64, u64)> {
    let (from, to) = hour_slot(day, hour)?;
    if from > now_ms {
        return None;
    }
    Some((from, to.min(now_ms)))
}

/// Monday (00:00 UTC) of the week containing `now_ms`.
fn week_monday(now_ms: u64) -> u64 {
    let days = day_index(now_ms);
    let wd = ((days + 3) % 7 + 7) % 7; // Monday=0 (1970-01-01 was Thursday)
    day_start(now_ms).saturating_sub(wd as u64 * DAY_MS)
}

/// Most recent `target` weekday (Monday=0) on or before today, at 00:00.
fn most_recent_weekday(now_ms: u64, target: u32) -> Option<u64> {
    let days = day_index(now_ms);
    let wd = (((days + 3) % 7 + 7) % 7) as u32;
    let back = (wd + 7 - target) % 7;
    day_start(now_ms).checked_sub(back as u64 * DAY_MS)
}

fn month_start(now_ms: u64) -> Option<u64> {
    let (y, m, _) = civil_from_days(day_index(now_ms));
    let days = days_from_civil(y, m, 1);
    if days < 0 {
        return None;
    }
    Some(days as u64 * DAY_MS)
}

fn month_len_days(y: i32, m: u32) -> Option<u64> {
    let (ny, nm) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
    u64::try_from(days_from_civil(ny, nm, 1) - days_from_civil(y, m, 1)).ok()
}

// ── civil-date math (Hinnant's algorithms, std-only) ─────────────────────────

/// Days since 1970-01-01 for a civil date (may be negative pre-1970).
fn days_from_civil(y: i32, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y as i64 - 1 } else { y as i64 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400; // [0, 399]
    let mp = (m as i64 + 9) % 12; // [0, 11]
    let doy = (153 * mp + 2) / 5 + d as i64 - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146097 + doe - 719468
}

/// Civil date for days since 1970-01-01.
fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 {
        (mp + 3) as u32
    } else {
        (mp - 9) as u32
    }; // [1, 12]
    (if m <= 2 { (y + 1) as i32 } else { y as i32 }, m, d)
}

// ── token parsers ────────────────────────────────────────────────────────────

/// Strict `YYYY-MM-DD` (also accepts `/` and `.` separators post-normalize).
fn parse_ymd(norm: &str) -> Option<(i32, u32, u32)> {
    let digits: String = norm
        .chars()
        .map(|c| if c == '/' || c == '.' { '-' } else { c })
        .collect();
    let parts: Vec<&str> = digits.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let (y, m, d) = (
        parts[0].parse::<i32>().ok()?,
        parts[1].parse::<u32>().ok()?,
        parts[2].parse::<u32>().ok()?,
    );
    if !(1000..=9999).contains(&y) || !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    Some((y, m, d))
}

/// Hour from a remainder like "a las 2pm", "at 14", "14:30", "las 9 am".
/// Token-based (connector words skipped, unknown words refuse) so substrings
/// of other words can never corrupt the parse. Returns 0-23, or `None` when
/// no hour token is present.
fn parse_hour(rest: &str) -> Option<u32> {
    let mut time_tok: Option<&str> = None;
    let mut meridiem: Option<bool> = None; // true = am, false = pm
    for w in rest.split_whitespace() {
        match w {
            "a" | "las" | "la" | "at" | "o'clock" | "de" | "en" => {}
            "am" => meridiem = Some(true),
            "pm" => meridiem = Some(false),
            _ if w.ends_with("am") || w.ends_with("pm") => {
                if time_tok.is_none() {
                    time_tok = Some(w);
                }
            }
            _ if w.chars().next().is_some_and(|c| c.is_ascii_digit()) => {
                if time_tok.is_none() {
                    time_tok = Some(w);
                }
            }
            _ => return None, // unknown word → not an hour expression
        }
    }
    let tok = time_tok?;
    let (num_part, glued) = if let Some(h) = tok.strip_suffix("pm") {
        (h, Some(false))
    } else if let Some(h) = tok.strip_suffix("am") {
        (h, Some(true))
    } else {
        (tok, None)
    };
    let hour_token = num_part.split(':').next()?.trim();
    if hour_token.is_empty() || !hour_token.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let h: u32 = hour_token.parse().ok()?;
    match glued.or(meridiem) {
        Some(true) => {
            // am: 12am → 0
            if !(1..=12).contains(&h) {
                return None;
            }
            Some(h % 12)
        }
        Some(false) => {
            // pm: 12pm → 12
            if !(1..=12).contains(&h) {
                return None;
            }
            Some(h % 12 + 12)
        }
        None => {
            if h > 23 {
                return None;
            }
            Some(h)
        }
    }
}

/// Relative spans: "hace N días|horas|minutos|semanas", "N <unit> ago",
/// "últimos|ultimos N <unit>", "last N <unit>". Returns (n, unit_ms).
fn parse_ago(words: &[&str]) -> Option<(u64, u64)> {
    let unit = |w: &str| match w {
        "minuto" | "minutos" | "minute" | "minutes" | "min" => Some(MIN_MS),
        "hora" | "horas" | "hour" | "hours" => Some(HOUR_MS),
        "dia" | "dias" | "day" | "days" => Some(DAY_MS),
        "semana" | "semanas" | "week" | "weeks" => Some(7 * DAY_MS),
        "mes" | "meses" | "month" | "months" => Some(30 * DAY_MS),
        _ => None,
    };
    // hace N <unit> | N <unit> ago | últimos N <unit> | last N <unit>
    if words.len() == 3 {
        if words[0] == "hace" {
            return Some((words[1].parse().ok()?, unit(words[2])?));
        }
        if words[2] == "ago" {
            return Some((words[0].parse().ok()?, unit(words[1])?));
        }
        if words[0] == "ultimos" || words[0] == "last" {
            return Some((words[1].parse().ok()?, unit(words[2])?));
        }
    }
    None
}

/// Weekday → Monday=0 index + whether a past marker ("pasado/a", "last")
/// forces the previous occurrence. Accepts optional "el"/"este" prefix.
fn parse_weekday(norm: &str) -> Option<(u32, bool)> {
    let past = contains_any(norm, &["pasado", "pasada", "last"]);
    let t = norm
        .replace("pasado", " ")
        .replace("pasada", " ")
        .replace("last", " ")
        .replace("el ", " ")
        .replace("este ", " ")
        .replace("this ", " ");
    let t = t.trim();
    // Reject multi-word leftovers ("el lunes a las 3" is hourly, handled elsewhere;
    // here only a bare weekday qualifies).
    if t.contains(' ') {
        // Allow "lunes pasado"-style trailing junk already stripped; anything
        // else with spaces is not a bare weekday.
        return None;
    }
    let wd = match t {
        "lunes" | "monday" | "mon" => 0,
        "martes" | "tuesday" | "tue" => 1,
        "miercoles" | "wednesday" | "wed" => 2,
        "jueves" | "thursday" | "thu" => 3,
        "viernes" | "friday" | "fri" => 4,
        "sabado" | "saturday" | "sat" => 5,
        "domingo" | "sunday" | "sun" => 6,
        "lun" | "mar" | "mie" | "jue" | "vie" | "sab" | "dom" => {
            return None; // ambiguous ES abbreviations — refuse, don't guess
        }
        _ => return None,
    };
    Some((wd, past))
}
