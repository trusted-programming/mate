use rustc_errors::Applicability;
use rustc_hir::{Expr, ExprKind, StmtKind};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_session::{declare_lint, declare_lint_pass};
use rustc_span::symbol::Symbol;
use utils::span_to_snippet_macro;

declare_lint! {
    pub FILTER_SIMPLE_FLIPPED,
    Warn,
    "suggest using explicit filter iterator"
}

declare_lint_pass!(FilterSimpleFlipped => [FILTER_SIMPLE_FLIPPED]);

impl<'tcx> LateLintPass<'tcx> for FilterSimpleFlipped {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'_>) {
        let hir_map = cx.tcx.hir();
        if let ExprKind::MethodCall(seg, _recv, args, span) = &expr.kind
            && seg.ident.name == Symbol::intern("for_each")
        {
            // A trait check is required here to make sure we are calling
            // the Iterator::for_each method instead of some other for_each method.
            // For now we just check argument count.
            assert_eq!(args.len(), 1);

            // Extract the for_each closure.
            let ExprKind::Closure(for_each_cls) = &args[0].kind else {
                return;
            };
            let cls_body = hir_map.body(for_each_cls.body);

            // Collect a set of local definitions, the expression we wish to analyze and
            // the statements following it
            let (pat_expr, local_defs_span, body_span) =
                match utils::get_pat_expr_and_spans(cls_body.value) {
                    Ok(v) => v,
                    _ => return,
                };

            // Check for an if with early return in the success branch.
            let ExprKind::If(cond, fail, then) = &pat_expr.kind else {
                return;
            };
            let ExprKind::Block(fail_block, _) = fail.kind else {
                return;
            };
            if fail_block.stmts.is_empty() {
                return;
            }
            let StmtKind::Semi(fail_e) = fail_block.stmts[0].kind else {
                return;
            };
            let ExprKind::Ret(_) = fail_e.kind else {
                return;
            };

            let src_map = cx.sess().source_map();
            let body_snip =
                body_span.map_or(String::new(), |sp| span_to_snippet_macro(src_map, sp));

            let then_snip = {
                match then {
                    Some(then) => {
                        let ExprKind::Block(then_block, _) = then.kind else {
                            return;
                        };
                        if !then_block.stmts.is_empty() {
                            let fst_span = then_block.stmts[0].span;
                            let lst_span = match then_block.expr {
                                None => then_block.stmts[then_block.stmts.len() - 1].span,
                                Some(e) => e.span,
                            };
                            span_to_snippet_macro(src_map, fst_span.to(lst_span))
                        } else {
                            then_block
                                .expr
                                .map_or(String::new(), |e| span_to_snippet_macro(src_map, e.span))
                        }
                    }
                    None => String::new(),
                }
            };

            let local_defs_snip =
                local_defs_span.map_or(String::new(), |sp| span_to_snippet_macro(src_map, sp));

            let pat_snip = if !cls_body.params.is_empty() {
                let fst_span = cls_body.params[0].span;
                let lst_span = cls_body.params[cls_body.params.len() - 1].span;
                span_to_snippet_macro(src_map, fst_span.to(lst_span))
            } else {
                String::new()
            };

            let cond_snip = span_to_snippet_macro(src_map, cond.span);
            let suggestion = format!("filter(|{pat_snip}| {{ {local_defs_snip} !({cond_snip}) }}).for_each(|{pat_snip}| {{ {local_defs_snip} {then_snip} {body_snip} }})");
            cx.struct_span_lint(
                FILTER_SIMPLE_FLIPPED,
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
