use bevy::prelude::*;
use bevy_enum_event::EnumEntityEvent;

#[derive(EnumEntityEvent, Clone, Copy)]
#[enum_event(propagate = &'static ::bevy::prelude::ChildOf)] // WITHOUT auto_propagate
#[allow(dead_code)]
enum TestEvent {
    Action { entity: Entity, data: u16 },
}

#[test]
fn test_generated_code() {
    // Just need to verify it compiles
    let e = Entity::from_bits(1);
    let _ = test_event::Action { entity: e, data: 5 };
}

#[test]
fn test_propagate_method_exists_no_auto() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let parent = app.world_mut().spawn(()).id();
    let child = app.world_mut().spawn(ChildOf(parent)).id();

    // Add observer on child that USES propagate() method
    app.world_mut()
        .entity_mut(child)
        .observe(|mut event: On<test_event::Action>| {
            // This line will fail to compile if propagate() doesn't exist
            event.propagate(false);
        });

    app.update();
}

#[derive(EnumEntityEvent, Clone, Copy)]
#[enum_event(auto_propagate, propagate = &'static ::bevy::prelude::ChildOf)]
#[allow(dead_code)]
enum TestEventAuto {
    Action { entity: Entity, data: u16 },
}

#[test]
fn test_propagate_method_exists_with_auto() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let parent = app.world_mut().spawn(()).id();
    let child = app.world_mut().spawn(ChildOf(parent)).id();

    // Add observer on child that USES propagate() method
    app.world_mut()
        .entity_mut(child)
        .observe(|mut event: On<test_event_auto::Action>| {
            // This line will fail to compile if propagate() doesn't exist
            event.propagate(false);
        });

    app.update();
}
