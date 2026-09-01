# Panel
panel-week = W
panel-month = M
panel-error = Err

# Popup
popup-rolling = 5h
popup-week = W
popup-month = M
resets-in = Setzt zurück in { $d }
updated-at = Aktualisiert: { $t }
error-line = Fehler: { $e }
no-data = Keine Daten
quit = Beenden

# Errors
error-no-api-key = Kein OpenCode-Go-API-Key (auth.json oder OPENCODE_API_KEY)

# Durations — "Tagen" im Dativ (Kontext "setzt zurück in ...")
duration-day =
    { $n ->
        [one] 1 Tag
       *[other] { $n } Tagen
    }
duration-hour =
    { $n ->
        [one] 1 Stunde
       *[other] { $n } Stunden
    }
duration-minute =
    { $n ->
        [one] 1 Minute
       *[other] { $n } Minuten
    }
duration-second =
    { $n ->
        [one] 1 Sekunde
       *[other] { $n } Sekunden
    }
