#![feature(rustc_private)]
#![warn(unused_extern_crates)]
#![feature(let_chains)]
#![feature(unwrap_infallible)]

extern crate rustc_errors;
extern crate rustc_hash;
extern crate rustc_hir;
extern crate rustc_hir_typeck;
extern crate rustc_middle;

mod variable_check;

use clippy_utils::higher::ForLoop;
use rustc_errors::Applicability;
use rustc_hir::{
    intravisit::{walk_expr, Visitor},
    Expr, ExprKind,
};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use utils::span_to_snippet_macro;
use variable_check::check_variables;

dylint_linting::declare_late_lint! {
    /// ### What it does
    /// parallelize iterators using rayon
    /// ### Why is this bad?
    /// parallel iters are often faster
    /// ### Known problems
    /// lots
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
    "suggest using for each"
}

impl<'tcx> LateLintPass<'tcx> for ForEach {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'_>) {
        if let Some(ForLoop {
            pat,
            arg,
            body,
            loop_id: _loop_id,
            span: _span,
        }) = ForLoop::hir(expr)
        {
            let src_map = cx.sess().source_map();

            // Make sure we ignore cases that require a try_foreach
            let mut validator = Validator {
                is_valid: true,
                has_continue: false,
            };
            validator.visit_expr(body);
            if !validator.is_valid || !check_variables(cx, body) {
                return;
            }
            // Check whether the iter is explicit
            // NOTE: since this is a syntax only check we are bound to miss cases.
            let mut explorer = IterExplorer::default();
            explorer.visit_expr(arg);
            let mc_snip: String = explorer.to_snip();

            let iter_snip = span_to_snippet_macro(src_map, arg.span);
            let pat_snip = span_to_snippet_macro(src_map, pat.span);
            let mut body_snip = span_to_snippet_macro(src_map, body.span);

            // TODO: this needs to be improved
            if validator.has_continue {
                body_snip = body_snip.replace("continue", "return");
            }
            cx.span_lint(FOR_EACH, expr.span, |diag| {
                diag.primary_message("use a for_each to enable iterator refinement");
                diag.multipart_suggestion(
                    "try using `for_each` on the iterator",
                    vec![(
                        expr.span,
                        format!("({iter_snip}){mc_snip}.for_each(|{pat_snip}| {body_snip});"),
                    )],
                    Applicability::MachineApplicable,
                );
            });
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
        if let ExprKind::MethodCall(path, expr, _expr_list, _span) = &ex.kind {
            // Get method identifier
            let mid = path.ident;
            // Check if it's an iter method
            // In theory, we could check all iter method names here.
            // Perhaps a hashset could be used.
            match mid.as_str() {
                "into_iter" | "iter" | "iter_mut" => self.is_iter = true,
                _ => {}
            }
            self.visit_expr(expr);
        }
    }
}

struct Validator {
    is_valid: bool,
    has_continue: bool,
}

impl Visitor<'_> for Validator {
    fn visit_expr(&mut self, ex: &Expr) {
        match &ex.kind {
            ExprKind::Loop(_, _, _, _)
            | ExprKind::Closure(_)
            | ExprKind::Ret(_)
            | ExprKind::Break(_, _) => self.is_valid = false,
            ExprKind::Continue(_) => self.has_continue = true,
            _ => walk_expr(self, ex),
        }
    }
}

#[test]
fn ui() {
    dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
}
