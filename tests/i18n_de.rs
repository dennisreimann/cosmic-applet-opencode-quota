static DE_LOCALE: std::sync::Once = std::sync::Once::new();

/// Deterministische Sprache für diesen Testprozess: DE. Muss vor dem ersten
/// Loader-Zugriff laufen (Once); der Loader initialisiert beim ersten Zugriff.
fn force_de() {
    DE_LOCALE.call_once(|| std::env::set_var("I18N_LANGUAGE", "de"));
}

#[test]
fn german_catalog_is_loaded() {
    force_de();
    assert_eq!(
        cosmic_applet_opencode_quota::fl!("no-data"),
        "Keine Daten"
    );
    assert_eq!(cosmic_applet_opencode_quota::fl!("quit"), "Beenden");
    assert_eq!(
        cosmic_applet_opencode_quota::fl!("resets-in", d = "2 Tagen"),
        "Setzt zurück in 2 Tagen"
    );
}
