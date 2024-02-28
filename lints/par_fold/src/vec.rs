use rustc_errors::Applicability;
use rustc_hir::{Expr, ExprKind, StmtKind};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_session::{declare_lint, declare_lint_pass};
use rustc_span::{sym, Symbol};
use utils::span_to_snippet_macro;

declare_lint! {
    pub WARN_PAR_FOLD_VEC,
    Warn,
    "suggest using parallel fold"
}

declare_lint_pass!(ParFoldVec => [WARN_PAR_FOLD_VEC]);
impl<'tcx> LateLintPass<'tcx> for ParFoldVec {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'_>) {
        if let ExprKind::MethodCall(path, recv, args, _span) = &expr.kind
            && path.ident.name == Symbol::intern("fold")
        {
            assert_eq!(args.len(), 2);
            let id_expr = args[0];
            let op_expr = args[1];

            // Check the penultimate statement of the fold for a `c.push(v)`
            // Quite a specific target, can we be more general?
            let ExprKind::Closure(op_cls) = op_expr.kind else {
                return;
            };
            let hir_map = cx.tcx.hir();
            let cls_body = hir_map.body(op_cls.body);

            let Ok(StmtKind::Semi(fold_op)) =
                if let ExprKind::Block(block, _) = &cls_body.value.kind {
                    match block.stmts.len() {
                        0 => Err(()),
                        l => {
                            if l == 1 && block.expr.is_none() {
                                Err(())
                            } else {
                                Ok(&block.stmts[l - 1])
                            }
                        }
                    }
                } else {
                    Err(())
                }
                .map(|s| s.kind)
            else {
                return;
            };

            let ExprKind::MethodCall(path, _, _, _) = fold_op.kind else {
                return;
            };
            if path.ident.name != Symbol::intern("push") {
                return;
            }

            // Check that this method is on a vec
            let base_ty = cx
                .tcx
                .typeck(expr.hir_id.owner.def_id)
                .node_type(id_expr.hir_id);
            let Some(adt) = base_ty.ty_adt_def() else {
                return;
            };
            if !cx.tcx.is_diagnostic_item(sym::Vec, adt.did()) {
                return;
            }

            // Assume that if we make it here, we can apply the pattern.
            let src_map = cx.sess().source_map();
            let cls_snip = span_to_snippet_macro(src_map, op_expr.span);
            let recv_snip = span_to_snippet_macro(src_map, recv.span);
            let id_snip = span_to_snippet_macro(src_map, id_expr.span);

            let fold_snip = format!("fold(|| Vec::new(), {cls_snip})");
            let reduce_snip = "reduce(|| Vec::new(), |mut a, b| { a.extend(b); a })";
            let mut extend_snip =
                format!("{{ {id_snip}.extend({recv_snip}.{fold_snip}.{reduce_snip}); {id_snip} }}");
            extend_snip = extend_snip.replace(".iter()", ".par_iter()");
            extend_snip = extend_snip.replace(".iter_mut()", ".par_iter_mut()");
            extend_snip = extend_snip.replace(".into_iter()", ".into_par_iter()");

            cx.span_lint(WARN_PAR_FOLD_VEC, expr.span, "sequential fold", |diag| {
                diag.span_suggestion(
                    expr.span,
                    "try using a parallel fold on the iterator",
                    extend_snip,
                    Applicability::MachineApplicable,
                );
            });
        }
    }
}
