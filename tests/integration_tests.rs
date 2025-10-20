//! Integration tests that actually fire events and verify observer behavior
//! These tests demonstrate real Bevy observer patterns without Arc/Mutex magic

use bevy::prelude::*;
use bevy_enum_event::EnumEntityEvent;

// ============================================================================
// Basic event firing
// ============================================================================

#[derive(Component)]
struct EventCounter(usize);

#[derive(EnumEntityEvent, Clone, Copy)]
#[allow(dead_code)]
enum BasicEvent {
    Triggered { entity: Entity },
}

#[test]
fn test_basic_event_firing() {
    let mut app = App::new();

    // Add observer that increments a counter
    app.add_observer(
        |_event: On<basic_event::Triggered>, mut query: Query<&mut EventCounter>| {
            for mut counter in &mut query {
                counter.0 += 1;
            }
        },
    );

    // Spawn an entity with a counter
    let entity_id = app.world_mut().spawn(EventCounter(0)).id();

    // Trigger event
    app.world_mut()
        .trigger(basic_event::Triggered { entity: entity_id });

    // Run the app to process observers
    app.update();

    // Verify counter was incremented
    let counter = app.world().get::<EventCounter>(entity_id).unwrap();
    assert_eq!(counter.0, 1, "Observer should have incremented the counter");
}

// ============================================================================
// Entity-specific vs global observers
// ============================================================================

#[derive(Component)]
struct EntityEventCount(usize);

#[derive(EnumEntityEvent, Clone, Copy)]
#[allow(dead_code)]
enum SpecificEvent {
    Happened { entity: Entity },
}

#[test]
fn test_entity_specific_vs_global_observers() {
    let mut app = App::new();

    // Add global observer (triggers for ALL events)
    app.add_observer(
        |_event: On<specific_event::Happened>, mut query: Query<&mut EntityEventCount>| {
            // Increment all counters (global effect)
            for mut count in &mut query {
                count.0 += 1;
            }
        },
    );

    // Spawn two entities with counters
    let entity1 = app.world_mut().spawn(EntityEventCount(0)).id();
    let entity2 = app.world_mut().spawn(EntityEventCount(0)).id();

    // Trigger event on entity1
    app.world_mut()
        .trigger(specific_event::Happened { entity: entity1 });
    app.update();

    // Both should have count of 1 (global observer affects both)
    let count1 = app.world().get::<EntityEventCount>(entity1).unwrap();
    let count2 = app.world().get::<EntityEventCount>(entity2).unwrap();
    assert_eq!(count1.0, 1, "Entity1 should have count 1 after first event");
    assert_eq!(
        count2.0, 1,
        "Entity2 should have count 1 from global observer"
    );

    // Trigger event on entity2
    app.world_mut()
        .trigger(specific_event::Happened { entity: entity2 });
    app.update();

    // Both should now have count of 2
    let count1 = app.world().get::<EntityEventCount>(entity1).unwrap();
    let count2 = app.world().get::<EntityEventCount>(entity2).unwrap();
    assert_eq!(
        count1.0, 2,
        "Entity1 should have count 2 from global observer"
    );
    assert_eq!(
        count2.0, 2,
        "Entity2 should have count 2 after second event"
    );
}

// ============================================================================
// Entity-targeted observers
// ============================================================================

#[derive(Component)]
struct HitCount(usize);

#[derive(EnumEntityEvent, Clone, Copy)]
#[allow(dead_code)]
enum TargetedEvent {
    Hit { entity: Entity },
}

#[test]
fn test_entity_targeted_observers() {
    let mut app = App::new();

    // Spawn two entities with hit counters
    let entity1 = app.world_mut().spawn(HitCount(0)).id();
    let entity2 = app.world_mut().spawn(HitCount(0)).id();

    // Add entity-specific observer on entity1
    app.world_mut().entity_mut(entity1).observe(
        |event: On<targeted_event::Hit>, mut query: Query<&mut HitCount>| {
            if let Ok(mut count) = query.get_mut(event.entity) {
                count.0 += 1;
            }
        },
    );

    // Add entity-specific observer on entity2
    app.world_mut().entity_mut(entity2).observe(
        |event: On<targeted_event::Hit>, mut query: Query<&mut HitCount>| {
            if let Ok(mut count) = query.get_mut(event.entity) {
                count.0 += 1;
            }
        },
    );

    // Trigger event on entity1 only
    app.world_mut()
        .trigger(targeted_event::Hit { entity: entity1 });
    app.update();

    // Verify only entity1's observer was triggered
    let count1 = app.world().get::<HitCount>(entity1).unwrap();
    let count2 = app.world().get::<HitCount>(entity2).unwrap();
    assert_eq!(count1.0, 1, "Entity1 should have been hit");
    assert_eq!(count2.0, 0, "Entity2 should not have been hit");

    // Trigger event on entity2 only
    app.world_mut()
        .trigger(targeted_event::Hit { entity: entity2 });
    app.update();

    // Verify only entity2's observer was triggered this time
    let count1 = app.world().get::<HitCount>(entity1).unwrap();
    let count2 = app.world().get::<HitCount>(entity2).unwrap();
    assert_eq!(count1.0, 1, "Entity1 count should still be 1");
    assert_eq!(count2.0, 1, "Entity2 should now have been hit");
}

// ============================================================================
// Custom target field
// ============================================================================

#[derive(Component)]
struct AttackCount(usize);

#[derive(Component)]
struct DefenseCount(usize);

#[derive(EnumEntityEvent, Clone, Copy)]
#[allow(dead_code)]
enum AttackEvent {
    Hit {
        #[enum_event(target)]
        attacker: Entity,
        defender: Entity,
    },
}

#[test]
fn test_custom_target_field() {
    let mut app = App::new();

    // Spawn attacker and defender with counters
    let attacker = app.world_mut().spawn(AttackCount(0)).id();
    let defender = app.world_mut().spawn(DefenseCount(0)).id();

    // Add observer on attacker (should trigger because attacker is the target)
    app.world_mut().entity_mut(attacker).observe(
        |event: On<attack_event::Hit>, mut query: Query<&mut AttackCount>| {
            if let Ok(mut count) = query.get_mut(event.attacker) {
                count.0 += 1;
            }
        },
    );

    // Add observer on defender (should NOT trigger because defender is not the target)
    app.world_mut().entity_mut(defender).observe(
        |event: On<attack_event::Hit>, mut query: Query<&mut DefenseCount>| {
            if let Ok(mut count) = query.get_mut(event.defender) {
                count.0 += 1;
            }
        },
    );

    // Trigger attack event
    app.world_mut()
        .trigger(attack_event::Hit { attacker, defender });
    app.update();

    // Verify only attacker observer was triggered
    let attack_count = app.world().get::<AttackCount>(attacker).unwrap();
    let defense_count = app.world().get::<DefenseCount>(defender).unwrap();

    assert_eq!(
        attack_count.0, 1,
        "Attacker observer should trigger (attacker is the event target)"
    );
    assert_eq!(
        defense_count.0, 0,
        "Defender observer should NOT trigger (defender is not the event target)"
    );
}

// ============================================================================
// Multiple events from same enum
// ============================================================================

#[derive(Component)]
struct LifecycleLog(Vec<String>);

#[derive(EnumEntityEvent, Clone)]
#[allow(dead_code)]
enum LifecycleEvent {
    Spawned { entity: Entity },
    Updated { entity: Entity },
    Destroyed { entity: Entity },
}

#[test]
fn test_multiple_events_from_enum() {
    let mut app = App::new();

    // Spawn entity with lifecycle log
    let entity_id = app.world_mut().spawn(LifecycleLog(vec![])).id();

    // Add observers for each event type
    app.add_observer(
        |_event: On<lifecycle_event::Spawned>, mut query: Query<&mut LifecycleLog>| {
            for mut log in &mut query {
                log.0.push("spawned".to_string());
            }
        },
    );

    app.add_observer(
        |_event: On<lifecycle_event::Updated>, mut query: Query<&mut LifecycleLog>| {
            for mut log in &mut query {
                log.0.push("updated".to_string());
            }
        },
    );

    app.add_observer(
        |_event: On<lifecycle_event::Destroyed>, mut query: Query<&mut LifecycleLog>| {
            for mut log in &mut query {
                log.0.push("destroyed".to_string());
            }
        },
    );

    // Trigger events in sequence
    app.world_mut()
        .trigger(lifecycle_event::Spawned { entity: entity_id });
    app.update();

    app.world_mut()
        .trigger(lifecycle_event::Updated { entity: entity_id });
    app.update();

    app.world_mut()
        .trigger(lifecycle_event::Destroyed { entity: entity_id });
    app.update();

    // Verify log
    let log = app.world().get::<LifecycleLog>(entity_id).unwrap();
    assert_eq!(log.0.len(), 3, "Should have 3 log entries");
    assert_eq!(log.0[0], "spawned");
    assert_eq!(log.0[1], "updated");
    assert_eq!(log.0[2], "destroyed");
}

// ============================================================================
// Event propagation with ChildOf (armor/goblin example)
// ============================================================================

#[derive(Component, Deref, DerefMut)]
struct HitPoints(u16);

#[derive(Component, Deref)]
struct Armor(u16);

#[derive(EnumEntityEvent, Clone, Copy)]
#[enum_event(auto_propagate, propagate)]
#[allow(dead_code)]
enum ArmorEvent {
    Attack { entity: Entity, damage: u16 },
}

#[test]
fn test_armor_goblin_propagation() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Spawn parent (goblin) with child (armor) to establish ChildOf relationship
    let goblin_id = app.world_mut().spawn(HitPoints(50)).id();
    let armor_id = app.world_mut().spawn((Armor(10), ChildOf(goblin_id))).id();

    // Add observer on goblin - takes damage if attack gets through armor
    app.world_mut().entity_mut(goblin_id).observe(
        |attack: On<armor_event::Attack>, mut hp_query: Query<&mut HitPoints>| {
            if let Ok(mut hp) = hp_query.get_mut(attack.entity) {
                hp.0 = hp.0.saturating_sub(attack.damage);
            }
        },
    );

    // Add observer on armor - blocks some damage, allows propagation if damage exceeds armor
    app.world_mut().entity_mut(armor_id).observe(
        |mut attack: On<armor_event::Attack>, armor_query: Query<&Armor>| {
            if let Ok(armor) = armor_query.get(attack.entity) {
                let damage_through = attack.damage.saturating_sub(**armor);

                if damage_through > 0 {
                    // Some damage gets through - let it propagate to parent
                    attack.damage = damage_through;
                    // propagate(true) is implicit with auto_propagate
                } else {
                    // Armor blocked all damage - stop propagation
                    attack.propagate(false);
                }
            }
        },
    );

    // Flush to ensure observers are registered
    app.update();

    // Test 1: Attack armor with 15 damage (armor blocks 10, so 5 should get through to goblin)
    app.world_mut().trigger(armor_event::Attack {
        entity: armor_id,
        damage: 15,
    });
    app.update();

    // Verify goblin took 5 damage (15 - 10 armor)
    let goblin_hp = app.world().get::<HitPoints>(goblin_id).unwrap();
    assert_eq!(
        **goblin_hp, 45,
        "Goblin should have 45 HP (50 - 5 damage that got through armor)"
    );

    // Test 2: Attack armor with 5 damage (armor blocks all of it)
    app.world_mut().trigger(armor_event::Attack {
        entity: armor_id,
        damage: 5,
    });
    app.update();

    // Verify goblin still has 45 HP (armor blocked all 5 damage)
    let goblin_hp = app.world().get::<HitPoints>(goblin_id).unwrap();
    assert_eq!(
        **goblin_hp, 45,
        "Goblin should still have 45 HP (armor blocked the 5 damage)"
    );

    // Test 3: Attack armor with 20 damage (armor blocks 10, 10 gets through)
    app.world_mut().trigger(armor_event::Attack {
        entity: armor_id,
        damage: 20,
    });
    app.update();

    // Verify goblin took 10 more damage
    let goblin_hp = app.world().get::<HitPoints>(goblin_id).unwrap();
    assert_eq!(
        **goblin_hp, 35,
        "Goblin should have 35 HP (45 - 10 damage that got through armor)"
    );
}

// ============================================================================
// Event propagation with custom relationship
// ============================================================================

#[derive(Component)]
#[relationship(relationship_target = ArmorParts)]
pub struct ArmorOf(Entity);

#[derive(Component)]
#[relationship_target(relationship = ArmorOf)]
pub struct ArmorParts(Vec<Entity>);


#[derive(EnumEntityEvent, Clone, Copy)]
#[enum_event(auto_propagate, propagate =  &'static crate::ArmorOf)]
#[allow(dead_code)]
enum CustomArmorEvent {
    Attack { entity: Entity, damage: u16 },
}

#[test]
fn test_armor_goblin_propagation_custom() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Spawn parent (goblin) with child (armor) to establish ChildOf relationship
    let goblin_id = app.world_mut().spawn(HitPoints(50)).id();
    let armor_id = app
        .world_mut()
        .spawn((Armor(10), ArmorOf(goblin_id)))
        .id();

    // Add observer on goblin - takes damage if attack gets through armor
    app.world_mut().entity_mut(goblin_id).observe(
        |attack: On<custom_armor_event::Attack>, mut hp_query: Query<&mut HitPoints>| {
            if let Ok(mut hp) = hp_query.get_mut(attack.entity) {
                hp.0 = hp.0.saturating_sub(attack.damage);
            }
        },
    );

    // Add observer on armor - blocks some damage, allows propagation if damage exceeds armor
    app.world_mut().entity_mut(armor_id).observe(
        |mut attack: On<custom_armor_event::Attack>, armor_query: Query<&Armor>| {
            if let Ok(armor) = armor_query.get(attack.entity) {
                let damage_through = attack.damage.saturating_sub(**armor);

                if damage_through > 0 {
                    // Some damage gets through - let it propagate to parent
                    attack.damage = damage_through;
                    // propagate(true) is implicit with auto_propagate
                } else {
                    // Armor blocked all damage - stop propagation
                    attack.propagate(false);
                }
            }
        },
    );

    // Flush to ensure observers are registered
    app.update();

    // Test 1: Attack armor with 15 damage (armor blocks 10, so 5 should get through to goblin)
    app.world_mut().trigger(custom_armor_event::Attack {
        entity: armor_id,
        damage: 15,
    });
    app.update();

    // Verify goblin took 5 damage (15 - 10 armor)
    let goblin_hp = app.world().get::<HitPoints>(goblin_id).unwrap();
    assert_eq!(
        **goblin_hp, 45,
        "Goblin should have 45 HP (50 - 5 damage that got through armor)"
    );

    // Test 2: Attack armor with 5 damage (armor blocks all of it)
    app.world_mut().trigger(custom_armor_event::Attack {
        entity: armor_id,
        damage: 5,
    });
    app.update();

    // Verify goblin still has 45 HP (armor blocked all 5 damage)
    let goblin_hp = app.world().get::<HitPoints>(goblin_id).unwrap();
    assert_eq!(
        **goblin_hp, 45,
        "Goblin should still have 45 HP (armor blocked the 5 damage)"
    );

    // Test 3: Attack armor with 20 damage (armor blocks 10, 10 gets through)
    app.world_mut().trigger(custom_armor_event::Attack {
        entity: armor_id,
        damage: 20,
    });
    app.update();

    // Verify goblin took 10 more damage
    let goblin_hp = app.world().get::<HitPoints>(goblin_id).unwrap();
    assert_eq!(
        **goblin_hp, 35,
        "Goblin should have 35 HP (45 - 10 damage that got through armor)"
    );
}

// ============================================================================
// Scenario 1: No enum-level propagation, variant-level definitions
// ============================================================================

#[derive(Component)]
struct DamageLog(Vec<String>);

// Custom relationship for Scenario 1
#[derive(Component)]
#[relationship(relationship_target = ShieldedBy)]
pub struct ShieldOf(Entity);

#[derive(Component)]
#[relationship_target(relationship = ShieldOf)]
pub struct ShieldedBy(Vec<Entity>);

// Enum with NO propagation at enum level
// Each variant demonstrates a different propagation configuration
#[derive(EnumEntityEvent, Clone, Copy)]
#[allow(dead_code)]
enum VariantLevelPropagateEvent {
    // Variant A: No propagation (baseline - should not propagate)
    NoPropagation { entity: Entity, damage: u16 },

    // Variant B: Basic propagate with default relationship (manual control)
    #[enum_event(propagate)]
    BasicPropagate { entity: Entity, damage: u16 },

    // Variant C: Auto-propagate with default relationship (ChildOf)
    #[enum_event(auto_propagate, propagate)]
    AutoPropagate { entity: Entity, damage: u16 },

    // Variant D: Auto-propagate with custom relationship (ShieldOf)
    #[enum_event(auto_propagate, propagate = &'static crate::ShieldOf)]
    AutoPropagateCustom { entity: Entity, damage: u16 },
}

#[test]
#[allow(clippy::too_many_lines)]
fn test_scenario1_no_enum_propagation_variant_definitions() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Setup hierarchy: Parent <- Child (using ChildOf)
    let parent = app.world_mut().spawn(DamageLog(vec![])).id();
    let child = app.world_mut().spawn((DamageLog(vec![]), ChildOf(parent))).id();

    // Setup hierarchy with custom relationship: Protected <- Shield (using ShieldOf)
    let protected = app.world_mut().spawn(DamageLog(vec![])).id();
    let shield = app.world_mut().spawn((DamageLog(vec![]), ShieldOf(protected))).id();

    // Add observers on parents
    app.world_mut().entity_mut(parent).observe(
        |event: On<variant_level_propagate_event::NoPropagation>, mut query: Query<&mut DamageLog>| {
            if let Ok(mut log) = query.get_mut(event.entity) {
                log.0.push(format!("parent_no_prop_{}", event.damage));
            }
        },
    );

    app.world_mut().entity_mut(parent).observe(
        |event: On<variant_level_propagate_event::BasicPropagate>, mut query: Query<&mut DamageLog>| {
            if let Ok(mut log) = query.get_mut(event.entity) {
                log.0.push(format!("parent_basic_{}", event.damage));
            }
        },
    );

    app.world_mut().entity_mut(parent).observe(
        |event: On<variant_level_propagate_event::AutoPropagate>, mut query: Query<&mut DamageLog>| {
            if let Ok(mut log) = query.get_mut(event.entity) {
                log.0.push(format!("parent_auto_{}", event.damage));
            }
        },
    );

    app.world_mut().entity_mut(protected).observe(
        |event: On<variant_level_propagate_event::AutoPropagateCustom>, mut query: Query<&mut DamageLog>| {
            if let Ok(mut log) = query.get_mut(event.entity) {
                log.0.push(format!("protected_auto_custom_{}", event.damage));
            }
        },
    );

    // Add observers on children
    app.world_mut().entity_mut(child).observe(
        |event: On<variant_level_propagate_event::NoPropagation>, mut query: Query<&mut DamageLog>| {
            if let Ok(mut log) = query.get_mut(event.entity) {
                log.0.push(format!("child_no_prop_{}", event.damage));
            }
        },
    );

    app.world_mut().entity_mut(child).observe(
        |mut event: On<variant_level_propagate_event::BasicPropagate>, mut query: Query<&mut DamageLog>| {
            if let Ok(mut log) = query.get_mut(event.entity) {
                log.0.push(format!("child_basic_{}", event.damage));
                // Manually enable propagation for this test
                event.propagate(true);
            }
        },
    );

    app.world_mut().entity_mut(child).observe(
        |event: On<variant_level_propagate_event::AutoPropagate>, mut query: Query<&mut DamageLog>| {
            if let Ok(mut log) = query.get_mut(event.entity) {
                log.0.push(format!("child_auto_{}", event.damage));
                // Auto-propagate is implicit - no need to call propagate(true)
            }
        },
    );

    app.world_mut().entity_mut(shield).observe(
        |event: On<variant_level_propagate_event::AutoPropagateCustom>, mut query: Query<&mut DamageLog>| {
            if let Ok(mut log) = query.get_mut(event.entity) {
                log.0.push(format!("shield_auto_custom_{}", event.damage));
                // Auto-propagate is implicit with custom relationship
            }
        },
    );

    app.update();

    // Test Variant A: NoPropagation - should NOT propagate to parent
    app.world_mut().trigger(variant_level_propagate_event::NoPropagation {
        entity: child,
        damage: 10,
    });
    app.update();

    let child_log = app.world().get::<DamageLog>(child).unwrap();
    assert_eq!(child_log.0.len(), 1, "Child should have received NoPropagation event");
    assert_eq!(child_log.0[0], "child_no_prop_10");

    let parent_log = app.world().get::<DamageLog>(parent).unwrap();
    assert_eq!(parent_log.0.len(), 0, "Parent should NOT receive NoPropagation event (no propagation)");

    // Test Variant B: BasicPropagate - should propagate when manually enabled
    app.world_mut().trigger(variant_level_propagate_event::BasicPropagate {
        entity: child,
        damage: 20,
    });
    app.update();

    let child_log = app.world().get::<DamageLog>(child).unwrap();
    assert_eq!(child_log.0.len(), 2);
    assert_eq!(child_log.0[1], "child_basic_20");

    let parent_log = app.world().get::<DamageLog>(parent).unwrap();
    assert_eq!(parent_log.0.len(), 1, "Parent should receive BasicPropagate event (manual propagation)");
    assert_eq!(parent_log.0[0], "parent_basic_20");

    // Test Variant C: AutoPropagate - should auto-propagate with ChildOf
    app.world_mut().trigger(variant_level_propagate_event::AutoPropagate {
        entity: child,
        damage: 30,
    });
    app.update();

    let child_log = app.world().get::<DamageLog>(child).unwrap();
    assert_eq!(child_log.0.len(), 3);
    assert_eq!(child_log.0[2], "child_auto_30");

    let parent_log = app.world().get::<DamageLog>(parent).unwrap();
    assert_eq!(parent_log.0.len(), 2, "Parent should receive AutoPropagate event (auto-propagation)");
    assert_eq!(parent_log.0[1], "parent_auto_30");

    // Test Variant D: AutoPropagateCustom - should auto-propagate with ShieldOf
    app.world_mut().trigger(variant_level_propagate_event::AutoPropagateCustom {
        entity: shield,
        damage: 40,
    });
    app.update();

    let shield_log = app.world().get::<DamageLog>(shield).unwrap();
    assert_eq!(shield_log.0.len(), 1);
    assert_eq!(shield_log.0[0], "shield_auto_custom_40");

    let protected_log = app.world().get::<DamageLog>(protected).unwrap();
    assert_eq!(protected_log.0.len(), 1, "Protected should receive event (auto-propagate with custom relationship)");
    assert_eq!(protected_log.0[0], "protected_auto_custom_40");
}

// ============================================================================
// Scenario 2: Enum-level propagation with variant overrides
// ============================================================================

// Custom relationship for Scenario 2
#[derive(Component)]
#[relationship(relationship_target = MountedBy)]
pub struct MountOf(Entity);

#[derive(Component)]
#[relationship_target(relationship = MountOf)]
pub struct MountedBy(Vec<Entity>);

// Enum with auto_propagate + ChildOf at enum level
// Variants demonstrate complete override behavior (not merge)
#[derive(EnumEntityEvent, Clone, Copy)]
#[enum_event(auto_propagate, propagate)]
#[allow(dead_code)]
enum EnumLevelPropagateEvent {
    // Variant A: No override - inherits enum-level (auto_propagate + ChildOf)
    InheritEnum { entity: Entity, value: u16 },

    // Variant B: Override with manual propagate + default relationship
    #[enum_event(propagate)]
    ManualDefault { entity: Entity, value: u16 },

    // Variant C: Override with auto_propagate + default relationship
    #[enum_event(auto_propagate, propagate)]
    AutoDefault { entity: Entity, value: u16 },

    // Variant D: Override with manual propagate + custom relationship (MountOf)
    #[enum_event(propagate = &'static crate::MountOf)]
    ManualCustom { entity: Entity, value: u16 },

    // Variant E: Override with auto_propagate + custom relationship (MountOf)
    #[enum_event(auto_propagate, propagate = &'static crate::MountOf)]
    AutoCustom { entity: Entity, value: u16 },
}

#[test]
#[allow(clippy::too_many_lines)]
fn test_scenario2_enum_propagation_with_variant_overrides() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Setup hierarchy with ChildOf relationship: Parent <- Child
    let parent_child = app.world_mut().spawn(DamageLog(vec![])).id();
    let child_child = app.world_mut().spawn((DamageLog(vec![]), ChildOf(parent_child))).id();

    // Setup hierarchy with MountOf relationship: Rider <- Mount
    let rider = app.world_mut().spawn(DamageLog(vec![])).id();
    let mount = app.world_mut().spawn((DamageLog(vec![]), MountOf(rider))).id();

    // === Variant A: InheritEnum (inherits auto_propagate + ChildOf) ===
    app.world_mut().entity_mut(parent_child).observe(
        |event: On<enum_level_propagate_event::InheritEnum>, mut query: Query<&mut DamageLog>| {
            if let Ok(mut log) = query.get_mut(event.entity) {
                log.0.push(format!("parent_inherit_{}", event.value));
            }
        },
    );

    app.world_mut().entity_mut(child_child).observe(
        |event: On<enum_level_propagate_event::InheritEnum>, mut query: Query<&mut DamageLog>| {
            if let Ok(mut log) = query.get_mut(event.entity) {
                log.0.push(format!("child_inherit_{}", event.value));
                // auto_propagate is implicit - no need to call propagate(true)
            }
        },
    );

    // === Variant B: ManualDefault (manual propagate + default ChildOf) ===
    app.world_mut().entity_mut(parent_child).observe(
        |event: On<enum_level_propagate_event::ManualDefault>, mut query: Query<&mut DamageLog>| {
            if let Ok(mut log) = query.get_mut(event.entity) {
                log.0.push(format!("parent_manual_def_{}", event.value));
            }
        },
    );

    app.world_mut().entity_mut(child_child).observe(
        |mut event: On<enum_level_propagate_event::ManualDefault>, mut query: Query<&mut DamageLog>| {
            if let Ok(mut log) = query.get_mut(event.entity) {
                log.0.push(format!("child_manual_def_{}", event.value));
                // Manual propagate - must explicitly enable
                event.propagate(true);
            }
        },
    );

    // === Variant C: AutoDefault (auto_propagate + default ChildOf) ===
    app.world_mut().entity_mut(parent_child).observe(
        |event: On<enum_level_propagate_event::AutoDefault>, mut query: Query<&mut DamageLog>| {
            if let Ok(mut log) = query.get_mut(event.entity) {
                log.0.push(format!("parent_auto_def_{}", event.value));
            }
        },
    );

    app.world_mut().entity_mut(child_child).observe(
        |event: On<enum_level_propagate_event::AutoDefault>, mut query: Query<&mut DamageLog>| {
            if let Ok(mut log) = query.get_mut(event.entity) {
                log.0.push(format!("child_auto_def_{}", event.value));
                // auto_propagate is implicit
            }
        },
    );

    // === Variant D: ManualCustom (manual propagate + MountOf) ===
    app.world_mut().entity_mut(rider).observe(
        |event: On<enum_level_propagate_event::ManualCustom>, mut query: Query<&mut DamageLog>| {
            if let Ok(mut log) = query.get_mut(event.entity) {
                log.0.push(format!("rider_manual_custom_{}", event.value));
            }
        },
    );

    app.world_mut().entity_mut(mount).observe(
        |mut event: On<enum_level_propagate_event::ManualCustom>, mut query: Query<&mut DamageLog>| {
            if let Ok(mut log) = query.get_mut(event.entity) {
                log.0.push(format!("mount_manual_custom_{}", event.value));
                // Manual propagate with custom relationship
                event.propagate(true);
            }
        },
    );

    // === Variant E: AutoCustom (auto_propagate + MountOf) ===
    app.world_mut().entity_mut(rider).observe(
        |event: On<enum_level_propagate_event::AutoCustom>, mut query: Query<&mut DamageLog>| {
            if let Ok(mut log) = query.get_mut(event.entity) {
                log.0.push(format!("rider_auto_custom_{}", event.value));
            }
        },
    );

    app.world_mut().entity_mut(mount).observe(
        |event: On<enum_level_propagate_event::AutoCustom>, mut query: Query<&mut DamageLog>| {
            if let Ok(mut log) = query.get_mut(event.entity) {
                log.0.push(format!("mount_auto_custom_{}", event.value));
                // auto_propagate is implicit with custom relationship
            }
        },
    );

    app.update();

    // === Test Variant A: InheritEnum ===
    // Should inherit enum-level: auto_propagate + ChildOf
    app.world_mut().trigger(enum_level_propagate_event::InheritEnum {
        entity: child_child,
        value: 10,
    });
    app.update();

    let child_log = app.world().get::<DamageLog>(child_child).unwrap();
    assert_eq!(child_log.0.len(), 1);
    assert_eq!(child_log.0[0], "child_inherit_10");

    let parent_log = app.world().get::<DamageLog>(parent_child).unwrap();
    assert_eq!(parent_log.0.len(), 1, "Parent should receive InheritEnum (inherited auto_propagate)");
    assert_eq!(parent_log.0[0], "parent_inherit_10");

    // === Test Variant B: ManualDefault ===
    // Override: manual propagate + default ChildOf
    app.world_mut().trigger(enum_level_propagate_event::ManualDefault {
        entity: child_child,
        value: 20,
    });
    app.update();

    let child_log = app.world().get::<DamageLog>(child_child).unwrap();
    assert_eq!(child_log.0.len(), 2);
    assert_eq!(child_log.0[1], "child_manual_def_20");

    let parent_log = app.world().get::<DamageLog>(parent_child).unwrap();
    assert_eq!(parent_log.0.len(), 2, "Parent should receive ManualDefault (manual propagate called)");
    assert_eq!(parent_log.0[1], "parent_manual_def_20");

    // === Test Variant C: AutoDefault ===
    // Override: auto_propagate + default ChildOf
    app.world_mut().trigger(enum_level_propagate_event::AutoDefault {
        entity: child_child,
        value: 30,
    });
    app.update();

    let child_log = app.world().get::<DamageLog>(child_child).unwrap();
    assert_eq!(child_log.0.len(), 3);
    assert_eq!(child_log.0[2], "child_auto_def_30");

    let parent_log = app.world().get::<DamageLog>(parent_child).unwrap();
    assert_eq!(parent_log.0.len(), 3, "Parent should receive AutoDefault (auto_propagate)");
    assert_eq!(parent_log.0[2], "parent_auto_def_30");

    // === Test Variant D: ManualCustom ===
    // Override: manual propagate + MountOf (custom relationship)
    app.world_mut().trigger(enum_level_propagate_event::ManualCustom {
        entity: mount,
        value: 40,
    });
    app.update();

    let mount_log = app.world().get::<DamageLog>(mount).unwrap();
    assert_eq!(mount_log.0.len(), 1);
    assert_eq!(mount_log.0[0], "mount_manual_custom_40");

    let rider_log = app.world().get::<DamageLog>(rider).unwrap();
    assert_eq!(rider_log.0.len(), 1, "Rider should receive ManualCustom (manual propagate with MountOf)");
    assert_eq!(rider_log.0[0], "rider_manual_custom_40");

    // === Test Variant E: AutoCustom ===
    // Override: auto_propagate + MountOf (custom relationship)
    app.world_mut().trigger(enum_level_propagate_event::AutoCustom {
        entity: mount,
        value: 50,
    });
    app.update();

    let mount_log = app.world().get::<DamageLog>(mount).unwrap();
    assert_eq!(mount_log.0.len(), 2);
    assert_eq!(mount_log.0[1], "mount_auto_custom_50");

    let rider_log = app.world().get::<DamageLog>(rider).unwrap();
    assert_eq!(rider_log.0.len(), 2, "Rider should receive AutoCustom (auto_propagate with MountOf)");
    assert_eq!(rider_log.0[1], "rider_auto_custom_50");
}
