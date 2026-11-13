# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

The **Unreleased** section accumulates changes as they land; its entries are
used verbatim as release notes when cutting a new release (the section is
then renamed to the new version and emptied).

## [Unreleased]

## [1.2.1] - 2026-11-13

### Fixed

- Dependency refresh closes the Dependabot findings (rustls ≥ 0.23.46,
  quinn-proto ≥ 0.11.15) via targeted `cargo update`; dirs updated to 7.
- libcosmic is now pinned to the verified revision so `cargo update`
  cannot silently move the applet's UI base (see AGENTS.md).

## [1.2.0] - 2026-11-13

### Fixed

- Cached snapshots are validated on load (percent finite 0–100, resetsAt
  valid RFC3339); malformed cache files are discarded instead of
  displayed as current data.
- A refresh error is now shown in the popup even while the last cached
  snapshot is displayed; cache write failures are logged instead of
  swallowed.
- `install.sh` also works from a source checkout (`target/release/…`),
  not only from an extracted release archive.

## [1.1.0] - 2026-10-23

### Fixed

- Rate-limited quota windows are handled gracefully: they stay valid with
  zero remaining, including windows that report no percentage.
- Unused rolling usage windows no longer produce misleading values.

## [1.0.0] - 2026-09-01

### Added

- Initial release: native COSMIC panel applet showing OpenCode Go quota
  usage in the panel — consumed share for the 5-hour window, week and
  month, with details popup including relative reset times. Icon and text
  follow the dark/light theme.

[Unreleased]: https://github.com/dennisreimann/cosmic-applet-opencode-quota/compare/v1.1.0...HEAD
[1.1.0]: https://github.com/dennisreimann/cosmic-applet-opencode-quota/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/dennisreimann/cosmic-applet-opencode-quota/releases/tag/v1.0.0
