//! General-purpose enum to Bevy event conversion macro.
//!
//! This crate provides derive macros that generate Bevy event types from enum variants.
//! For each variant, it creates a corresponding event struct in a `snake_case` module.
//!
//! # Quick Start
//!
//! ```rust
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
//! fn on_paused(paused: On<game_state::Paused>) {
//!     println!("Game paused!");
//! }
//! ```
//!
//! # Event Types
//!
//! Bevy 0.17 introduced two distinct event types:
//!
//! - **`Event`**: Global events that are not associated with any specific entity
//! - **`EntityEvent`**: Events that target a specific entity and trigger entity-specific observers
//!
//! This crate provides corresponding derive macros:
//!
//! - `#[derive(EnumEvent)]` - Generates `Event` types (see examples below)
//! - `#[derive(EnumEntityEvent)]` - Generates `EntityEvent` types (see [EnumEntityEvent examples](#enumentityevent))
//!
//! # Example: `EnumEvent` (Unit Variants)
//!
//! ```rust
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
//! ```rust
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
//! # Example: Variants with Data
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
//! ```rust
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
//! # Deref Feature (enabled by default)
//!
//! The `deref` feature provides ergonomic access to event data:
//!
//! - **Single-field variants**: Automatically implement `Deref`/`DerefMut` to the inner value
//! - **Multi-field variants**: Mark one field with `#[enum_event(deref)]` for deref access
//! - **No annotation**: Access fields directly by name
//!
#![cfg_attr(
    feature = "deref",
    doc = r#"
```
use bevy_enum_event::EnumEvent;
use std::ops::Deref;

#[derive(EnumEvent, Clone)]
enum NetworkEvent {
    MessageReceived(String),  // Single field - automatic deref
    PlayerScored { #[enum_event(deref)] player: u32, points: u32 },  // Multi-field with deref
    TeamScore { team: u32, points: u32 },  // No deref - access fields directly
}

// Single-field variant - automatic deref
let msg = network_event::MessageReceived("Hello".to_string());
let content: &String = msg.deref();
assert_eq!(content, "Hello");

// Multi-field with #[enum_event(deref)] - deref to the marked field
let scored = network_event::PlayerScored { player: 7, points: 120 };
let player: &u32 = scored.deref();
assert_eq!(*player, 7);

// No deref annotation - access fields directly
let team_score = network_event::TeamScore { team: 1, points: 50 };
assert_eq!(team_score.team, 1);
assert_eq!(team_score.points, 50);
```
"#
)]
//!
//! Disable the feature in `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! bevy_enum_event = { version = "0.2", default-features = false }
//! ```
//!
//! # Example: `EnumEntityEvent`
//!
//! ```rust
//! use bevy::prelude::*;
//! use bevy_enum_event::EnumEntityEvent;
//!
//! #[derive(EnumEntityEvent, Clone, Copy)]
//! enum PlayerEvent {
//!     Spawned { entity: Entity },
//!     Damaged { entity: Entity, amount: f32 },
//!     Destroyed { entity: Entity },
//! }
//!
//! fn on_player_damaged(damaged: On<player_event::Damaged>) {
//!     println!("Player {:?} took {} damage", damaged.entity, damaged.amount);
//! }
//! ```
//!
//! # EntityEvent Features
//!
//! ## Requirements
//!
//! - **Named fields only**: All variants must use `{ field: Type }` syntax
//! - **Entity field**: Each variant needs an `entity: Entity` field or a field marked `#[enum_event(target)]`
//! - **Triggering**: Use `commands.trigger(event)` or `world.trigger(event)`
//!
//! ## Custom Target Field
//!
//! By default, `EnumEntityEvent` looks for a field named `entity`.
//! Use `#[enum_event(target)]` to specify a different field:
//!
//! ```rust
//! use bevy::prelude::*;
//! use bevy_enum_event::EnumEntityEvent;
//!
//! #[derive(EnumEntityEvent, Clone, Copy)]
//! enum CombatEvent {
//!     Attack {
//!         #[enum_event(target)]
//!         attacker: Entity,
//!         defender: Entity,
//!     },
//! }
//! ```
//!
//! ## Event Propagation
//!
//! Enable event propagation to bubble events up entity hierarchies:
//!
//! ```rust
//! use bevy::prelude::*;
//! use bevy_enum_event::EnumEntityEvent;
//!
//! // Default propagation (uses ChildOf)
//! #[derive(EnumEntityEvent, Clone, Copy)]
//! #[enum_event(propagate)]
//! enum UiEvent {
//!     Click { entity: Entity },
//! }
//!
//! // Auto propagation (always bubbles up)
//! #[derive(EnumEntityEvent, Clone, Copy)]
//! #[enum_event(auto_propagate, propagate)]
//! enum SystemEvent {
//!     Update { entity: Entity },
//! }
//!
//! // Custom relationship type
//! #[derive(EnumEntityEvent, Clone, Copy)]
//! #[enum_event(propagate = &'static ::bevy::prelude::ChildOf)]
//! enum CustomEvent {
//!     Action { entity: Entity },
//! }
//! ```
//!
//! **Note**: Custom relationship types must be `pub` or use absolute paths (`::bevy::`, `crate::`)
//! because they're accessed from the generated module.

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

impl<'ast> Visit<'ast> for GenericsUsageCollector<'_> {
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
        .is_some_and(|segment| segment.ident == ident)
}

#[derive(Default)]
struct FieldAttrInfo {
    passthrough_attrs: Vec<Attribute>,
    has_deref: bool,
    has_deref_mut: bool,
    is_event_target: bool,
}

#[derive(Default)]
struct VariantAttrInfo {
    propagate_value: Option<proc_macro2::TokenStream>,
    has_auto_propagate: bool,
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
                } else if path_ends_with_ident(&meta.path, "target") {
                    info.is_event_target = true;
                }
                Ok(())
            }) {
                panic!("EnumEvent: failed to parse #[enum_event(...)] attribute: {err}");
            }
        } else if path_ends_with_ident(attr.path(), "event_target") {
            info.is_event_target = true;
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

fn analyze_variant_attrs(attrs: &[Attribute]) -> VariantAttrInfo {
    let mut info = VariantAttrInfo::default();

    for attr in attrs {
        if path_ends_with_ident(attr.path(), "enum_event") {
            if let Err(err) = attr.parse_nested_meta(|meta| {
                if path_ends_with_ident(&meta.path, "auto_propagate") {
                    info.has_auto_propagate = true;
                    Ok(())
                } else if path_ends_with_ident(&meta.path, "propagate") {
                    if meta.input.peek(syn::Token![=]) {
                        // Parse: propagate = <value>
                        meta.input.parse::<syn::Token![=]>()?;
                        let tokens: proc_macro2::TokenStream = meta.input.parse()?;
                        info.propagate_value = Some(tokens);
                    } else {
                        // Just: propagate (no value, uses default)
                        info.propagate_value = Some(quote! {});
                    }
                    Ok(())
                } else {
                    // Unknown attributes on variants are just ignored (could be other macro's attributes)
                    Ok(())
                }
            }) {
                panic!("EnumEvent: failed to parse variant #[enum_event(...)] attribute: {err}");
            }
        }
    }

    info
}

/// Derive macro that generates Bevy `Event` types from enum variants.
///
/// Generates a snake_case module containing one `Event` struct per enum variant.
/// Supports unit, tuple, and named field variants.
///
/// # Example
///
/// ```rust
/// use bevy_enum_event::EnumEvent;
///
/// #[derive(EnumEvent, Clone)]
/// enum Action {
///     Jump,
///     Run(f32),
///     Attack { damage: i32, critical: bool },
/// }
/// ```
///
/// Generates:
///
/// ```rust
/// pub mod action {
///     use bevy::prelude::Event;
///
///     #[derive(Event, Clone, Debug)]
///     pub struct Jump;
///
///     #[derive(Event, Clone, Debug)]
///     pub struct Run(pub f32);
///
///     #[derive(Event, Clone, Debug)]
///     pub struct Attack {
///         pub damage: i32,
///         pub critical: bool,
///     }
/// }
/// ```
///
/// # Deref Feature
///
/// When enabled (default), single-field variants automatically implement `Deref`/`DerefMut`.
/// For multi-field variants, mark one field with `#[enum_event(deref)]`.
///
/// # Panics
///
/// Panics if applied to a non-enum type.
#[proc_macro_derive(EnumEvent, attributes(enum_event, deref, deref_mut))]
pub fn derive_enum_events(input: TokenStream) -> TokenStream {
    derive_enum_event_impl(input, false)
}

/// Derive macro that generates Bevy `EntityEvent` types from enum variants.
///
/// Like `EnumEvent`, but generates entity-targeted events that trigger entity-specific observers.
///
/// # Requirements
///
/// - **Named fields only**: All variants must use `{ field: Type }` syntax
/// - **Entity field**: Must have `entity: Entity` or a field marked `#[enum_event(target)]`
///
/// # Example
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy_enum_event::EnumEntityEvent;
///
/// #[derive(EnumEntityEvent, Clone, Copy)]
/// enum PlayerEvent {
///     Spawned { entity: Entity },
///     Damaged { entity: Entity, amount: f32 },
/// }
/// ```
///
/// # Custom Target Field
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy_enum_event::EnumEntityEvent;
///
/// #[derive(EnumEntityEvent, Clone, Copy)]
/// enum CombatEvent {
///     Attack {
///         #[enum_event(target)]
///         attacker: Entity,
///         defender: Entity,
///     },
/// }
/// ```
///
/// # Propagation
///
/// Enable event bubbling up entity hierarchies:
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy_enum_event::EnumEntityEvent;
///
/// // Basic propagation (uses ChildOf)
/// #[derive(EnumEntityEvent, Clone, Copy)]
/// #[enum_event(propagate)]
/// enum UiEvent {
///     Click { entity: Entity },
/// }
///
/// // Auto propagation (always bubbles)
/// #[derive(EnumEntityEvent, Clone, Copy)]
/// #[enum_event(auto_propagate, propagate)]
/// enum SystemEvent {
///     Update { entity: Entity },
/// }
///
/// // Custom relationship
/// #[derive(EnumEntityEvent, Clone, Copy)]
/// #[enum_event(propagate = &'static ::bevy::prelude::ChildOf)]
/// enum CustomEvent {
///     Action { entity: Entity },
/// }
/// ```
///
/// # Variant-Level Propagation
///
/// Override enum-level settings for specific variants:
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy_enum_event::EnumEntityEvent;
///
/// #[derive(EnumEntityEvent, Clone, Copy)]
/// #[enum_event(propagate)]  // Default for all
/// #[allow(dead_code)]
/// enum MixedEvent {
///     Normal { entity: Entity },  // Uses default
///
///     #[enum_event(auto_propagate, propagate)]  // Override
///     AutoEvent { entity: Entity },
/// }
/// ```
///
/// **Note**: Custom relationships must be `pub` or use absolute paths (`::bevy::`, `crate::`).
#[proc_macro_derive(
    EnumEntityEvent,
    attributes(enum_event, event_target, deref, deref_mut)
)]
pub fn derive_enum_entity_events(input: TokenStream) -> TokenStream {
    derive_enum_event_impl(input, true)
}

#[allow(clippy::too_many_lines)]
fn derive_enum_event_impl(input: TokenStream, is_entity_event: bool) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = &input.ident;

    // Check for propagate and auto_propagate attributes on the enum
    // Can be: #[enum_event(propagate)]
    //         #[enum_event(propagate = &'static RelType)]
    //         #[enum_event(auto_propagate, propagate = &'static RelType)]
    let mut propagate_value: Option<proc_macro2::TokenStream> = None;
    let mut has_auto_propagate = false;

    for attr in &input.attrs {
        if path_ends_with_ident(attr.path(), "enum_event") {
            attr.parse_nested_meta(|meta| {
                if path_ends_with_ident(&meta.path, "auto_propagate") {
                    has_auto_propagate = true;
                    Ok(())
                } else if path_ends_with_ident(&meta.path, "propagate") {
                    if meta.input.peek(syn::Token![=]) {
                        // Parse: propagate = <value>
                        // Capture the remaining tokens as-is without parsing
                        meta.input.parse::<syn::Token![=]>()?;
                        // Parse the rest of the input as raw tokens
                        let tokens: proc_macro2::TokenStream = meta.input.parse()?;
                        propagate_value = Some(tokens);
                    } else {
                        // Just: propagate (no value, uses default)
                        propagate_value = Some(quote! {});
                    }
                    Ok(())
                } else {
                    Err(meta.error("unknown enum_event attribute"))
                }
            })
            .unwrap_or_else(|e| panic!("Failed to parse enum_event attribute: {e}"));
        }
    }

    // Extract variants from enum
    let variants = match &input.data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => panic!("EnumEvent can only be derived for enums"),
    };

    // Convert EnumName to snake_case for module name
    let module_name_str = to_snake_case(&enum_name.to_string());
    let module_name = syn::Ident::new(&module_name_str, enum_name.span());

    #[allow(clippy::items_after_statements)]
    fn adjust_propagate_type_for_module(ty: &mut syn::Type) {
        fn adjust_path(path: &mut syn::TypePath) {
            if path.path.leading_colon.is_some() {
                return;
            }

            if let Some(first) = path.path.segments.first() {
                let ident = &first.ident;
                let starts_with_crate = ident == "crate";
                let starts_with_super = ident == "super";
                let starts_with_self = ident == "self";

                if starts_with_crate || starts_with_super || starts_with_self {
                    return;
                }
            }

            path.path.segments.insert(0, syn::parse_quote!(super));
        }

        match ty {
            syn::Type::Reference(ref mut reference) => {
                adjust_propagate_type_for_module(&mut reference.elem);
            }
            syn::Type::Path(ref mut type_path) => adjust_path(type_path),
            _ => {}
        }
    }

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

        // Parse variant-level propagate attributes
        let variant_attr_info = analyze_variant_attrs(&variant.attrs);

        // Determine propagate settings for this variant:
        // - If variant has propagate settings, use those (override enum-level)
        // - Otherwise, use enum-level settings
        let variant_has_propagate = variant_attr_info.propagate_value.is_some();
        let variant_propagate_value = if variant_has_propagate {
            variant_attr_info.propagate_value.clone()
        } else {
            propagate_value.clone()
        };
        let variant_auto_propagate = if variant_has_propagate {
            variant_attr_info.has_auto_propagate
        } else {
            has_auto_propagate
        };

        let mut usage_collector =
            GenericsUsageCollector::new(&type_param_names, &lifetime_param_names);
        for field in &variant.fields {
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

        // For EntityEvent, check if the variant has an entity field
        let has_entity_field = if is_entity_event {
            match &variant.fields {
                Fields::Named(fields) => {
                    // Check for entity field or marked target field
                    let target_fields: Vec<_> = fields
                        .named
                        .iter()
                        .filter(|field| {
                            let info = analyze_field_attrs(&field.attrs);
                            info.is_event_target
                                || field
                                    .ident
                                    .as_ref()
                                    .is_some_and(|id| id == "entity")
                        })
                        .collect();

                    assert!(target_fields.len() <= 1,
                            "EnumEntityEvent: variant `{variant_ident}` has multiple fields marked as event target; only one field can be the target"
                        );

                    !target_fields.is_empty()
                }
                Fields::Unnamed(_) | Fields::Unit => false,
            }
        } else {
            false
        };

        assert!(!is_entity_event || has_entity_field,
                "EnumEntityEvent: variant `{variant_ident}` must have an `entity: Entity` field or a field marked with #[enum_event(target)]"
            );

        let event_derive = if is_entity_event {
            quote! { EntityEvent }
        } else {
            quote! { Event }
        };

        let struct_def = match &variant.fields {
            Fields::Unit => {
                // Unit variants cannot be EntityEvents
                assert!(!is_entity_event,
                        "EnumEntityEvent: variant `{variant_ident}` is a unit variant; entity events must have at least an entity field"
                    );

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
                // Tuple variants cannot be EntityEvents
                assert!(!is_entity_event,
                        "EnumEntityEvent: variant `{variant_ident}` is a tuple variant; entity events must use named fields with an `entity: Entity` field"
                    );

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

                assert!(!(field_count > 1 && deref_attr_fields > 1),
                        "EnumEvent: variant `{variant_ident}` has multiple fields marked for deref (e.g., #[enum_event(deref)]); only one field can be dereferenced"
                    );

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

                assert!(!(field_count > 1 && deref_attr_fields > 1),
                        "EnumEvent: variant `{variant_ident}` has multiple fields marked for deref (e.g., #[enum_event(deref)]); only one field can be dereferenced"
                    );

                let should_derive_deref =
                    cfg!(feature = "deref") && (field_count == 1 || deref_attr_fields == 1);

                let auto_mark_single_field =
                    should_derive_deref && deref_attr_fields == 0 && field_count == 1;

                let mut field_tokens: Vec<_> = field_infos
                    .iter()
                    .map(|(info, field_name, field_type)| {
                        let passthrough_attrs = info.passthrough_attrs.iter();
                        let mut marker_attrs = Vec::new();

                        // Add event_target attribute for EntityEvent
                        if is_entity_event && (info.is_event_target || field_name == "entity") {
                            marker_attrs.push(quote!(#[event_target]));
                        }

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

                // Note: We accept #[enum_event(propagate)] on the enum, but generate #[entity_event(propagate)]
                // on the struct because that's what Bevy's EntityEvent derive expects
                // Generate variant-specific propagate attributes
                let propagate_attr = if is_entity_event && variant_propagate_value.is_some() {
                    match variant_propagate_value.clone() {
                        Some(tokens) if tokens.is_empty() => {
                            if variant_auto_propagate {
                                quote! { #[entity_event(auto_propagate, propagate)] }
                            } else {
                                quote! { #[entity_event(propagate)] }
                            }
                        }
                        Some(tokens) => {
                            let adjusted_tokens = if let Ok(mut ty) = syn::parse2::<syn::Type>(tokens.clone()) {
                                adjust_propagate_type_for_module(&mut ty);
                                quote! { #ty }
                            } else {
                                quote! { #tokens }
                            };

                            if variant_auto_propagate {
                                quote! { #[entity_event(auto_propagate, propagate = #adjusted_tokens)] }
                            } else {
                                quote! { #[entity_event(propagate = #adjusted_tokens)] }
                            }
                        }
                        None => quote! {},
                    }
                } else {
                    quote! {}
                };

                if should_derive_deref {
                    uses_deref_derives = true;
                    quote! {
                        /// Event type corresponding to the enum variant.
                        #[allow(unused_lifetimes, unused_type_parameters)]
                        #[derive(#event_derive, Deref, DerefMut, Clone, Debug)]
                        #propagate_attr
                        pub struct #variant_ident #struct_generics_tokens #where_clause {
                            #(#field_tokens)*
                        }
                    }
                } else {
                    quote! {
                        /// Event type corresponding to the enum variant.
                        #[allow(unused_lifetimes, unused_type_parameters)]
                        #[derive(#event_derive, Clone, Debug)]
                        #propagate_attr
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

    let event_import = if is_entity_event {
        quote! {
            use bevy::prelude::{Entity, EntityEvent};
        }
    } else {
        quote! {
            use bevy::prelude::Event;
        }
    };

    let expanded = quote! {
        /// Generated module containing event types for each enum variant.
        pub mod #module_name {
            #event_import
            #deref_imports

            #(#struct_defs)*
            #(#additional_impls)*
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

