# bevy_enum_event

General-purpose enum to Bevy event conversion macro.

## Overview

`bevy_enum_event` provides derive macros that automatically generate Bevy event types from enum variants. For each variant in your enum, it creates a corresponding event struct organized in a snake_case module. Supports unit variants, tuple variants, and named field variants.

Starting with Bevy 0.17, there are two types of events:
- **`Event`**: Global events that are not associated with any specific entity
- **`EntityEvent`**: Events that target a specific entity and can trigger entity-specific observers

This crate provides corresponding derive macros for both:
- `#[derive(EnumEvent)]` - Generates `Event` types
- `#[derive(EnumEntityEvent)]` - Generates `EntityEvent` types

## Bevy Compatibility

|  Bevy   | bevy_enum_event |
|---------|-----------------|
| 0.17    | 0.2             |
| 0.16    | 0.1             |

## Features

- **Automatic event generation**: One macro generates all variant events
- **Support for data-carrying variants**: Enum variants can contain data (tuple or named fields)
- **Snake case module**: `PlayerState` → `player_state` module
- **Zero boilerplate**: No manual event struct definitions needed
- **Type-safe**: Each variant gets its own distinct event type
- **Generic-friendly**: Works with lifetimes, generic parameters, and `where` clauses
- **Bevy integration**: Generated events work seamlessly with Bevy's observer system
- **Entity event support**: Generate `EntityEvent` types with entity targeting and propagation
- **Deref support** (optional): Automatic `Deref` and `DerefMut` for single-field variants, or multi-field variants when you tag one field with `#[enum_event(deref)]`

## Installation

```toml
[dependencies]
bevy_enum_event = "0.2"
```

## Quick Start

### Unit Variants

```rust
use bevy::prelude::*;
use bevy_enum_event::EnumEvent;

#[derive(EnumEvent, Clone, Copy, Debug)]
enum PlayerState {
    Idle,
    Running,
    Jumping,
}
```

This automatically generates:

```rust
pub mod player_state {
    use bevy::prelude::Event;

    #[derive(Event, Clone, Copy, Debug)]
    pub struct Idle;

    #[derive(Event, Clone, Copy, Debug)]
    pub struct Running;

    #[derive(Event, Clone, Copy, Debug)]
    pub struct Jumping;
}
```

### Variants with Data

```rust
use bevy::prelude::*;
use bevy_enum_event::EnumEvent;

#[derive(EnumEvent, Clone)]
enum GameEvent {
    PlayerSpawned(Entity),
    ScoreChanged { #[enum_event(deref)] player: Entity, score: i32 },
    GameOver,
}
```

This generates:

```rust
pub mod game_event {
    use bevy::prelude::Event;

    #[derive(Event, Clone, Debug)]
    pub struct PlayerSpawned(pub Entity);

    #[derive(Event, Clone, Debug)]
    pub struct ScoreChanged {
        pub player: Entity,
        pub score: i32,
    }

    #[derive(Event, Clone, Debug)]
    pub struct GameOver;
}
```

## Usage with Bevy Observers

### Basic Example (Global Events)

```rust
use bevy::prelude::*;
use bevy_enum_event::EnumEvent;

#[derive(EnumEvent, Clone, Copy)]
enum GameState {
    MainMenu,
    Playing,
    Paused,
}

fn setup(app: &mut App) {
    app.observe(on_paused);
}

fn on_paused(paused: On<game_state::Paused>) {
    println!("Game paused!");
}
```

### Entity Events

Entity events target specific entities and trigger entity-specific observers:

```rust
use bevy::prelude::*;
use bevy_enum_event::EnumEntityEvent;

#[derive(EnumEntityEvent, Clone)]
enum PlayerEvent {
    Spawned { entity: Entity },
    Damaged { entity: Entity, amount: f32 },
    Destroyed { entity: Entity },
}

// Global observer - runs for ALL player events
fn on_any_player_damaged(damaged: On<player_event::Damaged>) {
    println!("A player took {} damage", damaged.amount);
}

// Entity-specific observer - only runs for events targeting a specific entity
fn setup_player(commands: &mut Commands) {
    commands.spawn_empty()
        .observe(|damaged: On<player_event::Damaged>| {
            println!("This specific player took {} damage", damaged.amount);
        });
}
```

### With Event Data

```rust
use bevy::prelude::*;
use bevy_enum_event::EnumEvent;

#[derive(EnumEvent, Clone)]
enum GameEvent {
    Victory(String),
    ScoreChanged { team: u32, score: i32 },
    GameOver,
}

fn on_score_changed(score: On<game_event::ScoreChanged>) {
    let event = score.event();
    println!("Team {} scored {} points", event.team, event.score);
}
```

## Feature: `deref` (enabled by default)

The `deref` feature automatically derives Bevy's `Deref` and `DerefMut` traits for enum variants with a single field (either tuple or named), providing ergonomic access to the inner value. For multi-field variants, you can opt into the same ergonomic access by adding `#[enum_event(deref)]` to the field you want to expose (the generated struct receives Bevy's `#[deref]` attribute). If a multi-field variant is not annotated with `#[enum_event(deref)]`, no deref functionality is generated and fields must be accessed directly by name.

### Example

```rust
use bevy::prelude::*;
use bevy_enum_event::EnumEvent;

#[derive(EnumEvent, Clone)]
enum EntityEvent {
    Spawned(Entity),
    Destroyed(Entity),
    HealthChanged { value: f32 },
    Scored { #[enum_event(deref)] player: Entity, points: u32 },
    TeamScore { team: u32, points: u32 },  // No deref annotation, no derive will happen
}

fn on_spawned(spawned: On<entity_event::Spawned>) {
    // With deref feature, single-field variants can access the Entity directly
    let entity: Entity = *spawned.event();
    println!("Entity spawned: {:?}", entity);
}

fn on_health_changed(health_changed: On<entity_event::HealthChanged>) {
    // Deref also works for named single-field variants
    let health: f32 = *health_changed.event();
    println!("Health changed to: {}", health);
}

fn on_scored(scored: On<entity_event::Scored>) {
    // Multi-field variants work when you mark one field with #[enum_event(deref)]
    let player: Entity = *scored.event();
    println!("Player {player:?} scored!");
}

fn on_team_score(team_scored: On<entity_event::TeamScore>) {
    // Multi-field without deref annotation - must access fields directly
    let event = team_scored.event();
    println!("Team {} scored {} points", event.team, event.points);
}
```

### Disabling the `deref` Feature

If you prefer not to have `Deref` and `DerefMut` automatically implemented, you can disable the default features:

```toml
[dependencies]
bevy_enum_event = { version = "0.2", default-features = false }
```

When disabled, you'll need to access fields directly:

```rust
fn on_spawned(spawned: On<game_event::Victory>) {
    let message: &String = &spawned.event().0;  // Access via .0
    println!("Victory: {}", message);
}
```

## EntityEvent Features

### Custom Target Field

By default, `EnumEntityEvent` uses a field named `entity` as the event target. You can specify a different field using `#[enum_event(target)]`:

```rust
use bevy::prelude::*;
use bevy_enum_event::EnumEntityEvent;

#[derive(EnumEntityEvent, Clone, Copy)]
enum CombatEvent {
    Attack {
        #[enum_event(target)]
        attacker: Entity,
        defender: Entity,
    },
}

// This event will trigger observers on the attacker entity
fn trigger_attack(mut commands: Commands, attacker: Entity, defender: Entity) {
    commands.trigger(combat_event::Attack { attacker, defender });
}
```

### Event Propagation

Enable event propagation up the entity hierarchy using `#[enum_event(propagate)]`. This allows events to "bubble up" through parent entities:

```rust
use bevy::prelude::*;
use bevy_enum_event::EnumEntityEvent;

// Default propagation (uses ChildOf relationship)
#[derive(EnumEntityEvent, Clone, Copy)]
#[enum_event(propagate)]
enum UiEvent {
    Click { entity: Entity },
    Hover { entity: Entity },
}

// With propagation enabled, you can:
// - Stop propagation with `click.propagate(false)`
// - Access the original target with `click.original_event_target()`
fn on_click(mut click: On<ui_event::Click>) {
    println!("Clicked on: {:?}", click.entity);

    // Stop the event from bubbling up to parent entities
    if should_stop_propagation() {
        click.propagate(false);
    }
}
```

#### Custom Propagation Relationships

You can specify a custom relationship type for propagation:

```rust
use bevy::prelude::*;
use bevy_enum_event::EnumEntityEvent;

// Use a custom relationship type
#[derive(EnumEntityEvent, Clone, Copy)]
#[enum_event(propagate = &'static ::bevy::hierarchy::ChildOf)]
enum HierarchyEvent {
    NodeChanged { entity: Entity },
    NodeAdded { entity: Entity },
}
```

#### Automatic Event Propagation

For events that should automatically bubble up the hierarchy, use `auto_propagate`:

```rust
use bevy::prelude::*;
use bevy_enum_event::EnumEntityEvent;

#[derive(Component)]
#[relationship(relationship_target = CustomRelationshipTarget)]
pub struct CustomRelationship(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = CustomRelationship, linked_spawn)]
pub struct CustomRelationshipTarget(Vec<Entity>);

// Events will automatically propagate up the CustomRelationship chain
// Note that as a submodule is created, super::CustomRelationship is used internally unless you specify an absolute path. The relationship and target, thus, need to be public.
#[derive(EnumEntityEvent, Clone, Copy)]
#[enum_event(auto_propagate, propagate = &'static CustomRelationship)]
enum NetworkEvent {
    DataReceived { entity: Entity },
}
```

This is useful when you want to propagate events along custom relationship types instead of the default `ChildOf` relationship.

### Important Notes for EntityEvent

- **Custom relationships must be exported**: If you provide `propagate = ...` with a custom relationship type, declare that type `pub` (or reference it via an absolute path such as `crate::` or `::bevy::`). Non-public relationships cannot be accessed from the generated module.
- **Named fields only**: `EnumEntityEvent` requires named fields (struct-style variants). Tuple and unit variants are not supported.
- **Entity field required**: Each variant must have either a field named `entity: Entity` or a field marked with `#[enum_event(target)]`.
- **Triggering events**: Use `commands.trigger(event)` or `world.trigger(event)` to trigger entity events.
- **Accessing the entity**: The target entity is available via `event.entity` (or your custom target field name).

## Snake Case Conversion

The macro intelligently converts enum names to snake_case module names:

- `LifeFSM` → `life_fsm`
- `PlayerState` → `player_state`
- `HTTPServer` → `http_server`
- `MyHTTPSConnection` → `my_https_connection`

## Generics & Lifetimes

All derives mirror the generic parameters, lifetimes, and `where` clauses from your enum onto the generated
event structs. This makes it straightforward to use `EnumEvent` with enums such as:

```rust
#[derive(EnumEvent, Clone)]
enum GenericEvent<'a, T>
where
    T: Clone + 'a,
{
    Borrowed(&'a T),
    Owned(T),
    Done,
}
```

The generated module exposes `generic_event::Borrowed<'a, T>`, `generic_event::Owned<'a, T>`, and
`generic_event::Done<'a, T>` types with identical bounds.

Unit event structs expose ergonomic constructors so you never have to juggle hidden `PhantomData`
markers by hand. Every unit variant implements `Default`, and when a phantom marker is required the
derive also emits a `new()` helper that seeds it for you. Tuple and named variants that require
phantom markers likewise receive `new(...)` helpers that accept only the original fields.

## Use Cases

- State machines (see [bevy_fsm](https://crates.io/crates/bevy_fsm))
- Game state transitions
- Entity lifecycle events
- Animation states
- Input modes
- Any enum-based event system
- Network message events
- Input/Output events with associated data

## AI Disclaimer

- Refactoring and documentation supported by Claude Code
- Minor editing supported by ChatGPT Codex
- The process and final releases are thoroughly supervised and checked by the author

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

