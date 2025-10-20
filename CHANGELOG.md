# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-10-20

### Added
- Initial release of `bevy_enum_event` macro
- Automatic event generation from enum variants
- Support for unit variants, tuple variants, and named field variants
- Snake case module generation from enum names
- Support for generics, lifetimes, and `where` clauses
- Optional `deref` feature (enabled by default) for ergonomic access to single-field variants
- `#[enum_event(deref)]` attribute for multi-field variants
- Automatic `PhantomData` handling for generic unit variants
- Comprehensive documentation and examples

### Features
- `deref` (default): Automatic `Deref` and `DerefMut` implementation for single-field variants

[0.1.0]: https://github.com/ffmulks/bevy_enum_event/releases/tag/v0.1.0
