#![feature(rustc_private)]

// We need to import them like this otherwise it doesn't work.
extern crate rustc_ast;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
mod lints;

use lints::rules::default_numeric_fallback::DefaultNumericFallback;
use lints::rules::missing_debug_implementations::MissingDebugImplementations;

use rustc_lint::LintStore;
use rustc_tools::with_lints;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.is_empty() {
        eprintln!("Missing file operand");
        return;
    }
    println!("Running lint example with arguments `{:?}`", args);
    with_lints(&args, vec![], |store: &mut LintStore| {
        store.register_late_pass(|_| Box::new(DefaultNumericFallback));
        store.register_late_pass(|_| Box::new(MissingDebugImplementations));
    })
    .unwrap();
}
