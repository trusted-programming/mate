use clippy_utils::sym;
use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_session::{declare_lint_pass, declare_tool_lint};

declare_tool_lint! {
    /// ### What it does
    ///
    /// ### Why is this bad?
    ///
    /// ### Known problems
    ///
    /// ### Example

    pub lint::ITER,
    Warn,
    "found a .iter method might be parallelizable.",
    report_in_external_macro: false
}

declare_lint_pass!(IterLint => [ITER]);
impl<'tcx> LateLintPass<'tcx> for IterLint {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'_>) {
        // Check our expr is calling a method with pattern matching

        if let hir::ExprKind::MethodCall(path, _,  _,_) = &expr.kind
            // Check if the name of this method is `our_fancy_method`
            && path.ident.name == sym!(iter)
            // We can check the type of the self argument whenever necessary.
            // (It's necessary if we want to check that method is specifically belonging to a specific trait,
            // for example, a `map` method could belong to user-defined trait instead of to `Iterator`)
            // See the next section for more information.
        {
            cx.struct_span_lint(
                ITER,
                expr.span,
                "found a .iter method might be parallelizable",
                |diag| {
                    diag.span_suggestion(expr.span, "consider substituting with", ".par_iter()", Applicability::MachineApplicable)
                },
            );
        }
    }
}
