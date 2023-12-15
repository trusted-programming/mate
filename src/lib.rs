#![feature(rustc_private)]
#![recursion_limit = "256"]
#![warn(unused_extern_crates)]
#![feature(let_chains)]

#[cfg(not(feature = "rlib"))]
dylint_linting::dylint_library!();

extern crate rustc_ast;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

mod lints;

#[allow(clippy::no_mangle_with_rust_abi)]
#[cfg_attr(not(feature = "rlib"), no_mangle)]
pub fn register_lints(_sess: &rustc_session::Session, lint_store: &mut rustc_lint::LintStore) {
    lint_store.register_early_pass(|| Box::new(lints::phase1::ForEach));
    lint_store.register_late_pass(|_| Box::new(lints::phase2::simple::FilterSimple));
    lint_store.register_late_pass(|_| Box::new(lints::phase2::simple_flipped::FilterSimpleFlipped));
    lint_store.register_late_pass(|_| Box::new(lints::phase3::fold::simple::FoldSimple));
    lint_store.register_late_pass(|_| Box::new(lints::rayon::RayonImport));
    lint_store.register_late_pass(|_| Box::new(lints::phase4::fold::simple::ParFoldSimple));
}

#[test]
fn ui() {
    dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
}
