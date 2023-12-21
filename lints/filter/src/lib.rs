#![feature(rustc_private)]
#![warn(unused_extern_crates)]
#![feature(let_chains)]

#[cfg(not(feature = "rlib"))]
dylint_linting::dylint_library!();

extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_session;
extern crate rustc_span;

mod simple;
mod simple_flipped;

#[allow(clippy::no_mangle_with_rust_abi)]
#[cfg_attr(not(feature = "rlib"), no_mangle)]
pub fn register_lints(_sess: &rustc_session::Session, lint_store: &mut rustc_lint::LintStore) {
    lint_store.register_late_pass(|_| Box::new(simple::FilterSimple));
    lint_store.register_late_pass(|_| Box::new(simple_flipped::FilterSimpleFlipped));
}

#[test]
fn ui() {
    dylint_testing::ui_test(
        env!("CARGO_PKG_NAME"),
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui"),
    );
}
