#![feature(rustc_private)]
#![warn(unused_extern_crates)]
#![feature(let_chains)]
#![feature(iter_intersperse)]

#[cfg(not(feature = "rlib"))]
dylint_linting::dylint_library!();

extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_session;
extern crate rustc_span;

use rustc_errors::Applicability;
use rustc_hir::{Expr, ExprKind};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_session::{declare_lint, declare_lint_pass};
use rustc_span::{sym, Symbol};
use utils::span_to_snippet_macro;

mod builder;

builder::map_collection!(MapVec, MAP_VEC, Vec, "push", "Vec");
builder::map_collection!(MapHashMap, MAP_HASHMAP, HashMap, "insert", "HashMap");
builder::map_collection!(MapHashSet, MAP_HASHSET, HashSet, "insert", "HashSet");
builder::map_collection!(MapBTreeMap, MAP_BTREEMAP, BTreeMap, "insert", "BTreeMap");
builder::map_collection!(MapBTreeSet, MAP_BTREESET, BTreeSet, "insert", "BTreeSet");

#[allow(clippy::no_mangle_with_rust_abi)]
#[cfg_attr(not(feature = "rlib"), no_mangle)]

pub fn register_lints(_sess: &rustc_session::Session, lint_store: &mut rustc_lint::LintStore) {
    lint_store.register_late_pass(|_| Box::new(MapVec));
    lint_store.register_late_pass(|_| Box::new(MapHashMap));
    lint_store.register_late_pass(|_| Box::new(MapHashSet));
    lint_store.register_late_pass(|_| Box::new(MapBTreeMap));
    lint_store.register_late_pass(|_| Box::new(MapBTreeSet));
}

#[test]
fn ui() {
    dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
}
