# Changelog

## [Unreleased]

### Added
- `#[enum_event(derive(...))]` helper attribute to forward additional derives
  onto the generated variant structs (e.g. `#[enum_event(derive(Copy, PartialEq))]`).
  Because a derive macro cannot observe the enum's sibling `#[derive(...)]`, the
  derives to forward are listed explicitly. Also supported at the variant level,
  where it is additive to the enum-level list.

### Fixed
- `Copy` (and other reasonable derives) are now forwarded to the generated
  event/message structs, not just to field-less unit variants
  ([#1](https://github.com/MolecularSadism/bevy_enum_event/issues/1)). Small
  events can now be copied instead of cloned.

## [0.4.0] - 2026-07-09

### Changed
- Migrated to Bevy 0.19 (dev-dependency bumped from `0.18` to `0.19`)
- Updated the Bevy compatibility table and installation instructions in the README

### Added
- `tests/bevy_019_compat.rs` covering Bevy 0.19-specific concerns:
  - Enum variants named after Bevy 0.19's newly-shortened prelude lifecycle
    events (`Add`, `Insert`, `Replace`, `Remove`, `Despawn`) generate
    non-colliding event/message/entity-event types and their observers still fire
  - Generated types remain plain events and are unaffected by Bevy 0.19's
    "resources are components" change

### Notes
- No source changes were required: the derive macros only depend on the public
  `Event`/`Message`/`EntityEvent` derives, `On<_>` observers,
  `MessageWriter`/`MessageReader`/`add_message`, and propagation attributes, all
  of which are unchanged in Bevy 0.19. The full existing test suite passes
  against Bevy 0.19 unmodified. Bevy 0.19 requires Rust 1.95.0 or newer.

## [0.3.2] - 2026-01-22

### Added
- Three-macro system for complete Bevy 0.17+ nomenclature support:
  - `EnumEvent` - Observer-based global events (triggered via `world.trigger()`)
  - `EnumMessage` - Buffered messages (written via `MessageWriter`, read via `MessageReader`)
  - `EnumEntityEvent` - Entity-targeted observer events with propagation
- Comprehensive tests for `EnumMessage` with `MessageWriter`/`MessageReader` integration
- Integration tests demonstrating all three patterns working together
- "Choosing the Right Macro" guide in README

### Changed
- Generated modules now include `use super::*;` for standard library type access
- Updated all documentation to reflect correct Bevy 0.17+ terminology

## [0.3.1] - 2026-01-20

### Fixed
- README update for crates.io (0.3.0 was published with outdated README)

## [0.3.0] - 2026-01-20

### Changed
- Migrated to Bevy 0.18
- Simplified documentation

Note: 0.3.0 was published to crates.io but not pushed to GitHub

## [0.2.0] - 2025-10-20

### Added
- `EnumEntityEvent` derive macro for entity-targeted events
- `#[enum_event(target)]` for custom target fields
- `#[enum_event(propagate)]` for event propagation
- `#[enum_event(auto_propagate)]` for automatic propagation
- Custom propagation relationships via `#[enum_event(propagate = &'static Type)]`
- Variant-level attribute overrides

### Changed
- Migrated to Bevy 0.17

## [0.1.0] - 2025-10-20

### Added
- `EnumMessage` derive macro for global messages (originally named `EnumEvent`)
- Support for unit, tuple, and named field variants
- `deref` feature (default) for ergonomic field access
- Full support for generics and lifetimes
