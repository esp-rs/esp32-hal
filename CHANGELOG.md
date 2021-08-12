# Change Log

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

## [v0.3.0] - 2021-08-12

### Additions
  - Basic I2C Support

### Fixed
  - Fix compilication errors around the `const_fn` feature.
  - Bumped `xtensa-lx`, `xtensa-lx-rt` & `esp32` to support newer compilers.

## [v0.2.0] - 2020-09-23

### Changed
  - Replace `xtenxa-lx6` with `xtensa-lx`, a silicon agnostic craate for the runtime and peripheral access of xtensa CPU's.

### Fixed
  - Update alloc to support the new `alloc_ref` nightly changes.
  - Clean up examples

## [v0.1.0] - 2020-09-15

- Initial release

[Unreleased]: https://github.com/esp-rs/esp32-hal/compare/v0.3.0...HEAD
[v0.3.0]: https://github.com/esp-rs/esp32-hal/compare/v0.2.0...v0.3.0
[v0.2.0]: https://github.com/esp-rs/esp32-hal/compare/v0.1.0...v0.2.0
[v0.1.0]: https://github.com/esp-rs/esp32-hal/tree/v0.1.0