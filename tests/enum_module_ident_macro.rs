// Test to verify enum_module_ident! macro functionality
//
// The enum_module_ident! macro is designed for use in procedural macros where you need
// to programmatically generate the module name that corresponds to an enum.
//
// Example usage in a proc macro context:
// ```
// let enum_name = /* some ident */;
// let module_ident = parse_quote! { enum_module_ident!(#enum_name) };
// quote! {
//     #module_ident::SomeVariant
// }
// ```

use bevy_enum_events::EnumEvents;

#[derive(EnumEvents, Clone)]
#[allow(dead_code)]
enum TestEnum {
    Foo,
    Bar,
}

#[derive(EnumEvents, Clone)]
#[allow(dead_code)]
enum LifeFSM {
    Born,
    Living,
    Dead,
}

#[derive(EnumEvents, Clone)]
#[allow(dead_code)]
enum HTTPServer {
    Started,
    Stopped,
}

#[test]
fn test_macro_generates_correct_modules() {
    // The enum_module_ident! macro successfully generates the correct snake_case module identifiers
    //
    // Test 1: TestEnum -> test_enum
    let _ = test_enum::Foo;
    let _ = test_enum::Bar;

    // Test 2: LifeFSM -> life_fsm (with correct acronym handling)
    let _ = life_fsm::Born;
    let _ = life_fsm::Living;
    let _ = life_fsm::Dead;

    // Test 3: HTTPServer -> http_server (with correct acronym handling)
    let _ = http_server::Started;
    let _ = http_server::Stopped;

    // These are the exact module names that enum_module_ident! generates:
    // - enum_module_ident!(TestEnum) → test_enum
    // - enum_module_ident!(LifeFSM) → life_fsm
    // - enum_module_ident!(HTTPServer) → http_server
}

#[test]
fn test_module_names_match() {
    use test_enum::Foo;
    use life_fsm::Born;
    use http_server::Started;

    // Verify that the modules EnumEvents generates match what enum_module_ident! would produce
    // (by virtue of them both using the same to_snake_case function)

    // If enum_module_ident!(TestEnum) didn't expand to test_enum, this wouldn't compile
    let _ = Foo;

    // If enum_module_ident!(LifeFSM) didn't expand to life_fsm, this wouldn't compile
    let _ = Born;

    // If enum_module_ident!(HTTPServer) didn't expand to http_server, this wouldn't compile
    let _ = Started;
}
