//! Basic example demonstrating the different types of enum variants supported by `bevy_enum_events`.
//!
//! This example shows:
//! - Unit variants (no data)
//! - Tuple variants (unnamed fields)
//! - Named field variants
//! - Mixed variants in a single enum
//! - Deref behavior for single-field variants
//! - Using the `enum_module_ident!` macro to get module names

#[allow(unused_imports)]
use bevy_enum_events::{enum_module_ident, EnumEvents};

// Example 1: Unit variants only (e.g., simple state machine)
#[derive(EnumEvents, Clone, Copy, Debug)]
#[allow(dead_code)]
enum PlayerState {
    Idle,
    Running,
    Jumping,
}

// Example 2: Mixed variants with data (realistic game events)
#[derive(EnumEvents, Clone, Debug)]
#[allow(dead_code)]
enum GameEvent {
    /// A player wins with their team name
    Victory(String),
    /// Score updated for a team
    ScoreChanged { team: u32, score: i32 },
    /// Game over with no additional data
    GameOver,
}

// Example 3: Single-field variants (benefit from deref feature)
#[derive(EnumEvents, Clone, Debug)]
#[allow(dead_code)]
enum NetworkEvent {
    MessageReceived(String),
    Disconnected,
}

fn main() {
    println!("=== bevy_enum_events Basic Example ===\n");

    // Working with unit variants
    println!("1. Unit Variants (PlayerState):");
    let idle = player_state::Idle;
    let running = player_state::Running;
    println!("  Created states: {idle:?} and {running:?}");
    println!("  Size of unit variant: {} bytes\n", std::mem::size_of_val(&idle));

    // Working with mixed variants
    println!("2. Mixed Variants (GameEvent):");
    let victory = game_event::Victory("Team Red".to_string());
    println!("  Victory event: {}", victory.0);

    let score = game_event::ScoreChanged { team: 1, score: 100 };
    println!("  Score event: Team {} scored {}", score.team, score.score);

    let game_over = game_event::GameOver;
    println!("  Game over: {game_over:?}\n");

    // Working with single-field variant and deref
    println!("3. Single-field Variants with Deref (NetworkEvent):");
    let msg = network_event::MessageReceived("Hello, Bevy!".to_string());

    #[cfg(feature = "deref")]
    {
        // With deref feature enabled (default), we can access the inner value directly
        println!("  Message (via deref): {}", &*msg);
        println!("  Message length (via deref): {} chars", msg.len());
    }

    #[cfg(not(feature = "deref"))]
    {
        // Without deref, access via .0
        println!("  Message (via .0): {}", msg.0);
        println!("  Message length (via .0): {} chars", msg.0.len());
    }

    println!("\nAll event types work seamlessly with Bevy's event system!");

    // Demonstrate enum_module_ident! macro
    println!("\n4. Using enum_module_ident! macro:");
    println!("  The enum_module_ident! macro converts enum names to module identifiers.");
    println!("  PlayerState → {}", stringify!(enum_module_ident!(PlayerState)));
    println!("  GameEvent → {}", stringify!(enum_module_ident!(GameEvent)));
    println!("  NetworkEvent → {}", stringify!(enum_module_ident!(NetworkEvent)));

    // This is particularly useful for library authors who need to programmatically
    // reference the generated module names in their own macros or code generation.
    println!("\n  This macro is useful for library authors building on bevy_enum_events.");
}
