#![feature(rustc_private)]
#![warn(unused_extern_crates)]
#![feature(let_chains)]

extern crate rustc_hir;
// extern crate rustc_middle;
extern crate rustc_span;

use clippy_utils::sym;
use clippy_utils::ty::implements_trait;
use rustc_hir::intravisit::{walk_expr, Visitor};
use rustc_hir::{Expr, ExprKind};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_span::sym;
// use rustc_middle::ty::TyKind;
// use rustc_session::{declare_lint, declare_lint_pass};
// use rustc_span::{Span, Symbol};
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
    pub PAR_ITER,
    Warn,
    "suggest using par iter"
}

impl Visitor<'_> for ParIter {
    fn visit_expr(&mut self, ex: &Expr) {
        match ex.kind {
            ExprKind::Closure(_c) => {
                dbg!("got a closure");
            }
            _ => walk_expr(self, ex),
        }
    }
}

impl<'tcx> LateLintPass<'tcx> for ParIter {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'_>) {
        if let ExprKind::MethodCall(path, _recv, args, span) = &expr.kind
            && path.ident.name == sym!(into_iter)
        {
            let src_map = cx.sess().source_map();
            let ty = cx.typeck_results().expr_ty(expr);

            // into_par_iter
            let trait_def_id =
                clippy_utils::get_trait_def_id(cx, &["rayon", "iter", "IntoParallelIterator"])
                    .unwrap();

            if implements_trait(cx, ty, trait_def_id, &[]) {
                // check that iterator type is Send
                let is_send = cx.tcx.get_diagnostic_item(sym::Send).map_or(false, |id| {
                    implements_trait(cx, cx.typeck_results().expr_ty(expr), id, &[])
                });
                if !is_send {
                    return;
                }
                // @todo check that all types inside the closures are Send and sync or Copy

                let id_snip = span_to_snippet_macro(src_map, expr.span);
                dbg!(id_snip);
            }

            // @todo par_iter

            // @todo par_iter_mut
        }
    }
}

#[test]
fn ui() {
    dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
}
