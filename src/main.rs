#![feature(rustc_private)]
#![feature(let_chains)]

// We need to import them like this otherwise it doesn't work.
extern crate rustc_ast;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

mod cargo_cli;
mod lints;

use anyhow::Result;
use cargo::core::Workspace;
use cargo::ops::{self, CompileOptions};
use cargo::util::command_prelude::CompileMode;
use cargo::util::important_paths::find_root_manifest_for_wd;
use cargo::util::Config;
// use cargo_cli::Cli;
// use clap::Parser;
use lints::parallel::{phase1, phase2, phase3, phase4};
// use lints::rules::default_numeric_fallback::DefaultNumericFallback;
// use lints::rules::missing_debug_implementations::MissingDebugImplementations;
use lazy_static::lazy_static;
use rustc_lint::LintStore;
use rustc_tools::with_lints_and_error_output;
use rustfix::{apply_suggestions, get_suggestions_from_json, Filter, Suggestion};
use std::collections::HashSet;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::sync::Mutex;

lazy_static! {
    static ref BUFFER: Mutex<Vec<u8>> = Mutex::new(Vec::new());
}

fn main() -> Result<()> {
    // let Cli::Mate(args) = Cli::parse();

    let config = Config::default()?;
    let manifest_path = find_root_manifest_for_wd(config.cwd())?;
    let workspace = Workspace::new(&manifest_path, &config)?;
    let compile_opts = CompileOptions::new(&config, CompileMode::Check { test: false })?;
    ops::compile(&workspace, &compile_opts)?;

    let package = workspace.current()?;
    let targets = package.targets();

    let target_directory = PathBuf::from("target");
    let build_mode = "debug"; // Change to "release" if needed
    let deps_directory = target_directory.join(build_mode).join("deps");

    // let category = args.category;

    for target in targets {
        if target.is_bin() || target.is_lib() {
            let mut rustc_args = Vec::new();
            let src_path = target.src_path().path().map_or(String::new(), |path| {
                path.to_str().unwrap_or_default().to_string()
            });
            rustc_args.push(String::new());
            rustc_args.push(src_path);

            // List all files in the deps directory
            for entry in fs::read_dir(&deps_directory)? {
                let entry = entry?;
                let path = entry.path();

                // Check if the file is an rlib
                if let Some(extension) = path.extension() {
                    if extension == "rlib" {
                        let path_string = path.to_string_lossy().into_owned();

                        rustc_args.push("--extern".to_string());
                        rustc_args.push(path_string);
                    }
                }
            }

            // Redirect stdout
            let original_stdout = io::stdout();
            let mut lock = original_stdout.lock();
            let mut buffer = io::Cursor::new(Vec::new());
            io::set_print(Some(Box::new(buffer.clone())));

            let error_output = "JSON";

            // Run the function that includes the compiler execution
            with_lints_and_error_output(
                &rustc_args,
                vec![],
                error_output,
                |store: &mut LintStore| {
                    store.register_late_pass(|_| Box::new(phase2::simple::FilterSimple));
                    store.register_late_pass(|_| Box::new(phase3::fold::simple::FoldSimple));
                    store.register_late_pass(|_| Box::new(phase4::fold::simple::ParFoldSimple));
                    store.register_early_pass(|| Box::new(phase1::ForEach));
                },
            )?;

            // Read from buffer
            let mut output = String::new();
            buffer.set_position(0); // Reset cursor to the start
            buffer.read_to_string(&mut output)?;

            let only = HashSet::new();
            // Step 2: Parse JSON output to get suggestions
            let suggestions: Vec<Suggestion> =
                get_suggestions_from_json(&output, &only, Filter::MachineApplicableOnly)?;

            // Step 3: Apply suggestions
            apply_suggestions(&output, &suggestions)?;

            // for category in &category {
            //     let _ignore_errors = match category {
            //         Category::Parallel => {}
            //         Category::Rules => with_lints(&rustc_args, vec![], |store: &mut LintStore| {
            //             store.register_late_pass(|_| Box::new(DefaultNumericFallback));
            //             store.register_late_pass(|_| Box::new(MissingDebugImplementations));
            //         }),
            //         Category::Safety => {
            //             with_lints(&rustc_args, vec![], |_store: &mut LintStore| {})
            //         }
            //     };
            // }
        }
    }

    Ok(())
}
