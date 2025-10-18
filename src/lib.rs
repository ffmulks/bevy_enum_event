//! General-purpose enum to Bevy event conversion macro.
//!
//! This crate provides a derive macro that generates Bevy event types from enum variants.
//! For each variant, it creates a corresponding event struct in a snake_case module.
//!
//! # Example
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

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// Converts PascalCase or camelCase to snake_case.
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
/// - All variants must be unit variants (no fields)
///
/// # Generated Code
///
/// For an enum named `MyEnum` with variants `VariantA`, `VariantB`, this macro generates:
///
/// ```rust,ignore
/// pub mod my_enum {
///     use bevy::prelude::Event;
///
///     #[derive(Event, Clone, Copy, Debug)]
///     pub struct VariantA;
///
///     #[derive(Event, Clone, Copy, Debug)]
///     pub struct VariantB;
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
///     Run,
///     Attack,
/// }
///
/// // Generated module:
/// // pub mod action {
/// //     pub struct Jump;
/// //     pub struct Run;
/// //     pub struct Attack;
/// // }
/// ```
#[proc_macro_derive(EnumEvents)]
pub fn derive_enum_events(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = &input.ident;

    // Extract variants from enum
    let variants = match &input.data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => panic!("EnumEvents can only be derived for enums"),
    };

    // Verify all variants are unit variants
    for variant in variants {
        if !matches!(variant.fields, Fields::Unit) {
            panic!("EnumEvents enum variants must be unit variants (no fields)");
        }
    }

    let variant_idents: Vec<_> = variants.iter().map(|v| &v.ident).collect();

    // Generate the module structure with variant event types
    // Convert EnumName to snake_case for module name
    let module_name_str = to_snake_case(&enum_name.to_string());
    let module_name = syn::Ident::new(&module_name_str, enum_name.span());

    let expanded = quote! {
        /// Generated module containing event types for each enum variant.
        pub mod #module_name {
            use bevy::prelude::Event;

            #(
                /// Event type corresponding to the enum variant.
                #[derive(Event, Clone, Copy, Debug)]
                pub struct #variant_idents;
            )*
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
