#![cfg(feature = "fsm")]

use bevy_enum_event::{EnumEvent, FSMTransition};
use bevy_fsm::FSMTransition as BevyFsmTransition;

#[allow(dead_code)]
#[derive(EnumEvent, FSMTransition, Clone, Copy, Debug, PartialEq)]
enum GenericTransition<T>
where
    T: Copy,
{
    Custom(T),
    Marker(::core::marker::PhantomData<T>),
}

#[test]
fn generic_fsm_transition_support() {
    let custom = generic_transition::Custom(42i32);
    let marker = generic_transition::Marker::<i32>(::core::marker::PhantomData);

    assert!(BevyFsmTransition::can_transition(
        GenericTransition::Custom(1),
        GenericTransition::Custom(2)
    ));

    // Prevent unused warnings
    let _ = (custom, marker);
}
