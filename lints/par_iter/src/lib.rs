#![feature(rustc_private)]
#![warn(unused_extern_crates)]
#![feature(let_chains)]

extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_hir_typeck;
extern crate rustc_infer;
extern crate rustc_middle;
extern crate rustc_span;

use clippy_utils::{get_trait_def_id, ty::implements_trait};
use rustc_errors::Applicability;
use rustc_hir::{
    def::Res,
    intravisit::{walk_expr, Visitor},
    Expr, ExprKind, HirId, Node,
};

use rustc_hir_typeck::expr_use_visitor::{Delegate, ExprUseVisitor, PlaceWithHirId};
use rustc_infer::infer::TyCtxtInferExt;
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_middle::mir::FakeReadCause;
use rustc_middle::ty::BorrowKind;
use rustc_middle::ty::Ty;
use rustc_span::{sym, Symbol};
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
                    )
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
// .consume_body(closure.body)
impl<'a, 'tcx> Visitor<'_> for Validator<'a, 'tcx> {
    fn visit_expr(&mut self, ex: &Expr) {
        if let ExprKind::Closure(closure) = ex.kind {
            if let Node::Expr(expr) = self.cx.tcx.hir().get(closure.body.hir_id) {
                let mut closure_visitor = ClosureVisitor {
                    cx: self.cx,
                    is_valid: true,
                };
                closure_visitor.visit_expr(expr);
                let mut checker = MutabilityChecker {
                    is_consume: false,
                    is_borrow: false,
                    is_mutable: false,
                    is_fake_read: false,
                    is_copy: false,
                    is_bind: false,
                };
                let infcx = self.cx.tcx.infer_ctxt().build();
                let mut expr_use_visitor = ExprUseVisitor::new(
                    &mut checker,
                    &infcx,
                    closure.def_id,
                    self.cx.param_env,
                    self.cx.typeck_results(),
                );
                let src_map = self.cx.sess().source_map();
                dbg!(span_to_snippet_macro(src_map, expr.span));
                expr_use_visitor.walk_expr(expr);
                dbg!(checker);
                self.is_valid &= closure_visitor.is_valid;

                // @todo use MIR to determine if the closure is Fn FnMut or FnOnce
                // let ty = self
                //     .cx
                //     .tcx
                //     .type_of(closure.def_id)
                //     .instantiate(self.cx.tcx, &[]);

                // if let TyKind::Closure(_def_id, args) = ty.kind() {
                //     match args.as_closure().kind() {
                //         ClosureKind::Fn => {
                //             self.is_valid &= true;
                //         }
                //         ClosureKind::FnMut | ClosureKind::FnOnce => {
                //             self.is_valid &= true;
                //         }
                //     }
                // }
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

#[derive(Debug)]
struct MutabilityChecker {
    is_consume: bool,
    is_borrow: bool,
    is_mutable: bool,
    is_fake_read: bool,
    is_copy: bool,
    is_bind: bool,
}

impl<'tcx> Delegate<'tcx> for MutabilityChecker {
    fn consume(&mut self, _place_with_id: &PlaceWithHirId<'tcx>, _diag_expr_id: HirId) {
        self.is_consume |= true;
    }

    fn borrow(
        &mut self,
        _place_with_id: &PlaceWithHirId<'tcx>,
        _diag_expr_id: HirId,
        _bk: BorrowKind,
    ) {
        self.is_borrow |= true;
    }

    fn mutate(&mut self, _assignee_place: &PlaceWithHirId<'tcx>, _diag_expr_id: HirId) {
        self.is_mutable |= true;
    }

    fn fake_read(
        &mut self,
        _place_with_id: &PlaceWithHirId<'tcx>,
        _cause: FakeReadCause,
        _diag_expr_id: HirId,
    ) {
        self.is_fake_read |= true;
    }
    // Provided methods
    fn copy(&mut self, place_with_id: &PlaceWithHirId<'tcx>, diag_expr_id: HirId) {
        self.is_copy |= true;
    }
    fn bind(&mut self, _binding_place: &PlaceWithHirId<'tcx>, diag_expr_id: HirId) {
        self.is_bind |= true;
    }
}

#[test]
fn ui() {
    dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
}
