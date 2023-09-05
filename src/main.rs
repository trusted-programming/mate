#![feature(rustc_private)]

mod lints;

use lints::test_lint::{OddFunctionLineCount, WarnGenerics};
use rustc_tools::rustc_lint::LintStore;
use rustc_tools::with_lints;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.is_empty() {
        eprintln!("Missing file operand");
        return;
    }
    println!("Running lint example with arguments `{:?}`", args);
    with_lints(&args, vec![], |store: &mut LintStore| {
        store.register_early_pass(|| Box::new(WarnGenerics));
        store.register_late_pass(|_| Box::new(OddFunctionLineCount));
    })
    .unwrap();
}
