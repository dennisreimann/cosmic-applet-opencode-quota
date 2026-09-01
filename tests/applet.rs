use cosmic_applet_opencode_quota::applet::{
    fmt_reset_in, fmt_updated, format_duration, panel_line, percent_label, percent_used, popup_rows,
};
use cosmic_applet_opencode_quota::quota::{QuotaSnapshot, QuotaWindow};
use std::sync::Once;

static EN_LOCALE: Once = Once::new();

/// Deterministische Sprache für diesen Testprozess: EN (Quell-Locale),
/// unabhängig von der Desktop-Locale des Rechners. Muss vor dem ersten
/// Format-Funktionsaufruf laufen (Once); der Loader initialisiert beim
/// ersten Zugriff.
fn force_en() {
    EN_LOCALE.call_once(|| std::env::set_var("I18N_LANGUAGE", "en"));
}

fn snap() -> QuotaSnapshot {
    QuotaSnapshot {
        rolling: QuotaWindow {
            percent_remaining: 97.0,
            resets_at: "2026-09-01T18:49:23.763Z".into(),
        },
        weekly: QuotaWindow {
            percent_remaining: 78.0,
            resets_at: "2026-09-07T00:00:00.763Z".into(),
        },
        monthly: QuotaWindow {
            percent_remaining: 89.0,
            resets_at: "2026-09-30T11:51:07.763Z".into(),
        },
        fetched_at_unix: 1_788_278_419,
    }
}

#[test]
fn label_rounds() {
    assert_eq!(percent_label(42.3), "42%");
    assert_eq!(percent_label(5.0), "5%");
    assert_eq!(percent_label(97.6), "98%");
}

#[test]
fn used_is_inverse_of_remaining() {
    assert_eq!(percent_used(97.0), 3.0);
    assert_eq!(percent_used(0.0), 100.0);
    assert_eq!(percent_used(100.0), 0.0);
    assert_eq!(percent_used(78.0), 22.0);
    // negative/über-100 bleiben geclampt
    assert_eq!(percent_used(110.0), 0.0);
    assert_eq!(percent_used(-5.0), 100.0);
}

#[test]
fn panel_line_shows_used() {
    force_en();
    assert_eq!(panel_line(&snap()), "3% · 22% W · 11% M");
}

#[test]
fn popup_rows_split_label_and_reset() {
    force_en();
    let rows = popup_rows(&snap());
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].0, "5h: 3%");
    assert!(rows[0].1.starts_with("Resets in "));
    assert_eq!(rows[1].0, "W: 22%");
    assert!(rows[1].1.starts_with("Resets in "));
    assert_eq!(rows[2].0, "M: 11%");
    assert!(rows[2].1.starts_with("Resets in "));
}

#[test]
fn fmt_reset_in_relative_or_raw() {
    force_en();
    let s = fmt_reset_in("2026-09-07T00:00:00Z");
    assert!(
        s.contains("hour") || s.contains("day") || s.contains("minute") || s.contains("second"),
        "erwartete englische Dauer, got {s}"
    );
    assert_eq!(fmt_reset_in("kaputt"), "kaputt");
}

#[test]
fn format_duration_english_plural() {
    force_en();
    assert_eq!(format_duration(1), "1 second");
    assert_eq!(format_duration(45), "45 seconds");
    // Sekunden werden ab der ersten Minute nicht mehr angezeigt.
    assert_eq!(format_duration(60), "1 minute");
    assert_eq!(format_duration(90), "1 minute");
    assert_eq!(format_duration(120), "2 minutes");
    assert_eq!(format_duration(3600), "1 hour");
    assert_eq!(format_duration(5400), "1 hour 30 minutes");
    assert_eq!(format_duration(7200), "2 hours");
    assert_eq!(format_duration(86_400), "1 day");
    assert_eq!(format_duration(90_000), "1 day 1 hour");
    assert_eq!(format_duration(172_800 + 7_200), "2 days 2 hours");
}

#[test]
fn updated_formats_local_time() {
    // Nur sicherstellen, dass eine lesbare Zeit (mit Doppelpunkt) entsteht.
    let s = fmt_updated(1_788_278_419);
    assert!(s.contains(':'), "erwartete HH:MM:SS-Struktur, got {s}");
}
