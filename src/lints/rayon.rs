use rustc_hir::{ItemKind, UseKind};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_session::{declare_lint_pass, declare_tool_lint};
use rustc_span::Span;

declare_tool_lint! {
    /// ### What it does
    ///
    /// ### Why is this bad?
    ///
    /// ### Known problems
    ///
    /// ### Example

    pub lint::RAYON_IMPORT,
    Warn,
    "check if rayon prelude is imported."
}
// TODO: add check for cargo.toml contains rayon

declare_lint_pass!(RayonImport => [RAYON_IMPORT]);
impl LateLintPass<'_> for RayonImport {
    fn check_crate(&mut self, cx: &LateContext<'_>) {
        let mut found_rayon_prelude = false;
        let mut last_use_span = None; // Keep track of the last `use` statement's span

        for item_id in cx.tcx.hir().items() {
            let item = cx.tcx.hir().item(item_id);
            match item.kind {
                ItemKind::Use(path, UseKind::Glob) => {
                    if path.segments.len() == 3 {
                        let first_segment = path.segments[0].ident.name.as_str();
                        let second_segment = path.segments[1].ident.name.as_str();
                        let third_segment = path.segments[2].ident.as_str();

                        if first_segment == "rayon"
                            && second_segment == "prelude"
                            && third_segment == "*"
                        {
                            found_rayon_prelude = true;
                            break;
                        }
                    }
                    last_use_span = Some(item.span); // Update the span for the last `use` statement
                }
                _ => {
                    if last_use_span.is_some() {
                        // We've found a non-`use` item and have seen at least one `use` item
                        break;
                    }
                }
            }
        }

        if !found_rayon_prelude {
            if let Some(span) = last_use_span {
                // Suggest adding the import after the last `use` statement
                let suggestion_span = Span::new(span.hi(), span.hi(), span.ctxt(), None);
                cx.struct_span_lint(
                    RAYON_IMPORT,
                    suggestion_span,
                    "rayon::prelude::* is not imported",
                    |diag| {
                        diag.span_suggestion(
                            suggestion_span,
                            "consider adding this import",
                            "\nuse rayon::prelude::*;".to_string(),
                            rustc_errors::Applicability::MachineApplicable,
                        )
                    },
                );
            }
        }
    }
}
