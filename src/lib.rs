//! Derive macros that generate Bevy event and message types from enum variants.
//!
//! Each variant becomes a separate struct in a `snake_case` module.
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
//! # Macros
//!
//! Bevy 0.17+ distinguishes between three event/message types:
//!
//! - `#[derive(EnumEvent)]` - Observer-based global events (triggered via `world.trigger()`)
//! - `#[derive(EnumEntityEvent)]` - Entity-targeted observer events with optional propagation
//! - `#[derive(EnumMessage)]` - Buffered messages (written via `MessageWriter`, read via `MessageReader`)
//!
//! # EnumEvent
//!
//! For observer-based events that are triggered globally and handled by observers.
//!
//! ```rust
//! use bevy_enum_event::EnumEvent;
//!
//! #[derive(EnumEvent, Clone)]
//! enum GameEvent {
//!     Victory(String),
//!     ScoreChanged { team: u32, score: i32 },
//!     GameOver,
//! }
//! // Generates: game_event::Victory, game_event::ScoreChanged, game_event::GameOver
//! // Each derives Event and is used with triggers and observers
//! ```
//!
//! # EnumMessage
//!
//! For buffered messages that are written/read between systems using `MessageWriter`/`MessageReader`.
//! These require registration with `app.add_message::<T>()`.
//!
//! ```rust
//! use bevy_enum_event::EnumMessage;
//!
//! #[derive(EnumMessage, Clone)]
//! enum NetworkMessage {
//!     Connected(String),
//!     Disconnected { reason: String },
//!     DataReceived { data: Vec<u8> },
//! }
//! // Generates: network_message::Connected, network_message::Disconnected, network_message::DataReceived
//! // Each derives Message and is used with MessageWriter/MessageReader
//! ```
//!
//! # Deref Feature (default)
//!
//! Single-field variants auto-implement `Deref`/`DerefMut`. For multi-field variants,
//! use `#[enum_event(deref)]`. Disable with `default-features = false`.
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
    PlayerScored { #[enum_event(deref)] player: u32, points: u32 },
}

let msg = network_event::MessageReceived("Hello".to_string());
assert_eq!(msg.deref(), "Hello");

let scored = network_event::PlayerScored { player: 7, points: 120 };
assert_eq!(*scored.deref(), 7);
```
"#
)]
//!
//! # Forwarding Derives
//!
//! Generated variant structs always derive `Clone` and `Debug` (unit variants
//! also derive `Copy` and `Default`). To add more derives — most commonly
//! `Copy` for small events — list them with `#[enum_event(derive(...))]`. A
//! derive macro cannot observe the enum's own `#[derive(...)]`, so the derives
//! to forward are named explicitly.
//!
//! ```rust
//! use bevy_enum_event::EnumEvent;
//!
//! #[derive(EnumEvent, Clone, Copy, PartialEq)]
//! #[enum_event(derive(Copy, PartialEq))]
//! enum Input {
//!     Jump,
//!     Move { dx: f32, dy: f32 },
//! }
//! // input::Jump and input::Move are both `Copy` + `PartialEq`.
//! ```
//!
//! Per-variant `#[enum_event(derive(...))]` is additive to the enum-level list,
//! so variants that can satisfy extra bounds can opt in individually.
//!
//! # EnumEntityEvent
//!
//! Entity-targeted events. Requires named fields with `entity: Entity` or `#[enum_event(target)]`.
//!
//! ```rust
//! use bevy::prelude::*;
//! use bevy_enum_event::EnumEntityEvent;
//!
//! #[derive(EnumEntityEvent, Clone, Copy)]
//! enum PlayerEvent {
//!     Damaged { entity: Entity, amount: f32 },
//! }
//! ```
//!
//! ## Custom Target
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
//! ## Propagation
//!
//! ```rust
//! use bevy::prelude::*;
//! use bevy_enum_event::EnumEntityEvent;
//!
//! #[derive(EnumEntityEvent, Clone, Copy)]
//! #[enum_event(propagate)]
//! enum UiEvent {
//!     Click { entity: Entity },
//! }
//!
//! #[derive(EnumEntityEvent, Clone, Copy)]
//! #[enum_event(auto_propagate, propagate)]
//! enum SystemEvent {
//!     Update { entity: Entity },
//! }
//!
//! #[derive(EnumEntityEvent, Clone, Copy)]
//! #[enum_event(propagate = &'static ::bevy::prelude::ChildOf)]
//! enum CustomEvent {
//!     Action { entity: Entity },
//! }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashSet;
use syn::{Attribute, Data, DeriveInput, Fields, parse_macro_input, visit::Visit};

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

/// Derives that the generated variant structs always add themselves. Requesting
/// any of these via `#[enum_event(derive(...))]` is a no-op so that a
/// forwarded list can be copied verbatim from the enum's own `#[derive(...)]`
/// without producing conflicting duplicate derives.
///
/// A proc-macro derive cannot observe the *other* derives applied to the same
/// item (the compiler strips the `#[derive(...)]` list before invoking us), so
/// forwarding is opt-in through the `#[enum_event(derive(...))]` helper
/// attribute rather than automatic.
const ALWAYS_DERIVED: &[&str] = &["Clone", "Debug"];

/// Additional derives that unit-variant structs always add on top of
/// [`ALWAYS_DERIVED`] (they are field-less, so these are always sound).
const ALWAYS_DERIVED_UNIT: &[&str] = &["Copy", "Default"];

fn path_last_ident(path: &syn::Path) -> Option<String> {
    path.segments
        .last()
        .map(|segment| segment.ident.to_string())
}

/// Parses forwarded derives from `#[enum_event(derive(...))]` on the given
/// attributes, appending each derive path to `out` while skipping duplicates.
fn collect_forwarded_derives(attrs: &[Attribute], out: &mut Vec<syn::Path>) {
    for attr in attrs {
        if !path_ends_with_ident(attr.path(), "enum_event") {
            continue;
        }

        if let Err(err) = attr.parse_nested_meta(|meta| {
            if path_ends_with_ident(&meta.path, "derive") {
                meta.parse_nested_meta(|derive_meta| {
                    if !out
                        .iter()
                        .any(|existing| paths_match(existing, &derive_meta.path))
                    {
                        out.push(derive_meta.path.clone());
                    }
                    Ok(())
                })
            } else {
                // Leave other keys (deref, target, propagate, ...) to their
                // dedicated parsers.
                skip_meta_value(&meta);
                Ok(())
            }
        }) {
            panic!("bevy_enum_event: failed to parse #[enum_event(derive(...))] attribute: {err}");
        }
    }
}

/// Two derive paths are considered the same forwarded derive when their final
/// segment matches (so `Copy` and `std::marker::Copy` do not both appear).
fn paths_match(a: &syn::Path, b: &syn::Path) -> bool {
    path_last_ident(a) == path_last_ident(b)
}

/// Consumes an optional `= value` or `(...)` payload attached to a nested meta
/// key we do not handle here, so `parse_nested_meta` can continue past it.
fn skip_meta_value(meta: &syn::meta::ParseNestedMeta) {
    if meta.input.peek(syn::Token![=]) {
        let _ = meta
            .value()
            .and_then(syn::parse::ParseBuffer::parse::<proc_macro2::TokenStream>);
    } else if meta.input.peek(syn::token::Paren) {
        let _ = meta.parse_nested_meta(|_| Ok(()));
    }
}

/// Builds the comma-prefixed token list of forwarded derives to splice into a
/// generated struct's `#[derive(...)]`, skipping any that the struct already
/// derives (`skip`).
fn forwarded_derive_tokens(forwarded: &[syn::Path], skip: &[&str]) -> proc_macro2::TokenStream {
    let filtered = forwarded
        .iter()
        .filter(|path| path_last_ident(path).is_none_or(|name| !skip.contains(&name.as_str())));
    quote! { #(, #filtered)* }
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
    forwarded_derives: Vec<syn::Path>,
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
                } else {
                    // Unknown keys belong to other macros; consume any payload
                    // so parsing can continue past them.
                    skip_meta_value(&meta);
                }
                Ok(())
            }) {
                panic!("bevy_enum_event: failed to parse #[enum_event(...)] attribute: {err}");
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
                } else if path_ends_with_ident(&meta.path, "derive") {
                    // Per-variant forwarded derives, additive to enum-level ones.
                    meta.parse_nested_meta(|derive_meta| {
                        if !info
                            .forwarded_derives
                            .iter()
                            .any(|existing| paths_match(existing, &derive_meta.path))
                        {
                            info.forwarded_derives.push(derive_meta.path.clone());
                        }
                        Ok(())
                    })
                } else {
                    // Unknown attributes on variants are just ignored (could be other macro's attributes)
                    skip_meta_value(&meta);
                    Ok(())
                }
            }) {
                panic!("EnumMessage: failed to parse variant #[enum_event(...)] attribute: {err}");
            }
        }
    }

    info
}

/// Generates Bevy `Event` types from enum variants for observer-based events.
///
/// Creates a `snake_case` module with one event struct per variant.
/// These events are triggered via `world.trigger()` and handled by observers.
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
/// // Generates: action::Jump, action::Run, action::Attack
/// // Each struct derives Event for use with triggers/observers
/// ```
#[proc_macro_derive(EnumEvent, attributes(enum_event, deref, deref_mut))]
pub fn derive_enum_events(input: TokenStream) -> TokenStream {
    derive_enum_event_impl(input, EventKind::Event)
}

/// Generates Bevy `Message` types from enum variants for buffered message passing.
///
/// Creates a `snake_case` module with one message struct per variant.
/// These messages are written via `MessageWriter` and read via `MessageReader`.
/// Each generated type must be registered with `app.add_message::<T>()`.
///
/// ```rust
/// use bevy_enum_event::EnumMessage;
///
/// #[derive(EnumMessage, Clone)]
/// enum NetworkMessage {
///     Connected(String),
///     Disconnected { reason: String },
///     DataReceived { data: Vec<u8> },
/// }
/// // Generates: network_message::Connected, network_message::Disconnected, network_message::DataReceived
/// // Each struct derives Message for use with MessageWriter/MessageReader
/// ```
#[proc_macro_derive(EnumMessage, attributes(enum_event, deref, deref_mut))]
pub fn derive_enum_messages(input: TokenStream) -> TokenStream {
    derive_enum_event_impl(input, EventKind::Message)
}

/// Generates Bevy `EntityEvent` types from enum variants.
///
/// Requires named fields with `entity: Entity` or `#[enum_event(target)]`.
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
/// # Propagation
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy_enum_event::EnumEntityEvent;
///
/// #[derive(EnumEntityEvent, Clone, Copy)]
/// #[enum_event(propagate)]
/// enum UiEvent {
///     Click { entity: Entity },
/// }
///
/// #[derive(EnumEntityEvent, Clone, Copy)]
/// #[enum_event(auto_propagate, propagate)]
/// enum SystemEvent {
///     Update { entity: Entity },
/// }
/// ```
#[proc_macro_derive(
    EnumEntityEvent,
    attributes(enum_event, event_target, deref, deref_mut)
)]
pub fn derive_enum_entity_events(input: TokenStream) -> TokenStream {
    derive_enum_event_impl(input, EventKind::EntityEvent)
}

/// Specifies which kind of Bevy event/message to generate.
#[derive(Clone, Copy, PartialEq, Eq)]
enum EventKind {
    /// Observer-based global events (`#[derive(Event)]`)
    Event,
    /// Buffered messages (`#[derive(Message)]`)
    Message,
    /// Entity-targeted observer events (`#[derive(EntityEvent)]`)
    EntityEvent,
}

#[allow(clippy::too_many_lines)]
fn derive_enum_event_impl(input: TokenStream, event_kind: EventKind) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = &input.ident;
    let is_entity_event = event_kind == EventKind::EntityEvent;

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
                } else if path_ends_with_ident(&meta.path, "derive") {
                    // Handled separately by `collect_forwarded_derives`; just
                    // consume the `(...)` payload here so parsing can continue.
                    skip_meta_value(&meta);
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
        _ => panic!("bevy_enum_event: macros can only be derived for enums"),
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
            syn::Type::Reference(reference) => {
                adjust_propagate_type_for_module(&mut reference.elem);
            }
            syn::Type::Path(type_path) => adjust_path(type_path),
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

    // Reasonable derives (e.g. `Copy`, `PartialEq`, `Hash`) are forwarded onto
    // every generated variant struct via `#[enum_event(derive(...))]`. A derive
    // macro cannot observe the enum's own `#[derive(...)]` list (the compiler
    // strips it before invoking us), so this opt-in helper attribute is how the
    // enum's derives get carried onto the generated events.
    let mut enum_forwarded_derives: Vec<syn::Path> = Vec::new();
    collect_forwarded_derives(&input.attrs, &mut enum_forwarded_derives);

    // Generate struct definitions for each variant
    let mut struct_defs = Vec::new();
    let mut additional_impls = Vec::new();
    let mut uses_deref_derives = false;

    for variant in variants {
        let variant_ident = &variant.ident;
        let struct_generics_tokens = struct_generics.clone();

        // Parse variant-level propagate attributes
        let variant_attr_info = analyze_variant_attrs(&variant.attrs);

        // Combine enum-level and variant-level forwarded derives (variant-level
        // is additive), then build the comma-prefixed token lists to splice into
        // each struct's `#[derive(...)]`. Field-bearing structs already derive
        // `Clone`/`Debug`; unit structs also derive `Copy`/`Default`, so those
        // are filtered out to avoid duplicate-derive errors.
        let mut variant_forwarded_derives = enum_forwarded_derives.clone();
        for path in &variant_attr_info.forwarded_derives {
            if !variant_forwarded_derives
                .iter()
                .any(|existing| paths_match(existing, path))
            {
                variant_forwarded_derives.push(path.clone());
            }
        }
        let forwarded_derives_field =
            forwarded_derive_tokens(&variant_forwarded_derives, ALWAYS_DERIVED);
        let forwarded_derives_unit = forwarded_derive_tokens(
            &variant_forwarded_derives,
            &[ALWAYS_DERIVED, ALWAYS_DERIVED_UNIT].concat(),
        );

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
                                || field.ident.as_ref().is_some_and(|id| id == "entity")
                        })
                        .collect();

                    assert!(
                        target_fields.len() <= 1,
                        "EnumEntityEvent: variant `{variant_ident}` has multiple fields marked as event target; only one field can be the target"
                    );

                    !target_fields.is_empty()
                }
                Fields::Unnamed(_) | Fields::Unit => false,
            }
        } else {
            false
        };

        assert!(
            !is_entity_event || has_entity_field,
            "EnumEntityEvent: variant `{variant_ident}` must have an `entity: Entity` field or a field marked with #[enum_event(target)]"
        );

        let event_derive = match event_kind {
            EventKind::EntityEvent => quote! { EntityEvent },
            EventKind::Message => quote! { Message },
            EventKind::Event => quote! { Event },
        };

        let struct_doc = match event_kind {
            EventKind::EntityEvent => "Entity event type corresponding to the enum variant.",
            EventKind::Message => "Message type corresponding to the enum variant.",
            EventKind::Event => "Event type corresponding to the enum variant.",
        };

        let struct_def = match &variant.fields {
            Fields::Unit => {
                // Unit variants cannot be EntityEvents
                assert!(
                    !is_entity_event,
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
                        #[doc = #struct_doc]
                        #[allow(unused_lifetimes, unused_type_parameters)]
                        #[derive(#event_derive, Clone, Copy, Debug, Default #forwarded_derives_unit)]
                        pub struct #variant_ident #struct_generics_tokens #where_clause {
                            #[doc(hidden)]
                            pub(crate) _phantom: #phantom_type,
                        }
                    }
                } else {
                    quote! {
                        #[doc = #struct_doc]
                        #[allow(unused_lifetimes, unused_type_parameters)]
                        #[derive(#event_derive, Clone, Copy, Debug, Default #forwarded_derives_unit)]
                        pub struct #variant_ident #struct_generics_tokens #where_clause;
                    }
                }
            }
            Fields::Unnamed(fields) => {
                // Tuple variants cannot be EntityEvents
                assert!(
                    !is_entity_event,
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

                assert!(
                    !(field_count > 1 && deref_attr_fields > 1),
                    "bevy_enum_event: variant `{variant_ident}` has multiple fields marked for deref (e.g., #[enum_event(deref)]); only one field can be dereferenced"
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
                        #[doc = #struct_doc]
                        #[allow(unused_lifetimes, unused_type_parameters)]
                        #[derive(#event_derive, Deref, DerefMut, Clone, Debug #forwarded_derives_field)]
                        pub struct #variant_ident #struct_generics_tokens(#(#field_tokens),*) #where_clause;
                    }
                } else {
                    quote! {
                        #[doc = #struct_doc]
                        #[allow(unused_lifetimes, unused_type_parameters)]
                        #[derive(#event_derive, Clone, Debug #forwarded_derives_field)]
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

                assert!(
                    !(field_count > 1 && deref_attr_fields > 1),
                    "bevy_enum_event: variant `{variant_ident}` has multiple fields marked for deref (e.g., #[enum_event(deref)]); only one field can be dereferenced"
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
                            let adjusted_tokens =
                                if let Ok(mut ty) = syn::parse2::<syn::Type>(tokens.clone()) {
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
                        #[doc = #struct_doc]
                        #[allow(unused_lifetimes, unused_type_parameters)]
                        #[derive(#event_derive, Deref, DerefMut, Clone, Debug #forwarded_derives_field)]
                        #propagate_attr
                        pub struct #variant_ident #struct_generics_tokens #where_clause {
                            #(#field_tokens)*
                        }
                    }
                } else {
                    quote! {
                        #[doc = #struct_doc]
                        #[allow(unused_lifetimes, unused_type_parameters)]
                        #[derive(#event_derive, Clone, Debug #forwarded_derives_field)]
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

    let event_import = match event_kind {
        EventKind::EntityEvent => quote! {
            use bevy::prelude::{Entity, EntityEvent};
        },
        EventKind::Message => quote! {
            use bevy::ecs::message::Message;
        },
        EventKind::Event => quote! {
            use bevy::prelude::Event;
        },
    };

    let module_doc = match event_kind {
        EventKind::EntityEvent => {
            "Generated module containing entity event types for each enum variant."
        }
        EventKind::Message => "Generated module containing message types for each enum variant.",
        EventKind::Event => "Generated module containing event types for each enum variant.",
    };

    let expanded = quote! {
        #[doc = #module_doc]
        pub mod #module_name {
            #[allow(unused_imports)]
            use super::*;
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
