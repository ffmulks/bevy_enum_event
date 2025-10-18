use bevy_enum_events::EnumEvents;

// Test unit variants
#[derive(EnumEvents, Clone, Copy, Debug, PartialEq)]
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
#[derive(EnumEvents, Clone, Debug)]
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
#[derive(EnumEvents, Clone, Debug)]
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
#[derive(EnumEvents, Clone, Debug)]
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
    #[derive(EnumEvents, Clone)]
    enum DerefTuple {
        Value(String),
    }

    let mut val = deref_tuple::Value("test".to_string());

    // Test Deref
    let s: &String = &*val;
    assert_eq!(s, "test");

    // Test DerefMut
    let s_mut: &mut String = &mut *val;
    s_mut.push_str("_modified");
    assert_eq!(val.0, "test_modified");
}

// Test deref for single-field named variant
#[cfg(feature = "deref")]
#[test]
fn test_deref_named_variant() {
    #[derive(EnumEvents, Clone)]
    enum DerefNamed {
        Value { data: String },
    }

    let mut val = deref_named::Value {
        data: "test".to_string(),
    };

    // Test Deref
    let s: &String = &*val;
    assert_eq!(s, "test");

    // Test DerefMut
    let s_mut: &mut String = &mut *val;
    s_mut.push_str("_modified");
    assert_eq!(val.data, "test_modified");
}

// Test that multi-field variants don't have deref
#[test]
fn test_multi_field_variants() {
    #[derive(EnumEvents, Clone)]
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

// Test Clone trait
#[test]
fn test_clone() {
    #[derive(EnumEvents, Clone)]
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
    #[derive(EnumEvents, Clone, Debug)]
    enum DebugEnum {
        Value(String),
    }

    let val = debug_enum::Value("test".to_string());
    let debug_str = format!("{val:?}");
    assert!(debug_str.contains("Value"));
}
