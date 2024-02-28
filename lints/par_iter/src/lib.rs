#![feature(rustc_private)]
#![warn(unused_extern_crates)]
#![feature(let_chains)]

extern crate rustc_errors;
extern crate rustc_hash;
extern crate rustc_hir;
extern crate rustc_hir_typeck;
extern crate rustc_infer;
extern crate rustc_middle;
extern crate rustc_span;

mod constants;
mod utils;
mod variable_check;

use clippy_utils::get_parent_expr;
use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_hir::intravisit::Visitor;
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_middle::ty::{ty_kind::TyKind, Ty};
use utils::{check_implements_par_iter, generate_suggestion, is_type_valid};
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
    pub PAR_ITER,
    Warn,
    "suggest using par iter"
}

impl<'tcx> LateLintPass<'tcx> for ParIter {
    // TODO: implement check crate to check if rayon is present
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'_>) {
        if let hir::ExprKind::MethodCall(path, recv, _args, _span) = &expr.kind
            && let Some(suggestion) = generate_suggestion(cx, expr, path)
        {
            let ty = cx.typeck_results().expr_ty(recv);

            if check_implements_par_iter(cx, recv) && is_type_valid(cx, ty) {
                let mut top_expr = *recv;

                while let Some(parent_expr) = get_parent_expr(cx, top_expr) {
                    if let hir::ExprKind::MethodCall(_, _, _, _) = parent_expr.kind {
                        top_expr = parent_expr; // Save the previous expression
                    } else {
                        break; // Stop if the parent expression is not a method call
                    }
                }

                let ty: Ty<'_> = cx.typeck_results().expr_ty(top_expr);

                // TODO: this needs to change and find a better solutions for returns
                if let TyKind::Adt(_, _) = ty.kind() {
                    return;
                }

                let mut validator = Validator { cx, is_valid: true };
                validator.visit_expr(top_expr);
                if !validator.is_valid {
                    return;
                }

                cx.span_lint(
                    PAR_ITER,
                    expr.span,
                    "found iterator that can be parallelized",
                    |diag| {
                        diag.multipart_suggestion(
                            "try using a parallel iterator",
                            vec![(expr.span, suggestion)],
                            Applicability::MachineApplicable,
                        );
                    },
                );
            }
        }
    }
}

struct Validator<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    is_valid: bool,
}

struct ExprVisitor<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    // expr_scope: hir::HirId,
    is_valid: bool,
}

impl<'a, 'tcx> hir::intravisit::Visitor<'_> for ExprVisitor<'a, 'tcx> {
    fn visit_qpath(
        &mut self,
        qpath: &'_ hir::QPath<'_>,
        id: hir::HirId,
        _span: rustc_span::Span,
    ) -> Self::Result {
        if let hir::def::Res::Local(hir_id) = self.cx.typeck_results().qpath_res(qpath, id) {
            if let hir::Node::Pat(pat) = self.cx.tcx.hir_node(hir_id) {
                self.visit_pat(pat);
            }
            if let hir::Node::Local(l) = self.cx.tcx.parent_hir_node(hir_id) {
                self.visit_local(l)
            }
        }
    }
    fn visit_pat(&mut self, pat: &hir::Pat) -> Self::Result {
        if let hir::PatKind::Binding(hir::BindingAnnotation(_, hir::Mutability::Mut), _, _, _) =
            pat.kind
        {
            self.is_valid = false;
        }
    }
    fn visit_local(&mut self, l: &'_ hir::Local<'_>) -> Self::Result {
        // TODO: figure out the scope
        // if let Some(local_scope) = self.cx.tcx.hir().get_enclosing_scope(l.hir_id)
        //     && self.expr_scope == local_scope
        // {
        //     return;
        // }
        if let Some(expr) = l.init {
            self.is_valid &= is_type_valid(
                self.cx,
                self.cx.tcx.typeck(expr.hir_id.owner).node_type(expr.hir_id),
            );
            hir::intravisit::walk_expr(self, expr)
        }
    }
    fn visit_block(&mut self, b: &'_ hir::Block<'_>) -> Self::Result {
        for stmt in b.stmts {
            // TODO: deal with scope
            // if let Some(scope) = self.cx.tcx.hir().get_enclosing_scope(stmt.hir_id) {
            //     self.expr_scope = scope;
            // }
            self.visit_stmt(stmt);
        }
    }
    fn visit_stmt(&mut self, s: &'_ hir::Stmt<'_>) -> Self::Result {
        match s.kind {
            hir::StmtKind::Expr(e) => self.visit_expr(e),
            hir::StmtKind::Item(_) => {}
            hir::StmtKind::Local(l) => self.visit_local(l),
            hir::StmtKind::Semi(e) => self.visit_expr(e),
        }
    }

    fn visit_expr(&mut self, ex: &hir::Expr) {
        match ex.kind {
            hir::ExprKind::Closure(closure) => {
                let body = self.cx.tcx.hir().body(closure.body);
                // pub fn_decl: &'hir FnDecl<'hir>,
                if let hir::Node::Expr(expr) = self.cx.tcx.hir_node(closure.body.hir_id) {
                    if self.is_valid {
                        self.is_valid &= check_variables(self.cx, closure.def_id, body);
                    }
                    self.visit_expr(expr);
                    hir::intravisit::walk_expr(self, expr);
                }
            }
            hir::ExprKind::Block(b, _) => {
                self.visit_block(b);
            }
            hir::ExprKind::Path(ref qpath) => self.visit_qpath(qpath, ex.hir_id, qpath.span()),

            // check return type of call if matches
            hir::ExprKind::Call(func, args) => {
                self.visit_expr(func);
                args.iter().for_each(|arg| {
                    self.visit_expr(arg);
                });
            }
            hir::ExprKind::AssignOp(_op, target, value) => {
                self.visit_expr(target);
                self.visit_expr(value);
            }
            hir::ExprKind::MethodCall(_path_segment, receiver, args, _span) => {
                self.visit_expr(receiver);
                args.iter().for_each(|arg| {
                    self.visit_expr(arg);
                })
            }
            // TODO: handle other cases
            _ => hir::intravisit::walk_expr(self, ex),
        }
    }
}

impl<'a, 'tcx> hir::intravisit::Visitor<'_> for Validator<'a, 'tcx> {
    fn visit_expr(&mut self, ex: &hir::Expr) {
        if let hir::ExprKind::MethodCall(_method_name, _receiver, args, _span) = ex.kind {
            args.iter().for_each(|arg| {
                let mut expr_visitor = ExprVisitor {
                    cx: self.cx,
                    // expr_scope: self.cx.tcx.hir().get_enclosing_scope(arg.hir_id).unwrap(),
                    is_valid: true,
                };

                expr_visitor.visit_expr(arg);
                hir::intravisit::walk_expr(self, ex);
                self.is_valid &= expr_visitor.is_valid;
            })
        }
    }
}

#[test]
fn ui() {
    dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
}
