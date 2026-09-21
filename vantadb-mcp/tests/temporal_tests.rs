// FIND-103: suite del traductor temporal determinista.
// RED→GREEN: estos tests definen el contrato; el primero en fallar señala
// el comportamiento exacto a implementar en `src/temporal.rs`.
// Anchor fijo: jueves 2026-09-17 12:00:00 UTC (determinista, sin reloj).
// `utc_ms` usa aritmética de calendario por loops — implementación
// independiente del algoritmo Hinnant del módulo bajo test.

use vantadb_mcp::{parse_temporal_expression, FALLBACK_LAST_DAYS};

const DAY: u64 = 86_400_000;
const HOUR: u64 = 3_600_000;
const MIN: u64 = 60_000;

fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

/// Unix-ms UTC por loops (independiente de `temporal.rs`).
fn utc_ms(y: i32, m: u32, d: u32, hh: u32, mm: u32) -> u64 {
    let lens = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut days: u64 = 0;
    for yy in 1970..y {
        days += if is_leap(yy) { 366 } else { 365 };
    }
    for mm_ in 1..m {
        days += lens[(mm_ - 1) as usize] as u64;
        if mm_ == 2 && is_leap(y) {
            days += 1;
        }
    }
    days += (d - 1) as u64;
    days * DAY + hh as u64 * HOUR + mm as u64 * MIN
}

/// Jueves 2026-09-17 12:00:00 UTC.
fn now() -> u64 {
    utc_ms(2026, 9, 17, 12, 0)
}

fn day(y: i32, m: u32, d: u32) -> (u64, u64) {
    let s = utc_ms(y, m, d, 0, 0);
    (s, s + DAY - 1)
}

fn hour(y: i32, m: u32, d: u32, h: u32) -> (u64, u64) {
    let s = utc_ms(y, m, d, h, 0);
    (s, s + HOUR - 1)
}

#[test]
fn hoy_is_start_of_today_to_now() {
    assert_eq!(
        parse_temporal_expression("hoy", now()),
        Some((utc_ms(2026, 9, 17, 0, 0), now()))
    );
    assert_eq!(
        parse_temporal_expression("TODAY", now()),
        Some((utc_ms(2026, 9, 17, 0, 0), now()))
    );
}

#[test]
fn ayer_is_full_previous_day() {
    assert_eq!(
        parse_temporal_expression("ayer", now()),
        Some(day(2026, 9, 16))
    );
    assert_eq!(
        parse_temporal_expression("yesterday", now()),
        Some(day(2026, 9, 16))
    );
    assert_eq!(
        parse_temporal_expression("anteayer", now()),
        Some(day(2026, 9, 15))
    );
}

#[test]
fn ayer_with_hour_is_that_hour_slot() {
    assert_eq!(
        parse_temporal_expression("ayer a las 2pm", now()),
        Some(hour(2026, 9, 16, 14))
    );
    assert_eq!(
        parse_temporal_expression("yesterday at 2pm", now()),
        Some(hour(2026, 9, 16, 14))
    );
    assert_eq!(
        parse_temporal_expression("ayer a las 14", now()),
        Some(hour(2026, 9, 16, 14))
    );
}

#[test]
fn hoy_with_hour_slots() {
    assert_eq!(
        parse_temporal_expression("hoy a las 9am", now()),
        Some(hour(2026, 9, 17, 9))
    );
    // 18:00 de hoy aún no empezó (now 12:00) → futuro → None.
    assert_eq!(parse_temporal_expression("hoy a las 6pm", now()), None);
    // 12pm empieza exactamente en now → slot degenerado [now, now].
    assert_eq!(
        parse_temporal_expression("a las 12pm", now()),
        Some((now(), now()))
    );
    assert_eq!(
        parse_temporal_expression("a las 12am", now()),
        Some(hour(2026, 9, 17, 0))
    );
}

#[test]
fn bare_hour_prefers_today_then_yesterday() {
    // 09:00 de hoy ya empezó → hoy.
    assert_eq!(
        parse_temporal_expression("a las 9", now()),
        Some(hour(2026, 9, 17, 9))
    );
    // 14:00 de hoy es futuro (now 12:00) → ayer.
    assert_eq!(
        parse_temporal_expression("a las 2pm", now()),
        Some(hour(2026, 9, 16, 14))
    );
}

#[test]
fn weeks_anchor_on_monday() {
    // Lunes 2026-09-14 → now.
    assert_eq!(
        parse_temporal_expression("esta semana", now()),
        Some((utc_ms(2026, 9, 14, 0, 0), now()))
    );
    assert_eq!(
        parse_temporal_expression("this week", now()),
        Some((utc_ms(2026, 9, 14, 0, 0), now()))
    );
    // Semana pasada completa Lun 7 → Dom 13.
    assert_eq!(
        parse_temporal_expression("la semana pasada", now()),
        Some((utc_ms(2026, 9, 7, 0, 0), utc_ms(2026, 9, 14, 0, 0) - 1))
    );
    assert_eq!(
        parse_temporal_expression("last week", now()),
        Some((utc_ms(2026, 9, 7, 0, 0), utc_ms(2026, 9, 14, 0, 0) - 1))
    );
}

#[test]
fn months_cover_first_to_anchor() {
    assert_eq!(
        parse_temporal_expression("este mes", now()),
        Some((utc_ms(2026, 9, 1, 0, 0), now()))
    );
    // Agosto completo.
    assert_eq!(
        parse_temporal_expression("el mes pasado", now()),
        Some((utc_ms(2026, 8, 1, 0, 0), utc_ms(2026, 9, 1, 0, 0) - 1))
    );
    // Borde año: desde enero, el mes pasado es diciembre del año previo.
    let jan = utc_ms(2026, 1, 15, 12, 0);
    assert_eq!(
        parse_temporal_expression("last month", jan),
        Some((utc_ms(2025, 12, 1, 0, 0), utc_ms(2026, 1, 1, 0, 0) - 1))
    );
}

#[test]
fn relative_spans_count_back_from_now() {
    let n = now();
    assert_eq!(
        parse_temporal_expression("hace 3 días", n),
        Some((n - 3 * DAY, n))
    );
    assert_eq!(
        parse_temporal_expression("3 days ago", n),
        Some((n - 3 * DAY, n))
    );
    assert_eq!(
        parse_temporal_expression("últimos 7 días", n),
        Some((n - 7 * DAY, n))
    );
    assert_eq!(
        parse_temporal_expression("hace 2 horas", n),
        Some((n - 2 * HOUR, n))
    );
    assert_eq!(
        parse_temporal_expression("5 minutes ago", n),
        Some((n - 5 * MIN, n))
    );
}

#[test]
fn weekdays_resolve_to_most_recent() {
    // Jueves 17 → lunes más reciente es el 14.
    assert_eq!(
        parse_temporal_expression("el lunes", now()),
        Some(day(2026, 9, 14))
    );
    assert_eq!(
        parse_temporal_expression("el lunes pasado", now()),
        Some(day(2026, 9, 7))
    );
    assert_eq!(
        parse_temporal_expression("last monday", now()),
        Some(day(2026, 9, 7))
    );
    // Acentos opcionales: miércoles 16.
    assert_eq!(
        parse_temporal_expression("miércoles", now()),
        Some(day(2026, 9, 16))
    );
    // Hoy es jueves → "thursday" resuelve al día completo de hoy.
    assert_eq!(
        parse_temporal_expression("thursday", now()),
        Some(day(2026, 9, 17))
    );
}

#[test]
fn explicit_dates_validate_by_round_trip() {
    assert_eq!(
        parse_temporal_expression("2026-09-16", now()),
        Some(day(2026, 9, 16))
    );
    // 2023 no fue bisiesto → Feb 29 inválido.
    assert_eq!(parse_temporal_expression("2023-02-29", now()), None);
    assert_eq!(
        parse_temporal_expression("2024-02-29", now()),
        Some(day(2024, 2, 29))
    );
    assert_eq!(parse_temporal_expression("2026-13-01", now()), None);
}

#[test]
fn future_and_garbage_are_none() {
    assert_eq!(parse_temporal_expression("mañana", now()), None);
    assert_eq!(parse_temporal_expression("tomorrow", now()), None);
    assert_eq!(parse_temporal_expression("2026-09-18", now()), None);
    assert_eq!(
        parse_temporal_expression("próximamente quizás", now()),
        None
    );
    assert_eq!(parse_temporal_expression("", now()), None);
    assert_eq!(parse_temporal_expression("hace tres días", now()), None); // solo dígitos
}

#[test]
fn fallback_window_is_thirty_days() {
    assert_eq!(FALLBACK_LAST_DAYS, 30);
}
