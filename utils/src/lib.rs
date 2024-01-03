#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_span;

use rustc_hir::{Expr, ExprKind, Stmt, StmtKind};
use rustc_span::source_map::SourceMap;
use rustc_span::{Span, SyntaxContext};

pub fn is_local_def(stmt: &Stmt) -> bool {
    match stmt.kind {
        StmtKind::Local(_) => true,
        StmtKind::Expr(e) | StmtKind::Semi(e) => {
            if let ExprKind::Block(b, _) = e.kind {
                b.stmts.iter().all(is_local_def) && b.expr.is_none()
            } else {
                false
            }
        }
        _ => false,
    }
}

pub fn get_pat_expr_and_spans<'a>(
    expr: &'a Expr<'a>,
) -> Result<(&'a Expr<'a>, Option<Span>, Option<Span>), ()> {
    let mut local_defs_span = None;
    let mut body_span = None;
    let pat_expr = if let ExprKind::Block(block, _) = &expr.kind {
        match block.stmts.len() {
            0 => {
                if block.expr.is_none() {
                    return Err(());
                }
                block.expr.unwrap()
            }
            _ => {
                let mut local_defs = vec![];
                let mut body = vec![];
                let mut add_locals = true;
                for s in block.stmts.iter() {
                    if is_local_def(s) & add_locals {
                        local_defs.push(s.span);
                    } else {
                        add_locals = false;
                        body.push(s);
                    }
                }
                if !local_defs.is_empty() {
                    let fst_span = local_defs[0];
                    let lst_span = local_defs[local_defs.len() - 1];
                    local_defs_span = Some(fst_span.to(lst_span));
                }
                if body.is_empty() {
                    match block.expr {
                        None => return Err(()),
                        Some(expr) => expr,
                    }
                } else {
                    match body.remove(0).kind {
                        StmtKind::Expr(e) | StmtKind::Semi(e) => {
                            if !body.is_empty() {
                                let fst_span = body[0].span;
                                let lst_span = match block.expr {
                                    None => body[body.len() - 1].span,
                                    Some(e) => e.span,
                                };
                                body_span = Some(fst_span.to(lst_span));
                            } else {
                                body_span = block.expr.map(|e| e.span);
                            }
                            e
                        }
                        _ => return Err(()),
                    }
                }
            }
        }
    } else {
        expr
    };
    Ok((pat_expr, local_defs_span, body_span))
}

pub fn span_to_snippet_macro(src_map: &SourceMap, span: Span) -> String {
    if span.ctxt() == SyntaxContext::root() {
        // It's not a macro, proceed as usual
        src_map.span_to_snippet(span).unwrap()
    } else {
        // TODO: Handle the macro case
        // The combined_span originates from a macro expansion
        // You might need a different approach to handle this case
        let callsite_span = span.source_callsite();
        src_map
            .span_to_snippet(callsite_span)
            .unwrap_or_else(|_| String::new())
    }
}
