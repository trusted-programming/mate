use rustc_hir::{intravisit::Visitor, Item, ItemKind, UseKind};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_session::{declare_lint, declare_lint_pass};
use rustc_span::{FileName, FileNameDisplayPreference};

declare_lint! {
    /// ### What it does
    /// Checks the rayon iterator trait are import.
    ///
    /// ### Why is this bad?
    /// Not importing can lead to missed opportunities for
    /// parallelization and performance optimization when using Rayon.
    ///
    /// ### Known problems
    /// This lint does not check if Rayon is actually used, so it might suggest
    /// imports that are unnecessary. but can be fixed using cargo fix.
    ///
    /// ### Example
    /// ```
    /// use rayon::iter::{IntoParallelIterator, ParallelIterator};
    /// ```

    pub RAYON_PRELUDE,
    Warn,
    "check if use statements for iterators are present."
}

declare_lint_pass!(RayonPrelude => [RAYON_PRELUDE]);

impl LateLintPass<'_> for RayonPrelude {
    fn check_mod(
        &mut self,
        cx: &LateContext<'_>,
        md: &'_ rustc_hir::Mod<'_>,
        _hir_id: rustc_hir::HirId,
    ) {
        // Skip linting if in build.rs file
        if let FileName::Real(f) = cx
            .sess()
            .source_map()
            .span_to_filename(cx.tcx.hir().root_module().spans.inner_span)
        {
            if f.to_string_lossy(FileNameDisplayPreference::Short)
                .ends_with("build.rs")
            {
                return;
            }
        }

        let hir = cx.tcx.hir();
        let mut use_statement_visitor = UseStatementVisitor { has_import: false };
        let module_item_ids = md.item_ids;
        for item_id in module_item_ids {
            let item = hir.item(*item_id);
            use_statement_visitor.visit_item(item);
        }

        // @todo remove #[allow(unused_imports)] and only add the import when it is going to be used
        let import_suggestion = "#[allow(unused_imports)]\nuse rayon::prelude::*;\n".to_string();
        let inject_use_span = md.spans.inject_use_span;
        if !use_statement_visitor.has_import {
            cx.span_lint(RAYON_PRELUDE, inject_use_span, |diag| {
                diag.primary_message("rayon::prelude::* is not imported");
                diag.span_suggestion(
                    inject_use_span,
                    "consider adding this import",
                    import_suggestion,
                    rustc_errors::Applicability::MachineApplicable,
                );
            });
        }
    }
}

struct UseStatementVisitor {
    has_import: bool,
}

impl<'v> Visitor<'v> for UseStatementVisitor {
    fn visit_item(&mut self, item: &'v Item<'v>) {
        if let ItemKind::Use(use_path, UseKind::Glob) = item.kind {
            let path: Vec<String> = use_path
                .segments
                .to_vec()
                .iter()
                .map(|x| x.ident.to_string())
                .collect();

            let trait_path = vec!["rayon".to_string(), "prelude".to_string()];
            self.has_import |= trait_path == path
        }
        // Continue walking the rest of the tree
        rustc_hir::intravisit::walk_item(self, item);
    }
}
