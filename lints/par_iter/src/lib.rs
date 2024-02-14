#![feature(rustc_private)]
#![warn(unused_extern_crates)]
#![feature(let_chains)]

extern crate rustc_hir;
extern crate rustc_span;

use clippy_utils::{get_trait_def_id, sym, ty::implements_trait};
use rustc_hir::{
    def::Res,
    intravisit::{walk_expr, Visitor},
    Expr, ExprKind, Node,
};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_span::source_map::SourceMap;
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

struct ClosureVisitor<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    is_valid: bool,
}

struct Validator<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    is_valid: bool,
}
impl<'a, 'tcx> Visitor<'_> for ClosureVisitor<'a, 'tcx> {
    fn visit_expr(&mut self, ex: &Expr) {
        match ex.kind {
            ExprKind::Path(path) => {
                let res: Res = self.cx.typeck_results().qpath_res(&path, ex.hir_id);

                if let Res::Local(hir_id) = res {
                    let ty = self.cx.tcx.type_of(hir_id);

                    let hir = self.cx.tcx.hir();
                    if let Some(node) = hir.find(hir_id) {
                        if let rustc_hir::Node::Pat(pat) = node {
                            let ty = self.cx.tcx.pat_ty(pat);
                            let ty = typeck_results.pat_ty(pat);

                            implements_trait(self.cx, ty, trait_id, args);
                            self.is_valid = true;
                        }
                    }
                }
            }
            _ => walk_expr(self, ex),
        }
    }
}

impl<'a, 'tcx> Visitor<'_> for Validator<'a, 'tcx> {
    fn visit_expr(&mut self, ex: &Expr) {
        match ex.kind {
            ExprKind::Closure(closure) => {
                let hir = self.cx.tcx.hir();
                let node = hir.get(closure.body.hir_id);
                if let Node::Expr(expr) = node {
                    let mut closure_visitor = ClosureVisitor {
                        cx: self.cx,
                        is_valid: true,
                    };
                    closure_visitor.visit_expr(expr);
                }
            }
            _ => walk_expr(self, ex),
        }
    }
}

impl<'tcx> LateLintPass<'tcx> for ParIter {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'_>) {
        if let ExprKind::MethodCall(path, _recv, _args, _span) = &expr.kind
            && (path.ident.name == sym!(into_iter)
                || path.ident.name == sym!(iter)
                || path.ident.name == sym!(iter_mut))
        {
            let ty = cx.typeck_results().expr_ty(expr);

            let mut implements_par_iter = false;

            let trait_defs = vec![
                get_trait_def_id(cx, &["rayon", "iter", "IntoParallelIterator"]),
                get_trait_def_id(cx, &["rayon", "iter", "ParallelIterator"]),
                // @todo get_trait_def_id(cx, &["rayon", "iter", "IndexedParallelIterator"]),
                // @todo get_trait_def_id(cx, &["rayon", "iter", "IntoParallelRefIterator"]),
                // @todo get_trait_def_id(cx, &["rayon", "iter", "IntoParallelRefMutIterator"]),
            ];

            for t in trait_defs {
                if let Some(trait_def_id) = t {
                    implements_par_iter =
                        implements_par_iter || implements_trait(cx, ty, trait_def_id, &[]);
                }
            }

            if !implements_par_iter {
                return;
            }
            dbg!("implements_par_iter");

            // @todo check that all types inside the closures are Send and sync or Copy
            let mut validator = Validator { cx, is_valid: true };
            let hir = cx.tcx.hir();

            let parent_node = hir.get_parent(expr.hir_id);
            match parent_node {
                Node::Expr(expr) => {
                    validator.visit_expr(expr);
                }
                // Handle other kinds of parent nodes as needed
                _ => {}
            }
            if !validator.is_valid {
                return;
            }

            dbg!("the END");
        }
    }
}

#[test]
fn ui() {
    dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
}
