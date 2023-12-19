use rustc_ast::ast::{Expr, ExprKind};
use rustc_ast::visit::{walk_expr, Visitor};
use rustc_errors::Applicability;
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_session::{declare_lint, declare_lint_pass};

declare_lint! {
    /// ### What it does
    ///
    /// ### Why is this bad?
    ///
    /// ### Known problems
    ///
    /// ### Example

    pub WARN_FOREACH,
    Warn,
    "use a for_each to enable iterator refinement."
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

declare_lint_pass!(ForEach => [WARN_FOREACH]);

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
                WARN_FOREACH,
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
