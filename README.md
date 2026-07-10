# bevy_enum_event

[![CI](https://github.com/MolecularSadism/bevy_enum_event/workflows/CI/badge.svg)](https://github.com/MolecularSadism/bevy_enum_event/actions)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/MolecularSadism/bevy_enum_event#license)
[![Bevy](https://img.shields.io/badge/Bevy-0.18-blue.svg)](https://bevyengine.org/)
[![Rust](https://img.shields.io/badge/rust-2021%20edition-orange.svg)](https://www.rust-lang.org/)

Derive macros that generate Bevy event and message types from enum variants.

## Overview

Transform enum variants into distinct Bevy event/message structs automatically. Each variant becomes a separate type in a snake_case module, eliminating boilerplate while preserving type safety.

```rust
use bevy::prelude::*;
use bevy_enum_event::EnumEvent;

#[derive(EnumEvent, Clone)]
enum GameEvent {
    Victory(String),
    ScoreChanged { team: u32, score: i32 },
    GameOver,
}

// Generates: game_event::Victory, game_event::ScoreChanged, game_event::GameOver
```

## Bevy Compatibility

| Bevy | bevy_enum_event |
|------|-----------------|
| 0.19 | 0.4             |
| 0.18 | 0.3             |
| 0.17 | 0.2             |
| 0.16 | 0.1             |

## Installation

```toml
[dependencies]
bevy_enum_event = "0.4.0"
```

## Macros

Bevy 0.17+ distinguishes between three event/message types:

- **`EnumEvent`** - Observer-based global events (triggered via `world.trigger()`)
- **`EnumEntityEvent`** - Entity-targeted observer events with optional propagation
- **`EnumMessage`** - Buffered messages (written via `MessageWriter`, read via `MessageReader`)

## EnumEvent

For observer-based events that are triggered globally and handled by observers.

```rust
use bevy_enum_event::EnumEvent;

#[derive(EnumEvent, Clone)]
enum PlayerState {
    Idle,                           // Unit variant
    Moving(f32),                    // Tuple variant
    Attacking { target: Entity },   // Named field variant
}
```

### Using with Triggers and Observers

```rust
fn setup(mut app: App) {
    app.add_observer(on_attacking);
}

fn on_attacking(event: On<player_state::Attacking>) {
    println!("Attacking {:?}", event.target);
}

// Trigger the event
fn trigger_attack(mut commands: Commands) {
    commands.trigger(player_state::Attacking { target: Entity::PLACEHOLDER });
}
```

### Deref Feature (default)

Single-field variants automatically implement `Deref`/`DerefMut`:

```rust
#[derive(EnumEvent, Clone)]
enum NetworkEvent {
    MessageReceived(String),  // Automatic deref to String
}

fn on_message(msg: On<network_event::MessageReceived>) {
    let content: &String = &**msg;
}
```

For multi-field variants, mark one field with `#[enum_event(deref)]`:

```rust
#[derive(EnumEvent, Clone)]
enum GameEvent {
    PlayerScored {
        #[enum_event(deref)]
        player: Entity,
        points: u32
    },
}
```

Disable with `default-features = false`.

### Forwarding Derives (`Copy`, `PartialEq`, ...)

The generated variant structs always derive `Clone` and `Debug` (and unit
variants additionally derive `Copy` and `Default`). To put additional derives
on the generated structs — most commonly `Copy`, since many events are tiny and
copy cheaply — list them with `#[enum_event(derive(...))]` on the enum:

```rust
#[derive(EnumEvent, Clone, Copy, PartialEq)]
#[enum_event(derive(Copy, PartialEq))]
enum Input {
    Jump,
    Move { dx: f32, dy: f32 },
}

// input::Jump, input::Move, ... are all `Copy` + `PartialEq`.
```

A derive macro cannot see the *other* derives written next to it (the compiler
consumes the `#[derive(...)]` list before the macro runs), so the derives to
forward must be listed explicitly in `#[enum_event(derive(...))]`. Any derive
whose bound is satisfied by the variant's fields is accepted; naming `Clone` or
`Debug` (or `Copy`/`Default` on a unit variant) is a harmless no-op since those
are already derived.

Derives can also be applied to a single variant — additive to the enum-level
list — which is useful when only some variants can satisfy the bound:

```rust
#[derive(EnumEvent, Clone)]
enum Signal {
    Message(String),          // holds a String: not `Copy`

    #[enum_event(derive(Copy))]
    Tick { frame: u64 },      // this variant's struct is `Copy`
}
```

## EnumMessage

For buffered messages that are written/read between systems using `MessageWriter`/`MessageReader`.
**Important:** Each message type must be registered with `app.add_message::<T>()`.

```rust
use bevy::prelude::*;
use bevy_enum_event::EnumMessage;

#[derive(EnumMessage, Clone)]
enum NetworkCommand {
    Connect { address: String },
    Disconnect,
    SendData(Vec<u8>),
}

fn setup(app: &mut App) {
    // REQUIRED: Register message types
    app.add_message::<network_command::Connect>();
    app.add_message::<network_command::Disconnect>();
    app.add_message::<network_command::SendData>();
}

fn send_commands(mut writer: MessageWriter<network_command::Connect>) {
    writer.write(network_command::Connect {
        address: "127.0.0.1:8080".to_string(),
    });
}

fn receive_commands(mut reader: MessageReader<network_command::Connect>) {
    for cmd in reader.read() {
        println!("Connecting to {}", cmd.address);
    }
}
```

## EnumEntityEvent

Entity-targeted events that trigger entity-specific observers.

### Requirements

- Named fields only (`{ field: Type }` syntax)
- Must have `entity: Entity` field or `#[enum_event(target)]` on a custom field

```rust
use bevy::prelude::*;
use bevy_enum_event::EnumEntityEvent;

#[derive(EnumEntityEvent, Clone, Copy)]
enum PlayerEvent {
    Spawned { entity: Entity },
    Damaged { entity: Entity, amount: f32 },
}

// Entity-specific observer
fn setup(mut commands: Commands) {
    commands.spawn_empty().observe(|damaged: On<player_event::Damaged>| {
        println!("This player took {} damage", damaged.amount);
    });
}
```

### Custom Target Field

```rust
#[derive(EnumEntityEvent, Clone, Copy)]
enum CombatEvent {
    Attack {
        #[enum_event(target)]
        attacker: Entity,
        defender: Entity,
    },
}
```

### Event Propagation

Events can bubble up entity hierarchies:

```rust
// Basic propagation (uses ChildOf)
#[derive(EnumEntityEvent, Clone, Copy)]
#[enum_event(propagate)]
enum UiEvent {
    Click { entity: Entity },
}

// Auto propagation (always bubbles)
#[derive(EnumEntityEvent, Clone, Copy)]
#[enum_event(auto_propagate, propagate)]
enum SystemEvent {
    Update { entity: Entity },
}

// Custom relationship
#[derive(EnumEntityEvent, Clone, Copy)]
#[enum_event(propagate = &'static ::bevy::prelude::ChildOf)]
enum CustomEvent {
    Action { entity: Entity },
}
```

Control propagation in observers:

```rust
fn on_click(mut click: On<ui_event::Click>) {
    click.propagate(true);  // Continue bubbling
    let original = click.original_event_target();
}
```

### Variant-Level Overrides

Override enum-level settings per variant:

```rust
#[derive(EnumEntityEvent, Clone, Copy)]
#[enum_event(propagate)]
enum MixedEvent {
    Normal { entity: Entity },  // Uses enum-level

    #[enum_event(auto_propagate, propagate)]
    AutoEvent { entity: Entity },  // Overrides with auto
}
```

## Generics & Lifetimes

Full support for generic parameters and lifetimes:

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

## Choosing the Right Macro

| Pattern | Macro | Use Case |
|---------|-------|----------|
| Global triggers + observers | `EnumEvent` | Game events, UI notifications, state changes |
| Buffered inter-system communication | `EnumMessage` | Network commands, async results, command queues |
| Entity-targeted observers | `EnumEntityEvent` | Entity interactions, damage systems, component events |

## License

MIT OR Apache-2.0
