use clippy_utils::visitors::for_each_expr;
use rustc_hash::FxHashSet;
use rustc_hir::{self as hir, def_id::LocalDefId};
use rustc_hir_typeck::expr_use_visitor::{self as euv};
use rustc_lint::LateContext;
use rustc_middle::{
    mir::FakeReadCause,
    ty::{self, Ty, UpvarId, UpvarPath},
};
use std::{collections::HashSet, ops::ControlFlow};

pub struct MutablyUsedVariablesCtxt<'tcx> {
    all_vars: FxHashSet<Ty<'tcx>>,
    copy_vars: FxHashSet<Ty<'tcx>>,
    prev_bind: Option<hir::HirId>,
    /// In async functions, the inner AST is composed of multiple layers until we reach the code
    /// defined by the user. Because of that, some variables are marked as mutably borrowed even
    /// though they're not. This field lists the `HirId` that should not be considered as mutable
    /// use of a variable.
    prev_move_to_closure: hir::HirIdSet,
}

// TODO: remove repetation is this two function almost identical
pub fn check_variables<'tcx>(cx: &LateContext<'tcx>, ex: &'tcx hir::Expr) -> bool {
    let MutablyUsedVariablesCtxt {
        mut all_vars,
        copy_vars,
        ..
    } = {
        let body_owner = ex.hir_id.owner.def_id;

        let mut ctx = MutablyUsedVariablesCtxt {
            all_vars: FxHashSet::default(),
            copy_vars: FxHashSet::default(),
            prev_bind: None,
            prev_move_to_closure: hir::HirIdSet::default(),
        };

        euv::ExprUseVisitor::for_clippy(cx, body_owner, &mut ctx)
            .walk_expr(ex)
            .into_ok();
        let mut checked_closures = FxHashSet::default();

        // We retrieve all the closures declared in the function because they will not be found
        // by `euv::Delegate`.
        let mut closures: FxHashSet<LocalDefId> = FxHashSet::default();
        for_each_expr(cx, ex, |expr| {
            if let hir::ExprKind::Closure(closure) = expr.kind {
                closures.insert(closure.def_id);
            }
            ControlFlow::<()>::Continue(())
        });
        check_closures(&mut ctx, cx, &mut checked_closures, closures);

        ctx
    };
    all_vars.retain(|var| !copy_vars.contains(var));
    all_vars.is_empty()
}

pub fn check_closures<'tcx, S: ::std::hash::BuildHasher>(
    ctx: &mut MutablyUsedVariablesCtxt<'tcx>,
    cx: &LateContext<'tcx>,
    checked_closures: &mut HashSet<hir::def_id::LocalDefId, S>,
    closures: HashSet<hir::def_id::LocalDefId, S>,
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
            .hir_node_by_def_id(closure)
            .associated_body()
            .map(|(_, body_id)| hir.body(body_id))
        {
            euv::ExprUseVisitor::for_clippy(cx, closure, &mut *ctx)
                .consume_body(body)
                .into_ok();
        }
    }
}

impl<'tcx> euv::Delegate<'tcx> for MutablyUsedVariablesCtxt<'tcx> {
    #[allow(clippy::if_same_then_else)]
    fn consume(&mut self, cmt: &euv::PlaceWithHirId<'tcx>, _: hir::HirId) {
        if let euv::Place {
            base:
                euv::PlaceBase::Local(_)
                | euv::PlaceBase::Upvar(UpvarId {
                    var_path: UpvarPath { hir_id: _ },
                    ..
                }),
            base_ty,
            ..
        } = &cmt.place
        {
            self.all_vars.insert(*base_ty);
        }
    }

    #[allow(clippy::if_same_then_else)]
    fn borrow(&mut self, _: &euv::PlaceWithHirId<'tcx>, _: hir::HirId, _: ty::BorrowKind) {}

    fn mutate(&mut self, _: &euv::PlaceWithHirId<'tcx>, _id: hir::HirId) {}
    fn copy(&mut self, cmt: &euv::PlaceWithHirId<'tcx>, _: hir::HirId) {
        if let euv::Place {
            base:
                euv::PlaceBase::Local(_)
                | euv::PlaceBase::Upvar(UpvarId {
                    var_path: UpvarPath { hir_id: _ },
                    ..
                }),
            base_ty,
            ..
        } = &cmt.place
        {
            self.copy_vars.insert(*base_ty);
        }
    }
    fn fake_read(
        &mut self,
        _: &rustc_hir_typeck::expr_use_visitor::PlaceWithHirId<'tcx>,
        _: FakeReadCause,
        _id: hir::HirId,
    ) {
    }

    fn bind(&mut self, _: &euv::PlaceWithHirId<'tcx>, id: hir::HirId) {
        self.prev_bind = Some(id);
    }
}
