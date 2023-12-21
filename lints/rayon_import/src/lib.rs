#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_span;

use rustc_hir::{ItemKind, UseKind};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_span::Span;

dylint_linting::declare_late_lint! {
    /// ### What it does
    /// add rayon import to crate root
    /// ### Why is this bad?
    /// no rayon no par_iter traits
    /// ### Example
    /// ```rust
    /// fn main() {}
    /// ```
    /// Use instead:
    /// ```rust
    /// use rayon::prelude::*;
    ///
    /// fn main () {}
    /// ```
    pub RAYON_IMPORT,
    Warn,
    "check if rayon prelude is imported"
}

// TODO: add check for cargo.toml contains rayon

impl LateLintPass<'_> for RayonImport {
    fn check_crate(&mut self, cx: &LateContext<'_>) {
        let mut found_rayon_prelude = false;

        for item_id in cx.tcx.hir().items() {
            let item = cx.tcx.hir().item(item_id);
            if let ItemKind::Use(path, UseKind::Glob) = &item.kind {
                if path.segments.len() == 2 {
                    let first_segment = path.segments[0].ident.name.as_str();
                    let second_segment = path.segments[1].ident.name.as_str();

                    if first_segment == "rayon" && second_segment == "prelude" {
                        found_rayon_prelude = true;
                        break;
                    }
                }
            }
        }

        if !found_rayon_prelude {
            // Get the span for the start of the crate
            let root_module = cx.tcx.hir().root_module();
            let inject_use_span = root_module.spans.inject_use_span;

            // Suggest adding the import at the start of the crate
            let suggestion_span = Span::new(
                inject_use_span.lo(),
                inject_use_span.lo(),
                inject_use_span.ctxt(),
                None,
            );

            cx.struct_span_lint(
                RAYON_IMPORT,
                suggestion_span,
                "rayon::prelude::* is not imported",
                |diag| {
                    diag.span_suggestion(
                        suggestion_span,
                        "consider adding this import",
                        // FIXME: would be nice to find a better solution
                        "#[allow(unused_imports)]\nuse rayon::prelude::*;\n".to_string(),
                        rustc_errors::Applicability::MachineApplicable,
                    )
                },
            );
        }
    }
}

#[test]
fn ui() {
    dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
}
