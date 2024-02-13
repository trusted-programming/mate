use rustc_errors::Applicability;
use rustc_hir::{Expr, ExprKind};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_span::Symbol;
use rustc_span::sym;
use utils::span_to_snippet_macro;

dylint_linting::declare_late_lint! {
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
    pub FOLD_VEC,
    Warn,
    "suggest using explicit fold"
}

impl<'tcx> LateLintPass<'tcx> for FoldVec {
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

            // Type check the accumulated parameter and assign the correct identity.
            // Attempting to match 'c.push(v);'

            let ExprKind::MethodCall(iseg, irecv, _iargs, _ispan) = &pat_expr.kind else { return; };
            // Vector specific insert method
            if iseg.ident.name != Symbol::intern("push") {
                return;
            }
            // Make sure the receiver is a variable/path to variable.
            let ExprKind::Path(_) = irecv.kind else { return; };

            // Make sure that this is the vec method call
            let ty = cx.tcx.typeck(irecv.hir_id.owner.def_id).node_type(irecv.hir_id);
            let Some(adt) = ty.ty_adt_def() else { return; };
            let did = adt.did();
            if !cx.tcx.is_diagnostic_item(sym::Vec, did) {
                return;
            }

            let src_map = cx.sess().source_map();
            let recv_snip = span_to_snippet_macro(src_map, recv.span);
            let irecv_snip = span_to_snippet_macro(src_map, irecv.span);
            let local_defs_snip =
                local_defs_span.map_or(String::new(), |sp| span_to_snippet_macro(src_map, sp));
            let pat_span = cls_body.params[0]
                .span
                .to(cls_body.params[cls_body.params.len() - 1].span);
            let pat_snip = span_to_snippet_macro(src_map, pat_span);
            let pat_expr_snip = span_to_snippet_macro(src_map, pat_expr.span);
            let suggestion =
                format!("{irecv_snip} = {recv_snip}.fold({irecv_snip}, |mut {irecv_snip}, {pat_snip}| {{ {local_defs_snip} {pat_expr_snip}; {irecv_snip} }})");

            cx.struct_span_lint(FOLD_VEC, expr.span, "implicit fold", |diag| {
                diag.span_suggestion(
                    expr.span,
                    "try using `fold` instead",
                    suggestion,
                    Applicability::MachineApplicable,
                )
            });
        }
    }
}
