# bevy_enum_event

General-purpose enum to Bevy event conversion macro.

## Overview

`bevy_enum_event` provides a derive macro that automatically generates Bevy event types from enum variants. For each variant in your enum, it creates a corresponding event struct organized in a snake_case module. Supports unit variants, tuple variants, and named field variants.

## Bevy Compatibility

|  Bevy   | bevy_fsm |
|---------|----------|
| 0.17    | -        |
| 0.16    | 0.1      |

## Features

- **Automatic event generation**: One macro generates all variant events
- **Support for data-carrying variants**: Enum variants can contain data (tuple or named fields)
- **Snake case module**: `PlayerState` → `player_state` module
- **Zero boilerplate**: No manual event struct definitions needed
- **Type-safe**: Each variant gets its own distinct event type
- **Bevy integration**: Generated events work seamlessly with Bevy's observer system
- **Deref support** (optional): Automatic `Deref` and `DerefMut` for single-field variants

## Installation

```toml
[dependencies]
bevy_enum_event = "0.1"
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

## Usage with Bevy Observers

### Basic Example

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

fn on_paused(trigger: Trigger<game_state::Paused>) {
    println!("Game paused!");
}
```

### With Event Data

```rust
use bevy::prelude::*;
use bevy_enum_event::EnumEvent;

#[derive(EnumEvent, Clone)]
enum EntityEvent {
    Spawned(Entity),
    Damaged { entity: Entity, amount: f32 },
    Destroyed(Entity),
}

fn on_entity_damaged(trigger: Trigger<entity_event::Damaged>) {
    let event = trigger.event();
    println!("Entity {:?} took {} damage", event.entity, event.amount);
}

fn on_entity_spawned(trigger: Trigger<entity_event::Spawned>) {
    // With the deref feature (enabled by default), you can access the Entity directly
    let entity: Entity = *trigger.event();
    println!("Entity spawned: {:?}", entity);
}
```

## Feature: `deref` (enabled by default)

The `deref` feature automatically implements `Deref` and `DerefMut` for enum variants with a single field (either tuple or named), allowing convenient access to the inner value.

### Example

```rust
use bevy::prelude::*;
use bevy_enum_event::EnumEvent;

#[derive(EnumEvent, Clone)]
enum EntityEvent {
    Spawned(Entity),
    Destroyed(Entity),
    HealthChanged { value: f32 },
}

fn on_spawned(trigger: Trigger<entity_event::Spawned>) {
    // With deref feature, you can access the Entity directly
    let entity: Entity = *trigger.event();
    println!("Entity spawned: {:?}", entity);
}

fn on_health_changed(trigger: Trigger<entity_event::HealthChanged>) {
    // Deref also works for named single-field variants
    let health: f32 = *trigger.event();
    println!("Health changed to: {}", health);
}
```

### Disabling the `deref` Feature

If you prefer not to have `Deref` and `DerefMut` automatically implemented, you can disable the default features:

```toml
[dependencies]
bevy_enum_event = { version = "0.1", default-features = false }
```

When disabled, you'll need to access fields directly:

```rust
fn on_spawned(trigger: Trigger<entity_event::Spawned>) {
    let entity: Entity = trigger.event().0;  // Access via .0
    println!("Entity spawned: {:?}", entity);
}
```

## Snake Case Conversion

The macro intelligently converts enum names to snake_case module names:

- `LifeFSM` → `life_fsm`
- `PlayerState` → `player_state`
- `HTTPServer` → `http_server`
- `MyHTTPSConnection` → `my_https_connection`

## Advanced: `enum_module_ident!` Macro

For library authors building on top of `bevy_enum_event`, the `enum_module_ident!` macro provides programmatic access to the generated module names:

```rust
use bevy_enum_event::enum_module_ident;

// Expands to the identifier: life_fsm
enum_module_ident!(LifeFSM);

// Can be used with stringify! to get the string representation
let module_name = stringify!(enum_module_ident!(PlayerState));
assert_eq!(module_name, "player_state");
```

### Use Cases

This macro is particularly useful for:
- Libraries like [`bevy_fsm`](https://crates.io/crates/bevy_fsm) that need to reference generated module names
- Code generation tools that work with `EnumEvent`
- Macros that compose with `EnumEvent`

Most users won't need this macro directly, as they can reference the generated modules by their snake_case names (e.g., `player_state::Idle`).

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
