use bevy::prelude::Entity;
use bevy_enum_event::{EnumEntityEvent, EnumEvent};

// Test unit variants
#[derive(EnumEvent, Clone, Copy, Debug, PartialEq)]
#[allow(dead_code)]
enum UnitEnum {
    A,
    B,
    C,
}

#[test]
fn test_unit_variants() {
    let a = unit_enum::A;
    let b = unit_enum::B;
    let c = unit_enum::C;

    // Verify they can be created and are distinct types
    assert_eq!(std::mem::size_of_val(&a), 0);
    assert_eq!(std::mem::size_of_val(&b), 0);
    assert_eq!(std::mem::size_of_val(&c), 0);
}

// Test tuple variants
#[derive(EnumEvent, Clone, Debug)]
#[allow(dead_code)]
enum TupleEnum {
    Single(u32),
    Multiple(String, i32),
    Empty,
}

#[test]
fn test_tuple_variants() {
    let single = tuple_enum::Single(42);
    assert_eq!(single.0, 42);

    let multiple = tuple_enum::Multiple("test".to_string(), 100);
    assert_eq!(multiple.0, "test");
    assert_eq!(multiple.1, 100);

    // Test that Empty variant can be created
    let empty = tuple_enum::Empty;
    assert_eq!(std::mem::size_of_val(&empty), 0);
}

// Test named field variants
#[derive(EnumEvent, Clone, Debug)]
#[allow(dead_code)]
enum NamedEnum {
    SingleField { value: u32 },
    MultipleFields { name: String, count: i32 },
    NoFields,
}

#[test]
fn test_named_field_variants() {
    let single = named_enum::SingleField { value: 42 };
    assert_eq!(single.value, 42);

    let multiple = named_enum::MultipleFields {
        name: "test".to_string(),
        count: 100,
    };
    assert_eq!(multiple.name, "test");
    assert_eq!(multiple.count, 100);

    // Test that NoFields variant can be created
    let no_fields = named_enum::NoFields;
    assert_eq!(std::mem::size_of_val(&no_fields), 0);
}

// Test mixed variants
#[derive(EnumEvent, Clone, Debug)]
#[allow(dead_code)]
enum MixedEnum {
    Unit,
    Tuple(String),
    Named { value: i32 },
}

#[test]
fn test_mixed_variants() {
    let unit = mixed_enum::Unit;
    assert_eq!(std::mem::size_of_val(&unit), 0);

    let tuple = mixed_enum::Tuple("hello".to_string());
    assert_eq!(tuple.0, "hello");

    let named = mixed_enum::Named { value: 42 };
    assert_eq!(named.value, 42);
}

// Test deref for single-field tuple variant
#[cfg(feature = "deref")]
#[test]
fn test_deref_tuple_variant() {
    #[derive(EnumEvent, Clone)]
    #[allow(dead_code)]
    enum DerefTuple {
        Value(String),
    }

    let mut val = deref_tuple::Value("test".to_string());

    // Test Deref
    let s: &String = &val;
    assert_eq!(s, "test");

    // Test DerefMut
    let s_mut: &mut String = &mut val;
    s_mut.push_str("_modified");
    assert_eq!(val.0, "test_modified");
}

// Test deref for single-field named variant
#[cfg(feature = "deref")]
#[test]
fn test_deref_named_variant() {
    #[derive(EnumEvent, Clone)]
    #[allow(dead_code)]
    enum DerefNamed {
        Value { data: String },
    }

    let mut val = deref_named::Value {
        data: "test".to_string(),
    };

    // Test Deref
    let s: &String = &val;
    assert_eq!(s, "test");

    // Test DerefMut
    let s_mut: &mut String = &mut val;
    s_mut.push_str("_modified");
    assert_eq!(val.data, "test_modified");
}

// Test that multi-field variants don't have deref
#[test]
fn test_multi_field_variants() {
    #[derive(EnumEvent, Clone)]
    #[allow(dead_code)]
    enum MultiField {
        Multiple(String, i32),
        MultipleNamed { a: String, b: i32 },
    }

    let m1 = multi_field::Multiple("test".to_string(), 42);
    assert_eq!(m1.0, "test");
    assert_eq!(m1.1, 42);

    let m2 = multi_field::MultipleNamed {
        a: "test".to_string(),
        b: 42,
    };
    assert_eq!(m2.a, "test");
    assert_eq!(m2.b, 42);
}

// Test deref support for multi-field variants when #[enum_event(deref)] is provided
#[cfg(feature = "deref")]
#[test]
fn test_multi_field_deref_with_attribute() {
    #[derive(EnumEvent, Clone)]
    #[allow(dead_code)]
    enum MultiFieldDeref {
        Tuple(#[enum_event(deref)] String, i32),
        Named {
            #[enum_event(deref)]
            value: String,
            other: i32,
        },
    }

    let mut tuple = multi_field_deref::Tuple("tuple".to_string(), 7);
    let tuple_ref: &String = &tuple;
    assert_eq!(tuple_ref, "tuple");

    let tuple_ref_mut: &mut String = &mut tuple;
    tuple_ref_mut.push_str("_updated");
    assert_eq!(tuple.0, "tuple_updated");

    let mut named = multi_field_deref::Named {
        value: "named".to_string(),
        other: 9,
    };
    let named_ref: &String = &named;
    assert_eq!(named_ref, "named");

    let named_ref_mut: &mut String = &mut named;
    named_ref_mut.push_str("_updated");
    assert_eq!(named.value, "named_updated");
    assert_eq!(named.other, 9);
}

// Test Clone trait
#[test]
fn test_clone() {
    #[derive(EnumEvent, Clone)]
    #[allow(dead_code)]
    enum CloneEnum {
        Value(String),
    }

    let original = clone_enum::Value("hello".to_string());
    let cloned = original.clone();
    assert_eq!(cloned.0, "hello");
}

// Test Debug trait
#[test]
fn test_debug() {
    #[derive(EnumEvent, Clone, Debug)]
    #[allow(dead_code)]
    enum DebugEnum {
        Value(String),
    }

    let val = debug_enum::Value("test".to_string());
    let debug_str = format!("{val:?}");
    assert!(debug_str.contains("Value"));
}

#[test]
fn test_generic_enum_support() {
    #[derive(EnumEvent, Clone, Debug)]
    #[allow(dead_code)]
    enum GenericEnum<T>
    where
        T: Clone + std::fmt::Debug,
    {
        Owned(T),
        Pair(T, u32),
        Unit,
    }

    #[derive(EnumEvent, Clone, Copy, Debug)]
    #[allow(dead_code)]
    enum BorrowedEnum<'event> {
        Reference(&'event i32),
        Unit,
    }

    let value = String::from("hello");
    let owned = generic_enum::Owned(value.clone());
    assert_eq!(owned.0, value);

    let pair = generic_enum::Pair(value.clone(), 7);
    assert_eq!(pair.0, value);
    assert_eq!(pair.1, 7);

    let _unit = generic_enum::Unit::<String>::default();

    let data = 42;
    let reference = borrowed_enum::Reference(&data);
    #[cfg(feature = "deref")]
    assert_eq!(**reference, 42);
    #[cfg(not(feature = "deref"))]
    assert_eq!(*reference.0, 42);

    let _borrowed_unit = borrowed_enum::Unit::default();
}

// ============================================================================
// EntityEvent Tests
// ============================================================================

// Test basic EntityEvent with entity field
#[test]
fn test_entity_event_basic() {
    #[derive(EnumEntityEvent, Clone, Copy)]
    #[allow(dead_code)]
    enum EntityAction {
        Spawned { entity: Entity },
        Destroyed { entity: Entity },
    }

    let entity = Entity::from_bits(42);
    let spawned = entity_action::Spawned { entity };
    assert_eq!(spawned.entity, entity);

    let destroyed = entity_action::Destroyed { entity };
    assert_eq!(destroyed.entity, entity);
}

// Test EntityEvent with additional fields
#[test]
fn test_entity_event_with_data() {
    #[derive(EnumEntityEvent, Clone)]
    #[allow(dead_code)]
    enum CombatEvent {
        Damaged { entity: Entity, amount: f32 },
        Healed { entity: Entity, amount: f32 },
    }

    let entity = Entity::from_bits(7);
    let damaged = combat_event::Damaged {
        entity,
        amount: 10.5,
    };
    assert_eq!(damaged.entity, entity);
    assert!((damaged.amount - 10.5).abs() < f32::EPSILON);

    let healed = combat_event::Healed {
        entity,
        amount: 5.0,
    };
    assert_eq!(healed.entity, entity);
    assert!((healed.amount - 5.0).abs() < f32::EPSILON);
}

// Test EntityEvent with custom target field
#[test]
fn test_entity_event_custom_target() {
    #[derive(EnumEntityEvent, Clone, Copy)]
    #[allow(dead_code)]
    enum AttackEvent {
        Hit {
            #[enum_event(target)]
            attacker: Entity,
            defender: Entity,
        },
    }

    let attacker = Entity::from_bits(1);
    let defender = Entity::from_bits(2);
    let hit = attack_event::Hit { attacker, defender };
    assert_eq!(hit.attacker, attacker);
    assert_eq!(hit.defender, defender);
}

// Test EntityEvent with propagate
#[test]
fn test_entity_event_propagate() {
    #[derive(EnumEntityEvent, Clone, Copy)]
    #[enum_event(propagate)]
    #[allow(dead_code)]
    enum UiEvent {
        Click { entity: Entity },
        Hover { entity: Entity },
    }

    let entity = Entity::from_bits(10);
    let click = ui_event::Click { entity };
    assert_eq!(click.entity, entity);

    let hover = ui_event::Hover { entity };
    assert_eq!(hover.entity, entity);
}

// Test EntityEvent with custom propagate relationship
// Note: This test just verifies the macro accepts the syntax and generates valid code
// The actual propagate relationship would be used at runtime by Bevy's observer system

pub type CustomRelationship = ::bevy::prelude::ChildOf;

#[derive(EnumEntityEvent, Clone, Copy)]
#[enum_event(propagate = &'static crate::CustomRelationship)]
#[allow(dead_code)]
enum HierarchyEvent {
    NodeAdded { entity: Entity },
    NodeRemoved { entity: Entity },
}

#[test]
fn test_entity_event_custom_propagate() {
    let entity = Entity::from_bits(20);
    let added = hierarchy_event::NodeAdded { entity };
    assert_eq!(added.entity, entity);

    let removed = hierarchy_event::NodeRemoved { entity };
    assert_eq!(removed.entity, entity);
}

// Test EntityEvent with deref on entity field
#[cfg(feature = "deref")]
#[test]
fn test_entity_event_single_field_deref() {
    #[derive(EnumEntityEvent, Clone, Copy)]
    #[allow(dead_code)]
    enum SingleFieldEntity {
        Spawned { entity: Entity },
    }

    let entity = Entity::from_bits(99);
    let spawned = single_field_entity::Spawned { entity };

    // With single field, deref should work
    let dereffed: &Entity = &spawned;
    assert_eq!(*dereffed, entity);
}

// Test EntityEvent with deref on custom field
#[cfg(feature = "deref")]
#[test]
fn test_entity_event_multi_field_deref() {
    #[derive(EnumEntityEvent, Clone, Copy)]
    #[allow(dead_code)]
    enum MultiFieldEntity {
        Scored {
            #[enum_event(deref)]
            entity: Entity,
            points: u32,
        },
    }

    let entity = Entity::from_bits(5);
    let scored = multi_field_entity::Scored {
        entity,
        points: 100,
    };

    // Deref should give us the entity field
    let dereffed: &Entity = &scored;
    assert_eq!(*dereffed, entity);
    assert_eq!(scored.points, 100);
}

// ============================================================================
// Forwarded derives (Copy, PartialEq, Eq, Hash, PartialOrd, Ord)
// ============================================================================

// Regression test for https://github.com/MolecularSadism/bevy_enum_event/issues/1
// A proc-macro derive cannot see the enum's sibling `#[derive(...)]`, so
// forwarding is opt-in via `#[enum_event(derive(...))]`. The listed derives
// must reach every generated variant struct, not just unit variants.
#[derive(EnumEvent, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[enum_event(derive(Copy, PartialEq, Eq, Hash, PartialOrd, Ord))]
#[allow(dead_code)]
enum ForwardedDerives {
    Unit,
    Tuple(u32),
    Struct { value: i32 },
}

// Helper functions that only compile if the bound is actually implemented.
fn assert_copy<T: Copy>(_: &T) {}
fn assert_eq_trait<T: Eq>(_: &T) {}
fn assert_hash<T: std::hash::Hash>(_: &T) {}
fn assert_ord<T: Ord>(_: &T) {}

#[test]
fn test_forwarded_derives_on_all_variant_kinds() {
    let unit = forwarded_derives::Unit;
    let tuple = forwarded_derives::Tuple(7);
    let strukt = forwarded_derives::Struct { value: -3 };

    // Copy: a copy leaves the original usable.
    let tuple_copy = tuple;
    assert_eq!(tuple, tuple_copy);
    assert_eq!(tuple.0, 7);

    let struct_copy = strukt;
    assert_eq!(strukt, struct_copy);
    assert_eq!(strukt.value, -3);

    // PartialEq / Eq
    assert_ne!(forwarded_derives::Tuple(1), forwarded_derives::Tuple(2));

    // Verify the trait bounds are genuinely satisfied for every variant kind.
    assert_copy(&unit);
    assert_copy(&tuple);
    assert_copy(&strukt);
    assert_eq_trait(&unit);
    assert_eq_trait(&tuple);
    assert_eq_trait(&strukt);
    assert_hash(&unit);
    assert_hash(&tuple);
    assert_hash(&strukt);
    assert_ord(&unit);
    assert_ord(&tuple);
    assert_ord(&strukt);
}

// Forwarded `Copy` must coexist with the auto-generated `Deref`/`DerefMut` on
// single-field variants.
#[derive(EnumEvent, Clone, Copy)]
#[enum_event(derive(Copy))]
#[allow(dead_code)]
enum SmallEvents {
    Health(u32),
    Position { xy: (f32, f32) },
}

#[test]
fn test_forwarded_copy_with_deref() {
    let health = small_events::Health(100);
    let health_copy = health;
    // Original still usable after the copy.
    assert_eq!(health.0, 100);
    assert_eq!(health_copy.0, 100);
    assert_copy(&health);

    let pos = small_events::Position { xy: (1.0, 2.0) };
    let pos_copy = pos;
    assert_eq!(pos.xy, (1.0, 2.0));
    assert_eq!(pos_copy.xy, (1.0, 2.0));
    assert_copy(&pos);
}

// Per-variant `#[enum_event(derive(...))]` is additive to the enum-level list.
#[derive(EnumEvent, Clone)]
#[enum_event(derive(PartialEq))]
#[allow(dead_code)]
enum MixedCopyability {
    // Not Copy: holds a String.
    Named(String),
    // Copy: opt in for just this variant, on top of the enum-level PartialEq.
    #[enum_event(derive(Copy))]
    Id {
        value: u64,
    },
}

#[test]
fn test_variant_level_forwarded_derives() {
    // Enum-level PartialEq reaches the non-Copy variant.
    assert_eq!(
        mixed_copyability::Named("a".to_string()),
        mixed_copyability::Named("a".to_string())
    );

    let id = mixed_copyability::Id { value: 42 };
    let id_copy = id;
    assert_eq!(id.value, 42);
    assert_eq!(id_copy.value, 42);
    assert_copy(&id);
    // Enum-level PartialEq also applies to the per-variant-augmented struct.
    assert_eq!(id, id_copy);
}
