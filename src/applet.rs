use crate::{api, cache, config, fl, quota::QuotaSnapshot};
use chrono::{DateTime, Local};
use cosmic::app::{Core, Task};
use cosmic::cosmic_theme::Spacing;
use cosmic::iced::platform_specific::shell::wayland::commands::popup::{destroy_popup, get_popup};
use cosmic::iced::{Alignment, Length, Rectangle, window};
use cosmic::widget::{autosize, button, column, container, divider, icon, row, space, text, Id};
use cosmic::{applet, theme, Action, Element};
use std::sync::LazyLock;
use std::time::Duration;

pub const APP_ID: &str = "net.d11n.CosmicOpencodeQuota";

const REFRESH_INTERVAL: Duration = Duration::from_secs(60);
const O_SVG: &[u8] = include_bytes!("../o.svg");

static AUTOSIZE_MAIN_ID: LazyLock<Id> = LazyLock::new(|| Id::new("autosize-main"));

/// Startet das Applet (aufgerufen aus `main`).
pub fn run() -> cosmic::iced::Result {
    cosmic::applet::run::<QuotaApplet>(())
}

#[derive(Debug, Clone)]
pub enum Message {
    Fetched(Result<QuotaSnapshot, String>),
    Tick,
    /// Button-Bounds (in Fenster-Koordinaten) für die Popup-Positionierung.
    TogglePopup(Rectangle),
    PopupClosed(window::Id),
    Quit,
}

#[derive(Default)]
pub struct QuotaApplet {
    core: Core,
    snapshot: Option<QuotaSnapshot>,
    error: Option<String>,
    popup: Option<window::Id>,
}

pub fn percent_label(p: f64) -> String {
    format!("{}%", p.round())
}

/// Umrechnung: verbrauchter Anteil statt verbleibender Quota.
pub fn percent_used(percent_remaining: f64) -> f64 {
    (100.0 - percent_remaining).clamp(0.0, 100.0)
}

/// Panel-Beschriftung: `3% · 22% W · 11% M` (5h · Woche · Monat) — jeweils
/// der VERBRAUCHTE Anteil (heraufzählend).
pub fn panel_line(snap: &QuotaSnapshot) -> String {
    format!(
        "{} · {} {} · {} {}",
        percent_label(percent_used(snap.rolling.percent_remaining)),
        percent_label(percent_used(snap.weekly.percent_remaining)),
        fl!("panel-week"),
        percent_label(percent_used(snap.monthly.percent_remaining)),
        fl!("panel-month")
    )
}

/// Popup-Wertzeilen als (Label+Verbrauch, Reset) — Reset rechtsbündig.
pub fn popup_rows(snap: &QuotaSnapshot) -> Vec<(String, String)> {
    vec![
        (
            format!(
                "{}: {}",
                fl!("popup-rolling"),
                percent_label(percent_used(snap.rolling.percent_remaining))
            ),
            fl!("resets-in", d = fmt_reset_in(&snap.rolling.resets_at)),
        ),
        (
            format!(
                "{}: {}",
                fl!("popup-week"),
                percent_label(percent_used(snap.weekly.percent_remaining))
            ),
            fl!("resets-in", d = fmt_reset_in(&snap.weekly.resets_at)),
        ),
        (
            format!(
                "{}: {}",
                fl!("popup-month"),
                percent_label(percent_used(snap.monthly.percent_remaining))
            ),
            fl!("resets-in", d = fmt_reset_in(&snap.monthly.resets_at)),
        ),
    ]
}

/// Sekunden als lokalisierte Dauer mit korrekter Ein-/Mehrzahl,
/// z.B. `1 Stunde 30 Minuten` oder `3 Tagen 4 Stunden`.
pub fn format_duration(secs: i64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    if h >= 24 {
        let d = h / 24;
        let h_rem = h % 24;
        if h_rem > 0 {
            format!(
                "{} {}",
                fl!("duration-day", n = d),
                fl!("duration-hour", n = h_rem)
            )
        } else {
            fl!("duration-day", n = d)
        }
    } else if h > 0 {
        if m > 0 {
            format!(
                "{} {}",
                fl!("duration-hour", n = h),
                fl!("duration-minute", n = m)
            )
        } else {
            fl!("duration-hour", n = h)
        }
    } else if m > 0 {
        fl!("duration-minute", n = m)
    } else {
        fl!("duration-second", n = secs)
    }
}

/// Zeit bis zum Reset, relativ formatiert (z.B. `3 Tagen 4 Stunden`).
pub fn fmt_reset_in(iso: &str) -> String {
    match DateTime::parse_from_rfc3339(iso) {
        Ok(reset) => {
            let reset_local = reset.with_timezone(&Local);
            let dur = reset_local.signed_duration_since(Local::now());
            format_duration(dur.num_seconds().max(0))
        }
        Err(_) => iso.to_string(),
    }
}

pub fn fmt_updated(unix: i64) -> String {
    DateTime::from_timestamp(unix, 0)
        .map(|d| d.with_timezone(&Local).format("%H:%M:%S").to_string())
        .unwrap_or_else(|| unix.to_string())
}

async fn fetch_snapshot() -> Result<QuotaSnapshot, String> {
    let api_key = config::resolve_api_key(config::auth_json_path().as_deref())
        .ok_or_else(|| fl!("error-no-api-key"))?;
    let client = reqwest::Client::new();
    let snap = api::fetch_quota(&client, &api_key)
        .await
        .map_err(|e| e.message)?;
    if let Some(path) = config::default_cache_path() {
        let _ = cache::save_snapshot(&path, &snap);
    }
    Ok(snap)
}

fn panel_element(state: &QuotaApplet) -> Element<'_, Message> {
    let line = match (&state.snapshot, &state.error) {
        (Some(s), _) => panel_line(s),
        (None, Some(_)) => fl!("panel-error"),
        (None, None) => "–".to_string(),
    };

    let mut handle = icon::from_svg_bytes(O_SVG);
    handle.symbolic = true;
    let (w, h) = state.core.applet.suggested_size(true);
    let logo = icon::icon(handle)
        .width(Length::Fixed(w as f32))
        .height(Length::Fixed(h as f32));

    if state.core.applet.is_horizontal() {
        row![logo, state.core.applet.text(line)]
            .spacing(4)
            .align_y(Alignment::Center)
            .into()
    } else {
        column![logo, state.core.applet.text(line)]
            .spacing(2)
            .align_x(Alignment::Center)
            .into()
    }
}

fn popup_element(state: &QuotaApplet) -> Element<'_, Message> {
    let Spacing { space_xxs, space_s, space_m, .. } = cosmic::theme::active().cosmic().spacing;
    // Nur vertikales Padding an der Spalte — horizontale Insets bekommt jede
    // Zeile selbst, damit das Hover-Highlight der Menü-Zeile randlos ist.
    // Oben mehr Luft (Titelzeile), unten knapp (Trennlinie sitzt direkt darüber).
    let mut col = column![]
        .padding([space_s, 0, space_xxs, 0])
        .spacing(space_xxs);

    // Kopfzeile: Titel links, "Aktualisiert" rechts.
    let updated = state
        .snapshot
        .as_ref()
        .map(|s| fl!("updated-at", t = fmt_updated(s.fetched_at_unix)));
    let head: Element<'_, Message> = match updated {
        Some(u) => row![
            text::heading("OpenCode Go"),
            space::horizontal(),
            text::body(u),
        ]
        .align_y(Alignment::Center)
        .padding([0, space_m])
        .into(),
        None => container(text::heading("OpenCode Go"))
            .padding([0, space_m])
            .into(),
    };
    col = col.push(head);

    // Trennlinie zwischen Kopfzeile und Werten.
    col = col.push(container(divider::horizontal::default()).padding([space_xxs, 0]));

    match &state.snapshot {
        Some(s) => {
            for (label, reset) in popup_rows(s) {
                col = col.push(
                    row![text::heading(label), space::horizontal(), text::body(reset)]
                        .padding([0, space_m]),
                );
            }
        }
        None => {
            let msg = match &state.error {
                Some(e) => fl!("error-line", e = e),
                None => fl!("no-data"),
            };
            col = col.push(container(text::body(msg)).padding([0, space_m]));
        }
    }

    // Trennlinie direkt über der Beenden-Zeile: kein unteres Padding,
    // damit der Abstand zur Menü-Zeile nicht zu groß wird.
    col = col.push(container(divider::horizontal::default()).padding([space_xxs, 0, 0, 0]));
    col = col.push(applet::menu_button(text::body(fl!("quit"))).on_press(Message::Quit));
    container(col).into()
}

impl cosmic::Application for QuotaApplet {
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = APP_ID;

    fn init(core: Core, _flags: ()) -> (Self, Task<Message>) {
        let snapshot = config::default_cache_path()
            .as_deref()
            .and_then(|p| cache::load_snapshot(Some(p)));
        (
            Self {
                core,
                snapshot,
                error: None,
                popup: None,
            },
            Task::perform(fetch_snapshot(), |res| Action::App(Message::Fetched(res))),
        )
    }

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Fetched(Ok(snap)) => {
                self.snapshot = Some(snap);
                self.error = None;
                Task::perform(tokio::time::sleep(REFRESH_INTERVAL), |_| {
                    Action::App(Message::Tick)
                })
            }
            Message::Fetched(Err(e)) => {
                self.error = Some(e);
                Task::perform(tokio::time::sleep(REFRESH_INTERVAL), |_| {
                    Action::App(Message::Tick)
                })
            }
            Message::Tick => {
                Task::perform(fetch_snapshot(), |res| Action::App(Message::Fetched(res)))
            }
            Message::TogglePopup(bounds) => {
                if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = window::Id::unique();
                    self.popup = Some(new_id);
                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    popup_settings.positioner.size = None;
                    // Anker = gesamtes Widget (nicht nur das Icon-Quadrat),
                    // damit das Popup zentriert darunter erscheint.
                    popup_settings.positioner.anchor_rect = Rectangle {
                        x: bounds.x as i32,
                        y: bounds.y as i32,
                        width: bounds.width.max(1.0) as i32,
                        height: bounds.height.max(1.0) as i32,
                    };
                    get_popup(popup_settings)
                }
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
                Task::none()
            }
            Message::Quit => {
                std::process::exit(0);
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let padding = self.core.applet.suggested_padding(true).0;
        let btn = button::custom(panel_element(self))
            .padding(if self.core.applet.is_horizontal() {
                [0, padding]
            } else {
                [padding, 0]
            })
            .class(theme::Button::AppletIcon)
            .on_press_with_rectangle(move |offset, bounds| {
                Message::TogglePopup(Rectangle {
                    x: bounds.x - offset.x,
                    y: bounds.y - offset.y,
                    width: bounds.width,
                    height: bounds.height,
                })
            });

        autosize::autosize(btn, AUTOSIZE_MAIN_ID.clone()).into()
    }

    fn view_window(&self, _id: window::Id) -> Element<'_, Message> {
        self.core
            .applet
            .popup_container(popup_element(self))
            .into()
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(applet::style())
    }
}
