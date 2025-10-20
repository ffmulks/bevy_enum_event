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
- **Deref support** (optional, enabled by default): Automatic `Deref` and `DerefMut` for ergonomic field access

## Installation

```toml
[dependencies]
bevy_enum_event = "0.2"
```

---

# Part 1: EnumEvent - Global Events

## Quick Start - Unit Variants

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

## Variants with Data

Enum variants can carry data using tuple or named field syntax:

```rust
use bevy::prelude::*;
use bevy_enum_event::EnumEvent;

#[derive(EnumEvent, Clone)]
enum GameEvent {
    PlayerSpawned(Entity),
    ScoreChanged { player: Entity, score: i32 },
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

## Using Events with Bevy Observers

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

### Accessing Event Data

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

## Deref Feature (enabled by default)

The `deref` feature provides ergonomic access to event data by automatically implementing `Deref` and `DerefMut`:

- **Single-field variants**: Automatically get deref to the inner value
- **Multi-field variants**: Mark one field with `#[enum_event(deref)]` for deref access
- **No annotation**: Access fields directly by name

### Example - Automatic Deref

```rust
use bevy::prelude::*;
use bevy_enum_event::EnumEvent;

#[derive(EnumEvent, Clone)]
enum NetworkEvent {
    MessageReceived(String),      // Single field - automatic deref
    HealthChanged { value: f32 },  // Single field - automatic deref
}

fn on_message(msg: On<network_event::MessageReceived>) {
    // Direct access to the String via deref
    let content: &String = &*msg.event();
    println!("Received: {}", content);
}

fn on_health(health: On<network_event::HealthChanged>) {
    // Direct access to the f32 via deref
    let value: f32 = *health.event();
    println!("Health: {}", value);
}
```

### Example - Multi-Field with Deref Annotation

```rust
use bevy::prelude::*;
use bevy_enum_event::EnumEvent;

#[derive(EnumEvent, Clone)]
enum GameEvent {
    // Mark the primary field for deref access
    PlayerScored { #[enum_event(deref)] player: Entity, points: u32 },

    // No annotation - access fields directly
    TeamScore { team: u32, points: u32 },
}

fn on_player_scored(scored: On<game_event::PlayerScored>) {
    // Deref gives you the player entity
    let player: Entity = *scored.event();
    // Other fields still accessible by name
    println!("Player {:?} scored {} points", player, scored.event().points);
}

fn on_team_score(scored: On<game_event::TeamScore>) {
    // No deref - access fields directly
    let event = scored.event();
    println!("Team {} scored {} points", event.team, event.points);
}
```

### Disabling Deref

If you prefer not to have `Deref` and `DerefMut` automatically implemented:

```toml
[dependencies]
bevy_enum_event = { version = "0.2", default-features = false }
```

When disabled, access fields directly:

```rust
fn on_message(msg: On<network_event::MessageReceived>) {
    let content: &String = &msg.event().0;  // Access via .0 for tuple variants
    println!("Received: {}", content);
}
```

## Advanced: Generics & Lifetimes

The macro preserves generic parameters, lifetimes, and `where` clauses from your enum:

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

Generated types: `generic_event::Borrowed<'a, T>`, `generic_event::Owned<'a, T>`, `generic_event::Done<'a, T>`.

Unit variants automatically implement `Default` and get a `new()` helper when phantom markers are needed. Tuple and named variants with phantom markers also receive `new(...)` helpers that accept only the original fields.

---

# Part 2: EnumEntityEvent - Entity-Targeted Events

`EntityEvent` types target specific entities and can trigger entity-specific observers, enabling fine-grained control over event handling.

## Important Requirements

Before diving into examples, note these requirements for `EnumEntityEvent`:

- **Named fields only**: All variants must use struct-style `{ field: Type }` syntax
- **Entity field required**: Each variant must have either:
  - A field named `entity: Entity`, OR
  - A field marked with `#[enum_event(target)]`
- **Triggering**: Use `commands.trigger(event)` or `world.trigger(event)`

## Basic EntityEvent Usage

```rust
use bevy::prelude::*;
use bevy_enum_event::EnumEntityEvent;

#[derive(EnumEntityEvent, Clone)]
enum PlayerEvent {
    Spawned { entity: Entity },
    Damaged { entity: Entity, amount: f32 },
    Destroyed { entity: Entity },
}

// Global observer - runs for ALL player damage events
fn on_any_player_damaged(damaged: On<player_event::Damaged>) {
    println!("A player took {} damage", damaged.amount);
}

// Entity-specific observer - only runs for events targeting this specific entity
fn setup_player(mut commands: Commands) {
    commands.spawn_empty()
        .observe(|damaged: On<player_event::Damaged>| {
            println!("This specific player took {} damage", damaged.amount);
        });
}
```

## Custom Target Field

By default, `EnumEntityEvent` looks for a field named `entity`. You can use a different field with `#[enum_event(target)]`:

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

## Event Propagation

Event propagation allows events to "bubble up" through entity hierarchies, similar to DOM event propagation in web browsers.

### Basic Propagation

Enable propagation with `#[enum_event(propagate)]`:

```rust
use bevy::prelude::*;
use bevy_enum_event::EnumEntityEvent;

// Default propagation uses the ChildOf relationship
#[derive(EnumEntityEvent, Clone, Copy)]
#[enum_event(propagate)]
enum UiEvent {
    Click { entity: Entity },
    Hover { entity: Entity },
}

fn on_click(mut click: On<ui_event::Click>) {
    println!("Clicked on: {:?}", click.entity);

    // Stop the event from bubbling up to parent entities
    click.propagate(false);

    // Access the original target that triggered the event
    let original = click.original_event_target();
}
```

### Auto Propagation

For events that should always bubble up without manual control:

```rust
#[derive(EnumEntityEvent, Clone, Copy)]
#[enum_event(auto_propagate, propagate)]
enum SystemEvent {
    Update { entity: Entity },
}
```

### Custom Propagation Relationships

Specify a custom relationship type for propagation instead of the default `ChildOf`:

```rust
use bevy::prelude::*;
use bevy_enum_event::EnumEntityEvent;

#[derive(Component)]
#[relationship(relationship_target = CustomRelationshipTarget)]
pub struct CustomRelationship(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = CustomRelationship, linked_spawn)]
pub struct CustomRelationshipTarget(Vec<Entity>);

// Propagate along CustomRelationship instead of ChildOf
// Note: Use absolute paths (::) or make the relationship public
#[derive(EnumEntityEvent, Clone, Copy)]
#[enum_event(propagate = &'static CustomRelationship)]
enum NetworkEvent {
    DataReceived { entity: Entity },
}
```

**Important**: Custom relationship types must be `pub` or referenced via absolute paths (`::bevy::`, `crate::`, etc.) because they're accessed from the generated module.

### Variant-Level Propagation

Override enum-level propagation settings for specific variants:

```rust
#[derive(EnumEntityEvent, Clone, Copy)]
#[enum_event(propagate)]  // Default for all variants
enum MixedEvent {
    Normal { entity: Entity },  // Uses enum-level propagate

    #[enum_event(auto_propagate, propagate)]  // Override with auto_propagate
    AutoEvent { entity: Entity },

    #[enum_event(propagate = &'static ::bevy::prelude::ChildOf)]  // Custom relationship
    CustomEvent { entity: Entity },
}

```

---

# Additional Information

## Snake Case Conversion

The macro intelligently converts enum names to snake_case module names:

- `LifeFSM` → `life_fsm`
- `PlayerState` → `player_state`
- `HTTPServer` → `http_server`
- `MyHTTPSConnection` → `my_https_connection`

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

