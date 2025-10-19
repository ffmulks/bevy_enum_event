//! General-purpose enum to Bevy event conversion macro.
//!
//! This crate provides a derive macro that generates Bevy event types from enum variants.
//! For each variant, it creates a corresponding event struct in a `snake_case` module.
//!
//! # Example (Unit Variants)
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_enum_event::EnumEvent;
//!
//! #[derive(EnumEvent, Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
//! use bevy_enum_event::EnumEvent;
//!
//! #[derive(EnumEvent, Clone)]
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
//! field automatically derive `Deref` and `DerefMut`. Multi-field variants can opt into the same
//! behavior by marking one field with `#[enum_event(deref)]`, allowing direct access to that inner value:
//!
#![cfg_attr(
    feature = "deref",
    doc = r#"
```
use bevy_enum_event::EnumEvent;
use std::ops::Deref;

#[derive(EnumEvent, Clone)]
enum NetworkEvent {
    MessageReceived(String),
    Disconnected,
    PlayerScored { #[enum_event(deref)] player: u32, points: u32 },
}

// Test that deref works
let msg = network_event::MessageReceived("Hello".to_string());
let content: &String = msg.deref();
assert_eq!(content, "Hello");

let scored = network_event::PlayerScored { player: 7, points: 120 };
let player: &u32 = scored.deref();
assert_eq!(*player, 7);
```
"#
)]
//!
//!
//! To disable this feature, add the following to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! bevy_enum_event = { version = "0.1", default-features = false }
//! ```
//!
//! # Usage with Observers
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_enum_event::EnumEvent;
//!
//! #[derive(EnumEvent, Clone, Copy)]
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
//! provides access to the module name that `EnumEvent` would generate.
//!
//! ```ignore
//! use bevy_enum_event::enum_module_ident;
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
use std::collections::HashSet;
use syn::{parse_macro_input, visit::Visit, Attribute, Data, DeriveInput, Fields};

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

struct GenericsUsageCollector<'a> {
    type_names: &'a [String],
    lifetime_names: &'a [String],
    pub used_types: HashSet<String>,
    pub used_lifetimes: HashSet<String>,
}

impl<'a> GenericsUsageCollector<'a> {
    fn new(type_names: &'a [String], lifetime_names: &'a [String]) -> Self {
        Self {
            type_names,
            lifetime_names,
            used_types: HashSet::new(),
            used_lifetimes: HashSet::new(),
        }
    }
}

impl<'a, 'ast> Visit<'ast> for GenericsUsageCollector<'a> {
    fn visit_type_path(&mut self, type_path: &'ast syn::TypePath) {
        if type_path.qself.is_none() {
            if let Some(ident) = type_path.path.get_ident() {
                let ident_str = ident.to_string();
                if self.type_names.iter().any(|name| name == &ident_str) {
                    self.used_types.insert(ident_str);
                }
            }
        }
        syn::visit::visit_type_path(self, type_path);
    }

    fn visit_lifetime(&mut self, lifetime: &'ast syn::Lifetime) {
        let ident_str = lifetime.ident.to_string();
        if self.lifetime_names.iter().any(|name| name == &ident_str) {
            self.used_lifetimes.insert(ident_str);
        }
        syn::visit::visit_lifetime(self, lifetime);
    }
}

fn path_ends_with_ident(path: &syn::Path, ident: &str) -> bool {
    path.segments
        .last()
        .map(|segment| segment.ident == ident)
        .unwrap_or(false)
}

#[derive(Default)]
struct FieldAttrInfo {
    passthrough_attrs: Vec<Attribute>,
    has_deref: bool,
    has_deref_mut: bool,
}

fn analyze_field_attrs(attrs: &[Attribute]) -> FieldAttrInfo {
    let mut info = FieldAttrInfo::default();

    for attr in attrs {
        if path_ends_with_ident(attr.path(), "enum_event") {
            if let Err(err) = attr.parse_nested_meta(|meta| {
                if path_ends_with_ident(&meta.path, "deref") {
                    info.has_deref = true;
                } else if path_ends_with_ident(&meta.path, "deref_mut") {
                    info.has_deref_mut = true;
                    info.has_deref = true;
                }
                Ok(())
            }) {
                panic!("EnumEvent: failed to parse #[enum_event(...)] attribute: {err}");
            }
        } else if path_ends_with_ident(attr.path(), "deref") {
            info.has_deref = true;
        } else if path_ends_with_ident(attr.path(), "deref_mut") {
            info.has_deref_mut = true;
            info.has_deref = true;
        } else {
            info.passthrough_attrs.push(attr.clone());
        }
    }

    info
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
/// use bevy_enum_event::EnumEvent;
///
/// #[derive(EnumEvent)]
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
/// This generates the same module name that `EnumEvent` would create, allowing
/// programmatic access to generated module names in consuming crates.
///
/// # Example
///
/// ```ignore
/// use bevy_enum_event::enum_module_ident;
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
/// This macro is primarily useful for library authors building on top of `bevy_enum_event`,
/// such as:
/// - The `bevy_fsm` crate, which needs to reference generated module names
/// - Code generation tools that work with `EnumEvent`
/// - Macros that compose with `EnumEvent`
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
#[proc_macro_derive(EnumEvent, attributes(enum_event, deref, deref_mut))]
pub fn derive_enum_events(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = &input.ident;

    // Extract variants from enum
    let variants = match &input.data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => panic!("EnumEvent can only be derived for enums"),
    };

    // Convert EnumName to snake_case for module name
    let module_name_str = to_snake_case(&enum_name.to_string());
    let module_name = syn::Ident::new(&module_name_str, enum_name.span());

    let generics = input.generics.clone();
    let struct_generics = if generics.params.is_empty() {
        quote! {}
    } else {
        let params = generics.params.iter();
        quote! { <#(#params),*> }
    };
    let where_clause = generics.where_clause.as_ref();
    let type_params: Vec<(String, syn::Ident)> = generics
        .type_params()
        .map(|param| (param.ident.to_string(), param.ident.clone()))
        .collect();
    let lifetime_params: Vec<(String, syn::Lifetime)> = generics
        .lifetimes()
        .map(|param| {
            let lt = param.lifetime.clone();
            (lt.ident.to_string(), lt)
        })
        .collect();
    let type_param_names: Vec<String> = type_params.iter().map(|(name, _)| name.clone()).collect();
    let lifetime_param_names: Vec<String> = lifetime_params
        .iter()
        .map(|(name, _)| name.clone())
        .collect();

    // Generate struct definitions for each variant
    let mut struct_defs = Vec::new();
    let mut additional_impls = Vec::new();
    let mut uses_deref_derives = false;

    for variant in variants {
        let variant_ident = &variant.ident;
        let struct_generics_tokens = struct_generics.clone();

        let mut usage_collector =
            GenericsUsageCollector::new(&type_param_names, &lifetime_param_names);
        for field in variant.fields.iter() {
            usage_collector.visit_type(&field.ty);
        }
        let unused_type_params: Vec<_> = type_params
            .iter()
            .filter(|(name, _)| !usage_collector.used_types.contains(name))
            .map(|(_, ident)| ident.clone())
            .collect();
        let unused_lifetimes: Vec<_> = lifetime_params
            .iter()
            .filter(|(name, _)| !usage_collector.used_lifetimes.contains(name))
            .map(|(_, lifetime)| lifetime.clone())
            .collect();
        let phantom_entries: Vec<_> = unused_type_params
            .iter()
            .map(|ident| quote! { #ident })
            .chain(unused_lifetimes.iter().map(|lt| {
                quote! { &#lt () }
            }))
            .collect();
        let phantom_type = if phantom_entries.is_empty() {
            None
        } else {
            Some(quote! { ::core::marker::PhantomData<(#(#phantom_entries ,)*)> })
        };
        let mut extra_impl = None;

        let struct_def = match &variant.fields {
            Fields::Unit => {
                if let Some(phantom_type) = phantom_type.clone() {
                    let (impl_generics_impl, ty_generics_impl, where_clause_impl) =
                        generics.split_for_impl();
                    extra_impl = Some(quote! {
                        impl #impl_generics_impl #variant_ident #ty_generics_impl #where_clause_impl {
                            #[inline]
                            pub const fn new() -> Self {
                                Self {
                                    _phantom: ::core::marker::PhantomData,
                                }
                            }
                        }
                    });
                    quote! {
                        /// Event type corresponding to the enum variant.
                        #[allow(unused_lifetimes, unused_type_parameters)]
                        #[derive(Event, Clone, Copy, Debug, Default)]
                        pub struct #variant_ident #struct_generics_tokens #where_clause {
                            #[doc(hidden)]
                            pub(crate) _phantom: #phantom_type,
                        }
                    }
                } else {
                    quote! {
                        /// Event type corresponding to the enum variant.
                        #[allow(unused_lifetimes, unused_type_parameters)]
                        #[derive(Event, Clone, Copy, Debug, Default)]
                        pub struct #variant_ident #struct_generics_tokens #where_clause;
                    }
                }
            }
            Fields::Unnamed(fields) => {
                let struct_generics_tokens = struct_generics_tokens.clone();
                let field_infos: Vec<_> = fields
                    .unnamed
                    .iter()
                    .map(|field| {
                        let info = analyze_field_attrs(&field.attrs);
                        (info, &field.ty)
                    })
                    .collect();
                let field_count = field_infos.len();
                let deref_attr_fields = field_infos
                    .iter()
                    .filter(|(info, _)| info.has_deref)
                    .count();

                if field_count > 1 && deref_attr_fields > 1 {
                    panic!(
                        "EnumEvent: variant `{}` has multiple fields marked for deref (e.g., #[enum_event(deref)]); only one field can be dereferenced",
                        variant_ident
                    );
                }

                let should_derive_deref =
                    cfg!(feature = "deref") && (field_count == 1 || deref_attr_fields == 1);

                let mut field_tokens: Vec<_> = field_infos
                    .iter()
                    .map(|(info, ty)| {
                        let passthrough_attrs = info.passthrough_attrs.iter();
                        let mut marker_attrs = Vec::new();

                        if should_derive_deref {
                            if info.has_deref {
                                marker_attrs.push(quote!(#[deref]));
                            }
                            if info.has_deref_mut {
                                marker_attrs.push(quote!(#[deref_mut]));
                            }
                        }

                        quote! {
                            #(#passthrough_attrs)*
                            #(#marker_attrs)*
                            pub #ty
                        }
                    })
                    .collect();

                if let Some(phantom_type) = phantom_type.clone() {
                    field_tokens.push(quote! {
                        #[doc(hidden)]
                        pub(crate) #phantom_type
                    });

                    let (impl_generics_impl, ty_generics_impl, where_clause_impl) =
                        generics.split_for_impl();
                    let arg_idents: Vec<_> = (0..field_infos.len())
                        .map(|index| {
                            syn::Ident::new(&format!("__arg{index}"), variant_ident.span())
                        })
                        .collect();
                    let arg_defs: Vec<_> = field_infos
                        .iter()
                        .enumerate()
                        .map(|(idx, (_, ty))| {
                            let ident = &arg_idents[idx];
                            quote! { #ident: #ty }
                        })
                        .collect();
                    let arg_values = arg_idents.iter();

                    extra_impl = Some(quote! {
                        impl #impl_generics_impl #variant_ident #ty_generics_impl #where_clause_impl {
                            #[inline]
                            pub fn new(#(#arg_defs),*) -> Self {
                                Self(#(#arg_values),*, ::core::marker::PhantomData)
                            }
                        }
                    });
                }

                if should_derive_deref {
                    uses_deref_derives = true;
                    quote! {
                        /// Event type corresponding to the enum variant.
                        #[allow(unused_lifetimes, unused_type_parameters)]
                        #[derive(Event, Deref, DerefMut, Clone, Debug)]
                        pub struct #variant_ident #struct_generics_tokens(#(#field_tokens),*) #where_clause;
                    }
                } else {
                    quote! {
                        /// Event type corresponding to the enum variant.
                        #[allow(unused_lifetimes, unused_type_parameters)]
                        #[derive(Event, Clone, Debug)]
                        pub struct #variant_ident #struct_generics_tokens(#(#field_tokens),*) #where_clause;
                    }
                }
            }
            Fields::Named(fields) => {
                let struct_generics_tokens = struct_generics_tokens.clone();
                let field_infos: Vec<_> = fields
                    .named
                    .iter()
                    .map(|field| {
                        let info = analyze_field_attrs(&field.attrs);
                        let field_name = field
                            .ident
                            .as_ref()
                            .expect("Named fields must have identifiers")
                            .clone();
                        (info, field_name, &field.ty)
                    })
                    .collect();
                let field_count = field_infos.len();
                let deref_attr_fields = field_infos
                    .iter()
                    .filter(|(info, _, _)| info.has_deref)
                    .count();

                if field_count > 1 && deref_attr_fields > 1 {
                    panic!(
                        "EnumEvent: variant `{}` has multiple fields marked for deref (e.g., #[enum_event(deref)]); only one field can be dereferenced",
                        variant_ident
                    );
                }

                let should_derive_deref =
                    cfg!(feature = "deref") && (field_count == 1 || deref_attr_fields == 1);

                let auto_mark_single_field =
                    should_derive_deref && deref_attr_fields == 0 && field_count == 1;

                let mut field_tokens: Vec<_> = field_infos
                    .iter()
                    .map(|(info, field_name, field_type)| {
                        let passthrough_attrs = info.passthrough_attrs.iter();
                        let mut marker_attrs = Vec::new();

                        if should_derive_deref {
                            if info.has_deref {
                                marker_attrs.push(quote!(#[deref]));
                            }
                            if info.has_deref_mut {
                                marker_attrs.push(quote!(#[deref_mut]));
                            } else if auto_mark_single_field {
                                marker_attrs.push(quote!(#[deref]));
                            }
                        } else if auto_mark_single_field {
                            marker_attrs.push(quote!(#[deref]));
                        }

                        quote! {
                            #(#passthrough_attrs)*
                            #(#marker_attrs)*
                            pub #field_name: #field_type,
                        }
                    })
                    .collect();

                if let Some(phantom_type) = phantom_type.clone() {
                    field_tokens.push(quote! {
                        #[doc(hidden)]
                        pub(crate) _phantom: #phantom_type,
                    });

                    let (impl_generics_impl, ty_generics_impl, where_clause_impl) =
                        generics.split_for_impl();
                    let arg_defs: Vec<_> = field_infos
                        .iter()
                        .map(|(_, field_name, field_type)| {
                            quote! { #field_name: #field_type }
                        })
                        .collect();
                    let field_names: Vec<_> = field_infos
                        .iter()
                        .map(|(_, field_name, _)| field_name)
                        .collect();

                    extra_impl = Some(quote! {
                        impl #impl_generics_impl #variant_ident #ty_generics_impl #where_clause_impl {
                            #[inline]
                            pub fn new(#(#arg_defs),*) -> Self {
                                Self {
                                    #(#field_names),*,
                                    _phantom: ::core::marker::PhantomData,
                                }
                            }
                        }
                    });
                }

                if should_derive_deref {
                    uses_deref_derives = true;
                    quote! {
                        /// Event type corresponding to the enum variant.
                        #[allow(unused_lifetimes, unused_type_parameters)]
                        #[derive(Event, Deref, DerefMut, Clone, Debug)]
                        pub struct #variant_ident #struct_generics_tokens #where_clause {
                            #(#field_tokens)*
                        }
                    }
                } else {
                    quote! {
                        /// Event type corresponding to the enum variant.
                        #[allow(unused_lifetimes, unused_type_parameters)]
                        #[derive(Event, Clone, Debug)]
                        pub struct #variant_ident #struct_generics_tokens #where_clause {
                            #(#field_tokens)*
                        }
                    }
                }
            }
        };

        struct_defs.push(struct_def);
        if let Some(extra) = extra_impl {
            additional_impls.push(extra);
        }
    }

    let deref_imports = if cfg!(feature = "deref") && uses_deref_derives {
        quote! {
            use bevy::prelude::{Deref, DerefMut};
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        /// Generated module containing event types for each enum variant.
        pub mod #module_name {
            use bevy::prelude::Event;
            #deref_imports

            #(#struct_defs)*
            #(#additional_impls)*
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for generating a default FSMTransition implementation (requires `fsm` feature).
///
/// This macro generates a permissive `FSMTransition` implementation that allows all state
/// transitions. Use this for simple state machines where any transition should be allowed.
///
/// # Requirements
///
/// - Can be applied to any enum (doesn't require `EnumEvent` or `FSMState`)
/// - Requires `fsm` feature to be enabled
/// - Depends on `bevy_fsm::FSMTransition` trait
///
/// # Generated Code
///
/// Generates an implementation of `FSMTransition` with `can_transition` always returning `true`.
///
/// # Example (Zero Boilerplate)
///
/// ```rust,ignore
/// use bevy::prelude::*;
/// use bevy_enum_event::{EnumEvent, FSMTransition, FSMState};
///
/// #[derive(Component, EnumEvent, FSMTransition, FSMState, Clone, Copy, Debug)]
/// enum GameState {
///     MainMenu,
///     Playing,
///     GameOver,
/// }
///
/// // All transitions are allowed automatically!
/// // MainMenu -> Playing ✅
/// // Playing -> GameOver ✅
/// // GameOver -> MainMenu ✅
/// ```
///
/// # Example (Custom Rules - Don't Derive)
///
/// If you need custom transition logic, don't derive `FSMTransition`:
///
/// ```rust,ignore
/// use bevy::prelude::*;
/// use bevy_enum_event::{EnumEvent, FSMState};
/// use bevy_fsm::FSMTransition;
///
/// // No FSMTransition derive here!
/// #[derive(Component, EnumEvent, FSMState, Clone, Copy, Debug)]
/// enum LifeFSM {
///     Alive,
///     Dying,
///     Dead,
/// }
///
/// // Manually implement for custom rules
/// impl FSMTransition for LifeFSM {
///     fn can_transition(from: Self, to: Self) -> bool {
///         matches!((from, to),
///             (LifeFSM::Alive, LifeFSM::Dying) |
///             (LifeFSM::Dying, LifeFSM::Dead)) || from == to
///     }
/// }
/// ```
#[cfg(feature = "fsm")]
#[proc_macro_derive(FSMTransition)]
pub fn derive_fsm_transition(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = &input.ident;
    let generics = input.generics.clone();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Verify it's an enum (though not strictly necessary for FSMTransition)
    if !matches!(&input.data, Data::Enum(_)) {
        panic!("FSMTransition can only be derived for enums");
    }

    let expanded = quote! {
        impl #impl_generics bevy_fsm::FSMTransition for #enum_name #ty_generics #where_clause {
            /// Default implementation: allows all transitions.
            ///
            /// This is auto-generated by `#[derive(FSMTransition)]`.
            fn can_transition(_from: Self, _to: Self) -> bool {
                true
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for generating FSM state infrastructure (requires `fsm` feature).
///
/// This macro extends `EnumEvent` with finite state machine functionality by implementing
/// both the `FSMTransition` trait (with a default "allow all" implementation) and the
/// `FSMState` trait (with variant-specific event triggering). It must be used alongside
/// `#[derive(EnumEvent)]`.
///
/// # Requirements
///
/// - Must be applied to the same enum as `#[derive(EnumEvent)]`
/// - The enum must only have unit variants (no tuple or named fields)
/// - Requires `fsm` feature to be enabled
/// - Depends on types from `bevy_fsm` crate: `Enter<T>`, `Exit<T>`, `Transition<F, T>`, `FSMTransition`, `FSMState`
///
/// # Generated Code
///
/// For an enum named `MyFSM`, this generates:
///
/// 1. **FSMTransition implementation** (default: allows all transitions)
/// 2. **FSMState implementation** with three methods:
///    - `trigger_enter_variant(ec, state)` - Fires `Enter<module::Variant>` events
///    - `trigger_exit_variant(ec, state)` - Fires `Exit<module::Variant>` events
///    - `trigger_transition_variant(ec, from, to)` - Fires `Transition<module::From, module::To>` events
///
/// # Example (Zero Boilerplate - All Transitions Allowed)
///
/// ```rust,ignore
/// use bevy::prelude::*;
/// use bevy_enum_event::{EnumEvent, FSMState};
///
/// // Just two derives - no FSMTransition implementation needed!
/// #[derive(Component, EnumEvent, FSMState, Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// enum GameState {
///     MainMenu,
///     Playing,
///     GameOver,
/// }
///
/// // All transitions are allowed by default
/// // MainMenu -> Playing ✅
/// // Playing -> GameOver ✅
/// // GameOver -> MainMenu ✅ (even backwards transitions work!)
/// ```
///
/// # Example (Custom Transition Rules)
///
/// Override the default `FSMTransition` implementation to add custom rules:
///
/// ```rust,ignore
/// use bevy::prelude::*;
/// use bevy_enum_event::{EnumEvent, FSMState};
/// use bevy_fsm::FSMTransition;
///
/// #[derive(Component, EnumEvent, FSMState, Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// enum LifeFSM {
///     Alive,
///     Dying,
///     Dead,
/// }
///
/// // Override the default to add custom transition rules
/// impl FSMTransition for LifeFSM {
///     fn can_transition(from: Self, to: Self) -> bool {
///         matches!((from, to),
///             (LifeFSM::Alive, LifeFSM::Dying) |
///             (LifeFSM::Dying, LifeFSM::Dead)) || from == to
///     }
/// }
///
/// // Now transitions are restricted:
/// // Alive -> Dying ✅
/// // Dying -> Dead ✅
/// // Dead -> Alive ❌ (blocked by custom rules)
/// ```
///
/// # Panics
///
/// - Panics if applied to a non-enum type
/// - Panics if any variant has fields (only unit variants are supported for FSM)
#[cfg(feature = "fsm")]
#[proc_macro_derive(FSMState)]
pub fn derive_derive_fsm_state(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = &input.ident;
    let generics = input.generics.clone();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let turbofish = if generics.params.is_empty() {
        quote! {}
    } else {
        quote! { ::#ty_generics }
    };

    // Extract variants from enum
    let variants = match &input.data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => panic!("FSMState can only be derived for enums"),
    };

    // Verify all variants are unit variants
    for variant in variants {
        if !matches!(variant.fields, Fields::Unit) {
            panic!("FSMState enum variants must be unit variants (no fields). Variant '{}' has fields.", variant.ident);
        }
    }

    let variant_idents: Vec<_> = variants.iter().map(|v| &v.ident).collect();

    // Generate the module name (same as EnumEvent uses)
    let module_name_str = to_snake_case(&enum_name.to_string());
    let fsm_module_name = syn::Ident::new(&module_name_str, enum_name.span());

    // Generate Enter event triggers for each variant
    let enter_triggers: Vec<_> = variant_idents
        .iter()
        .map(|variant| {
            let turbofish = turbofish.clone();
            let variant_ty = quote! { #fsm_module_name::#variant #ty_generics };
            let variant_value = quote! { #fsm_module_name::#variant #turbofish::default() };
            quote! {
                #enum_name::#variant => {
                    ec.trigger(bevy_fsm::Enter::<#variant_ty> {
                        state: #variant_value,
                    });
                }
            }
        })
        .collect();

    // Generate Exit event triggers for each variant
    let exit_triggers: Vec<_> = variant_idents
        .iter()
        .map(|variant| {
            let turbofish = turbofish.clone();
            let variant_ty = quote! { #fsm_module_name::#variant #ty_generics };
            let variant_value = quote! { #fsm_module_name::#variant #turbofish::default() };
            quote! {
                #enum_name::#variant => {
                    ec.trigger(bevy_fsm::Exit::<#variant_ty> {
                        state: #variant_value,
                    });
                }
            }
        })
        .collect();

    // Generate all pairs of transition types (N × N combinations)
    let mut transition_triggers = Vec::new();
    for from_variant in &variant_idents {
        for to_variant in &variant_idents {
            let turbofish_from = turbofish.clone();
            let turbofish_to = turbofish.clone();
            let from_ty = quote! { #fsm_module_name::#from_variant #ty_generics };
            let to_ty = quote! { #fsm_module_name::#to_variant #ty_generics };
            let from_value = quote! { #fsm_module_name::#from_variant #turbofish_from::default() };
            let to_value = quote! { #fsm_module_name::#to_variant #turbofish_to::default() };
            transition_triggers.push(quote! {
                (#enum_name::#from_variant, #enum_name::#to_variant) => {
                    ec.trigger(bevy_fsm::Transition::<#from_ty, #to_ty> {
                        from: #from_value,
                        to: #to_value,
                    });
                }
            });
        }
    }

    let expanded = quote! {
        // Implement the FSMState trait methods
        impl #impl_generics bevy_fsm::FSMState for #enum_name #ty_generics #where_clause {
            /// Triggers variant-specific Enter event.
            ///
            /// This method is generated by `#[derive(FSMState)]` and is used internally
            /// by the bevy_fsm framework to fire Enter events for specific state variants.
            fn trigger_enter_variant(ec: &mut bevy::prelude::EntityCommands, state: Self) {
                match state {
                    #(#enter_triggers)*
                }
            }

            /// Triggers variant-specific Exit event.
            ///
            /// This method is generated by `#[derive(FSMState)]` and is used internally
            /// by the bevy_fsm framework to fire Exit events for specific state variants.
            fn trigger_exit_variant(ec: &mut bevy::prelude::EntityCommands, state: Self) {
                match state {
                    #(#exit_triggers)*
                }
            }

            /// Triggers variant-specific Transition event.
            ///
            /// This method is generated by `#[derive(FSMState)]` and is used internally
            /// by the bevy_fsm framework to fire Transition events between specific state variants.
            fn trigger_transition_variant(ec: &mut bevy::prelude::EntityCommands, from: Self, to: Self) {
                match (from, to) {
                    #(#transition_triggers)*
                }
            }
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
