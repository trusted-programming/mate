#![feature(rustc_private)]
#![feature(let_chains)]

// We need to import them like this otherwise it doesn't work.
extern crate rustc_ast;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_lint_defs;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

mod cargo_cli;
mod lints;

use cargo_cli::{Category, Cli};
use clap::Parser;
use lints::parallel::{phase1, phase2, phase3, phase4};
use lints::rules::default_numeric_fallback::DefaultNumericFallback;
use lints::rules::missing_debug_implementations::MissingDebugImplementations;
use lints::safety::permission::RawPointerPermission;
use lints::safety::translation::RawPointerTranslation;
use rustc_lint::LintStore;
use rustc_tools::with_lints;
use std::fs;
use std::path::PathBuf;
use toml::{self, Table};

fn main() {
    let Cli::Mate(args) = Cli::parse();

    let category = args.category;
    let mut entrypoints_path = args.entrypoints_path.unwrap_or_else(|| {
        find_entrypoints().unwrap_or_else(|| {
            println!("no valid entrypoint found!");
            std::process::exit(1);
        })
    });
    entrypoints_path.insert(0, String::new());

    for category in category {
        let _ignore_errors = match category {
            Category::Parallel => with_lints(&entrypoints_path, vec![], |store: &mut LintStore| {
                store.register_late_pass(|_| Box::new(phase2::simple::FilterSimple));
                store.register_late_pass(|_| Box::new(phase3::fold::simple::FoldSimple));
                store.register_late_pass(|_| Box::new(phase4::fold::simple::ParFoldSimple));
                store.register_early_pass(|| Box::new(phase1::ForEach));
            }),
            Category::Rules => with_lints(&entrypoints_path, vec![], |store: &mut LintStore| {
                store.register_late_pass(|_| Box::new(DefaultNumericFallback));
                store.register_late_pass(|_| Box::new(MissingDebugImplementations));
            }),
            Category::Safety => with_lints(&entrypoints_path, vec![], |store: &mut LintStore| {
                store.register_late_pass(|_| {
                    Box::new(RawPointerPermission::new("analysis_results"))
                });
                store.register_late_pass(|_| {
                    Box::new(RawPointerTranslation::new("analysis_results"))
                });
            }),
        };
    }
}

// Function to add paths from a toml array to entrypoints
fn add_paths_from_array(array: &[toml::Value], entrypoints: &mut Vec<String>) {
    for item in array.iter().filter_map(|val| val.as_table()) {
        if let Some(path) = item.get("path").and_then(|val| val.as_str()) {
            entrypoints.push(path.to_string());
        }
    }
}

fn find_entrypoints() -> Option<Vec<String>> {
    let mut entrypoints = Vec::new();
    let content = fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");
    let cargo_toml: Table = content.parse().expect("Failed to parse Cargo.toml");

    // Function to add paths from a toml array to entrypoints defined elsewhere...
    // Assuming add_paths_from_array is defined elsewhere and correctly adds paths

    // Add library paths
    if let Some(toml::Value::Array(libs)) = cargo_toml.get("lib") {
        add_paths_from_array(libs, &mut entrypoints);
    }

    // Add binary paths
    if let Some(toml::Value::Array(bins)) = cargo_toml.get("bin") {
        add_paths_from_array(bins, &mut entrypoints);
    }

    // Add default library and binary paths if they exist
    for default_path in &["src/lib.rs", "src/main.rs"] {
        let path = PathBuf::from(*default_path);
        if path.exists() {
            entrypoints.push(path.to_string_lossy().into_owned());
        }
    }
    if entrypoints.is_empty() {
        None
    } else {
        Some(entrypoints)
    }
}
