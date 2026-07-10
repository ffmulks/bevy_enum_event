//! Compatibility tests targeting Bevy 0.19-specific changes.
//!
//! Bevy 0.19 shortened the built-in component lifecycle events and pulled them
//! into the prelude under bare names: `OnAdd` -> `Add`, `OnInsert` -> `Insert`,
//! `OnReplace` -> `Replace`, `OnRemove` -> `Remove`, `OnDespawn` -> `Despawn`.
//!
//! Because generated modules emit `use super::*;`, a user enum whose variants
//! collide with those prelude names could theoretically clash with the imported
//! Bevy types. A locally-defined `struct Add { .. }` shadows the glob-imported
//! `bevy::prelude::Add`, so generation must remain sound. These tests lock that
//! in so a future Bevy prelude change can't silently break variant naming.

use bevy::prelude::*;
use bevy_enum_event::{EnumEntityEvent, EnumEvent, EnumMessage};

// Variants named exactly after Bevy 0.19's prelude lifecycle events.
#[derive(EnumEvent, Clone, Debug)]
#[allow(dead_code)]
enum LifecycleNamedEvent {
    Add,
    Insert(u32),
    Replace { value: u32 },
    Remove,
    Despawn { reason: String },
}

#[test]
fn test_lifecycle_named_event_variants_do_not_collide() {
    // Each generated struct must be constructible and distinct from Bevy's
    // prelude types of the same bare name.
    let _: lifecycle_named_event::Add = lifecycle_named_event::Add;
    let insert = lifecycle_named_event::Insert(7);
    assert_eq!(insert.0, 7);
    let replace = lifecycle_named_event::Replace { value: 9 };
    assert_eq!(replace.value, 9);
    let _: lifecycle_named_event::Remove = lifecycle_named_event::Remove;
    let despawn = lifecycle_named_event::Despawn {
        reason: "gone".to_string(),
    };
    assert_eq!(despawn.reason, "gone");
}

#[derive(Resource, Default)]
struct LifecycleEventLog(Vec<String>);

#[test]
fn test_lifecycle_named_event_observers_fire() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<LifecycleEventLog>();

    // Observing our generated `Add`/`Replace` types must not be confused with
    // Bevy's own component lifecycle `Add`/`Replace` events.
    app.add_observer(
        |_ev: On<lifecycle_named_event::Add>, mut log: ResMut<LifecycleEventLog>| {
            log.0.push("add".to_string());
        },
    );
    app.add_observer(
        |ev: On<lifecycle_named_event::Replace>, mut log: ResMut<LifecycleEventLog>| {
            log.0.push(format!("replace_{}", ev.value));
        },
    );

    app.update();
    app.world_mut().trigger(lifecycle_named_event::Add);
    app.world_mut()
        .trigger(lifecycle_named_event::Replace { value: 42 });
    app.update();

    let log = app.world().resource::<LifecycleEventLog>();
    assert_eq!(log.0, vec!["add".to_string(), "replace_42".to_string()]);
}

// Same naming stress test for messages.
#[derive(EnumMessage, Clone, Debug)]
#[allow(dead_code)]
enum LifecycleNamedMessage {
    Add(u32),
    Remove { id: u32 },
}

#[test]
fn test_lifecycle_named_message_variants() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<lifecycle_named_message::Add>();

    app.world_mut()
        .write_message(lifecycle_named_message::Add(3));

    let messages = app
        .world()
        .resource::<Messages<lifecycle_named_message::Add>>();
    assert_eq!(messages.len(), 1);
}

// Same naming stress test for entity events.
#[derive(EnumEntityEvent, Clone, Copy)]
#[allow(dead_code)]
enum LifecycleNamedEntityEvent {
    Insert { entity: Entity },
    Despawn { entity: Entity, code: u32 },
}

#[derive(Component)]
struct EntityEventLog(Vec<String>);

#[test]
fn test_lifecycle_named_entity_event_observers_fire() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let entity = app.world_mut().spawn(EntityEventLog(vec![])).id();
    app.world_mut().entity_mut(entity).observe(
        |ev: On<lifecycle_named_entity_event::Despawn>, mut query: Query<&mut EntityEventLog>| {
            if let Ok(mut log) = query.get_mut(ev.entity) {
                log.0.push(format!("despawn_{}", ev.code));
            }
        },
    );

    app.update();
    app.world_mut()
        .trigger(lifecycle_named_entity_event::Despawn { entity, code: 5 });
    app.update();

    let log = app.world().get::<EntityEventLog>(entity).unwrap();
    assert_eq!(log.0, vec!["despawn_5".to_string()]);
}

// Bevy 0.19 made `#[derive(Resource)]` also implement `Component`. Confirm our
// generated event/message types are independent of that and don't accidentally
// pick up resource/component semantics through the glob import.
#[derive(EnumEvent, Clone, Debug)]
#[allow(dead_code)]
enum PlainEvent {
    Fired { code: u32 },
}

#[test]
fn test_generated_types_are_plain_events() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let observed = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let observed_clone = observed.clone();
    app.add_observer(move |ev: On<plain_event::Fired>| {
        observed_clone.store(ev.code, std::sync::atomic::Ordering::SeqCst);
    });

    app.update();
    app.world_mut().trigger(plain_event::Fired { code: 99 });
    app.update();

    assert_eq!(observed.load(std::sync::atomic::Ordering::SeqCst), 99);
}
