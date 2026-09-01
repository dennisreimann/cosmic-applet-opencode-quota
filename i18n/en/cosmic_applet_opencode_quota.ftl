# Panel
panel-week = W
panel-month = M
panel-error = Err

# Popup
popup-rolling = 5h
popup-week = W
popup-month = M
resets-in = Resets in { $d }
updated-at = Updated: { $t }
error-line = Error: { $e }
no-data = No data
quit = Quit

# Errors
error-no-api-key = No OpenCode Go API key (auth.json or OPENCODE_API_KEY)

# Durations (number included for composition in Rust)
duration-day =
    { $n ->
        [one] 1 day
       *[other] { $n } days
    }
duration-hour =
    { $n ->
        [one] 1 hour
       *[other] { $n } hours
    }
duration-minute =
    { $n ->
        [one] 1 minute
       *[other] { $n } minutes
    }
duration-second =
    { $n ->
        [one] 1 second
       *[other] { $n } seconds
    }
