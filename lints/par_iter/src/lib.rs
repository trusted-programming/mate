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
use rustc_middle::middle::resolve_bound_vars::ObjectLifetimeDefault;
use rustc_middle::query::Key;
use rustc_middle::ty::GenericArg;
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

            let implement_par_iter = check_implements_par_iter(cx, expr);
            // check that all types inside the closures are Send and sync or Copy
            let parent_node = cx.tcx.hir().get_parent(expr.hir_id);
            if let Node::Expr(parent_expr) = parent_node {
                let mut validator = Validator {
                    cx,
                    is_valid: true,
                    implement_par_iter,
                };
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

struct ClosureVisitor<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    is_valid: bool,
    implement_par_iter: bool,
}

struct Validator<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    is_valid: bool,
    implement_par_iter: bool,
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
                        if !self.implement_par_iter {
                            self.is_valid &= check_implements_ref_par_iter(self.cx, expr);
                        }
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
                    implement_par_iter: self.implement_par_iter,
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

fn check_implements_par_iter<'tcx>(cx: &LateContext<'tcx>, expr: &'tcx Expr<'_>) -> bool {
    let trait_paths = [
        ["rayon", "iter", "IntoParallelIterator"],
        ["rayon", "iter", "ParallelIterator"],
        // @todo ["rayon", "iter", "IndexedParallelIterator"],
        // @todo ["rayon", "iter", "IntoParallelRefMutIterator"],
        // Add more traits as needed
    ];
    let ty = cx.typeck_results().expr_ty(expr);
    let mut args = vec![];

    for path in trait_paths {
        if let Some(trait_def_id) = get_trait_def_id(cx, &path) {
            if path[2] == "IntoParallelRefIterator" {
                args.push(convert_to_generic_arg(cx, ty));
            }
            if implements_trait(cx, ty, trait_def_id, &args) {
                return true;
            }
        }
    }

    false
}

fn check_implements_ref_par_iter<'tcx>(cx: &LateContext<'tcx>, expr: &'tcx Expr<'_>) -> bool {
    let path = ["rayon", "iter", "IntoParallelRefIterator"];

    let ty = cx.typeck_results().expr_ty(expr);
    let mut args = vec![];

    if let Some(trait_def_id) = get_trait_def_id(cx, &path) {
        if path[2] == "IntoParallelRefIterator" {
            args.push(convert_to_generic_arg(cx, ty));
        }
        if implements_trait(cx, ty, trait_def_id, &args) {
            return true;
        }
    }
    false
}

fn convert_to_generic_arg<'tcx>(cx: &LateContext<'tcx>, ty: Ty<'_>) -> GenericArg<'tcx> {
    if let Some(ty_def_id) = ty.ty_def_id() {
        let object_lifetime_default: ObjectLifetimeDefault =
            cx.tcx.object_lifetime_default(ty_def_id);

        match object_lifetime_default {
            ObjectLifetimeDefault::Empty => {
                // For an empty default, use an erased region as a placeholder.
                let erased_region = cx.tcx.lifetimes.re_erased;
                GenericArg::from(erased_region)
            }
            ObjectLifetimeDefault::Static => {
                // For a static lifetime, use the 'static region.
                let static_region = cx.tcx.lifetimes.re_static;
                GenericArg::from(static_region)
            }
            ObjectLifetimeDefault::Ambiguous => {
                // @todo For an ambiguous lifetime, use an erased region as a placeholder.

                let erased_region = cx.tcx.lifetimes.re_erased;
                GenericArg::from(erased_region)
            }
            ObjectLifetimeDefault::Param(_def_id) => {
                //@todo implement this properly
                // For a parameterized lifetime, create a region based on the DefId.

                let region = cx.tcx.lifetimes.re_erased;

                // let region = cx
                //     .tcx
                //     .mk_region(ty::RegionKind::ReEarlyBound(ty::EarlyBoundRegion {
                //         def_id,
                //         index: 0, // Assuming index is 0, adjust as needed.
                //         name: tcx
                //             .def_path(def_id)
                //             .last()
                //             .unwrap()
                //             .data
                //             .get_opt_name()
                //             .unwrap(), // Get the name from the DefId.
                //     }));
                GenericArg::from(region)
            }
        }
    } else {
        let erased_region = cx.tcx.lifetimes.re_erased;
        GenericArg::from(erased_region)
    }
}

#[test]
fn ui() {
    dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
}
