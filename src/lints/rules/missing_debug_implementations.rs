use rustc_hir as hir;
use rustc_lint::{LateContext, LateLintPass, Level, LintContext};
use rustc_session::{declare_lint_pass, declare_tool_lint};
use rustc_span::symbol::sym;

declare_tool_lint! {
/// The `missing_debug_implementations` lint detects missing
    /// implementations of [`fmt::Debug`] for public types.
    ///
    /// [`fmt::Debug`]: https://doc.rust-lang.org/std/fmt/trait.Debug.html
    ///
    /// ### Example
    ///
    /// ```rust,compile_fail
    /// #![deny(missing_debug_implementations)]
    /// pub struct Foo;
    /// # fn main() {}
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// Having a `Debug` implementation on all types can assist with
    /// debugging, as it provides a convenient way to format and display a
    /// value. Using the `#[derive(Debug)]` attribute will automatically
    /// generate a typical implementation, or a custom implementation can be
    /// added by manually implementing the `Debug` trait.    pub lint::MISSING_DEBUG_IMPLEMENTATIONS,
    pub lint::MISSING_DEBUG_IMPLEMENTATIONS,
    Warn,
    "detects missing implementations of [`fmt::Debug`] for public types",
    report_in_external_macro: false
}

declare_lint_pass!(MissingDebugImplementations => [MISSING_DEBUG_IMPLEMENTATIONS]);

impl LateLintPass<'_> for MissingDebugImplementations {
    fn check_item(&mut self, cx: &LateContext<'_>, item: &hir::Item<'_>) {
        if !cx.effective_visibilities.is_reachable(item.owner_id.def_id) {
            return;
        }

        match item.kind {
            hir::ItemKind::Struct(..) | hir::ItemKind::Union(..) | hir::ItemKind::Enum(..) => {}
            _ => return,
        }

        // Avoid listing trait impls if the trait is allowed.
        let (level, _) = cx
            .tcx
            .lint_level_at_node(MISSING_DEBUG_IMPLEMENTATIONS, item.hir_id());
        if level == Level::Allow {
            return;
        }

        let Some(debug) = cx.tcx.get_diagnostic_item(sym::Debug) else {
            return;
        };

        let has_impl = cx
            .tcx
            .non_blanket_impls_for_ty(debug, cx.tcx.type_of(item.owner_id).instantiate_identity())
            .next()
            .is_some();

        if !has_impl {
            cx.struct_span_lint(
                MISSING_DEBUG_IMPLEMENTATIONS,
                item.span,
                "consider using a more meaningful DEBUG",
                |diag| diag,
            );
        }
    }
}
