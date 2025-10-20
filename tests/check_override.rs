use bevy::prelude::*;
use bevy_enum_event::EnumEntityEvent;

// Test that variant-level completely overrides enum-level (not combined)
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

// Compile test - if this compiles, it means the attributes are correctly applied
fn _compile_test() {
    let e = Entity::from_bits(1);
    let _ = auto_propagate_override_event::InheritAuto { entity: e };
    let _ = auto_propagate_override_event::NoAutoCustomRel { entity: e };
    let _ = auto_propagate_override_event::WithAutoCustomRel { entity: e };
}
