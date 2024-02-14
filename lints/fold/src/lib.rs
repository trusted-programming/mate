#![feature(rustc_private)]
#![warn(unused_extern_crates)]
#![feature(let_chains)]

#[cfg(not(feature = "rlib"))]
dylint_linting::dylint_library!();

#[cfg(feature = "rlib")]
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_session;
extern crate rustc_span;

mod hashmap;
mod simple;
mod vec;

#[allow(clippy::no_mangle_with_rust_abi)]
#[cfg_attr(not(feature = "rlib"), no_mangle)]

pub fn register_lints(_sess: &rustc_session::Session, lint_store: &mut rustc_lint::LintStore) {
    lint_store.register_late_pass(|_| Box::new(simple::FoldSimple));
    lint_store.register_late_pass(|_| Box::new(vec::FoldVec));
    lint_store.register_late_pass(|_| Box::new(hashmap::FoldHashmap));
}

#[test]
fn ui() {
    dylint_testing::ui_test(
        env!("CARGO_PKG_NAME"),
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui"),
    );
}
