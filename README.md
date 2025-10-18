# bevy_enum_events

General-purpose enum to Bevy event conversion macro.

## Overview

`bevy_enum_events` provides a derive macro that automatically generates Bevy event types from enum variants. For each variant in your enum, it creates a corresponding event struct organized in a snake_case module.

## Features

- **Automatic event generation**: One macro generates all variant events
- **Snake case module**: `PlayerState` → `player_state` module
- **Zero boilerplate**: No manual event struct definitions needed
- **Type-safe**: Each variant gets its own distinct event type
- **Bevy integration**: Generated events work seamlessly with Bevy's observer system

## Installation

```toml
[dependencies]
bevy_enum_events = "0.1"
```

## Quick Start

```rust
use bevy::prelude::*;
use bevy_enum_events::EnumEvents;

#[derive(EnumEvents, Clone, Copy, Debug)]
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

## Usage with Bevy Observers

```rust
use bevy::prelude::*;
use bevy_enum_events::EnumEvents;

#[derive(EnumEvents, Clone, Copy)]
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

## Requirements

- Enum must contain only unit variants (no data fields)
- Example:
  ```rust
  // ✅ Valid
  #[derive(EnumEvents)]
  enum State {
      A,
      B,
      C,
  }

  // ❌ Invalid - has data
  #[derive(EnumEvents)]
  enum State {
      A(u32),  // Error: variants must be unit variants
      B,
  }
  ```

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

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
