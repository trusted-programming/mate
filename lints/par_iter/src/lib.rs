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

mod constants;
mod utils;
mod variable_check;

use clippy_utils::{get_parent_expr, get_trait_def_id};
use rustc_data_structures::fx::FxHashSet;
use rustc_errors::Applicability;
use rustc_hir::intravisit::Visitor;
use rustc_hir::{self as hir};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_middle::ty::{self, ty_kind::TyKind, Ty};
use rustc_span::sym;
use utils::{check_implements_par_iter, check_trait_impl, generate_suggestion, is_type_valid};
use variable_check::check_variables;

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
            let ty = cx.typeck_results().expr_ty(recv);

            let par_iter_traits = check_implements_par_iter(cx, recv);
            if !par_iter_traits.is_empty() && is_type_valid(cx, ty) {
                // TODO: issue with into_par_iter() need to check directly with
                // parallel iterator
                // let mut implemented_methods: Vec<&AssocItems> = Vec::new();

                let mut allowed_methods: FxHashSet<&str> = FxHashSet::default();
                allowed_methods.insert("into_iter");
                allowed_methods.insert("iter");
                allowed_methods.insert("iter_mut");

                if let Some(parallel_iterator_def_id) =
                    get_trait_def_id(cx, &["rayon", "iter", "ParallelIterator"])
                    && let Some(parallel_indexed_iterator_def_id) =
                        get_trait_def_id(cx, &["rayon", "iter", "IndexedParallelIterator"])
                {
                    let parallel_iterator_associated_items =
                        cx.tcx.associated_items(parallel_iterator_def_id);
                    // Filter out only methods from the associated items
                    let parallel_iterator_methods: Vec<_> = parallel_iterator_associated_items
                        .in_definition_order()
                        .filter(|item| matches!(item.kind, ty::AssocKind::Fn))
                        .map(|item| item.name.as_str())
                        .collect();

                    let parallel_indexed_iterator_associated_items =
                        cx.tcx.associated_items(parallel_indexed_iterator_def_id);
                    // Filter out only methods from the associated items
                    let parallel_indexed_iterator_methods: Vec<_> =
                        parallel_indexed_iterator_associated_items
                            .in_definition_order()
                            .filter(|item| matches!(item.kind, ty::AssocKind::Fn))
                            .map(|item| item.name.as_str())
                            .collect();
                    allowed_methods.extend(parallel_iterator_methods);
                    allowed_methods.extend(parallel_indexed_iterator_methods);
                } else {
                    return;
                }

                let mut top_expr = *recv;

                while let Some(parent_expr) = get_parent_expr(cx, top_expr) {
                    if let hir::ExprKind::MethodCall(method_name, _, _, _) = parent_expr.kind {
                        if !allowed_methods.contains(method_name.ident.as_str()) {
                            return;
                        }
                        top_expr = parent_expr;
                    } else {
                        break;
                    }
                }

                let ty: Ty<'_> = cx.typeck_results().expr_ty(top_expr);
                // TODO: find a way to deal with iterators returns
                if check_trait_impl(cx, ty, sym::Iterator) {
                    return;
                }

                // TODO: this needs to change and find a better solutions for returns
                if let TyKind::Adt(_, _) = ty.kind() {
                    return;
                }

                let mut validator = Validator { cx, is_valid: true };
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
}

impl<'a, 'tcx> hir::intravisit::Visitor<'_> for Validator<'a, 'tcx> {
    fn visit_expr(&mut self, ex: &hir::Expr) {
        if let hir::ExprKind::MethodCall(_method_name, _receiver, args, _span) = ex.kind {
            if !self.is_valid {
                return;
            }
            for arg in args {
                if let hir::ExprKind::Closure(closure) = arg.kind {
                    let body = self.cx.tcx.hir().body(closure.body);

                    self.is_valid &= check_variables(self.cx, closure.def_id, body);
                }
            }
        }
    }
}

#[test]
fn ui() {
    dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
}
