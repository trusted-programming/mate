use rustc_errors::Applicability;
use rustc_hir::{BinOpKind, Expr, ExprKind, StmtKind};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_session::{declare_lint_pass, declare_tool_lint};
use rustc_span::Symbol;

enum MonoidType {
    Mul,
    Add,
}

declare_tool_lint! {
    pub lint::WARN_FOLD_SIMPLE,
    Warn,
    "suggest using explicit fold"
}

declare_lint_pass!(FoldSimple => [WARN_FOLD_SIMPLE]);

impl<'tcx> LateLintPass<'tcx> for FoldSimple {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'_>) {
        // TODO: extract to helper function
        let hir_map = cx.tcx.hir();
        // See notes in phase2/simple.rs for limitations here.
        if let ExprKind::MethodCall(seg, recv, args, _span) = &expr.kind
            && seg.ident.name == Symbol::intern("for_each") {
            assert_eq!(args.len(), 1);

            let ExprKind::Closure(for_each_cls) = &args[0].kind else { return; };
            let cls_body = hir_map.body(for_each_cls.body);

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
                            match &block.stmts[0].kind {
                                StmtKind::Expr(expr) | StmtKind::Semi(expr) => expr,
                                _ => return,
                            }
                        }
                        _ => return,
                    }
                }
                _ => cls_body.value
            };

            // Match an assign operator expression
            if let ExprKind::AssignOp(op, lhs, rhs) = &top_expr.kind {

                // Is the operator additive or multiplicative.
                // This effects the choice of identity.
                let mon_ty = match op.node {
                    BinOpKind::Add | BinOpKind::Sub | BinOpKind::BitXor | BinOpKind::BitOr => MonoidType::Add,
                    BinOpKind::Mul | BinOpKind::BitAnd => MonoidType::Mul,
                    _ => return,
                };

                // Type check the accumulated parameter and assign the correct identity.
                let lhs_ty = cx.tcx.typeck(lhs.hir_id.owner.def_id).node_type(lhs.hir_id);
                let id_snip = if lhs_ty.is_integral() {
                    match mon_ty {
                        MonoidType::Add => "0",
                        MonoidType::Mul => "1",
                    }
                } else if lhs_ty.is_bool() {
                    match mon_ty {
                        MonoidType::Add => "false",
                        MonoidType::Mul => "true",
                    }
                } else {
                    return;
                };

                let src_map = cx.sess().source_map();
                let recv_snip = src_map.span_to_snippet(recv.span).unwrap();
                assert!(cls_body.params.len() > 0);
                let pat_span = cls_body.params[0].span.to(cls_body.params[cls_body.params.len() - 1].span);
                let pat_snip = src_map.span_to_snippet(pat_span).unwrap();
                let rhs_snip = src_map.span_to_snippet(rhs.span).unwrap();
                let op_snip = src_map.span_to_snippet(op.span).unwrap();
                let lhs_snip = src_map.span_to_snippet(lhs.span).unwrap();
                let suggestion = format!("{lhs_snip} {op_snip} {recv_snip}.map(|{pat_snip}| {rhs_snip}).fold({id_snip}, |mut {lhs_snip}, v| {{ {lhs_snip} {op_snip} v; {lhs_snip} }})");

                cx.struct_span_lint(
                    WARN_FOLD_SIMPLE,
                    expr.span,
                    "implicit fold",
                    |diag| {
                        diag.span_suggestion(
                            expr.span,
                            "try using `fold` instead",
                            suggestion,
                            Applicability::MachineApplicable)
                    },
                );
            }
        }
    }
}
