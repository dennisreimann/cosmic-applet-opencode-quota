# AGENTS.md

Guidelines and context for AI agents working on this repository.

## Project

**cosmic-applet-opencode-quota** — A native COSMIC panel applet for Pop!_OS that
displays OpenCode Go quota usage in the panel (colored icon + three percentage
values). Written in Rust with `libcosmic`.

## Key architecture decision (history context)

The original approach was an ksni StatusNotifierItem tray. **That was completely
scrapped**: cosmic-applet-status-area forces every SNI icon into a fixed square
(`app.rs: Length::Fixed` on both axes) — readable wide text is physically
impossible there. The correct path is a **native COSMIC applet**
(`cosmic::applet::run`) that renders arbitrary text at panel height with
`core.applet.text(...)` (automatic theming: white/black depending on dark/light).

## Fetch failure semantics (keep last state)

`Message::Fetched(Ok)` replaces the snapshot and clears the error;
`Message::Fetched(Err)` only records the error — the last snapshot (values
and the "Updated" clock) stays untouched, so a broken connection never
blanks the panel or popup. The panel shows the error text only when there
has never been a successful fetch (snapshot is `None`). Do not "fix" this
by clearing state on errors.

## Data source

- **API:** `GET https://opencode.ai/zen/go/v1/usage` — headers
  `Authorization: Bearer <key>`, `Accept: application/json`.
- **Auth key:** `~/.local/share/opencode/auth.json` → JSON object `opencode-go`
  with `{ "type": "api", "key": "..." }`. Fallback: env `OPENCODE_API_KEY`.
- **Response schema (verified):** root object with `usage` containing three
  mandatory windows: `rolling` (5h), `weekly`, `monthly`. Each window:
  `{ "status": "ok", "percent": <0-100>, "resetsAt": "<RFC3339 offset>" }`.
  **`percent` is the CONSUMED share.** Remaining = `100 - percent`.
  A missing or malformed window fails the whole response (contract error).
- **Display:** the applet shows the **consumed** share (`100 - percent_remaining`,
  counting up), format: `16% · 77% W · 38% M` (W = weekly, M = monthly).
- **No OpenCode Zen balance:** there is no official balance/credit API (neither
  for Zen nor Go — docs point to the web console). The only source would be
  scraping billing pages with a browser cookie — deliberately **not** implemented.

## Module structure

- `src/config.rs` — auth.json path, cache path, `resolve_api_key` (auth.json before env).
- `src/i18n.rs` — Fluent loader (`fl!` macro, catalogs in `i18n/<lang>/cosmic_applet_opencode_quota.ftl`,
  language chosen from the desktop locale, `I18N_LANGUAGE` override for tests).
- `src/quota.rs` — parser/validator for the `/usage` response (pure data transform, isolated testing).
- `src/api.rs` — HTTP fetch: `fetch_quota`/`fetch_quota_at`, 10s timeout, 1 retry
  (retryable: 408/425/429/5xx), auth header, error classification (`ApiError { message, retryable }`).
- `src/cache.rs` — cache read/write (`~/.cache/opencode-quota/cache.json`,
  `QuotaSnapshot` serde). Shows the last known state immediately at startup.
- `src/applet.rs` — the libcosmic applet (panel view, popup, 60s refresh chain).

## libcosmic applet patterns (important)

- Dependency: `libcosmic` from Git with `default-features = false` and explicit
  features, including `"applet"`, `"applet-token"`, `"multi-window"`, `"tokio"`,
  `"wayland"`, `"winit"`. (Without the `applet` feature, `cosmic::applet` does not exist.)
- `cosmic::app::Task<M>` is an alias for `iced::Task<cosmic::Action<M>>` — the
  Application trait methods return `Task<Message>`. `cosmic::Task` (root) is the
  bare `iced::Task` — do not confuse the two.
- Refresh via a self-rescheduling task chain: `init` → `Task::perform(fetch, …)` →
  `Fetched` → `Task::perform(tokio::time::sleep(60s), …)` → `Tick` → fetch …,
  instead of an active subscription.
- **Popup rule:** follow the case-sensitive selector API of the current libcosmic
  revision — the API changes frequently between revisions. Official reference
  pattern: `examples/applet` in the libcosmic checkout (`~/.cargo/git/checkouts/...`),
  pattern `on_press_with_rectangle` + `Message::Surface(surface::Action)` +
  `cosmic::task::message(Action::Cosmic(app::Action::Surface(a)))` + boxed view.

## References to COSMIC applet internals (cosmic-applets repo)

For panel/applet behavior see the source of `pop-os/cosmic-applets`
(status-notifier watcher, size/anchor handling) and `pop-os/libcosmic`
(`src/applet/mod.rs`, `src/widget/autosize.rs`, `src/widget/rectangle_tracker/`).

## Testing & building

- Test suite: `cargo test` — integration tests in `tests/` (api, applet, cache, config, i18n_de, quota).
- Translations: always add new strings via `fl!("message-id")` in `src/applet.rs`; create message IDs
  in both catalogs (`i18n/en/…`, `i18n/de/…`) — the `fl!` macro validates against the EN catalog
  at compile time.
- Lint: `cargo clippy --all-targets -- -D warnings`.
- **Before EVERY release (mandatory):** run `cargo update` first, check
  for available major bumps of the direct dependencies in `Cargo.toml`
  and apply sensible ones, then run the full test suite and clippy again.
  Never tag a release from a stale Cargo.lock — v1.2.0 shipped outdated
  transitive dependencies (Dependabot warnings); the patch release fixed
  that.
- Release: `cargo build --release` (10–15 min, iced/libcosmic are heavy).
- Install (user-local):
  - Binary → `~/.local/bin/cosmic-applet-opencode-quota`
  - Applet discovery → `~/.local/share/applications/net.d11n.CosmicOpencodeQuota.desktop`
    (desktop entry with `X-CosmicApplet=true`, `Exec=<absolute path>`, `NoDisplay=true`).
  - Icon → `~/.local/share/icons/hicolor/scalable/apps/net.d11n.CosmicOpencodeQuota.svg` (the `assets/o.svg`).
- Panel configuration: `~/.config/cosmic/com.system76.CosmicPanel.Panel/v1/plugins_wings`
  (RON `Some((left, right_wing))`). The applet sits in the right wing in front of
  `com.system76.CosmicAppletStatusArea`. The panel hot-reloads applets on config changes
  (toggle remove + re-add to force a respawn).

## Reliability patterns (v1.2.0)

- **Cache validation:** `QuotaSnapshot` deserializes through a validating
  custom `Deserialize` impl — each window's percent must be finite 0–100 and
  `resetsAt` valid RFC3339; otherwise `cache::load_snapshot` returns `None`
  (fresh data is fetched instead). The API parser (`quota::normalize`)
  enforces the same shape, so cache and API share one contract.
- **Error display:** a refresh error is shown in the popup (`error-line`)
  even while the last cached snapshot is still displayed — old data stays,
  but is not presented as fresh. Cache write failures are logged via
  `tracing::warn` instead of being swallowed.
- **Installer:** `install.sh` works from a source checkout
  (`target/release/…`) as well as an extracted release archive, from any cwd.

## Deferred: multi-provider balance

Research notes from the retired spec (`docs/superpowers/specs/`), folded
here so the verified API facts survive:

- **PPQ.ai — usable, documented balance endpoint:** OpenAI-compatible,
  base `https://api.ppq.ai`, keys `sk-…` (Bearer). Documented:
  `POST https://api.ppq.ai/credits/balance` — auth via Bearer key, `api-key:`
  header, or `{"credit_id": "…"}` body; response `{"balance": <USDT>}`.
  Pay-per-query; `402 Payment Required` signals too-low balance.
  Source of truth: `https://ppq.ai/llms.txt`. PPQ data is absolute USD
  remaining, NOT a percent — color thresholds must be independent
  (e.g. yellow < $5, red < $1).
- **Z.ai — blocked:** no balance/billing API for API keys (checked
  openapi.json + llms.txt, probed ~16 candidate `/api/biz/*` paths — all
  404/405); billing lives only in the web console with session cookies.
  Balance would require cookie scraping — deliberately NOT implemented
  (project principle). Revisit only if Z.ai ships an official endpoint.
- **No OpenCode Zen balance exists** either (see Data source above).
- **Design direction if resumed (Option A, recommended):** small
  `QuotaSource` trait/enum; per provider fetcher, key resolution
  (generalize `resolve_api_key`), cache file and display block.
  OpenCode Go stays the percent-provider; PPQ is a balance-provider with
  `$X.XX` formatting. Display decision open: second panel segment vs
  popup-only row. Open questions: PPQ key location, panel-vs-popup,
  dollar thresholds.
