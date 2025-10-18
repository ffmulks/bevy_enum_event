//! General-purpose enum to Bevy event conversion macro.
//!
//! This crate provides a derive macro that generates Bevy event types from enum variants.
//! For each variant, it creates a corresponding event struct in a `snake_case` module.
//!
//! # Example (Unit Variants)
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_enum_events::EnumEvents;
//!
//! #[derive(EnumEvents, Clone, Copy, Debug, PartialEq, Eq, Hash)]
//! enum PlayerState {
//!     Idle,
//!     Running,
//!     Jumping,
//! }
//! ```
//!
//! This generates:
//!
//! ```rust,ignore
//! pub mod player_state {
//!     use bevy::prelude::Event;
//!
//!     #[derive(Event, Clone, Copy, Debug)]
//!     pub struct Idle;
//!
//!     #[derive(Event, Clone, Copy, Debug)]
//!     pub struct Running;
//!
//!     #[derive(Event, Clone, Copy, Debug)]
//!     pub struct Jumping;
//! }
//! ```
//!
//! # Example (Variants with Data)
//!
//! ```
//! use bevy_enum_events::EnumEvents;
//!
//! #[derive(EnumEvents, Clone)]
//! enum GameEvent {
//!     Victory(String),
//!     ScoreChanged { team: u32, score: i32 },
//!     GameOver,
//! }
//! ```
//!
//! This generates:
//!
//! ```rust,ignore
//! pub mod game_event {
//!     use bevy::prelude::Event;
//!
//!     #[derive(Event, Clone, Debug)]
//!     pub struct Victory(pub String);
//!
//!     #[derive(Event, Clone, Debug)]
//!     pub struct ScoreChanged {
//!         pub team: u32,
//!         pub score: i32,
//!     }
//!
//!     #[derive(Event, Clone, Debug)]
//!     pub struct GameOver;
//! }
//! ```
//!
//! # Feature: `deref` (enabled by default)
//!
//! When the `deref` feature is enabled (which it is by default), enum variants with a single
//! field or tuple value will automatically derive `Deref` and `DerefMut`, allowing direct access
//! to the inner value:
//!
#![cfg_attr(feature = "deref", doc = r#"
```
use bevy_enum_events::EnumEvents;
use std::ops::Deref;

#[derive(EnumEvents, Clone)]
enum NetworkEvent {
    MessageReceived(String),
    Disconnected,
}

// Test that deref works
let msg = network_event::MessageReceived("Hello".to_string());
let content: &String = msg.deref();
assert_eq!(content, "Hello");
```
"#)]
//!
//!
//! To disable this feature, add the following to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! bevy_enum_events = { version = "0.1", default-features = false }
//! ```
//!
//! # Usage with Observers
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_enum_events::EnumEvents;
//!
//! #[derive(EnumEvents, Clone, Copy)]
//! enum GameState {
//!     MainMenu,
//!     Playing,
//!     Paused,
//! }
//!
//! fn on_paused(trigger: Trigger<game_state::Paused>) {
//!     println!("Game paused!");
//! }
//! ```

//! # Helper Macro: `enum_module_ident!`
//!
//! For advanced use cases (like building wrapper crates), the `enum_module_ident!` macro
//! provides access to the module name that `EnumEvents` would generate.
//!
//! ```ignore
//! use bevy_enum_events::enum_module_ident;
//!
//! // This expands to the identifier: life_fsm
//! let module_name = stringify!(enum_module_ident!(LifeFSM));
//! assert_eq!(module_name, "life_fsm");
//! ```
//!
//! This is particularly useful for libraries like `bevy_fsm` that need to programmatically
//! reference the generated module names.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// Converts `PascalCase` or `camelCase` to `snake_case`.
///
/// Handles acronyms gracefully: `FSMState` → `fsm_state`, `HTTPServer` → `http_server`
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();

    for (i, &ch) in chars.iter().enumerate() {
        if ch.is_uppercase() {
            let is_first = i == 0;
            let prev_is_lower = i > 0 && chars[i - 1].is_lowercase();
            let next_is_lower = i + 1 < chars.len() && chars[i + 1].is_lowercase();

            // Add underscore if:
            // 1. Previous char is lowercase (camelCase -> snake_case)
            // 2. This is uppercase, next is lowercase, and we're not first (handles acronyms)
            if !is_first && (prev_is_lower || next_is_lower) {
                result.push('_');
            }

            result.push(ch.to_lowercase().next().unwrap());
        } else {
            result.push(ch);
        }
    }
    result
}

/// Derive macro that generates Bevy event types from enum variants.
///
/// # Requirements
///
/// - Can only be derived for enums
/// - Supports unit variants, tuple variants, and named field variants
///
/// # Panics
///
/// Panics if applied to a non-enum type (struct, union, etc.)
///
/// # Generated Code
///
/// For an enum named `MyEnum` with various variant types, this macro generates:
///
/// ```rust,ignore
/// pub mod my_enum {
///     use bevy::prelude::Event;
///
///     #[derive(Event, Clone, Copy, Debug)]
///     pub struct VariantA;
///
///     #[derive(Event, Clone, Debug)]
///     pub struct VariantB(pub String);
///
///     #[derive(Event, Clone, Debug)]
///     pub struct VariantC {
///         pub field1: i32,
///         pub field2: String,
///     }
/// }
/// ```
///
/// # Example
///
/// ```rust,no_run
/// use bevy_enum_events::EnumEvents;
///
/// #[derive(EnumEvents)]
/// enum Action {
///     Jump,
///     Run(f32),  // speed
///     Attack { damage: i32, critical: bool },
/// }
///
/// // Generated module:
/// // pub mod action {
/// //     pub struct Jump;
/// //     pub struct Run(pub f32);
/// //     pub struct Attack { pub damage: i32, pub critical: bool }
/// // }
/// ```
///
/// # Deref Feature
///
/// When the `deref` feature is enabled (default), single-field variants automatically
/// implement `Deref` and `DerefMut` for convenient access to the inner value.
/// Procedural macro that converts a type identifier to its `snake_case` module identifier.
///
/// This generates the same module name that `EnumEvents` would create, allowing
/// programmatic access to generated module names in consuming crates.
///
/// # Example
///
/// ```ignore
/// use bevy_enum_events::enum_module_ident;
///
/// // Expands to the identifier: life_fsm
/// enum_module_ident!(LifeFSM);
///
/// // Can be used with stringify! to get the string representation
/// let module_name = stringify!(enum_module_ident!(PlayerState));
/// assert_eq!(module_name, "player_state");
/// ```
///
/// # Use Cases
///
/// This macro is primarily useful for library authors building on top of `bevy_enum_events`,
/// such as:
/// - The `bevy_fsm` crate, which needs to reference generated module names
/// - Code generation tools that work with `EnumEvents`
/// - Macros that compose with `EnumEvents`
///
/// Most users won't need this macro directly, as they can reference the generated modules
/// by their `snake_case` names directly (e.g., `player_state::Idle`).
#[proc_macro]
pub fn enum_module_ident(input: TokenStream) -> TokenStream {
    let ident = parse_macro_input!(input as syn::Ident);
    let module_name_str = to_snake_case(&ident.to_string());
    let module_ident = syn::Ident::new(&module_name_str, ident.span());

    TokenStream::from(quote! { #module_ident })
}

/// # Panics
///
/// Panics if applied to a non-enum type (struct, union, etc.).
#[proc_macro_derive(EnumEvents)]
pub fn derive_enum_events(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = &input.ident;

    // Extract variants from enum
    let variants = match &input.data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => panic!("EnumEvents can only be derived for enums"),
    };

    // Convert EnumName to snake_case for module name
    let module_name_str = to_snake_case(&enum_name.to_string());
    let module_name = syn::Ident::new(&module_name_str, enum_name.span());

    // Generate struct definitions for each variant
    let struct_defs: Vec<_> = variants
        .iter()
        .map(|variant| {
            let variant_ident = &variant.ident;

            match &variant.fields {
                Fields::Unit => {
                    // Unit variant: generate a unit struct
                    quote! {
                        /// Event type corresponding to the enum variant.
                        #[derive(Event, Clone, Copy, Debug)]
                        pub struct #variant_ident;
                    }
                }
                Fields::Unnamed(fields) => {
                    // Tuple variant: generate a tuple struct
                    let field_types: Vec<_> = fields.unnamed.iter().map(|f| &f.ty).collect();
                    let is_single_field = field_types.len() == 1;

                    if is_single_field && cfg!(feature = "deref") {
                        // Single field with deref feature: add Deref and DerefMut
                        let field_type = &field_types[0];
                        quote! {
                            /// Event type corresponding to the enum variant.
                            #[derive(Event, Clone, Debug)]
                            pub struct #variant_ident(pub #field_type);

                            #[cfg(feature = "deref")]
                            impl ::core::ops::Deref for #variant_ident {
                                type Target = #field_type;

                                fn deref(&self) -> &Self::Target {
                                    &self.0
                                }
                            }

                            #[cfg(feature = "deref")]
                            impl ::core::ops::DerefMut for #variant_ident {
                                fn deref_mut(&mut self) -> &mut Self::Target {
                                    &mut self.0
                                }
                            }
                        }
                    } else {
                        // Multiple fields or deref disabled: just the struct
                        quote! {
                            /// Event type corresponding to the enum variant.
                            #[derive(Event, Clone, Debug)]
                            pub struct #variant_ident(#(pub #field_types),*);
                        }
                    }
                }
                Fields::Named(fields) => {
                    // Named fields variant: generate a struct with named fields
                    let field_defs: Vec<_> = fields.named.iter().collect();
                    let is_single_field = field_defs.len() == 1;

                    if is_single_field && cfg!(feature = "deref") {
                        // Single field with deref feature: add Deref and DerefMut
                        let field = &field_defs[0];
                        let field_name = field.ident.as_ref().unwrap();
                        let field_type = &field.ty;

                        quote! {
                            /// Event type corresponding to the enum variant.
                            #[derive(Event, Clone, Debug)]
                            pub struct #variant_ident {
                                pub #field_name: #field_type,
                            }

                            #[cfg(feature = "deref")]
                            impl ::core::ops::Deref for #variant_ident {
                                type Target = #field_type;

                                fn deref(&self) -> &Self::Target {
                                    &self.#field_name
                                }
                            }

                            #[cfg(feature = "deref")]
                            impl ::core::ops::DerefMut for #variant_ident {
                                fn deref_mut(&mut self) -> &mut Self::Target {
                                    &mut self.#field_name
                                }
                            }
                        }
                    } else {
                        // Multiple fields or deref disabled: just the struct
                        quote! {
                            /// Event type corresponding to the enum variant.
                            #[derive(Event, Clone, Debug)]
                            pub struct #variant_ident {
                                #(pub #field_defs),*
                            }
                        }
                    }
                }
            }
        })
        .collect();

    let expanded = quote! {
        /// Generated module containing event types for each enum variant.
        pub mod #module_name {
            use bevy::prelude::Event;

            #(#struct_defs)*
        }
    };

    TokenStream::from(expanded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake_case_conversion() {
        assert_eq!(to_snake_case("LifeFSM"), "life_fsm");
        assert_eq!(to_snake_case("PlayerState"), "player_state");
        assert_eq!(to_snake_case("HTTPServer"), "http_server");
        assert_eq!(to_snake_case("FSM"), "fsm");
        assert_eq!(to_snake_case("MyHTTPSConnection"), "my_https_connection");
    }
}
