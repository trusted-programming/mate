#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_span;

use rustc_errors::Applicability;
use rustc_hir::intravisit::{walk_expr, Visitor};
use rustc_hir::{Expr, ExprKind};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_span::Symbol;
use utils::span_to_snippet_macro;

dylint_linting::declare_late_lint! {
    /// ### What it does
    ///
    /// ### Why is this bad?
    ///
    /// ### Known problems
    /// Remove if none.
    ///
    /// ### Example
    /// ```rust
    /// // example code where a warning is issued
    /// ```
    /// Use instead:
    /// ```rust
    /// // example code that does not raise a warning
    /// ```
    pub FOR_EACH,
    Warn,
    "it warns that a for loop can be replaced by a for_each"
}

#[derive(Default)]
struct Validator {
    is_invalid: bool,
    is_async: bool,
}

impl Visitor<'_> for Validator {
    fn visit_expr(&mut self, ex: &Expr) {
        match &ex.kind {
            ExprKind::Loop(_, _, _, _)
            | ExprKind::Closure(_)
            | ExprKind::Ret(_)
            | ExprKind::Break(_, _) => self.is_invalid = true,
            _ => walk_expr(self, ex),
        }
    }
}

impl<'tcx> LateLintPass<'tcx> for ForEach {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr) {
        // Match on for loop expressions
        if let ExprKind::Loop(block, label, loop_source, span) = &expr.kind {
            // Make sure we ignore cases that require a try_foreach
            let mut validator = Validator::default();
            validator.visit_block(block);

            if validator.is_invalid || validator.is_async {
                return;
            }

            let src_map = cx.sess().source_map();
            let iter_snip = span_to_snippet_macro(src_map, iter.span);
            let pat_snip = span_to_snippet_macro(src_map, loop_source.span);
            let block_snip = span_to_snippet_macro(src_map, block.span);

            // This could be handled better
            let block_snip = block_snip.replace("continue", "return");

            // Assumes into_iter can be applied, more checks can be done to see if iter or iter_mut
            // may apply.
            let suggestion = format!(
                "({}).into_iter().for_each(|{}| {});",
                iter_snip, pat_snip, block_snip
            );

            cx.struct_span_lint(
                FOR_EACH,
                expr.span,
                "use a for_each to enable iterator refinement",
                |diag| {
                    diag.span_suggestion(
                        expr.span,
                        "try using `for_each` on the iterator",
                        suggestion,
                        Applicability::MachineApplicable,
                    )
                },
            );
        }
    }
}

#[test]
fn ui() {
    dylint_testing::ui_test(
        env!("CARGO_PKG_NAME"),
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui"),
    );
}
