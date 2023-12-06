use rustc_errors::Applicability;
use rustc_hir::{Expr, ExprKind};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_middle::ty::TyKind;
use rustc_session::{declare_lint_pass, declare_tool_lint};
use rustc_span::Symbol;

declare_tool_lint! {
    pub lint::WARN_PAR_FOLD_SIMPLE,
    Warn,
    "suggest using parallel fold"
}

declare_lint_pass!(ParFoldSimple => [WARN_PAR_FOLD_SIMPLE]);
impl<'tcx> LateLintPass<'tcx> for ParFoldSimple {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'_>) {
        if let ExprKind::MethodCall(path, recv, args, _span) = &expr.kind
            && path.ident.name == Symbol::intern("fold") {
            assert_eq!(args.len(), 2);
            let id_expr = args[0];
            let op_expr = args[1];

            // Are we in the reduce case?
            // i.e. closure arg types are equal
            let ExprKind::Closure(_op_cls) = op_expr.kind else { return; };
            let type_chk = cx.tcx.typeck(expr.hir_id.owner.def_id);
            let op_ty = type_chk.node_type(op_expr.hir_id);
            let TyKind::Closure(_, io_tys) = op_ty.kind() else { return; };

            // TODO: Check how robust this is.
            let op_cls_arg_tys = io_tys.as_closure().sig().input(0).skip_binder().tuple_fields();
            assert_eq!(op_cls_arg_tys.len(), 2);
            if op_cls_arg_tys[0] != op_cls_arg_tys[1] {
                return;
            }

            let mut ir = crate::lints::phase4::IterRenaming::new();
            ir.traverse_iter_chain(recv);

            let src_map = cx.sess().source_map();
            let id_snip = src_map.span_to_snippet(id_expr.span).unwrap();

            let suggestion = "reduce".to_string();
            let suggestion2 = format!("|| {}", id_snip);
            ir.suggestions.extend_from_slice(&[(path.ident.span, suggestion), (id_expr.span, suggestion2)]);

            cx.struct_span_lint(WARN_PAR_FOLD_SIMPLE, expr.span, "sequential fold", |diag| {
                diag.multipart_suggestion(
                    "try using a parallel fold on the iterator",
                    ir.suggestions,
                    Applicability::MachineApplicable
                )
            });
        }
    }
}
