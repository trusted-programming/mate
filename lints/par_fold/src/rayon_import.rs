use rustc_hir::{ItemKind, UseKind};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_session::{declare_lint, declare_lint_pass};
use rustc_span::Span;

declare_lint! {
    /// ### What it does
    /// Checks if `rayon::prelude::*` is imported in the project.
    ///
    /// ### Why is this bad?
    /// Not importing `rayon::prelude::*` can lead to missed opportunities for
    /// parallelization and performance optimization when using Rayon.
    ///
    /// ### Known problems
    /// This lint does not check if Rayon is actually used, so it might suggest
    /// imports that are unnecessary.
    ///
    /// ### Example
    /// ```
    /// use rayon::prelude::*;
    /// ```

    pub RAYON_IMPORT,
    Warn,
    "check if rayon prelude is imported."
}

declare_lint_pass!(RayonImport => [RAYON_IMPORT]);

impl LateLintPass<'_> for RayonImport {
    fn check_crate(&mut self, cx: &LateContext<'_>) {
        let mut found_rayon_prelude = false;

        for item_id in cx.tcx.hir().items() {
            if let ItemKind::Use(path, UseKind::Glob) = &cx.tcx.hir().item(item_id).kind {
                if path
                    .segments
                    .iter()
                    .map(|seg| seg.ident.name.as_str())
                    .eq(["rayon", "prelude"])
                {
                    found_rayon_prelude = true;
                    break;
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
                        "#[allow(unused_imports)]\nuse rayon::prelude::*;\n".to_string(),
                        rustc_errors::Applicability::MachineApplicable,
                    )
                },
            );
        }
    }
}
