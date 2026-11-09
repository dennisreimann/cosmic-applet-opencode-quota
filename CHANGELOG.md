# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

The **Unreleased** section accumulates changes as they land; its entries are
used verbatim as release notes when cutting a new release (the section is
then renamed to the new version and emptied).

## [Unreleased]

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
