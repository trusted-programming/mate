#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;
extern crate rustc_errors;

use rustc_ast::ast::{Expr, ExprKind};
use rustc_ast::visit::{walk_expr, Visitor};
use rustc_errors::Applicability;
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};

dylint_linting::declare_early_lint! {
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

struct Validator {
    has_early_ret: bool,
    is_async: bool,
}

impl Visitor<'_> for Validator {
    fn visit_expr(&mut self, ex: &Expr) {
        match &ex.kind {
            ExprKind::Try(_) | ExprKind::Ret(_) | ExprKind::Break(_, _) => {
                self.has_early_ret = true
            }
            ExprKind::Await(e, _) => {
                self.is_async = true;
                self.visit_expr(e)
            }
            _ => walk_expr(self, ex),
        }
    }
}

impl EarlyLintPass for ForEach {
    fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &Expr) {
        // Match on for loop expressions
        if let ExprKind::ForLoop(pat, iter, block, _) = &expr.kind {
            // Make sure we ignore cases that require a try_foreach
            let mut cft = Validator {
                has_early_ret: false,
                is_async: false,
            };
            cft.visit_block(block);
            cft.visit_expr(iter);
            if cft.has_early_ret || cft.is_async {
                return;
            }

            let src_map = cx.sess().source_map();
            let iter_snip = src_map.span_to_snippet(iter.span).unwrap();
            let pat_snip = src_map.span_to_snippet(pat.span).unwrap();
            let block_snip = src_map.span_to_snippet(block.span).unwrap();

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
