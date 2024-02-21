use rustc_errors::Applicability;
use rustc_hir::{BinOpKind, Expr, ExprKind};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_session::{declare_lint, declare_lint_pass};
use rustc_span::Symbol;
use utils::span_to_snippet_macro;

declare_lint! {
    /// ### What it does
    ///
    /// ### Why is this bad?
    ///
    /// ### Known problems
    /// Remove if none.
    ///
    /// ### Example
    /// ```rust
    /// // example code where a warning is issued
    /// ```
    /// Use instead:
    /// ```rust
    /// // example code that does not raise a warning
    /// ```
    pub FOLD_SIMPLE,
    Warn,
    "suggest using explicit fold"
}
declare_lint_pass!(FoldSimple => [FOLD_SIMPLE]);

enum MonoidType {
    Mul,
    Add,
}

impl<'tcx> LateLintPass<'tcx> for FoldSimple {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'_>) {
        // TODO: extract to helper function
        let hir_map = cx.tcx.hir();
        // See notes in phase2/simple.rs for limitations here.
        if let ExprKind::MethodCall(seg, recv, args, _span) = &expr.kind
            && seg.ident.name == Symbol::intern("for_each")
        {
            assert_eq!(args.len(), 1);

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

            // We should only have one statement left
            if body_span.is_some() {
                return;
            }

            // Match an assign operator expression
            let ExprKind::AssignOp(op, lhs, rhs) = &pat_expr.kind else {
                return;
            };

            // Is the operator additive or multiplicative.
            // This effects the choice of identity.
            let mon_ty = match op.node {
                BinOpKind::Add | BinOpKind::Sub | BinOpKind::BitXor | BinOpKind::BitOr => {
                    MonoidType::Add
                }
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
            let recv_snip = span_to_snippet_macro(src_map, recv.span);
            let local_defs_snip =
                local_defs_span.map_or(String::new(), |sp| span_to_snippet_macro(src_map, sp));
            let pat_span = cls_body.params[0]
                .span
                .to(cls_body.params[cls_body.params.len() - 1].span);
            let pat_snip = span_to_snippet_macro(src_map, pat_span);
            let rhs_snip = span_to_snippet_macro(src_map, rhs.span);
            let op_snip = span_to_snippet_macro(src_map, op.span);
            let lhs_snip = span_to_snippet_macro(src_map, lhs.span);
            let suggestion = format!("{lhs_snip} {op_snip} {recv_snip}.map(|{pat_snip}| {local_defs_snip} {rhs_snip}).fold({id_snip}, |mut {lhs_snip}, v| {{ {lhs_snip} {op_snip} v; {lhs_snip} }})");

            cx.span_lint(FOLD_SIMPLE, expr.span, "implicit fold", |diag| {
                diag.span_suggestion(
                    expr.span,
                    "try using `fold` instead",
                    suggestion,
                    Applicability::MachineApplicable,
                );
            });
        }
    }
}
