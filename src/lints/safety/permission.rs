use std::path::Path;

use rustc_hir::TyKind;
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_lint_defs::Applicability;
use rustc_session::{declare_tool_lint, impl_lint_pass};

use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Qualifiers<Qualifier> {
    pub fn_data: HashMap<String, HashMap<String, Vec<Qualifier>>>,
    pub struct_data: HashMap<String, HashMap<String, Vec<Qualifier>>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
pub enum Ownership {
    Owning,
    Transient,
    Unknown,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
/// [`Mutability::Mut`] ⊑ [`Mutability::Imm`]
pub enum Mutability {
    Imm,
    Mut,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
/// [`Fatness::Arr`] ⊑ [`Fatness::Ptr`]
pub enum Fatness {
    Arr,
    Ptr,
}

declare_tool_lint! {
    pub lint::RAW_POINTER_PERMISSION,
    Warn,
    "suggest if pointer permissions are understood",
    report_in_external_macro: false
}
impl_lint_pass!(RawPointerPermission => [RAW_POINTER_PERMISSION]);

#[allow(unused)]
pub struct RawPointerPermission {
    ownership: Qualifiers<Ownership>,
    mutability: Qualifiers<Mutability>,
    fatness: Qualifiers<Fatness>,
}

impl RawPointerPermission {
    pub fn new<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        RawPointerPermission {
            ownership: serde_json::from_str(
                &std::fs::read_to_string(&path.as_ref().join("ownership.json")).unwrap(),
            )
            .unwrap(),
            mutability: serde_json::from_str(
                &std::fs::read_to_string(&path.as_ref().join("mutability.json")).unwrap(),
            )
            .unwrap(),
            fatness: serde_json::from_str(
                &std::fs::read_to_string(&path.as_ref().join("fatness.json")).unwrap(),
            )
            .unwrap(),
        }
    }
}

// fn ownership_type_to_type_str(ty: &Ty, tcx: TyCtxt, ownership: &Vec<Ownership>) -> String {
//     let inner_span = {
//         let mut ty = ty;
//         loop {
//             let TyKind::Ptr(mut_ty) = ty.kind else {
//                 break ty.span;
//             };
//             ty = mut_ty.ty;
//         }
//     };
//     let inner_repr = tcx.sess.source_map().span_to_snippet(inner_span).unwrap();
//     let mut type_str = inner_repr;
//     for ownership in ownership.into_iter().rev() {
//         type_str = match ownership {
//             Ownership::Owning => format!("Option<Box<{}>>", type_str),
//             _ => format!("*mut {}", type_str),
//         }
//     }
//     type_str
// }

impl LateLintPass<'_> for RawPointerPermission {
    fn check_item(&mut self, cx: &LateContext<'_>, item: &'_ rustc_hir::Item<'_>) {
        if item.span.from_expansion() {
            return;
        }
        use rustc_hir::ItemKind::*;
        let def_id = item.owner_id.def_id.to_def_id();
        match item.kind {
            Fn(..) => {}
            Struct(variant_data, _) => {
                let struct_summary = self
                    .ownership
                    .struct_data
                    .get(&cx.tcx.def_path_str(def_id))
                    .unwrap();
                for field_def in variant_data.fields().iter() {
                    let field_summary = struct_summary.get(field_def.ident.as_str()).unwrap();
                    let ownership = field_summary;
                    if !ownership.is_empty()
                        && ownership
                            .iter()
                            .any(|&ownership| ownership == Ownership::Owning)
                    {
                        let mut suggestions = vec![];
                        let inner_span = {
                            let mut ty = field_def.ty;
                            loop {
                                let TyKind::Ptr(mut_ty) = ty.kind else {
                                    break ty.span;
                                };
                                ty = mut_ty.ty;
                            }
                        };
                        let inner_repr = cx
                            .tcx
                            .sess
                            .source_map()
                            .span_to_snippet(inner_span)
                            .unwrap();

                        let mut type_str = inner_repr;
                        for ownership in ownership.into_iter().rev() {
                            type_str = match ownership {
                                Ownership::Owning => format!("Option<Box<{}>>", type_str),
                                _ => format!("*mut {}", type_str),
                            }
                        }

                        suggestions.push((field_def.ty.span, type_str));

                        cx.struct_span_lint(
                            RAW_POINTER_PERMISSION,
                            field_def.span,
                            "pointer permissions".to_owned(),
                            |diag| {
                                diag.multipart_suggestion(
                                    "try using a better type",
                                    suggestions,
                                    Applicability::MaybeIncorrect,
                                )
                            },
                        );
                    }
                }
            }
            _ => {}
        }
    }
}
