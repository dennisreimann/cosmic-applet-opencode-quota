use cosmic_applet_opencode_quota::applet::{
    fmt_reset_in, fmt_updated, format_duration, panel_line, panel_segments, percent_label,
    percent_used, popup_rows, usage_level, UsageLevel,
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
    assert_eq!(rows[0].name, "5h");
    assert_eq!(rows[0].percent, "3%");
    assert!(rows[0].reset.as_deref().unwrap().starts_with("Resets in "));
    assert_eq!(rows[1].name, "W");
    assert_eq!(rows[1].percent, "22%");
    assert!(rows[1].reset.as_deref().unwrap().starts_with("Resets in "));
    assert_eq!(rows[2].name, "M");
    assert_eq!(rows[2].percent, "11%");
    assert!(rows[2].reset.as_deref().unwrap().starts_with("Resets in "));
}

#[test]
fn popup_rolling_reset_hidden_when_unused() {
    force_en();
    // 5h-Fenster völlig unangetastet: kein Reset-Text (der Timer läuft
    // trotzdem, ist aber ohne Verbrauch nicht aussagekräftig).
    let mut s = snap();
    s.rolling.percent_remaining = 100.0;
    let rows = popup_rows(&s);
    assert_eq!(rows[0].percent, "0%");
    assert_eq!(rows[0].reset, None);
    // Woche/Monat behalten ihren Reset auch bei 0% Verbrauch (Periodenende).
    s.weekly.percent_remaining = 100.0;
    s.monthly.percent_remaining = 100.0;
    let rows = popup_rows(&s);
    assert!(rows[1].reset.is_some());
    assert!(rows[2].reset.is_some());
}

#[test]
fn usage_level_thresholds() {
    // unter 80% verbraucht: normal
    assert_eq!(usage_level(97.0), UsageLevel::Normal); // 3% used
    assert_eq!(usage_level(20.6), UsageLevel::Normal); // 79.4 → zeigt "79%"
    assert_eq!(usage_level(20.5), UsageLevel::Warning); // 79.5 → rundet auf 80
    // ab 80% verbraucht: gelb
    assert_eq!(usage_level(20.0), UsageLevel::Warning); // exakt 80
    assert_eq!(usage_level(15.0), UsageLevel::Warning); // 85
    // ab 100% verbraucht: rot
    assert_eq!(usage_level(0.0), UsageLevel::Critical); // 100
    // Rundung entspricht dem ANGEZEIGTEN Wert
    assert_eq!(usage_level(20.4), UsageLevel::Warning); // 79.6 → zeigt "80%"
    assert_eq!(usage_level(0.4), UsageLevel::Critical); // 99.6 → zeigt "100%"
    assert_eq!(usage_level(21.0), UsageLevel::Normal); // 79.0 → zeigt "79%"
}

#[test]
fn panel_segments_color_percentages_only() {
    force_en();
    let mut s = snap();
    s.rolling.percent_remaining = 85.0; // 15% used → normal
    s.weekly.percent_remaining = 15.0; // 85% used → gelb
    s.monthly.percent_remaining = 0.0; // 100% used → rot
    let segs = panel_segments(&s);
    assert_eq!(
        segs,
        vec![
            ("15%".to_string(), UsageLevel::Normal),
            (" · ".to_string(), UsageLevel::Normal),
            ("85%".to_string(), UsageLevel::Warning),
            (" W · ".to_string(), UsageLevel::Normal),
            ("100%".to_string(), UsageLevel::Critical),
            (" M".to_string(), UsageLevel::Normal),
        ]
    );
    // Zusammengesetzt ergibt das weiterhin die bekannte Zeile.
    let joined: String = segs.iter().map(|(t, _)| t.as_str()).collect();
    assert_eq!(joined, panel_line(&s));
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
