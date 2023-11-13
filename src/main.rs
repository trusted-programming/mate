#![feature(rustc_private)]
#![feature(let_chains)]

// We need to import them like this otherwise it doesn't work.
extern crate clap;
extern crate rustc_ast;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

mod cargo_cli;
mod lints;

use cargo_cli::{CargoCli, Category};
use clap::Parser;
use lints::parallel::{phase1, phase2, phase3, phase4};
use lints::rules::default_numeric_fallback::DefaultNumericFallback;
use lints::rules::missing_debug_implementations::MissingDebugImplementations;
use rustc_lint::LintStore;
use rustc_tools::with_lints;

fn main() {
    let CargoCli::Mate(args) = CargoCli::parse();

    let manifest_arg = args
        .manifest_path
        .as_ref()
        .map(|path| path.to_str().unwrap_or_default())
        .unwrap_or_default();
    let category = args.category;
    let args_for_linting = vec!["--manifest-path".to_string(), manifest_arg.to_string()];

    for category in category {
        match category {
            Category::Parallel => with_lints(&args_for_linting, vec![], |store: &mut LintStore| {
                store.register_late_pass(|_| Box::new(phase2::simple::FilterSimple));
                store.register_late_pass(|_| Box::new(phase3::fold::simple::FoldSimple));
                store.register_late_pass(|_| Box::new(phase4::fold::simple::ParFoldSimple));
                store.register_early_pass(|| Box::new(phase1::ForEach));
            })
            .unwrap(),
            Category::Rules => with_lints(&args_for_linting, vec![], |store: &mut LintStore| {
                store.register_late_pass(|_| Box::new(DefaultNumericFallback));
                store.register_late_pass(|_| Box::new(MissingDebugImplementations));
            })
            .unwrap(),
            Category::Safety => {
                with_lints(&args_for_linting, vec![], |_store: &mut LintStore| {}).unwrap()
            }
        }
    }
}
