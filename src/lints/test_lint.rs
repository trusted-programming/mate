use rustc_tools::rustc_ast;
use rustc_tools::rustc_hir::def_id::LocalDefId;
use rustc_tools::rustc_hir::intravisit::FnKind;
use rustc_tools::rustc_hir::{Body, FnDecl};
use rustc_tools::rustc_lint::{
    EarlyContext, EarlyLintPass, LateContext, LateLintPass, LintContext,
};
use rustc_tools::rustc_session::{declare_lint_pass, declare_tool_lint};
use rustc_tools::rustc_span::Span;

declare_tool_lint! {
    // `lint` is the name of the binary here. It's required when creating a lint.
    pub lint::WARN_GENERICS,
    Warn,
    "warns if any item has generics",
    report_in_external_macro: false
}
declare_lint_pass!(WarnGenerics => [WARN_GENERICS]);

// `WarnGenerics` was declared in `declare_lint_pass`.
impl EarlyLintPass for WarnGenerics {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &rustc_ast::Item) {
        if let Some(generics) = item.kind.generics() {
            if generics.params.is_empty() {
                return;
            }
            cx.struct_span_lint(WARN_GENERICS, generics.span, "generics are ugly", |diag| {
                diag
            });
        }
    }
}

declare_tool_lint! {
    // `lint` is the name of the binary here. It's required when creating a lint.
    pub lint::ODD_FUNCTION_LINE_COUNT,
    Deny,
    "errors if a function has an odd number of lines",
    report_in_external_macro: false
}
declare_lint_pass!(OddFunctionLineCount => [ODD_FUNCTION_LINE_COUNT]);

// `OddFunctionLineCount` was declared in `declare_lint_pass`.
impl LateLintPass<'_> for OddFunctionLineCount {
    fn check_fn(
        &mut self,
        cx: &LateContext<'_>,
        _: FnKind<'_>,
        _: &FnDecl<'_>,
        body: &Body<'_>,
        span: Span,
        _: LocalDefId,
    ) {
        if span.from_expansion() {
            // If this function was generated from a macro, we don't want to check it.
            return;
        }
        // In here, to get the number of lines in the function, we don't take the function's
        // `Span` but its body's instead.
        if let Ok(code) = cx.sess().source_map().span_to_snippet(body.value.span) {
            // We need to remove the parens (`{}`) from the string.
            let code = &code[1..code.len() - 1];
            if !code.is_empty() && code.split('\n').count() % 2 != 0 {
                // This is an odd number of lines, we don't allow that!
                cx.struct_span_lint(
                    ODD_FUNCTION_LINE_COUNT,
                    span,
                    "functions with odd number of lines should not exist",
                    |diag| diag,
                );
            }
        }
    }
}
