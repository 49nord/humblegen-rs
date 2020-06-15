//! Implementation of humblespec embeds as an AST transformation.
//!
//! Embeds allow for re-use of struct defintions in humblespec.
//! This is useful for CRUD APIs (see the following example).
//!
//! # Example
//!
//! ```text
//! struct Monster {
//!     id: i32,
//!     .. MonsterData,
//! }
//!
//! struct MonsterData {
//!     name: str,
//!     hp: i32,
//! }
//! ```
//!
//! is equivalent to:
//!
//! ```text
//! struct Monster {
//!     id: i32,
//!     name: str,
//!     hp: i32,
//! }
//!
//! struct MonsterData {
//!     name: str,
//!     hp: i32,
//! }
//! ```
//!
//! # Rules
//!
//! - `MAX_EMBED_DEPTH` limits the maximum depth to which embeds are resolved.
//!   Exceeding that limit results in a panic.
//! - No need for declare-before-use.
//!
//! # Limitations
//!
//! - The transformation does not perform any collision checks.
//!   We rely on the rust compiler for that.
//!
//! - Embed-loops are not explicitly checked for but, since they are equivalent
//!   to infintely deep embeds, will result in a panic due to transgression of
//!   the `MAX_EMBED_DEPTH` limit.
//!
//! # Implementation:
//!
//! - AST representation of an embed is a bit hacky, see `FieldDefPair::is_embed`
//! - Fixed-point iteration that resolves embeds by one level per iteration.
//! - AST updates are performed in two phases (collect, update) in order to paciy
//!   the borrow checker and avoid iterator invalidation.

use crate::ast::*;
use std::collections::HashMap;
use std::iter::FromIterator;

const MAX_EMBED_DEPTH: usize = 10;

pub(crate) fn resolve_embeds(spec: &mut Spec) {
    let changed = std::cell::Cell::new(true);
    for _ in (0..=MAX_EMBED_DEPTH).take_while(|_| changed.get()) {
        changed.set(spec_resolve_embeds_one_level(spec));
    }
    if changed.get() {
        panic!("maximum embed depth is {}", MAX_EMBED_DEPTH);
    }
}

fn spec_resolve_embeds_one_level(spec: &mut Spec) -> bool {
    let mut changed = false;

    let all_structs_field_nodes: HashMap<String, &'_ Vec<FieldNode>> = HashMap::from_iter(
        spec.iter()
            .filter_map(|spec_item| match spec_item {
                SpecItem::StructDef(def) => Some(vec![(def.name.clone(), &def.fields.0)]),
                _ => None,
            })
            .flatten(),
    );

    // find the Vec<FieldNodes> that require expansion ("replacement") and queue those replacement operations
    // in hash map `replacements`
    let mut replacements: HashMap<String, Vec<FieldNode>> = HashMap::new();
    let struct_defs = spec
        .iter()
        .filter_map(|spec_item| match spec_item {
            SpecItem::StructDef(def) => Some(vec![(def.name.clone(), &def.fields.0)]),
            SpecItem::EnumDef(def) => Some(
                def.variants
                    .iter()
                    .filter_map(|v| {
                        v.variant_type
                            .struct_fields()
                            .map(|sf| (format!("{}.{}", def.name, v.name), &sf.0))
                    })
                    .collect::<Vec<_>>(),
            ),
            _ => None,
        })
        .flatten();
    for (struct_name, field_nodes) in struct_defs {
        let new_field_nodes = field_nodes
            .iter()
            .map(|field_node| {
                if field_node.pair.is_embed() {
                    changed = true;
                    let embedded_field_nodes = all_structs_field_nodes
                        .get(&field_node.pair.name)
                        .unwrap_or_else(|| {
                            panic!(
                                "humble spec references unknown type {:?} in embed",
                                field_node.pair.name
                            )
                        });
                    (*embedded_field_nodes).clone()
                } else {
                    vec![field_node.clone()]
                }
            })
            .flatten();

        let replacements = replacements.entry(struct_name).or_default();
        replacements.extend(new_field_nodes);
    }
    drop(all_structs_field_nodes);

    // apply replacements
    let struct_defs_mut = spec
        .iter_mut()
        .filter_map(|spec_item| match spec_item {
            SpecItem::StructDef(def) => Some(vec![(def.name.clone(), &mut def.fields.0)]),
            SpecItem::EnumDef(def) => Some({
                let enum_name = def.name.clone();
                def.variants
                    .iter_mut()
                    .filter_map(|v| {
                        let anon_struct_name = format!("{}.{}", enum_name, v.name);
                        v.variant_type
                            .struct_fields_mut()
                            .map(|sf| (anon_struct_name, &mut sf.0))
                    })
                    .collect::<Vec<_>>()
            }),
            _ => None,
        })
        .flatten();
    for (struct_name, struct_field_nodes_ptr) in struct_defs_mut {
        let (_, new_field_nodes) = match replacements.remove_entry(&struct_name) {
            Some(n) => n,
            None => continue,
        };

        *struct_field_nodes_ptr = new_field_nodes;
    }

    changed
}
