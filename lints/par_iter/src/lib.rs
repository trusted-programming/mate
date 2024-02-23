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
    BindingAnnotation, Expr, ExprKind, Mutability, Node, PatKind, PathSegment,
};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_middle::ty::{GenericArg, GenericArgKind, Ty};
use rustc_span::{sym, Symbol};

dylint_linting::declare_late_lint! {
    /// ### What it does
    /// parallelize iterators using rayon
    /// ### Why is this bad?
    /// parrallel iters are often faster
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
    pub PAR_ITER,
    Warn,
    "suggest using par iter"
}

impl<'tcx> LateLintPass<'tcx> for ParIter {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'_>) {
        if let ExprKind::MethodCall(path, _recv, _args, _span) = &expr.kind
            && (check_implements_par_iter(cx, expr) || check_implements_ref_par_iter(cx, expr))
            && let Node::Expr(parent_expr) = cx.tcx.hir().get_parent(expr.hir_id)
            && let Some(suggestion) = generate_suggestion(cx, expr, path)
        {
            let mut validator = Validator { cx, is_valid: true };
            validator.visit_expr(parent_expr);
            if !validator.is_valid {
                return;
            }

            cx.span_lint(
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

struct Validator<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    is_valid: bool,
}

impl<'a, 'tcx> Visitor<'_> for Validator<'a, 'tcx> {
    fn visit_expr(&mut self, ex: &Expr) {
        match ex.kind {
            ExprKind::Closure(closure) => {
                if let Node::Expr(expr) = self.cx.tcx.hir_node(closure.body.hir_id) {
                    walk_expr(self, expr);
                }
            }
            ExprKind::Path(ref path) => {
                if let Res::Local(hir_id) = self.cx.typeck_results().qpath_res(path, ex.hir_id) {
                    if let Node::Pat(pat) = self.cx.tcx.hir_node(hir_id) {
                        if let PatKind::Binding(BindingAnnotation(_, Mutability::Mut), _, _, _) =
                            pat.kind
                        {
                            self.is_valid = false;
                        }
                    }
                    if let Node::Local(local) = self.cx.tcx.hir().get_parent(hir_id) {
                        if let Some(expr) = local.init {
                            self.is_valid &= is_type_valid(
                                self.cx,
                                self.cx.tcx.typeck(expr.hir_id.owner).node_type(expr.hir_id),
                            );
                        }
                    }
                }
            }
            _ => walk_expr(self, ex),
        }
    }
}

fn is_type_valid<'tcx>(cx: &LateContext<'tcx>, ty: Ty<'tcx>) -> bool {
    let is_send = check_trait_impl(cx, ty, sym::Send);
    let is_sync = check_trait_impl(cx, ty, sym::Sync);
    let is_copy = check_trait_impl(cx, ty, sym::Copy);
    is_copy || (is_send && is_sync)
}

fn check_trait_impl<'tcx>(cx: &LateContext<'tcx>, ty: Ty<'tcx>, trait_name: Symbol) -> bool {
    cx.tcx
        .get_diagnostic_item(trait_name)
        .map_or(false, |trait_id| implements_trait(cx, ty, trait_id, &[]))
}

fn generate_suggestion(
    cx: &LateContext<'_>,
    expr: &Expr<'_>,
    path: &PathSegment,
) -> Option<String> {
    let method_name = &path.ident.name.to_string()[..];
    let replacement = match method_name {
        "into_iter" => Some("into_par_iter"),
        "iter" => Some("par_iter"),
        "iter_mut" => Some("par_iter_mut"),
        _ => None,
    };

    if let Some(r) = replacement {
        cx.sess()
            .source_map()
            .span_to_snippet(expr.span)
            .map(|s| Some(s.replace(method_name, r)))
            .unwrap_or_else(|_| None)
    } else {
        None
    }
}

fn check_implements_par_iter<'tcx>(cx: &LateContext<'tcx>, expr: &'tcx Expr<'_>) -> bool {
    let trait_paths = [
        ["rayon", "iter", "IntoParallelIterator"],
        ["rayon", "iter", "ParallelIterator"],
        ["rayon", "iter", "IndexedParallelIterator"],
    ];
    let ty = cx.typeck_results().expr_ty(expr);

    trait_paths.iter().any(|path| {
        get_trait_def_id(cx, path).map_or(false, |trait_def_id| {
            implements_trait(cx, ty, trait_def_id, &[])
        })
    })
}

fn check_implements_ref_par_iter<'tcx>(cx: &LateContext<'tcx>, expr: &'tcx Expr<'_>) -> bool {
    let trait_paths = [
        ["rayon", "iter", "IntoParallelRefMutIterator"],
        ["rayon", "iter", "IntoParallelRefIterator"],
    ];
    let ty = cx.typeck_results().expr_ty(expr);

    let lt;
    if let Some(lifetime) = ty
        .walk()
        .find(|t| matches!(t.unpack(), GenericArgKind::Lifetime(_)))
    {
        lt = lifetime;
    } else {
        let static_region = cx.tcx.lifetimes.re_static;
        lt = GenericArg::from(static_region);
    }

    trait_paths.iter().any(|path| {
        get_trait_def_id(cx, path).map_or(false, |trait_def_id| {
            implements_trait(cx, ty, trait_def_id, &[lt])
        })
    })
}

#[test]
fn ui() {
    dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
}
