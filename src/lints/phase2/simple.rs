use rustc_errors::Applicability;
use rustc_hir::{Expr, ExprKind};
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

            // Collect a set of local definitions, the expression we wish to analyze and
            // the statements following it
            let (pat_expr, local_defs_span, body_span) =
                match crate::lints::get_pat_expr_and_spans(&cls_body.value) {
                    Ok(v) => v,
                    _ => return,
                };

            // We should only have one statement left
            if body_span.is_some() { return }

            // Check for a single branched if.
            let ExprKind::If(cond, then, None) = &pat_expr.kind else {
                return;
            };
            if let ExprKind::Let(_) = cond.kind {
                return;
            }

            let src_map = cx.sess().source_map();
            let ExprKind::Block(then_block, _) = then.kind else {
                return;
            };
            let then_snip = if !then_block.stmts.is_empty() {
                let fst_span = then_block.stmts[0].span;
                let lst_span = match then_block.expr {
                    None => then_block.stmts[then_block.stmts.len() - 1].span,
                    Some(e) => e.span,
                };
                src_map.span_to_snippet(fst_span.to(lst_span)).unwrap()
            } else {
                then_block
                    .expr
                    .map_or(String::new(), |e| src_map.span_to_snippet(e.span).unwrap())
            };

            let local_defs_snip =
                local_defs_span.map_or(String::new(), |sp| src_map.span_to_snippet(sp).unwrap());

            let pat_snip = if !cls_body.params.is_empty() {
                let fst_span = cls_body.params[0].span;
                let lst_span = cls_body.params[cls_body.params.len() - 1].span;
                src_map.span_to_snippet(fst_span.to(lst_span)).unwrap()
            } else {
                String::new()
            };

            let cond_snip = src_map.span_to_snippet(cond.span).unwrap();
            let suggestion = format!("filter(|{pat_snip}| {{ let {pat_snip} = ({pat_snip}).as_ref(); {local_defs_snip} {cond_snip} }}).for_each(|{pat_snip}| {{ {local_defs_snip} {then_snip} }})");
            cx.struct_span_lint(
                WARN_FILTER_SIMPLE,
                *span,
                "implicit filter inside `for_each`",
                |diag| {
                    diag.span_suggestion(
                        *span,
                        "try lifting the filter iterator",
                        suggestion,
                        Applicability::MachineApplicable,
                    )
                },
            );
        }
    }
}
