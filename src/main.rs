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
use cargo_cli::{Category, Cli};
use clap::Parser;
use lints::parallel::{phase1, phase2, phase3, phase4};
use lints::rules::default_numeric_fallback::DefaultNumericFallback;
use lints::rules::missing_debug_implementations::MissingDebugImplementations;
use rustc_lint::LintStore;
use rustc_tools::with_lints;
use std::env;

fn main() -> Result<()> {
    let Cli::Mate(args) = Cli::parse();

    let config = Config::default()?;
    let manifest_path = find_root_manifest_for_wd(config.cwd())?;
    let workspace = Workspace::new(&manifest_path, &config)?;
    let compile_opts = CompileOptions::new(&config, CompileMode::Check { test: false })?;
    ops::compile(&workspace, &compile_opts)?;

    // Determine the build profile
    let profile = if let Ok(profile) = env::var("PROFILE") {
        profile
    } else {
        // Default to debug if PROFILE environment variable is not set
        "debug".to_string()
    };
    let deps_directory = workspace.target_dir().join(&profile).join("deps");

    let rustc_dependency_dir = format!("-L {}", deps_directory.display());

    let package = workspace.current()?;
    let targets = package.targets();

    let category = args.category;

    for target in targets {
        let mut rustc_args = Vec::new();
        rustc_args.push(String::new());
        let src_path = target.src_path().path().map_or(String::new(), |path| {
            path.to_str().unwrap_or_default().to_string()
        });

        rustc_args.push(src_path);
        rustc_args.push(rustc_dependency_dir.clone());

        if target.is_bin() || target.is_lib() {
            for category in &category {
                let _ignore_errors = match category {
                    Category::Parallel => {
                        with_lints(&rustc_args, vec![], |store: &mut LintStore| {
                            store.register_late_pass(|_| Box::new(phase2::simple::FilterSimple));
                            store
                                .register_late_pass(|_| Box::new(phase3::fold::simple::FoldSimple));
                            store.register_late_pass(|_| {
                                Box::new(phase4::fold::simple::ParFoldSimple)
                            });
                            store.register_early_pass(|| Box::new(phase1::ForEach));
                        })
                    }
                    Category::Rules => with_lints(&rustc_args, vec![], |store: &mut LintStore| {
                        store.register_late_pass(|_| Box::new(DefaultNumericFallback));
                        store.register_late_pass(|_| Box::new(MissingDebugImplementations));
                    }),
                    Category::Safety => {
                        with_lints(&rustc_args, vec![], |_store: &mut LintStore| {})
                    }
                };
            }
        }
    }

    Ok(())
}
