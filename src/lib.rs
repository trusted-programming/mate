#![feature(rustc_private)]
#![warn(unused_extern_crates)]

dylint_linting::dylint_library!();

extern crate rustc_lint;
extern crate rustc_session;

#[allow(clippy::no_mangle_with_rust_abi)]
#[no_mangle]
pub fn register_lints(sess: &rustc_session::Session, lint_store: &mut rustc_lint::LintStore) {
    // PHASE 1
    for_each::register_lints(sess, lint_store);
    // PHASE 2

    // PHASE 3

    // PREPARE RAYON FOR PHASE 4
    rayon_import::register_lints(sess, lint_store);
    // PHASE 4
}
