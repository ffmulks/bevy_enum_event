# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2025-10-20

### Added
- Migrated to Bevy 0.17
- Introduced `EnumEntityEvent` derive macro for entity-targeted events
- Support for `#[enum_event(target)]` attribute to specify custom target fields
- Support for `#[enum_event(propagate)]` to enable event propagation up entity hierarchies
- Support for `#[enum_event(auto_propagate)]` for automatic event propagation
- Custom propagation relationships via `#[enum_event(propagate = &'static RelType)]`
- Variant-level propagate attributes that override enum-level settings for fine-grained control
- Comprehensive documentation for EntityEvent features
- Integration tests demonstrating observer behavior and event propagation
- README sections covering `auto_propagate` usage and custom relationship visibility

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