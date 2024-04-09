#![feature(rustc_private)]
#![warn(unused_extern_crates)]
#![feature(let_chains)]

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

use clippy_utils::get_parent_expr;
use rustc_data_structures::fx::FxHashSet;
use rustc_errors::Applicability;
use rustc_hir::intravisit::{walk_expr, Visitor};
use rustc_hir::{self as hir, GenericArg};
use rustc_infer::infer::type_variable::{TypeVariableOrigin, TypeVariableOriginKind};
use rustc_infer::infer::TyCtxtInferExt;
use rustc_infer::traits::{Obligation, ObligationCause};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_middle::query::Key;
use rustc_middle::ty::{AliasTy, Binder, ProjectionPredicate};
use rustc_span::{sym, Span};
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
        if let hir::ExprKind::MethodCall(path, recv, _args, span) = &expr.kind
            && let Some(suggestion) = generate_suggestion(cx, expr, path)
        {
            let ty = cx.typeck_results().expr_ty(recv);
            let par_iter_traits = check_implements_par_iter(cx, recv);
            if !par_iter_traits.is_empty() && is_type_valid(cx, ty) {
                // TODO: issue with into_par_iter() need to check directly with
                // parallel iterator
                //
                let mut allowed_methods: FxHashSet<&str> =
                    ["into_iter", "iter", "iter_mut", "map_or"]
                        .into_iter()
                        .collect();
                for par_iter_trait in par_iter_traits {
                    allowed_methods.extend(get_methods(cx, par_iter_trait, ty, span));
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
use clippy_utils::ty::make_normalized_projection;

fn get_methods<'tcx>(
    cx: &LateContext<'tcx>,
    trait_def_id: hir::def_id::DefId,
    original_ty: rustc_middle::ty::Ty,
    span: Span,
) -> Vec<&'tcx str> {
    let res = Vec::new();

    let tcx = cx.tcx;
    let infcx = tcx.infer_ctxt().build();

    if let Some(ty_def_id) = original_ty.ty_def_id()
        && let param_env = tcx.param_env(ty_def_id)
        && let Some(projection) =
            make_normalized_projection(tcx, param_env, trait_def_id, sym::Item, vec![])
    {
        let cause = ObligationCause::dummy();
        let origin = TypeVariableOrigin {
            kind: TypeVariableOriginKind::TypeInference,
            span,
        };
        let projection_ty = infcx.next_ty_var(origin);

        let projection = ProjectionPredicate {
            projection_ty: AliasTy::new(tcx, trait_def_id, vec![]),
            term: tcx.mk_ty_var(tcx.next_ty_var_id()), // Or the specific type you expect the projection to equal.
        };

        let norm_ty = infcx.next_ty_var(TypeVariableOrigin {
            kind: TypeVariableOriginKind::TypeInference,
            span: cause.span,
        });

        // Create a projection obligation
        let obligation = Obligation::new(
            cause.clone(),
            param_env,
            projection.to_predicate(tcx, norm_ty),
        );

        let ocx = ObligationCtxt::new(&infcx);

        // FIXME: what is obligation
        ocx.register_obligation(obligation);
        let some_errors = ocx.select_where_possible();
    }
    // FIXME: what is assoc_ty

    res
}

#[test]
fn ui() {
    dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
}
