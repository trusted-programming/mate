#![feature(rustc_private)]
#![warn(unused_extern_crates)]
#![feature(let_chains)]
#![feature(unwrap_infallible)]

extern crate rustc_data_structures;
extern crate rustc_errors;
extern crate rustc_hash;
extern crate rustc_hir;
extern crate rustc_hir_typeck;
extern crate rustc_infer;
extern crate rustc_middle;
extern crate rustc_span;
extern crate rustc_trait_selection;

mod constants;
mod variable_check;

use clippy_utils::{get_parent_expr, get_trait_def_id};
use rustc_data_structures::fx::FxHashSet;
use rustc_errors::Applicability;
use rustc_hir::intravisit::{walk_expr, Visitor};
use rustc_hir::{self as hir};
use rustc_infer::infer::TyCtxtInferExt;
use rustc_infer::traits::{Obligation, ObligationCause};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_middle::ty::{self, GenericArgs};
use rustc_span::sym;
use rustc_trait_selection::traits::ObligationCtxt;
use variable_check::{
    check_implements_par_iter, check_trait_impl, check_variables, generate_suggestion,
    is_type_valid,
};

dylint_linting::declare_late_lint! {
    /// ### What it does
    /// parallelize iterators using rayon
    /// ### Why is this bad?
    /// parallel iters are often faster
    /// ### Known problems
    /// lots
    /// changing to par iterators will cause the loss of ordering
    /// ### Example
    /// ```rust
    /// (0..100).into_iter().for_each(|x| println!("{:?}", x));
    /// ```
    /// Use instead:
    /// ```rust
    /// use rayon::iter::*;
    ///
    /// (0..100).into_par_iter().for_each(|x| println!("{:?}", x));
    /// ```
    pub PAR_ITER,
    Warn,
    "suggest using par iter"
}

impl<'tcx> LateLintPass<'tcx> for ParIter {
    // TODO: implement check crate to check if rayon is present
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'_>) {
        if let hir::ExprKind::MethodCall(path, recv, _args, _span) = &expr.kind
            && let Some(suggestion) = generate_suggestion(cx, expr, path)
        {
            let par_iter_traits = check_implements_par_iter(cx, recv);
            if !par_iter_traits.is_empty() && is_type_valid(cx, cx.typeck_results().expr_ty(recv)) {
                // TODO: issue with into_par_iter() need to check directly with
                // parallel iterator

                let mut allowed_methods: FxHashSet<&str> =
                    ["into_iter", "iter", "iter_mut", "map_or"]
                        .into_iter()
                        .collect();
                for into_par_iter_trait in par_iter_traits {
                    allowed_methods.extend(get_all_methods(cx, into_par_iter_trait, recv));
                }

                let mut top_expr = *recv;
                let mut found_iter_method = false;
                let mut is_mut = false;

                while let Some(parent_expr) = get_parent_expr(cx, top_expr) {
                    match parent_expr.kind {
                        hir::ExprKind::MethodCall(method_name, _, _, _) => {
                            if ["into_iter", "iter", "iter_mut"]
                                .contains(&method_name.ident.as_str())
                            {
                                if found_iter_method {
                                    break;
                                }
                                found_iter_method = true;
                                if method_name.ident.as_str() == "iter_mut" {
                                    is_mut = true;
                                }
                            }
                            if !allowed_methods.contains(method_name.ident.as_str()) {
                                return;
                            }
                            top_expr = parent_expr;
                        }
                        hir::ExprKind::Closure(_) => top_expr = parent_expr,
                        _ => break,
                    }
                }

                // TODO: find a way to deal with iterators returns
                if check_trait_impl(cx, cx.typeck_results().expr_ty(top_expr), sym::Iterator) {
                    return;
                }

                let mut validator = Validator {
                    cx,
                    is_valid: true,
                    is_mut,
                };
                validator.visit_expr(top_expr);
                if !validator.is_valid {
                    return;
                }

                cx.span_lint(PAR_ITER, expr.span, |diag| {
                    diag.primary_message("found iterator that can be parallelized");
                    diag.multipart_suggestion(
                        "try using a parallel iterator",
                        vec![(expr.span, suggestion)],
                        Applicability::MachineApplicable,
                    );
                });
            }
        }
    }
}

struct Validator<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    is_valid: bool,
    is_mut: bool,
}

impl<'a, 'tcx> hir::intravisit::Visitor<'_> for Validator<'a, 'tcx> {
    fn visit_expr(&mut self, ex: &hir::Expr) {
        if let hir::ExprKind::MethodCall(_method_name, _receiver, args, _span) = ex.kind {
            if !self.is_valid {
                return;
            }
            let ex_ty = self.cx.typeck_results().expr_ty(ex);
            self.is_valid &= is_type_valid(self.cx, ex_ty);

            for arg in args {
                if let hir::ExprKind::Closure(closure) = arg.kind {
                    let mut params = hir::HirIdSet::default();
                    let body = self.cx.tcx.hir().body(closure.body);

                    for param in body.params {
                        if let hir::PatKind::Binding(_, hir_id, _, _) = param.pat.kind {
                            params.insert(hir_id);
                        }
                    }

                    self.is_valid &=
                        check_variables(self.cx, closure.def_id, body, &params, self.is_mut);
                }
            }
        }
        walk_expr(self, ex)
    }
}

fn get_all_methods<'tcx>(
    cx: &LateContext<'tcx>,
    into_iter_trait: hir::def_id::DefId,
    original_expr: &hir::Expr,
) -> Vec<&'tcx str> {
    let mut res = Vec::new();
    if let (Some(parallel_iterator_def_id), Some(parallel_indexed_iterator_def_id)) = (
        get_trait_def_id(cx.tcx, &["rayon", "iter", "ParallelIterator"]),
        get_trait_def_id(cx.tcx, &["rayon", "iter", "IndexedParallelIterator"]),
    ) {
        let tcx = cx.tcx;
        let infcx = tcx.infer_ctxt().build();
        let ocx = ObligationCtxt::new(&infcx);
        let param_env = tcx.param_env(into_iter_trait);

        // Create a new inference variable, ?new
        let ty = infcx.next_ty_var(original_expr.span);

        let projection = ty::Binder::dummy(ty::PredicateKind::Clause(ty::ClauseKind::Projection(
            ty::ProjectionPredicate {
                projection_term: ty::AliasTerm::new(tcx, into_iter_trait, GenericArgs::empty()),
                term: ty.into(),
            },
        )));

        let obligation = Obligation::new(tcx, ObligationCause::dummy(), param_env, projection);

        ocx.register_obligation(obligation);

        // let errors = ocx.select_where_possible();
        // if errors.is_empty() {
        //     dbg!("no errors"); // TODO: do something else here
        // }

        // TODO: use the previous steps to determine which ids should be run
        let ids = &[parallel_iterator_def_id, parallel_indexed_iterator_def_id];
        for def_id in ids {
            let associated_items = cx.tcx.associated_items(def_id);
            // Filter out only methods from the associated items
            let methods: Vec<&str> = associated_items
                .in_definition_order()
                .filter(|item| matches!(item.kind, ty::AssocKind::Fn))
                .map(|item| item.name.as_str())
                .collect();
            res.extend(methods);
        }
    }

    res
}

#[test]
fn ui() {
    dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
}
