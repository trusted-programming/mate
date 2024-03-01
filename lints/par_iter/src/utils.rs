use clippy_utils::{get_trait_def_id, ty::implements_trait};
use rustc_hir as hir;
use rustc_lint::{LateContext, LintContext};
use rustc_middle::ty::{GenericArg, GenericArgKind, Ty};
use rustc_span::{sym, Symbol};

use crate::constants::{TRAIT_PATHS, TRAIT_REF_PATHS};

pub(crate) fn check_implements_par_iter<'tcx>(
    cx: &'tcx LateContext,
    expr: &'tcx hir::Expr<'_>,
) -> Vec<hir::def_id::DefId> {
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
    let mut implemented_traits = Vec::new();

    for trait_path in TRAIT_PATHS {
        if let Some(trait_def_id) = get_trait_def_id(cx, trait_path)
            && implements_trait(cx, ty, trait_def_id, &[])
        {
            implemented_traits.push(trait_def_id);
        }
    }
    for trait_path in TRAIT_REF_PATHS {
        if let Some(trait_def_id) = get_trait_def_id(cx, trait_path)
            && implements_trait(cx, ty, trait_def_id, &[lt])
        {
            implemented_traits.push(trait_def_id);
        }
    }
    implemented_traits
}

fn check_trait_impl<'tcx>(cx: &LateContext<'tcx>, ty: Ty<'tcx>, trait_name: Symbol) -> bool {
    cx.tcx
        .get_diagnostic_item(trait_name)
        .map_or(false, |trait_id| implements_trait(cx, ty, trait_id, &[]))
}

pub(crate) fn is_type_valid<'tcx>(cx: &LateContext<'tcx>, ty: Ty<'tcx>) -> bool {
    let is_send = check_trait_impl(cx, ty, sym::Send);
    let is_sync = check_trait_impl(cx, ty, sym::Sync);
    let is_copy = check_trait_impl(cx, ty, sym::Copy);
    is_copy || (is_send && is_sync)
}

pub(crate) fn generate_suggestion(
    cx: &LateContext<'_>,
    expr: &hir::Expr<'_>,
    path: &hir::PathSegment,
) -> Option<String> {
    let method_name = &*path.ident.name.to_string();
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
            .map_or_else(|_| None, |s| Some(s.replace(method_name, r)))
    } else {
        None
    }
}
