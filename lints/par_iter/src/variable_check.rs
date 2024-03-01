use std::ops::ControlFlow;

use clippy_utils::visitors::for_each_expr_with_closures;
use rustc_hash::FxHashSet;
use rustc_hir as hir;
use rustc_hir_typeck::expr_use_visitor as euv;
use rustc_infer::infer::{InferCtxt, TyCtxtInferExt};
use rustc_lint::LateContext;
use rustc_middle::{
    hir::map::associated_body,
    mir::FakeReadCause,
    ty::{self, TyCtxt, UpvarId, UpvarPath},
};

pub(crate) struct MutablyUsedVariablesCtxt<'tcx> {
    mutably_used_vars: hir::HirIdSet,
    prev_bind: Option<hir::HirId>,
    /// In async functions, the inner AST is composed of multiple layers until we reach the code
    /// defined by the user. Because of that, some variables are marked as mutably borrowed even
    /// though they're not. This field lists the `HirId` that should not be considered as mutable
    /// use of a variable.
    prev_move_to_closure: hir::HirIdSet,
    aliases: hir::HirIdMap<hir::HirId>,
    async_closures: FxHashSet<hir::def_id::LocalDefId>,
    tcx: TyCtxt<'tcx>,
}

pub(crate) fn check_variables<'tcx>(
    cx: &LateContext<'tcx>,
    body_owner: hir::def_id::LocalDefId,
    body: &hir::Body<'tcx>,
) -> bool {
    let MutablyUsedVariablesCtxt {
        mutably_used_vars, ..
    } = {
        let mut ctx = MutablyUsedVariablesCtxt {
            mutably_used_vars: hir::HirIdSet::default(),
            prev_bind: None,
            prev_move_to_closure: hir::HirIdSet::default(),
            aliases: hir::HirIdMap::default(),
            async_closures: FxHashSet::default(),
            tcx: cx.tcx,
        };
        let infcx = cx.tcx.infer_ctxt().build();
        euv::ExprUseVisitor::new(
            &mut ctx,
            &infcx,
            body_owner,
            cx.param_env,
            cx.typeck_results(),
        )
        .consume_body(body);

        let mut checked_closures = FxHashSet::default();

        // We retrieve all the closures declared in the function because they will not be found
        // by `euv::Delegate`.
        let mut closures: FxHashSet<hir::def_id::LocalDefId> = FxHashSet::default();
        for_each_expr_with_closures(cx, cx.tcx.hir().body(body.id()).value, |expr| {
            if let hir::ExprKind::Closure(closure) = expr.kind {
                closures.insert(closure.def_id);
            }
            ControlFlow::<()>::Continue(())
        });
        check_closures(&mut ctx, cx, &infcx, &mut checked_closures, closures);

        ctx
    };
    mutably_used_vars.is_empty()
}

pub(crate) fn check_closures<'tcx>(
    ctx: &mut MutablyUsedVariablesCtxt<'tcx>,
    cx: &LateContext<'tcx>,
    infcx: &InferCtxt<'tcx>,
    checked_closures: &mut FxHashSet<hir::def_id::LocalDefId>,
    closures: FxHashSet<hir::def_id::LocalDefId>,
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
    fn add_mutably_used_var(&mut self, mut used_id: hir::HirId) {
        while let Some(id) = self.aliases.get(&used_id) {
            self.mutably_used_vars.insert(used_id);
            used_id = *id;
        }
        self.mutably_used_vars.insert(used_id);
    }

    fn would_be_alias_cycle(&self, alias: hir::HirId, mut target: hir::HirId) -> bool {
        while let Some(id) = self.aliases.get(&target) {
            if *id == alias {
                return true;
            }
            target = *id;
        }
        false
    }

    fn add_alias(&mut self, alias: hir::HirId, target: hir::HirId) {
        // This is to prevent alias loop.
        if alias == target || self.would_be_alias_cycle(alias, target) {
            return;
        }
        self.aliases.insert(alias, target);
    }

    // The goal here is to find if the current scope is unsafe or not. It stops when it finds
    // a function or an unsafe block.
    fn is_in_unsafe_block(&self, item: hir::HirId) -> bool {
        let hir = self.tcx.hir();
        for (parent, node) in hir.parent_iter(item) {
            if let Some(fn_sig) = hir.fn_sig_by_hir_id(parent) {
                return fn_sig.header.is_unsafe();
            } else if let hir::Node::Block(block) = node {
                if matches!(block.rules, hir::BlockCheckMode::UnsafeBlock(_)) {
                    return true;
                }
            }
        }
        false
    }
}

impl<'tcx> euv::Delegate<'tcx> for MutablyUsedVariablesCtxt<'tcx> {
    #[allow(clippy::if_same_then_else)]
    fn consume(&mut self, cmt: &euv::PlaceWithHirId<'tcx>, id: hir::HirId) {
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
                && matches!(base_ty.ref_mutability(), Some(ty::Mutability::Mut))
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
    fn borrow(&mut self, cmt: &euv::PlaceWithHirId<'tcx>, id: hir::HirId, borrow: ty::BorrowKind) {
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
                    && base_ty.ref_mutability() == Some(ty::Mutability::Mut))
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
            if let hir::Node::Expr(hir::Expr {
                kind:
                    hir::ExprKind::Call(
                        _,
                        [hir::Expr {
                            kind: hir::ExprKind::Closure(hir::Closure { def_id, .. }),
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

    fn mutate(&mut self, cmt: &euv::PlaceWithHirId<'tcx>, _id: hir::HirId) {
        self.prev_bind = None;
        if let euv::Place {
            projections: _,
            base:
                euv::PlaceBase::Local(vid)
                | euv::PlaceBase::Upvar(UpvarId {
                    var_path: UpvarPath { hir_id: vid },
                    ..
                }),
            ..
        } = &cmt.place
        {
            self.add_mutably_used_var(*vid);
        }
    }

    fn copy(&mut self, cmt: &euv::PlaceWithHirId<'tcx>, id: hir::HirId) {
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
        _id: hir::HirId,
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

    fn bind(&mut self, cmt: &euv::PlaceWithHirId<'tcx>, id: hir::HirId) {
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
