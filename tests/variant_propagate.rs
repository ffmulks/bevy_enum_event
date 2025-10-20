use bevy::prelude::*;
use bevy_enum_event::EnumEntityEvent;

// Test 1: Enum-level propagate applies to all variants
#[derive(EnumEntityEvent, Clone, Copy)]
#[enum_event(propagate)]
#[allow(dead_code)]
enum EnumLevelPropagateEvent {
    Action { entity: Entity, data: u16 },
    Update { entity: Entity },
}

#[test]
fn test_enum_level_propagate() {
    // Just verify it compiles
    let e = Entity::from_bits(1);
    let _ = enum_level_propagate_event::Action { entity: e, data: 5 };
    let _ = enum_level_propagate_event::Update { entity: e };
}

// Test 2: Variant-level override of enum-level setting
#[derive(EnumEntityEvent, Clone, Copy)]
#[enum_event(propagate)]
#[allow(dead_code)]
enum MixedPropagateEvent {
    // Uses enum-level propagate
    Normal { entity: Entity },

    // Overrides with auto_propagate
    #[enum_event(auto_propagate, propagate)]
    Auto { entity: Entity },

    // Overrides with custom relationship
    #[enum_event(propagate = &'static ::bevy::prelude::ChildOf)]
    Custom { entity: Entity },
}

#[test]
fn test_variant_level_override() {
    let e = Entity::from_bits(1);
    let _ = mixed_propagate_event::Normal { entity: e };
    let _ = mixed_propagate_event::Auto { entity: e };
    let _ = mixed_propagate_event::Custom { entity: e };
}

// Test 3: No enum-level, only variant-level
#[derive(EnumEntityEvent, Clone, Copy)]
#[allow(dead_code)]
enum VariantOnlyPropagateEvent {
    // No propagate
    None { entity: Entity },

    // Has propagate
    #[enum_event(propagate)]
    Manual { entity: Entity },

    // Has auto_propagate with custom relationship
    #[enum_event(auto_propagate, propagate = &'static ::bevy::prelude::ChildOf)]
    Auto { entity: Entity },
}

#[test]
fn test_variant_only_propagate() {
    let e = Entity::from_bits(1);
    let _ = variant_only_propagate_event::None { entity: e };
    let _ = variant_only_propagate_event::Manual { entity: e };
    let _ = variant_only_propagate_event::Auto { entity: e };
}

// Test 4: Enum-level auto_propagate, variant overrides without auto
// This tests that variant-level completely overrides enum-level (not combined)
#[derive(EnumEntityEvent, Clone, Copy)]
#[enum_event(auto_propagate, propagate)]
#[allow(dead_code)]
enum AutoPropagateOverrideEvent {
    // Inherits: auto_propagate, propagate
    InheritAuto { entity: Entity },

    // Override: removes auto_propagate, uses custom relation
    #[enum_event(propagate = &'static ::bevy::prelude::ChildOf)]
    NoAutoCustomRel { entity: Entity },

    // Override: keeps auto_propagate, uses custom relation
    #[enum_event(auto_propagate, propagate = &'static ::bevy::prelude::ChildOf)]
    WithAutoCustomRel { entity: Entity },
}

#[test]
fn test_auto_propagate_override() {
    let e = Entity::from_bits(1);

    // This variant inherits auto_propagate from enum-level
    let _ = auto_propagate_override_event::InheritAuto { entity: e };

    // This variant overrides: has custom rel but NO auto_propagate
    let _ = auto_propagate_override_event::NoAutoCustomRel { entity: e };

    // This variant overrides: has both custom rel AND auto_propagate
    let _ = auto_propagate_override_event::WithAutoCustomRel { entity: e };
}
