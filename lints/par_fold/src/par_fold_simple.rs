use rustc_errors::Applicability;
use rustc_hir::{Expr, ExprKind, HirId};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_middle::ty::TyKind;
use rustc_session::{declare_lint, declare_lint_pass};
use rustc_span::{Span, Symbol};
use utils::span_to_snippet_macro;

declare_lint! {
    pub WARN_PAR_FOLD_SIMPLE,
    Warn,
    "suggest using parallel fold"
}

declare_lint_pass!(ParFoldSimple => [WARN_PAR_FOLD_SIMPLE]);
impl<'tcx> LateLintPass<'tcx> for ParFoldSimple {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'_>) {
        if let ExprKind::MethodCall(path, recv, args, _span) = &expr.kind
            && path.ident.name == Symbol::intern("fold")
        {
            assert_eq!(args.len(), 2);
            let id_expr = args[0];
            let op_expr = args[1];

            // Are we in the reduce case?
            // i.e. closure arg types are equal
            let ExprKind::Closure(_op_cls) = op_expr.kind else {
                return;
            };
            let type_chk = cx.tcx.typeck(expr.hir_id.owner.def_id);
            let op_ty = type_chk.node_type(op_expr.hir_id);
            let TyKind::Closure(_, io_tys) = op_ty.kind() else {
                return;
            };

            // TODO: Check how robust this is.
            let op_cls_arg_tys = io_tys
                .as_closure()
                .sig()
                .input(0)
                .skip_binder()
                .tuple_fields();
            assert_eq!(op_cls_arg_tys.len(), 2);
            if op_cls_arg_tys[0] != op_cls_arg_tys[1] {
                return;
            }

            let mut ir = IterRenaming::new();
            ir.traverse_iter_chain(recv);

            let src_map = cx.sess().source_map();
            let id_snip = span_to_snippet_macro(src_map, id_expr.span);

            let suggestion = "reduce".to_string();
            let suggestion2 = format!("|| {}", id_snip);
            ir.suggestions
                .extend_from_slice(&[(path.ident.span, suggestion), (id_expr.span, suggestion2)]);

            cx.struct_span_lint(WARN_PAR_FOLD_SIMPLE, expr.span, "sequential fold", |diag| {
                diag.multipart_suggestion(
                    "try using a parallel fold on the iterator",
                    ir.suggestions,
                    Applicability::MachineApplicable,
                );
            });
        }
    }
}

// Traverse an iterator chain and rename all occurrences
// of sequential iterator calls to parallel ones.
struct IterRenaming {
    suggestions: Vec<(Span, String)>,
    seen: Vec<HirId>,
}

impl IterRenaming {
    fn new() -> Self {
        IterRenaming {
            suggestions: vec![],
            seen: vec![],
        }
    }

    fn traverse_iter_chain(&mut self, expr: &Expr) {
        if self.seen.contains(&expr.hir_id) {
            return;
        }
        self.seen.push(expr.hir_id);

        if let ExprKind::MethodCall(path, recv, args, _span) = &expr.kind {
            // TODO: Optimize this.
            let seq_names = vec![
                Symbol::intern("iter"),
                Symbol::intern("iter_mut"),
                Symbol::intern("into_iter"),
            ];
            let par_names = vec!["par_iter", "par_iter_mut", "into_par_iter"];
            for (sm, pm) in seq_names.into_iter().zip(par_names.into_iter()) {
                if path.ident.name == sm {
                    self.suggestions.push((path.ident.span, pm.to_string()));
                    break;
                }
            }
            self.traverse_iter_chain(recv);
            args.iter().for_each(|e| self.traverse_iter_chain(e));
        }
    }
}
