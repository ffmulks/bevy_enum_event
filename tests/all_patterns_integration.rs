//! Comprehensive integration tests for all three patterns:
//! - `EnumEvent`: Observer-based global events (triggered via `world.trigger()`)
//! - `EnumMessage`: Buffered messages (written via `MessageWriter`, read via `MessageReader`)
//! - `EnumEntityEvent`: Entity-targeted observer events with propagation
//!
//! This test file demonstrates and verifies the correct usage of each pattern
//! according to Bevy 0.17+ nomenclature.

use bevy::prelude::*;
use bevy_enum_event::{EnumEntityEvent, EnumEvent, EnumMessage};

// ============================================================================
// Test EnumEvent - Observer-based global events
// ============================================================================

#[derive(EnumEvent, Clone, Debug)]
#[allow(dead_code)]
enum GlobalGameEvent {
    LevelStarted { level: u32 },
    LevelCompleted { level: u32, score: i32 },
    GamePaused,
    GameResumed,
}

#[derive(Resource, Default)]
struct GlobalEventLog(Vec<String>);

#[test]
fn test_enum_event_with_triggers_and_observers() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<GlobalEventLog>();

    // Add global observers
    app.add_observer(
        |event: On<global_game_event::LevelStarted>, mut log: ResMut<GlobalEventLog>| {
            log.0.push(format!("level_started_{}", event.level));
        },
    );

    app.add_observer(
        |event: On<global_game_event::LevelCompleted>, mut log: ResMut<GlobalEventLog>| {
            log.0
                .push(format!("level_completed_{}_{}", event.level, event.score));
        },
    );

    app.add_observer(
        |_event: On<global_game_event::GamePaused>, mut log: ResMut<GlobalEventLog>| {
            log.0.push("game_paused".to_string());
        },
    );

    app.update();

    // Trigger events
    app.world_mut()
        .trigger(global_game_event::LevelStarted { level: 1 });
    app.update();

    app.world_mut().trigger(global_game_event::LevelCompleted {
        level: 1,
        score: 1000,
    });
    app.update();

    app.world_mut().trigger(global_game_event::GamePaused);
    app.update();

    // Verify log
    let log = app.world().resource::<GlobalEventLog>();
    assert_eq!(log.0.len(), 3);
    assert_eq!(log.0[0], "level_started_1");
    assert_eq!(log.0[1], "level_completed_1_1000");
    assert_eq!(log.0[2], "game_paused");
}

// ============================================================================
// Test EnumMessage - Buffered messages
// ============================================================================

#[derive(EnumMessage, Clone, Debug)]
#[allow(dead_code)]
enum NetworkCommand {
    Connect { address: String },
    Disconnect,
    SendData(Vec<u8>),
}

#[derive(Resource, Default)]
struct NetworkCommandLog(Vec<String>);

#[derive(Resource, Default)]
struct MessagesSent(bool);

fn send_network_commands(
    mut connect_writer: MessageWriter<network_command::Connect>,
    mut disconnect_writer: MessageWriter<network_command::Disconnect>,
    mut data_writer: MessageWriter<network_command::SendData>,
    mut sent: ResMut<MessagesSent>,
) {
    if sent.0 {
        return;
    }
    sent.0 = true;
    connect_writer.write(network_command::Connect {
        address: "127.0.0.1:8080".to_string(),
    });
    data_writer.write(network_command::SendData(vec![1, 2, 3, 4]));
    disconnect_writer.write(network_command::Disconnect);
}

fn process_connect_commands(
    mut reader: MessageReader<network_command::Connect>,
    mut log: ResMut<NetworkCommandLog>,
) {
    for cmd in reader.read() {
        log.0.push(format!("connect_{}", cmd.address));
    }
}

fn process_disconnect_commands(
    mut reader: MessageReader<network_command::Disconnect>,
    mut log: ResMut<NetworkCommandLog>,
) {
    for _cmd in reader.read() {
        log.0.push("disconnect".to_string());
    }
}

fn process_data_commands(
    mut reader: MessageReader<network_command::SendData>,
    mut log: ResMut<NetworkCommandLog>,
) {
    for cmd in reader.read() {
        log.0.push(format!("send_data_{:?}", cmd.0));
    }
}

#[test]
fn test_enum_message_with_writer_reader() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<NetworkCommandLog>();
    app.init_resource::<MessagesSent>();

    // Register message types - REQUIRED for EnumMessage
    app.add_message::<network_command::Connect>();
    app.add_message::<network_command::Disconnect>();
    app.add_message::<network_command::SendData>();

    // Add systems
    app.add_systems(
        Update,
        (
            send_network_commands,
            (
                process_connect_commands,
                process_disconnect_commands,
                process_data_commands,
            )
                .after(send_network_commands),
        ),
    );

    // Run frames
    app.update();
    app.update();

    // Verify log
    let log = app.world().resource::<NetworkCommandLog>();
    assert_eq!(log.0.len(), 3, "Should have 3 commands processed");
    assert!(log.0.contains(&"connect_127.0.0.1:8080".to_string()));
    assert!(log.0.contains(&"disconnect".to_string()));
    assert!(log.0.contains(&"send_data_[1, 2, 3, 4]".to_string()));
}

// ============================================================================
// Test EnumEntityEvent - Entity-targeted observer events
// ============================================================================

#[derive(EnumEntityEvent, Clone, Copy)]
#[allow(dead_code)]
enum EntityHealthEvent {
    Damaged { entity: Entity, amount: u32 },
    Healed { entity: Entity, amount: u32 },
    Died { entity: Entity },
}

#[derive(Component)]
struct Health(u32);

#[derive(Component)]
struct HealthLog(Vec<String>);

#[test]
fn test_enum_entity_event_with_entity_observers() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Spawn entity with health
    let entity = app.world_mut().spawn((Health(100), HealthLog(vec![]))).id();

    // Add entity-specific observer
    app.world_mut().entity_mut(entity).observe(
        |event: On<entity_health_event::Damaged>,
         mut query: Query<(&mut Health, &mut HealthLog)>| {
            if let Ok((mut health, mut log)) = query.get_mut(event.entity) {
                health.0 = health.0.saturating_sub(event.amount);
                log.0.push(format!("damaged_{}", event.amount));
            }
        },
    );

    app.world_mut().entity_mut(entity).observe(
        |event: On<entity_health_event::Healed>,
         mut query: Query<(&mut Health, &mut HealthLog)>| {
            if let Ok((mut health, mut log)) = query.get_mut(event.entity) {
                health.0 += event.amount;
                log.0.push(format!("healed_{}", event.amount));
            }
        },
    );

    app.update();

    // Trigger entity events
    app.world_mut()
        .trigger(entity_health_event::Damaged { entity, amount: 30 });
    app.update();

    app.world_mut()
        .trigger(entity_health_event::Healed { entity, amount: 10 });
    app.update();

    app.world_mut()
        .trigger(entity_health_event::Damaged { entity, amount: 20 });
    app.update();

    // Verify health and log
    let health = app.world().get::<Health>(entity).unwrap();
    let log = app.world().get::<HealthLog>(entity).unwrap();

    assert_eq!(health.0, 60, "Health should be 100 - 30 + 10 - 20 = 60");
    assert_eq!(log.0.len(), 3);
    assert_eq!(log.0[0], "damaged_30");
    assert_eq!(log.0[1], "healed_10");
    assert_eq!(log.0[2], "damaged_20");
}

// ============================================================================
// Test EnumEntityEvent with propagation
// ============================================================================

#[derive(EnumEntityEvent, Clone, Copy)]
#[enum_event(auto_propagate, propagate)]
#[allow(dead_code)]
enum PropagatingEvent {
    Signal { entity: Entity, value: u32 },
}

#[derive(Component)]
struct SignalLog(Vec<String>);

#[test]
fn test_enum_entity_event_propagation() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Create parent-child hierarchy
    let parent = app.world_mut().spawn(SignalLog(vec![])).id();
    let child = app
        .world_mut()
        .spawn((SignalLog(vec![]), ChildOf(parent)))
        .id();

    // Add observer on parent
    app.world_mut().entity_mut(parent).observe(
        |event: On<propagating_event::Signal>, mut query: Query<&mut SignalLog>| {
            if let Ok(mut log) = query.get_mut(event.entity) {
                log.0.push(format!("parent_received_{}", event.value));
            }
        },
    );

    // Add observer on child
    app.world_mut().entity_mut(child).observe(
        |event: On<propagating_event::Signal>, mut query: Query<&mut SignalLog>| {
            if let Ok(mut log) = query.get_mut(event.entity) {
                log.0.push(format!("child_received_{}", event.value));
            }
            // auto_propagate means it automatically propagates to parent
        },
    );

    app.update();

    // Trigger event on child
    app.world_mut().trigger(propagating_event::Signal {
        entity: child,
        value: 42,
    });
    app.update();

    // Verify both received the event (child first, then parent via propagation)
    let child_log = app.world().get::<SignalLog>(child).unwrap();
    let parent_log = app.world().get::<SignalLog>(parent).unwrap();

    assert_eq!(child_log.0.len(), 1);
    assert_eq!(child_log.0[0], "child_received_42");

    assert_eq!(parent_log.0.len(), 1);
    assert_eq!(parent_log.0[0], "parent_received_42");
}

// ============================================================================
// Combined usage - All three patterns in one app
// ============================================================================

// EnumEvent for UI notifications (global)
// The enum only exists to generate the `ui_notification` module used below.
#[derive(EnumEvent, Clone, Debug)]
#[allow(dead_code)]
enum UiNotification {
    ShowToast(String),
    HideAllToasts,
}

// EnumMessage for command queue (buffered)
// The enum only exists to generate the `game_command` module used below.
#[derive(EnumMessage, Clone, Debug)]
#[allow(dead_code)]
enum GameCommand {
    SpawnEnemy { kind: String },
    DespawnAll,
}

// EnumEntityEvent for entity interactions (targeted)
// The enum only exists to generate the `interaction_event` module used below.
#[derive(EnumEntityEvent, Clone, Copy)]
#[allow(dead_code)]
enum InteractionEvent {
    Clicked { entity: Entity },
    Hovered { entity: Entity },
}

#[derive(Resource, Default)]
struct CombinedTestLog(Vec<String>);

fn spawn_enemy_command(mut writer: MessageWriter<game_command::SpawnEnemy>) {
    writer.write(game_command::SpawnEnemy {
        kind: "zombie".to_string(),
    });
}

fn process_spawn_commands(
    mut reader: MessageReader<game_command::SpawnEnemy>,
    mut log: ResMut<CombinedTestLog>,
) {
    for cmd in reader.read() {
        log.0.push(format!("spawn_enemy_{}", cmd.kind));
    }
}

#[test]
fn test_combined_usage_all_patterns() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<CombinedTestLog>();

    // Register message type
    app.add_message::<game_command::SpawnEnemy>();

    // Add message system
    app.add_systems(
        Update,
        (
            spawn_enemy_command,
            process_spawn_commands.after(spawn_enemy_command),
        ),
    );

    // Add global observer for UI notifications
    app.add_observer(
        |event: On<ui_notification::ShowToast>, mut log: ResMut<CombinedTestLog>| {
            log.0.push(format!("ui_toast_{}", event.0));
        },
    );

    // Spawn an entity for interaction events
    let button = app.world_mut().spawn(()).id();

    // Add entity observer
    app.world_mut().entity_mut(button).observe(
        |_event: On<interaction_event::Clicked>, mut log: ResMut<CombinedTestLog>| {
            log.0.push("button_clicked".to_string());
        },
    );

    app.update();

    // Trigger UI notification (EnumEvent)
    app.world_mut()
        .trigger(ui_notification::ShowToast("Game started!".to_string()));
    app.update();

    // Trigger entity interaction (EnumEntityEvent)
    app.world_mut()
        .trigger(interaction_event::Clicked { entity: button });
    app.update();

    // Run more frames for message processing
    app.update();

    // Verify all patterns worked
    let log = app.world().resource::<CombinedTestLog>();
    assert!(
        log.0.iter().any(|s| s == "ui_toast_Game started!"),
        "UI notification should be logged"
    );
    assert!(
        log.0.iter().any(|s| s == "button_clicked"),
        "Button click should be logged"
    );
    assert!(
        log.0.iter().any(|s| s == "spawn_enemy_zombie"),
        "Spawn command should be logged"
    );
}
