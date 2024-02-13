#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;
extern crate rustc_errors;

use rustc_ast::ast::{Expr, ExprKind};
use rustc_ast::visit::{walk_expr, Visitor};
use rustc_errors::Applicability;
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use utils::span_to_snippet_macro;

dylint_linting::declare_early_lint! {
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
    pub FOR_EACH,
    Warn,
    "it warns that a for loop can be replaced by a for_each"
}

#[derive(Default)]
struct Validator {
    is_invalid: bool,
    is_async: bool,
}

impl Visitor<'_> for Validator {
    fn visit_expr(&mut self, ex: &Expr) {
        match &ex.kind {
            ExprKind::ForLoop(_, _, _, _)
            | ExprKind::Loop(_, _, _)
            | ExprKind::Closure(_)
            | ExprKind::Try(_)
            | ExprKind::Ret(_)
            | ExprKind::Break(_, _) => self.is_invalid = true,
            ExprKind::Await(e, _) => {
                self.is_async = true;
                self.visit_expr(e)
            }

            _ => walk_expr(self, ex),
        }
    }
}

#[derive(Default)]
struct IterExplorer {
    is_iter: bool,
}

impl IterExplorer {
    fn to_snip(&self) -> String {
        if self.is_iter {
            String::new()
        } else {
            ".into_iter()".to_string()
        }
    }
}

impl Visitor<'_> for IterExplorer {
    fn visit_expr(&mut self, ex: &'_ Expr) {
        match &ex.kind {
            ExprKind::MethodCall(mc) => {
                // Get method identifier
                let mid = mc.seg.ident;
                // Check if it's an iter method
                // In theory, we could check all iter method names here.
                // Perhaps a hashset could be used.
                match mid.as_str() {
                    "into_iter" | "iter" | "iter_mut" => self.is_iter = true,
                    _ => {}
                }
                self.visit_expr(&mc.receiver);
            }
            _ => {}
        }
    }
}

impl EarlyLintPass for ForEach {
    fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &Expr) {
        // Match on for loop expressions
        if let ExprKind::ForLoop(pat, iter, block, _) = &expr.kind {
            // Make sure we ignore cases that require a try_foreach
            let mut validator = Validator::default();
            validator.visit_block(block);
            validator.visit_expr(iter);
            if validator.is_invalid || validator.is_async {
                return;
            }

            // Check whether the iter is explicit
            // NOTE: since this is a syntax only check we are bound to miss cases.
            let mut explorer = IterExplorer::default();
            explorer.visit_expr(iter);
            let mc_snip = explorer.to_snip();

            let src_map = cx.sess().source_map();
            let iter_snip = span_to_snippet_macro(src_map, iter.span);
            let pat_snip = span_to_snippet_macro(src_map, pat.span);
            let block_snip = span_to_snippet_macro(src_map, block.span);

            // This could be handled better
            let block_snip = block_snip.replace("continue", "return");

            let suggestion = format!(
                "({}){}.for_each(|{}| {});",
                iter_snip, mc_snip, pat_snip, block_snip
            );

            cx.struct_span_lint(
                FOR_EACH,
                expr.span,
                "use a for_each to enable iterator refinement",
                |diag| {
                    diag.span_suggestion(
                        expr.span,
                        "try using `for_each` on the iterator",
                        suggestion,
                        Applicability::MachineApplicable,
                    )
                },
            );
        }
    }
}

#[test]
fn ui() {
    dylint_testing::ui_test(
        env!("CARGO_PKG_NAME"),
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui"),
    );
}
