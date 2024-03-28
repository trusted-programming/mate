use clippy_utils::visitors::for_each_expr_with_closures;
use rustc_hash::FxHashSet;
use rustc_hir as hir;
use rustc_hir_typeck::expr_use_visitor as euv;
use rustc_infer::infer::{InferCtxt, TyCtxtInferExt};
use rustc_lint::LateContext;
use rustc_middle::{
    hir::map::associated_body,
    mir::FakeReadCause,
    ty::{self, Ty, UpvarId, UpvarPath},
};
use std::{collections::HashSet, ops::ControlFlow};

pub struct UsedVariablesCtxt<'tcx> {
    pub all_vars: FxHashSet<Ty<'tcx>>,
    pub copy_vars: FxHashSet<Ty<'tcx>>,
    mut_vars: FxHashSet<Ty<'tcx>>,
    borrow_vars: FxHashSet<Ty<'tcx>>,
    prev_bind: Option<hir::HirId>,
    /// In async functions, the inner AST is composed of multiple layers until we reach the code
    /// defined by the user. Because of that, some variables are marked as mutably borrowed even
    /// though they're not. This field lists the `HirId` that should not be considered as mutable
    /// use of a variable.
    prev_move_to_closure: hir::HirIdSet,
}

// TODO: remove repetation is this two function almost identical
pub fn check_variables<'tcx>(
    cx: &LateContext<'tcx>,
    ex: &'tcx hir::Expr,
) -> UsedVariablesCtxt<'tcx> {
    let body_owner = ex.hir_id.owner.def_id;

    let mut ctx = UsedVariablesCtxt {
        all_vars: FxHashSet::default(),
        copy_vars: FxHashSet::default(),
        borrow_vars: FxHashSet::default(),
        mut_vars: FxHashSet::default(),
        prev_bind: None,
        prev_move_to_closure: hir::HirIdSet::default(),
    };
    let infcx = cx.tcx.infer_ctxt().build();
    euv::ExprUseVisitor::new(
        &mut ctx,
        &infcx,
        body_owner,
        cx.param_env,
        cx.typeck_results(),
    )
    .walk_expr(ex);

    let mut checked_closures = FxHashSet::default();

    // We retrieve all the closures declared in the function because they will not be found
    // by `euv::Delegate`.
    let mut closures: FxHashSet<hir::def_id::LocalDefId> = FxHashSet::default();
    for_each_expr_with_closures(cx, ex, |expr| {
        if let hir::ExprKind::Closure(closure) = expr.kind {
            closures.insert(closure.def_id);
        }
        ControlFlow::<()>::Continue(())
    });
    check_closures(&mut ctx, cx, &infcx, &mut checked_closures, closures);

    ctx
}

pub fn check_closures<'tcx, S: ::std::hash::BuildHasher>(
    ctx: &mut UsedVariablesCtxt<'tcx>,
    cx: &LateContext<'tcx>,
    infcx: &InferCtxt<'tcx>,
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
            .opt_hir_node_by_def_id(closure)
            .and_then(associated_body)
            .map(|(_, body_id)| hir.body(body_id))
        {
            euv::ExprUseVisitor::new(ctx, infcx, closure, cx.param_env, cx.typeck_results())
                .consume_body(body);
        }
    }
}

impl<'tcx> euv::Delegate<'tcx> for UsedVariablesCtxt<'tcx> {
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
    fn borrow(&mut self, cmt: &euv::PlaceWithHirId<'tcx>, _: hir::HirId, _: ty::BorrowKind) {
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
            self.borrow_vars.insert(*base_ty);
        }
    }

    fn mutate(&mut self, cmt: &euv::PlaceWithHirId<'tcx>, _id: hir::HirId) {
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
            self.mut_vars.insert(*base_ty);
        }
    }
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
