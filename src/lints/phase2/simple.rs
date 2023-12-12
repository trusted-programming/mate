use rustc_errors::Applicability;
use rustc_hir::{Expr, ExprKind, StmtKind};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_session::{declare_lint_pass, declare_tool_lint};
use rustc_span::symbol::Symbol;

declare_tool_lint! {
    pub lint::WARN_FILTER_SIMPLE,
    Warn,
    "suggest using explicit filter iterator"
}

declare_lint_pass!(FilterSimple => [WARN_FILTER_SIMPLE]);
impl<'tcx> LateLintPass<'tcx> for FilterSimple {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'_>) {
        // TODO: extract to helper function
        let hir_map = cx.tcx.hir();
        if let ExprKind::MethodCall(seg, _recv, args, span) = &expr.kind
            && seg.ident.name == Symbol::intern("for_each") {
            // A trait check is required here to make sure we are calling
            // the Iterator::for_each method instead of some other for_each method.
            // For now we just check argument count.
            assert_eq!(args.len(), 1);

            // Extract the for_each closure.
            let ExprKind::Closure(for_each_cls) = &args[0].kind else { return; };
            let cls_body = hir_map.body(for_each_cls.body);

            // Currently this will only create a lint if there's a single top-level if statement.
            // It is entirely reasonable to extract local declarations here as well.
            // This would enable more cases to be matched on.
            let top_expr = match cls_body.value.kind {
                ExprKind::Block(block, _) => {
                    match block.stmts.len() {
                        0 => {
                            if block.expr.is_none() {
                                return;
                            }
                            block.expr.unwrap()
                        }
                        // TODO: Does this always return?
                        1 => {
                            if block.expr.is_some() {
                                return;
                            }
                            let StmtKind::Expr(expr) = &block.stmts[0].kind else { return; };
                            expr
                        }
                        _ => return,
                    }
                }
                _ => cls_body.value
            };

            // Check for a single branched if.
            if let ExprKind::If(cond, then, None) = &top_expr.kind {
                let src_map = cx.sess().source_map();
                let pat_snip =
                    if !cls_body.params.is_empty() {
                        let fst_span = cls_body.params[0].span;
                        let lst_span = cls_body.params[cls_body.params.len() - 1].span;
                        src_map.span_to_snippet(fst_span.to(lst_span)).unwrap()
                    } else { String::new() };
                let cond_snip = src_map.span_to_snippet(cond.span).unwrap();
                let then_snip = src_map.span_to_snippet(then.span).unwrap();
                let suggestion = format!("filter(|&{pat_snip}| {cond_snip}).for_each(|{pat_snip}| {then_snip})");
                cx.struct_span_lint(
                    WARN_FILTER_SIMPLE,
                    *span,
                    "implicit filter inside `for_each`",
                    |diag| {
                        diag.span_suggestion(
                            *span,
                            "try lifting the filter iterator",
                            suggestion,
                            Applicability::MachineApplicable)
                    });
            }
        }
    }
}
