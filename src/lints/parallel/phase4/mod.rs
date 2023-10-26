use clippy_utils::sym;
use rustc_hir::{Expr, ExprKind, HirId};
use rustc_span::{Span, Symbol};

pub mod fold;

// Traverse an iterator chain and rename all occurrences
// of sequential iterator calls to parallel ones.
struct IterRenaming {
    suggestions: Vec<(Span, String)>,
    seen: Vec<HirId>,
}

impl IterRenaming {
    fn new() -> Self {
        IterRenaming { suggestions: vec![], seen: vec![] }
    }

    fn traverse_iter_chain(&mut self, expr: &Expr) {
        if self.seen.contains(&expr.hir_id) {
            return;
        }
        self.seen.push(expr.hir_id);

        if let ExprKind::MethodCall(path, recv, args, _span) = &expr.kind {
            // TODO: Optimize this.
            let seq_names =
                vec![Symbol::intern("iter"),
                     Symbol::intern("iter_mut"),
                     Symbol::intern("into_iter")];
            let par_names =
                vec!["par_iter",
                     "par_iter_mut",
                     "into_par_iter"];
            for (sm, pm) in seq_names.into_iter().zip(par_names.into_iter()) {
                if path.ident.name == sm {
                    self.suggestions.push((path.ident.span, pm.to_string()));
                    break;
                }
            }
            self.traverse_iter_chain(recv);
            args.iter().for_each(|e| self.traverse_iter_chain(e));
        }
    }
}
