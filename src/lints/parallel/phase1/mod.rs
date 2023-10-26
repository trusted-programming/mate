use rustc_ast::ast::{Expr, ExprKind};
use rustc_errors::Applicability;
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_session::{declare_lint_pass, declare_tool_lint};

declare_tool_lint! {
    /// ### What it does
    ///
    /// ### Why is this bad?
    ///
    /// ### Known problems
    ///
    /// ### Example

    pub lint::WARN_FOREACH,
    Warn,
    "use a for_each to enable iterator refinement."
}

declare_lint_pass!(ForEach => [WARN_FOREACH]);
impl EarlyLintPass for ForEach {
    fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &Expr) {
        // Match on for loop expressions
        if let ExprKind::ForLoop(pat, iter, block, _) = &expr.kind {
            // TODO: Check the iter and see if it requires an into_iter or not.
            // TODO: Check the block and replace instances of `continue` with `return`.
            
            let src_map = cx.sess().source_map();
            let iter_snip = src_map.span_to_snippet(iter.span).unwrap();
            let pat_snip = src_map.span_to_snippet(pat.span).unwrap();
            let block_snip = src_map.span_to_snippet(block.span).unwrap();
            let suggestion = format!("{}.for_each(|{}| {});", iter_snip, pat_snip, block_snip);

            cx.struct_span_lint(
                WARN_FOREACH, 
                expr.span,
                "use a for_each to enable iterator refinement",
                |diag| {
                    diag.span_suggestion(expr.span, "try using `for_each` on the iterator", suggestion, Applicability::MachineApplicable)
            });
        }
    }
}
