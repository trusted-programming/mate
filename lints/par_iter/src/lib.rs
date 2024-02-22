#![feature(rustc_private)]
#![warn(unused_extern_crates)]
#![feature(let_chains)]

extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;

use clippy_utils::{get_trait_def_id, ty::implements_trait};
use rustc_errors::Applicability;
use rustc_hir::BindingAnnotation;
use rustc_hir::Mutability;
use rustc_hir::PatKind;
use rustc_hir::{
    def::Res,
    intravisit::{walk_expr, Visitor},
    Expr, ExprKind, Node,
};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_middle::ty::GenericArg;
use rustc_middle::ty::GenericArgKind;
use rustc_middle::ty::Ty;
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
        if let ExprKind::MethodCall(path, _recv, _args, _span) = &expr.kind {
            let method_name = path.ident.name.to_string();
            if let Some(replacement) = get_replacement_method(&method_name) {
                let suggestion = generate_suggestion(cx, expr, &method_name, replacement);

                // check that all types inside the closures are Send and sync or Copy
                let parent_node = cx.tcx.hir().get_parent(expr.hir_id);
                if let Node::Expr(parent_expr) = parent_node {
                    if !check_implements_par_iter(cx, expr)
                        && !check_implements_ref_par_iter(cx, expr)
                    {
                        return;
                    };
                    let mut validator = Validator { cx, is_valid: true };
                    validator.visit_expr(parent_expr);
                    if !validator.is_valid {
                        return;
                    }
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
        } else {
            walk_expr(self, ex)
        }
    }
}
// .consume_body(closure.body)
impl<'a, 'tcx> Visitor<'_> for Validator<'a, 'tcx> {
    fn visit_expr(&mut self, ex: &Expr) {
        if let ExprKind::Closure(closure) = ex.kind {
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

fn get_replacement_method(method_name: &str) -> Option<&str> {
    match method_name {
        "into_iter" => Some("into_par_iter"),
        "iter" => Some("par_iter"),
        "iter_mut" => Some("par_iter_mut"),
        _ => None,
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
