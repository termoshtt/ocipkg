# Changelog

- All notable changes to this project will be documented in this file.
  - The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
  - and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

In addition to original Keep-a-Changelog, we use following rules:

- Use [GitHub Flavored Markdown](https://github.github.com/gfm/)
- Each line in changes SHOULD include a link to Pull Request in GitHub
- Each Pull Request MUST add a line in this file
  - This will be checked by GitHub Actions
- Each Pull Request MAY correspond to one or more lines in this file

## Unreleased

### Added

### Changed
- Fallback to use ~/.ocipkg/config.json if XDG_RUNTIME_DIR not set https://github.com/termoshtt/ocipkg/pull/67

### Fixed

### Internal

## 0.2.2 - 2022-08-11

### Changed
- Set MSRV to 1.57.0 https://github.com/termoshtt/ocipkg/pull/66

## 0.2.1 - 2022-08-11

Hot Fix of 0.2.1 because rejected from crates.io

### Fixed
- Add Cargo.toml metadata of ocipkg-cli crate

## 0.2.0 - 2022-08-11

### Changed
- Split ocipkg-cli crate https://github.com/termoshtt/ocipkg/pull/65

### Fixed
- Drop vergen and clap-vergen which not work with cargo-install https://github.com/termoshtt/ocipkg/pull/64

## 0.1.2 - 2022-08-08

HotFix to 0.1.1

### Fixed
- Make vergen build-dependency optional https://github.com/termoshtt/ocipkg/pull/63

## 0.1.1 - 2022-08-08

This has been yanked because it cannot compile without `cli` feature.

### Added
- `version` subcommand https://github.com/termoshtt/ocipkg/pull/62
- Start CHANGELOG.md https://github.com/termoshtt/ocipkg/pull/58

## 0.1.0 - 2022-07-25

Initial release. This file describes changes from this release.
