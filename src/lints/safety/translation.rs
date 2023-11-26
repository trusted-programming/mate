use std::{collections::HashMap, path::Path};

use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_lint_defs::Applicability;
use rustc_session::{declare_tool_lint, impl_lint_pass};
use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct TranslationSummary {
    pub fn_data: HashMap<String, String>,
    pub struct_data: HashMap<String, String>,
}

declare_tool_lint! {
    pub lint::RAW_POINTER_TRANSLATION,
    Warn,
    "suggest translation",
    report_in_external_macro: false
}
impl_lint_pass!(RawPointerTranslation => [RAW_POINTER_TRANSLATION]);

pub struct RawPointerTranslation {
    translation: TranslationSummary,
}

impl RawPointerTranslation {
    pub fn new<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        RawPointerTranslation {
            translation: serde_json::from_str(
                &std::fs::read_to_string(&path.as_ref().join("translation.json")).unwrap(),
            )
            .unwrap(),
        }
    }
}

impl LateLintPass<'_> for RawPointerTranslation {
    fn check_item(&mut self, cx: &LateContext<'_>, item: &'_ rustc_hir::Item<'_>) {
        if item.span.from_expansion() {
            return;
        }
        use rustc_hir::ItemKind::*;
        let def_id = item.owner_id.def_id.to_def_id();
        match item.kind {
            Fn(..) => {
                let translation = self
                    .translation
                    .fn_data
                    .get(&cx.tcx.def_path_str(def_id))
                    .unwrap_or_else(|| panic!("failed to find {}", &cx.tcx.def_path_str(def_id)));
                if translation.contains("Box") || translation.contains("&mut") {
                    let mut suggestions = vec![];
                    suggestions.push((item.span, translation.to_owned()));
                    cx.struct_span_lint(
                        RAW_POINTER_TRANSLATION,
                        item.span,
                        "refactor suggestions".to_owned(),
                        |diag| {
                            diag.multipart_suggestion(
                                "try refactor the entire function",
                                suggestions,
                                Applicability::MachineApplicable,
                            )
                        },
                    );
                }
            }
            Struct(..) => {
                let Some(translation) = self
                    .translation
                    .struct_data
                    .get(&cx.tcx.def_path_str(def_id))
                else {
                    return;
                };
                if translation.contains("Box") || translation.contains("&mut") {
                    let mut suggestions = vec![];
                    suggestions.push((item.span, translation.to_owned()));
                    cx.struct_span_lint(
                        RAW_POINTER_TRANSLATION,
                        item.span,
                        "refactor suggestions".to_owned(),
                        |diag| {
                            diag.multipart_suggestion(
                                "try retype the struct",
                                suggestions,
                                Applicability::MachineApplicable,
                            )
                        },
                    );
                }
            }
            _ => {}
        }
    }
}
