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

        for item_id in cx.tcx.hir().items() {
            let item = cx.tcx.hir().item(item_id);
            if let ItemKind::Use(path, UseKind::Glob) = &item.kind {
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
            }
        }

        if !found_rayon_prelude {
            // Get the span for the start of the crate
            let root_module = cx.tcx.hir().root_module();
            let inject_use_span = root_module.spans.inject_use_span;

            // Suggest adding the import at the start of the crate
            let suggestion_span = Span::new(
                inject_use_span.lo(),
                inject_use_span.lo(),
                inject_use_span.ctxt(),
                None,
            );

            cx.struct_span_lint(
                RAYON_IMPORT,
                suggestion_span,
                "rayon::prelude::* is not imported",
                |diag| {
                    diag.span_suggestion(
                        suggestion_span,
                        "consider adding this import",
                        "\nuse rayon::prelude::*;\n".to_string(),
                        rustc_errors::Applicability::MachineApplicable,
                    )
                },
            );
        }
    }
}
