macro_rules! map_collection {
    ($struct_name:ident, $lint_name:ident, $type_symbol:ident, $method_name:literal, $type_name:literal) => {

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
    pub $lint_name,
    Warn,
    "suggest using a map/collect"
}

declare_lint_pass!($struct_name => [$lint_name]);

impl<'tcx> LateLintPass<'tcx> for $struct_name {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'_>) {
        /*
         * Intended pattern
         * recv.for_each(|pat| { local_defs; c.collection_insert(v); })
         * --->
         * c.extend(recv.map(|pat| { local_defs; v }))
         */

        if let ExprKind::MethodCall(seg, recv, arg, _) = &expr.kind
            && seg.ident.name == Symbol::intern("for_each")
        {
            // for_each will only ever have one argument, and it's a closure
            assert_eq!(arg.len(), 1);
            let ExprKind::Closure(for_each_cls) = &arg[0].kind else {
                return;
            };

            let hir_map = cx.tcx.hir();
            let cls_body = hir_map.body(for_each_cls.body);

            // Collect a set of local definitions, the expression we wish to analyze and
            // the statements following it
            let Some((Some(pat_expr), local_defs_span, body_span)) =
                utils::get_pat_expr_and_spans(cls_body.value)
            else {
                return;
            };

            // We should only have one statement left
            if body_span.is_some() {
                return;
            }

            // Type check the accumulated parameter and assign the correct identity.
            // Attempting to match 'c.push(v);'

            let ExprKind::MethodCall(seg, coll, args, _) = &pat_expr.kind else {
                return;
            };

            // Collection update method
            if seg.ident.name != Symbol::intern($method_name) {
                return;
            }
            // Make sure the receiver is a variable/path to variable.
            let ExprKind::Path(_) = coll.kind else {
                return;
            };

            // Make sure the method is actually the collection method
            let ty = cx
                .tcx
                .typeck(coll.hir_id.owner.def_id)
                .node_type(coll.hir_id);
            let Some(adt) = ty.ty_adt_def() else {
                return;
            };
            if !cx.tcx.is_diagnostic_item(sym::$type_symbol, adt.did()) {
                return;
            }

            // Suggestion creation
            let src_map = cx.sess().source_map();
            let recv = span_to_snippet_macro(src_map, recv.span);
            let coll = span_to_snippet_macro(src_map, coll.span);
            let local_defs =
                local_defs_span.map_or(String::new(), |sp| span_to_snippet_macro(src_map, sp));

            let type_args = std::iter::repeat("_").take(args.len()).intersperse(",").collect::<String>();

            let args_span = args[0]
                .span
                .to(args[args.len() - 1].span);
            let args = {
                let snip = span_to_snippet_macro(src_map, args_span);
                if args.len() > 1 {
                    format!("({snip})")
                } else { snip }
            };

            let pat_span = cls_body.params[0]
                .span
                .to(cls_body.params[cls_body.params.len() - 1].span);
            let pat = span_to_snippet_macro(src_map, pat_span);

            let suggestion =
                format!("{coll}.extend({recv}.map(|{pat}| {{ {local_defs} {args} }}).collect::<{}<{type_args}>>())", $type_name);

            cx.span_lint($lint_name, expr.span, "implicit map", |diag| {
                diag.span_suggestion(
                    expr.span,
                    "try using `map` instead",
                    suggestion,
                    Applicability::MachineApplicable,
                );
            });
        }
    }
}

};}

pub(crate) use map_collection;
