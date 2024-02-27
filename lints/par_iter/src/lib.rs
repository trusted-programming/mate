#![feature(rustc_private)]
#![warn(unused_extern_crates)]
#![feature(let_chains)]

extern crate rustc_ast;
extern crate rustc_errors;
extern crate rustc_hash;
extern crate rustc_hir;
extern crate rustc_hir_typeck;
extern crate rustc_infer;
extern crate rustc_middle;
extern crate rustc_span;

mod constants;
mod utils;

use std::ops::ControlFlow;

use clippy_utils::{get_parent_expr, visitors::for_each_expr_with_closures};
use rustc_ast::Mutability;
use rustc_errors::Applicability;
use rustc_hash::FxHashSet;
use rustc_hir::{
    self as hir,
    def::Res,
    def_id::LocalDefId,
    intravisit::{walk_expr, Visitor},
    BlockCheckMode, Closure, Expr, ExprKind, HirId, HirIdMap, HirIdSet, Node,
};
use rustc_hir_typeck::expr_use_visitor as euv;
use rustc_infer::infer::{InferCtxt, TyCtxtInferExt};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_middle::{
    hir::map::associated_body,
    mir::FakeReadCause,
    ty::{self, ty_kind::TyKind, Ty, TyCtxt, UpvarId, UpvarPath},
};
use utils::{check_implements_par_iter, generate_suggestion, is_type_valid};

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

impl<'a, 'tcx> Visitor<'_> for ExprVisitor<'a, 'tcx> {
    fn visit_qpath(
        &mut self,
        qpath: &'_ hir::QPath<'_>,
        id: hir::HirId,
        _span: rustc_span::Span,
    ) -> Self::Result {
        if let Res::Local(hir_id) = self.cx.typeck_results().qpath_res(qpath, id) {
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
            walk_expr(self, expr)
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
                        // Collect variables mutably used and spans which will need dereferencings from the
                        // function body.
                        let MutablyUsedVariablesCtxt {
                            mutably_used_vars, ..
                        } = {
                            let mut ctx = MutablyUsedVariablesCtxt {
                                mutably_used_vars: HirIdSet::default(),
                                prev_bind: None,
                                prev_move_to_closure: HirIdSet::default(),
                                aliases: HirIdMap::default(),
                                async_closures: FxHashSet::default(),
                                tcx: self.cx.tcx,
                            };
                            let infcx = self.cx.tcx.infer_ctxt().build();
                            euv::ExprUseVisitor::new(
                                &mut ctx,
                                &infcx,
                                closure.def_id,
                                self.cx.param_env,
                                self.cx.typeck_results(),
                            )
                            .consume_body(body);

                            let mut checked_closures = FxHashSet::default();

                            // We retrieve all the closures declared in the function because they will not be found
                            // by `euv::Delegate`.
                            let mut closures: FxHashSet<LocalDefId> = FxHashSet::default();
                            for_each_expr_with_closures(self.cx, body, |expr| {
                                if let ExprKind::Closure(closure) = expr.kind {
                                    closures.insert(closure.def_id);
                                }
                                ControlFlow::<()>::Continue(())
                            });
                            check_closures(
                                &mut ctx,
                                self.cx,
                                &infcx,
                                &mut checked_closures,
                                closures,
                            );

                            ctx
                        };
                        self.is_valid &= mutably_used_vars.is_empty()
                    }
                    self.visit_expr(expr);
                    walk_expr(self, expr);
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
            _ => walk_expr(self, ex),
        }
    }
}

impl<'a, 'tcx> Visitor<'_> for Validator<'a, 'tcx> {
    fn visit_expr(&mut self, ex: &hir::Expr) {
        if let hir::ExprKind::MethodCall(_method_name, _receiver, args, _span) = ex.kind {
            args.iter().for_each(|arg| {
                let mut expr_visitor = ExprVisitor {
                    cx: self.cx,
                    // expr_scope: self.cx.tcx.hir().get_enclosing_scope(arg.hir_id).unwrap(),
                    is_valid: true,
                };

                expr_visitor.visit_expr(arg);
                walk_expr(self, ex);
                self.is_valid &= expr_visitor.is_valid;
            })
        }
    }
}
struct MutablyUsedVariablesCtxt<'tcx> {
    mutably_used_vars: HirIdSet,
    prev_bind: Option<HirId>,
    /// In async functions, the inner AST is composed of multiple layers until we reach the code
    /// defined by the user. Because of that, some variables are marked as mutably borrowed even
    /// though they're not. This field lists the `HirId` that should not be considered as mutable
    /// use of a variable.
    prev_move_to_closure: HirIdSet,
    aliases: HirIdMap<HirId>,
    async_closures: FxHashSet<LocalDefId>,
    tcx: TyCtxt<'tcx>,
}

fn check_closures<'tcx>(
    ctx: &mut MutablyUsedVariablesCtxt<'tcx>,
    cx: &LateContext<'tcx>,
    infcx: &InferCtxt<'tcx>,
    checked_closures: &mut FxHashSet<LocalDefId>,
    closures: FxHashSet<LocalDefId>,
) {
    let hir = cx.tcx.hir();
    for closure in closures {
        if !checked_closures.insert(closure) {
            continue;
        }
        ctx.prev_bind = None;
        ctx.prev_move_to_closure.clear();
        if let Some(body) = cx
            .tcx
            .opt_hir_node_by_def_id(closure)
            .and_then(associated_body)
            .map(|(_, body_id)| hir.body(body_id))
        {
            euv::ExprUseVisitor::new(ctx, infcx, closure, cx.param_env, cx.typeck_results())
                .consume_body(body);
        }
    }
}
impl<'tcx> MutablyUsedVariablesCtxt<'tcx> {
    fn add_mutably_used_var(&mut self, mut used_id: HirId) {
        while let Some(id) = self.aliases.get(&used_id) {
            self.mutably_used_vars.insert(used_id);
            used_id = *id;
        }
        self.mutably_used_vars.insert(used_id);
    }

    fn would_be_alias_cycle(&self, alias: HirId, mut target: HirId) -> bool {
        while let Some(id) = self.aliases.get(&target) {
            if *id == alias {
                return true;
            }
            target = *id;
        }
        false
    }

    fn add_alias(&mut self, alias: HirId, target: HirId) {
        // This is to prevent alias loop.
        if alias == target || self.would_be_alias_cycle(alias, target) {
            return;
        }
        self.aliases.insert(alias, target);
    }

    // The goal here is to find if the current scope is unsafe or not. It stops when it finds
    // a function or an unsafe block.
    fn is_in_unsafe_block(&self, item: HirId) -> bool {
        let hir = self.tcx.hir();
        for (parent, node) in hir.parent_iter(item) {
            if let Some(fn_sig) = hir.fn_sig_by_hir_id(parent) {
                return fn_sig.header.is_unsafe();
            } else if let Node::Block(block) = node {
                if matches!(block.rules, BlockCheckMode::UnsafeBlock(_)) {
                    return true;
                }
            }
        }
        false
    }
}

impl<'tcx> euv::Delegate<'tcx> for MutablyUsedVariablesCtxt<'tcx> {
    #[allow(clippy::if_same_then_else)]
    fn consume(&mut self, cmt: &euv::PlaceWithHirId<'tcx>, id: HirId) {
        if let euv::Place {
            base:
                euv::PlaceBase::Local(vid)
                | euv::PlaceBase::Upvar(UpvarId {
                    var_path: UpvarPath { hir_id: vid },
                    ..
                }),
            base_ty,
            ..
        } = &cmt.place
        {
            if let Some(bind_id) = self.prev_bind.take() {
                if bind_id != *vid {
                    self.add_alias(bind_id, *vid);
                }
            } else if !self.prev_move_to_closure.contains(vid)
                && matches!(base_ty.ref_mutability(), Some(Mutability::Mut))
            {
                self.add_mutably_used_var(*vid);
            } else if self.is_in_unsafe_block(id) {
                // If we are in an unsafe block, any operation on this variable must not be warned
                // upon!
                self.add_mutably_used_var(*vid);
            }
            self.prev_bind = None;
            // FIXME(rust/#120456) - is `swap_remove` correct?
            self.prev_move_to_closure.swap_remove(vid);
        }
    }

    #[allow(clippy::if_same_then_else)]
    fn borrow(&mut self, cmt: &euv::PlaceWithHirId<'tcx>, id: HirId, borrow: ty::BorrowKind) {
        self.prev_bind = None;
        if let euv::Place {
            base:
                euv::PlaceBase::Local(vid)
                | euv::PlaceBase::Upvar(UpvarId {
                    var_path: UpvarPath { hir_id: vid },
                    ..
                }),
            base_ty,
            ..
        } = &cmt.place
        {
            // If this is a mutable borrow, it was obviously used mutably so we add it. However
            // for `UniqueImmBorrow`, it's interesting because if you do: `array[0] = value` inside
            // a closure, it'll return this variant whereas if you have just an index access, it'll
            // return `ImmBorrow`. So if there is "Unique" and it's a mutable reference, we add it
            // to the mutably used variables set.
            if borrow == ty::BorrowKind::MutBorrow
                || (borrow == ty::BorrowKind::UniqueImmBorrow
                    && base_ty.ref_mutability() == Some(Mutability::Mut))
            {
                self.add_mutably_used_var(*vid);
            } else if self.is_in_unsafe_block(id) {
                // If we are in an unsafe block, any operation on this variable must not be warned
                // upon!
                self.add_mutably_used_var(*vid);
            }
        } else if borrow == ty::ImmBorrow {
            // If there is an `async block`, it'll contain a call to a closure which we need to
            // go into to ensure all "mutate" checks are found.
            if let Node::Expr(Expr {
                kind:
                    ExprKind::Call(
                        _,
                        [Expr {
                            kind: ExprKind::Closure(Closure { def_id, .. }),
                            ..
                        }],
                    ),
                ..
            }) = self.tcx.hir_node(cmt.hir_id)
            {
                self.async_closures.insert(*def_id);
            }
        }
    }

    fn mutate(&mut self, cmt: &euv::PlaceWithHirId<'tcx>, _id: HirId) {
        self.prev_bind = None;
        if let euv::Place {
            projections,
            base:
                euv::PlaceBase::Local(vid)
                | euv::PlaceBase::Upvar(UpvarId {
                    var_path: UpvarPath { hir_id: vid },
                    ..
                }),
            ..
        } = &cmt.place
        {
            if !projections.is_empty() {
                self.add_mutably_used_var(*vid);
            }
        }
    }

    fn copy(&mut self, cmt: &euv::PlaceWithHirId<'tcx>, id: HirId) {
        if let euv::Place {
            base:
                euv::PlaceBase::Local(vid)
                | euv::PlaceBase::Upvar(UpvarId {
                    var_path: UpvarPath { hir_id: vid },
                    ..
                }),
            ..
        } = &cmt.place
        {
            if self.is_in_unsafe_block(id) {
                self.add_mutably_used_var(*vid);
            }
        }
        self.prev_bind = None;
    }

    fn fake_read(
        &mut self,
        cmt: &rustc_hir_typeck::expr_use_visitor::PlaceWithHirId<'tcx>,
        cause: FakeReadCause,
        _id: HirId,
    ) {
        if let euv::Place {
            base:
                euv::PlaceBase::Upvar(UpvarId {
                    var_path: UpvarPath { hir_id: vid },
                    ..
                }),
            ..
        } = &cmt.place
        {
            if let FakeReadCause::ForLet(Some(inner)) = cause {
                // Seems like we are inside an async function. We need to store the closure `DefId`
                // to go through it afterwards.
                self.async_closures.insert(inner);
                self.add_alias(cmt.hir_id, *vid);
                self.prev_move_to_closure.insert(*vid);
                self.prev_bind = None;
            }
        }
    }

    fn bind(&mut self, cmt: &euv::PlaceWithHirId<'tcx>, id: HirId) {
        self.prev_bind = Some(id);
        if let euv::Place {
            base:
                euv::PlaceBase::Local(vid)
                | euv::PlaceBase::Upvar(UpvarId {
                    var_path: UpvarPath { hir_id: vid },
                    ..
                }),
            ..
        } = &cmt.place
        {
            if self.is_in_unsafe_block(id) {
                self.add_mutably_used_var(*vid);
            }
        }
    }
}

#[test]
fn ui() {
    dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
}
