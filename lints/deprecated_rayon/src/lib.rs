#![feature(rustc_private)]
#![warn(unused_extern_crates)]
#![feature(let_chains)]

extern crate rustc_errors;
extern crate rustc_hir;

use clippy_utils::get_parent_expr;
use rustc_errors::Applicability;
use rustc_hir::{self as hir};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use utils::span_to_snippet_macro;

dylint_linting::declare_late_lint! {
    /// ### What it does
    ///
    /// ### Why is this bad?
    ///
    /// ### Known problems
    ///
    ///
    /// ### Example
    /// ```rust
    ///
    /// ```
    /// Use instead:
    /// ```rust
    ///
    ///
    ///
    /// ```
    pub DEPRECATED_RAYON,
    Warn,
    "suggest replacing a deprecated method"
}

impl<'tcx> LateLintPass<'tcx> for DeprecatedRayon {
    // TODO: implement check crate to check if rayon is present
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'_>) {
        if let hir::ExprKind::MethodCall(path, recv, _args, _span) = &expr.kind
            && ["par_iter", "into+par_iter", "par_iter_mut"].contains(&path.ident.as_str())
        {
            let mut top_expr = *recv;
            while let Some(parent_expr) = get_parent_expr(cx, top_expr) {
                match parent_expr.kind {
                    hir::ExprKind::MethodCall(method_name, _, _, _) => {
                        top_expr = parent_expr;
                        let snippet =
                            span_to_snippet_macro(cx.sess().source_map(), parent_expr.span);
                        let suggestion_text = match method_name.ident.as_str() {
                            "find" => Some(snippet.replace(".find(", ".find_first(")),
                            "position" => Some(snippet.replace(".position(", ".position_first(")),
                            _ => None,
                        };

                        if let Some(suggestion) = suggestion_text {
                            cx.span_lint(
                                DEPRECATED_RAYON,
                                top_expr.span,
                                "found a deprecated rayon method",
                                |diag| {
                                    diag.span_suggestion(
                                        parent_expr.span,
                                        "try use this instead",
                                        suggestion,
                                        Applicability::MachineApplicable,
                                    );
                                },
                            );
                        }
                    }
                    _ => break,
                }
            }
        }
    }
}

#[test]
fn ui() {
    dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
}
