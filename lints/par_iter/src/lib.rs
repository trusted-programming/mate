#![feature(rustc_private)]
#![warn(unused_extern_crates)]
#![feature(let_chains)]

extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;

use clippy_utils::{get_trait_def_id, ty::implements_trait};
use rustc_errors::Applicability;
use rustc_hir::{
    def::Res,
    intravisit::{walk_expr, Visitor},
    Expr, ExprKind, Node,
};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_middle::ty::Ty;
use rustc_span::{sym, Symbol};

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

impl<'tcx> LateLintPass<'tcx> for ParIter {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'_>) {
        if let ExprKind::MethodCall(path, _recv, _args, _span) = &expr.kind {
            let method_name = path.ident.name.to_string();
            let replacement = get_replacement_method(&method_name);
            if replacement.is_empty() {
                return;
            }

            let suggestion = generate_suggestion(cx, expr, &method_name, replacement);

            let ty = cx.typeck_results().expr_ty(expr);

            let trait_paths = [
                ["rayon", "iter", "IntoParallelIterator"],
                ["rayon", "iter", "ParallelIterator"],
                // @todo ["rayon", "iter", "IndexedParallelIterator"],
                // @todo ["rayon", "iter", "IntoParallelRefIterator"],
                // @todo ["rayon", "iter", "IntoParallelRefMutIterator"],
                // Add more traits as needed
            ];

            let implements_par_iter = trait_paths
                .iter()
                .filter_map(|path| get_trait_def_id(cx, path))
                .any(|trait_def_id| implements_trait(cx, ty, trait_def_id, &[]));

            if !implements_par_iter {
                return;
            }

            // check that all types inside the closures are Send and sync or Copy
            let parent_node = cx.tcx.hir().get_parent(expr.hir_id);
            if let Node::Expr(parent_expr) = parent_node {
                let mut validator = Validator { cx, is_valid: true };
                validator.visit_expr(parent_expr);
                if !validator.is_valid {
                    return;
                }
            }

            cx.struct_span_lint(
                PAR_ITER,
                expr.span,
                "found iterator that can be parallelized",
                |diag| {
                    diag.multipart_suggestion(
                        "try using a parallel iterator",
                        vec![(expr.span, suggestion)],
                        Applicability::MachineApplicable,
                    );
                },
            );
        }
    }
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
        if let ExprKind::Path(ref path) = ex.kind {
            if let Res::Local(hir_id) = self.cx.typeck_results().qpath_res(path, ex.hir_id) {
                if let Node::Local(local) = self.cx.tcx.hir().get_parent(hir_id) {
                    if let Some(expr) = local.init {
                        self.is_valid &= is_type_valid(
                            self.cx,
                            self.cx.tcx.typeck(expr.hir_id.owner).node_type(expr.hir_id),
                        );
                    };
                }
            }
        } else {
            walk_expr(self, ex)
        }
    }
}

impl<'a, 'tcx> Visitor<'_> for Validator<'a, 'tcx> {
    fn visit_expr(&mut self, ex: &Expr) {
        if let ExprKind::Closure(closure) = ex.kind {
            // @fixme check that this works
            if let Node::Expr(expr) = self.cx.tcx.hir_node(closure.body.hir_id) {
                let mut closure_visitor = ClosureVisitor {
                    cx: self.cx,
                    is_valid: true,
                };
                closure_visitor.visit_expr(expr);
                self.is_valid &= closure_visitor.is_valid;
            }
        } else {
            walk_expr(self, ex)
        }
    }
}
fn is_type_valid<'tcx>(cx: &LateContext<'tcx>, ty: Ty<'tcx>) -> bool {
    let implements_send = check_trait_impl(cx, ty, sym::Send);
    let implements_sync = check_trait_impl(cx, ty, sym::Sync);
    let is_copy = check_trait_impl(cx, ty, sym::Copy);
    is_copy || (implements_send && implements_sync)
}

fn check_trait_impl<'tcx>(cx: &LateContext<'tcx>, ty: Ty<'tcx>, trait_name: Symbol) -> bool {
    cx.tcx
        .get_diagnostic_item(trait_name)
        .map_or(false, |trait_id| implements_trait(cx, ty, trait_id, &[]))
}

fn get_replacement_method(method_name: &str) -> &str {
    match method_name {
        "into_iter" => "into_par_iter",
        "iter" => "par_iter",
        "iter_mut" => "par_iter_mut",
        _ => "",
    }
}

fn generate_suggestion(
    cx: &LateContext<'_>,
    expr: &Expr<'_>,
    method_name: &str,
    replacement: &str,
) -> String {
    cx.sess()
        .source_map()
        .span_to_snippet(expr.span)
        .map(|s| s.replace(method_name, replacement))
        .unwrap_or_else(|_| String::from("/* error: unable to generate suggestion */"))
}

#[test]
fn ui() {
    dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
}
