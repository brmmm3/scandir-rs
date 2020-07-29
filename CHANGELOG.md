# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.9.3] - 2020-07-27

### Added

- Improved pytest test cases.

### Changed

- In benchmark.py:
  - Use Linux kernel 5.5.5 as platform independent a reference.
  - Accept optional parameter for temporary directory base.
  - Benchmark directory C:\Windows on Windows and /usr on other platforms.

### Fixed

- scandir didn't execute.
- Fix performance issue with Walk.
- Correctly return Python exceptions.
- Make build_wheels.sh version independent.
- Make examples platform independent.
- Fix typo in README.md.

## [0.9.2] - 2020-07-26

### Changed

- Provide Windows wheels without debug information.

## [0.9.1] - 2020-07-26

### Changed

- Update to latest versions of Rust and dependencies.

## [0.9.0] - 2020-01-27

### Added

- Add DirEntryExt and DirEntryFull.
- Arguments for directory and file filtering.

### Changed

- Replaced arguments `metadata` and `metadata_ext` with `return_type`.
- Update documentation.

## [0.8.0] - 2020-01-19

### Added

- Add getters to DirEntry.

### Changed

- Update documentation.

### Fixed

- Correctly count hardlinks.
- Update jwalk to get correct extended metadata (size and hardlinks).
  https://github.com/brmmm3/jwalk/tree/jwalk-0.4.1-alpha.1

## [0.7.2] - 2020-01-10

### Changed

- Change default return_type for Walk to RETURN_TYPE_WALK.

## [0.7.1] - 2020-01-10

### Changed

- Update documentation.

## [0.7.0] - 2020-01-09

- First release.
