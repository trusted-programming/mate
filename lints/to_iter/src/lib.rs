#![feature(rustc_private)]
#![warn(unused_extern_crates)]
#![feature(let_chains)]

extern crate rustc_errors;
extern crate rustc_hash;
extern crate rustc_hir;
extern crate rustc_hir_typeck;
extern crate rustc_infer;
extern crate rustc_middle;
extern crate rustc_span;

mod variable_check;

use clippy_utils::source::snippet_indent;
use clippy_utils::{higher::ForLoop, ty::implements_trait};
use clippy_utils::{is_lang_item_or_ctor, is_res_lang_ctor, ty};
use rustc_errors::Applicability;
use rustc_hir::{
    intravisit::{walk_expr, Visitor},
    Expr, ExprKind,
};
use rustc_hir::{LangItem, QPath};
use rustc_middle::ty::Ty;
use rustc_span::symbol::sym;

use rustc_lint::{LateContext, LateLintPass, LintContext};
use utils::span_to_snippet_macro;
use variable_check::check_variables;

dylint_linting::declare_late_lint! {
    /// ### What it does
    /// Convert a for loop into it's for_each equivalent
    /// ### Why is this bad?
    /// Offers opportunities for parallelisms
    /// ### Known problems
    /// lots
    ///
    /// ### Example
    /// ```rust
    /// // example code where a warning is issued
    /// ```
    /// Use instead:
    /// ```rust
    /// // example code that does not raise a warning
    /// ```
    pub TO_ITER,
    Warn,
    "suggest using `(try_)for_each`"
}

impl<'tcx> LateLintPass<'tcx> for ToIter {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'_>) {
        if let Some(ForLoop {
            pat,
            arg,
            body,
            loop_id: _loop_id,
            span: _span,
        }) = ForLoop::hir(expr)
        {
            let mut validator = Validator {
                cx,
                is_valid: true,
                has_continue: false,
                ret_ty: None,
            };
            validator.visit_expr(body);
            if !validator.is_valid {
                return;
            }

            let mut used_vars = check_variables(cx, body);
            used_vars
                .all_vars
                .retain(|v| !used_vars.copy_vars.contains(v));
            if !used_vars.all_vars.is_empty() {
                return;
            }

            // Check if we need to convert to iterator explicitly
            let src_map = cx.sess().source_map();
            let mut iter_snip = span_to_snippet_macro(src_map, arg.span);

            // Check if the argument is a Range
            let is_range = {
                let range_items = [
                    LangItem::Range,
                    LangItem::RangeTo,
                    LangItem::RangeFrom,
                    LangItem::RangeToInclusive,
                    LangItem::RangeInclusiveNew,
                ];

                let langs = cx.tcx.lang_items();
                let mut range_langs = langs.iter().filter(|(li, _)| range_items.contains(&li));
                match &arg.kind {
                    ExprKind::Struct(QPath::LangItem(li, _), _, _) => {
                        range_langs.any(|(ri, _)| ri == *li)
                    }
                    ExprKind::Call(l, _) => {
                        if let ExprKind::Path(QPath::LangItem(li, _)) = &l.kind {
                            range_langs.any(|(ri, _)| ri == *li)
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            };

            // Check if we need to convert to an iterator.
            // We explicitly call into_iter on Range to allow for better linting with par_iter.
            // TODO: When do we need extra parens
            let ty = cx.typeck_results().expr_ty(arg);
            if !cx
                .tcx
                .lang_items()
                .iterator_trait()
                .map_or(false, |id| implements_trait(cx, ty, id, &[]))
                || is_range
            {
                iter_snip = format!("({iter_snip}).into_iter()");
            } else {
                iter_snip = format!("({iter_snip})");
            }
            let pat_snip = span_to_snippet_macro(src_map, pat.span);

            // Compute the body span for the inner stmts of the block.
            // This is required in the case of try_for_each so we can add the extra return
            // statement.
            let body_span = if let ExprKind::Block(block, _) = &body.kind {
                let first_span = if block.stmts.is_empty() {
                    block.expr.map(|e| e.span)
                } else {
                    Some(block.stmts[0].span)
                };
                if let Some(sp) = first_span {
                    let last_span = if let Some(e) = block.expr {
                        e.span
                    } else {
                        block.stmts[block.stmts.len() - 1].span
                    };
                    Some(sp.to(last_span))
                } else {
                    None
                }
            } else {
                Some(body.span)
            };
            let mut body_snip =
                body_span.map_or(String::new(), |s| span_to_snippet_macro(src_map, s));

            if validator.has_continue && validator.ret_ty.is_none() {
                body_snip = body_snip.replace("continue", "return");
            }

            // Make sure to terminate the last statement with a semicolon
            // TODO: Are we missing anything here
            if !body_snip.trim_end().ends_with([';', '}']) {
                body_snip = format!("{};", body_snip.trim_end());
            }

            // Acquire the indentation of the loop expr and it's body for nicer formatting in the
            // sugggestion
            let outer_indent = snippet_indent(cx, expr.span).unwrap_or("".to_string());
            let indent = body_span
                .map_or(None, |s| snippet_indent(cx, s))
                .unwrap_or(format!("{outer_indent}    "));

            let sugg = if let Some(ty) = validator.ret_ty {
                let constr = if ty::is_type_diagnostic_item(cx, ty, sym::Option) {
                    "Some"
                } else if ty::is_type_diagnostic_item(cx, ty, sym::Result) {
                    "Ok"
                } else {
                    return;
                };
                format!(
                    "{iter_snip}.try_for_each(|{pat_snip}| {{\n{indent}{body_snip}\n{indent}return {constr}(());\n{outer_indent}}})?;"
                )
            } else {
                format!(
                    "{iter_snip}.for_each(|{pat_snip}| {{\n{indent}{body_snip}\n{outer_indent}}});"
                )
            };

            cx.span_lint(TO_ITER, expr.span, "use an iterator", |diag| {
                diag.span_suggestion(
                    expr.span,
                    "try using an iterator",
                    sugg,
                    Applicability::MachineApplicable,
                );
            });
        }
    }
}

struct Validator<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    is_valid: bool,
    has_continue: bool,
    ret_ty: Option<Ty<'tcx>>,
}

impl<'a, 'tcx> Visitor<'_> for Validator<'a, 'tcx> {
    fn visit_expr(&mut self, ex: &Expr) {
        match &ex.kind {
            ExprKind::Loop(_, _, _, _) | ExprKind::Closure(_) | ExprKind::Break(_, _) => {
                self.is_valid = false
            }
            ExprKind::Continue(d) => {
                // We don't support skipping outer loops
                if d.label.is_some() {
                    self.is_valid = false;
                } else {
                    self.has_continue = true;
                }
            }
            ExprKind::Ret(v) => {
                let Some(v) = v else {
                    self.is_valid = false;
                    return;
                };
                let v_ty = self.cx.typeck_results().expr_ty(v);

                // Check if implements Try trait
                if !self
                    .cx
                    .tcx
                    .lang_items()
                    .try_trait()
                    .map_or(false, |id| ty::implements_trait(self.cx, v_ty, id, &[]))
                {
                    self.is_valid = false;
                    return;
                }

                // Brute forces a check that the return type is the associated type Output.
                // TODO: Probably a nicer way to do this.
                match &v.kind {
                    // Must statically know that the return value is None
                    ExprKind::Path(qp) => {
                        let res = self.cx.typeck_results().qpath_res(qp, v.hir_id);
                        let is_opt = ty::is_type_diagnostic_item(self.cx, v_ty, sym::Option);
                        let is_none = is_res_lang_ctor(self.cx, res, LangItem::OptionNone);
                        if !(is_opt && is_none) {
                            self.is_valid = false;
                            return;
                        }
                    }
                    ExprKind::Call(l, _) => match &l.kind {
                        // Must statically know that the return value is
                        // Err(_) or Try::from_residual(_)
                        ExprKind::Path(qp) => {
                            let res = self.cx.typeck_results().qpath_res(qp, l.hir_id);
                            let is_err = ty::is_type_diagnostic_item(self.cx, v_ty, sym::Result)
                                && is_res_lang_ctor(self.cx, res, LangItem::ResultErr);
                            let is_from_residual = is_lang_item_or_ctor(
                                self.cx,
                                res.def_id(),
                                LangItem::TryTraitFromResidual,
                            );
                            if !is_err && !is_from_residual {
                                self.is_valid = false;
                                return;
                            }
                        }
                        _ => {
                            self.is_valid = false;
                            return;
                        }
                    },
                    _ => {
                        self.is_valid = false;
                        return;
                    }
                }
                self.ret_ty = Some(v_ty);
            }
            _ => walk_expr(self, ex),
        }
    }
}

#[test]
fn ui() {
    dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
}
