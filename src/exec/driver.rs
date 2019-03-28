

use rustc::hir::Guard::If;
use rustc::hir::Arm;
use rustc::hir::Block;
use rustc::hir::Stmt;

use rustc::hir::Unsafety;
use rustc::hir::def_id::DefId;
use rustc_driver::{Callbacks};
use rustc_interface::interface::Compiler;
use rustc::hir::map::Map;
use rustc::hir::itemlikevisit::ItemLikeVisitor;
use rustc::hir::{Expr,HirVec,BlockCheckMode, StmtKind, ImplItemKind, ExprKind,Item, TraitItem, ImplItem,ItemKind, HirId,BodyId};
use std::collections::HashSet;



struct GetTcntx {

}

impl Callbacks for GetTcntx {
    fn after_analysis(&mut self, compiler: &Compiler) -> bool {
        compiler.session().abort_if_errors();
        compiler.global_ctxt().unwrap().peek_mut().enter(|tcx| {
            let _ids = collect_target_func_ids(tcx.hir());
            println!("{:?}",_ids);
        });

        compiler.session().abort_if_errors();

        false
    }
}



struct IdCollector<'a,'hir: 'a> {
    ids: HashSet<HirId>,
    comp_ctx: &'a Map<'hir>
}





fn statement_contains_unsafe(st: &Stmt) -> bool {
    match &st.node {
        StmtKind::Local(lcl) => if let Some(xp) = &lcl.init {
            node_contains_unsafe(&xp.node)
        } else {
            false
        }
        StmtKind::Expr(expr) => node_contains_unsafe(&expr.node),
        StmtKind::Semi(expr) => node_contains_unsafe(&expr.node),
        _ => false
    }
}

fn vec_contains_unsafe(exprs: &HirVec<Expr>) -> bool {
    exprs.iter().any(|x|node_contains_unsafe(&x.node))
}

fn block_contains_unsafe(blk: &Block) -> bool {
                blk.rules != BlockCheckMode::DefaultBlock
            || blk.stmts.iter().any(|x|statement_contains_unsafe(x)) ||
                if let Some(exp) = &blk.expr {
                    node_contains_unsafe(&exp.node)
                } else {
                    false
                }
}

fn arms_contain_unsafe(arms: &HirVec<Arm>) -> bool {
    arms.iter().map(|x|(x.body, x.guard)).any(|(bdy,grd)|node_contains_unsafe(&bdy.node) || if let Some(If(xp)) = &grd {
        node_contains_unsafe(&xp.node)
    } else {
        false
    })
}

fn node_contains_unsafe(ep: &ExprKind) -> bool {
    match ep {
        ExprKind::Block(blk,_) => {
            block_contains_unsafe(blk)
        } ,
        ExprKind::Box(pxp) => node_contains_unsafe(&pxp.node),
        ExprKind::Array(pxp) =>  vec_contains_unsafe(pxp),
        ExprKind::Call(_,pxp) => vec_contains_unsafe(pxp),
        ExprKind::MethodCall(_,_,pxp) => vec_contains_unsafe(pxp),
        ExprKind::Tup(pxp) => vec_contains_unsafe(pxp),
        ExprKind::Binary(_,f,s) => node_contains_unsafe(&f.node) || node_contains_unsafe(&s.node),
        ExprKind::Unary(_, xp) => node_contains_unsafe(&xp.node),
        ExprKind::Lit(_) => false,
        ExprKind::Cast(xp,_) => node_contains_unsafe(&xp.node),
        ExprKind::Type(_,_) => false,
        ExprKind::If(cond,bod,els) => node_contains_unsafe(&cond.node) || 
        node_contains_unsafe(&bod.node) || if let Some(x) = &els {
            node_contains_unsafe(&x.node)
        } else {
            false
        },
        ExprKind::While(_,blk,_) => block_contains_unsafe(blk),
        ExprKind::Loop(blk,_,_) => block_contains_unsafe(blk),
        ExprKind::Match(xp,arms,_) => node_contains_unsafe(&xp.node) || arms_contain_unsafe(arms),
        ExprKind::Closure(_,_,_,_,_) => false,
        ExprKind::Assign(fxp,sxp) => node_contains_unsafe(&fxp.node) || node_contains_unsafe(&sxp.node),
        ExprKind::AssignOp(_, fxp, sxp) => node_contains_unsafe(&fxp.node) || node_contains_unsafe(&sxp.node),
        ExprKind::Field(exp,_) => node_contains_unsafe(&exp.node),
        ExprKind::Index(fxp,sxp) => node_contains_unsafe(&fxp.node) || node_contains_unsafe(&sxp.node),
        ExprKind::Path(_) => false,
        ExprKind::AddrOf(_,exp) => node_contains_unsafe(&exp.node),
        ExprKind::Break(_,_) => false,
        ExprKind::InlineAsm(_,_,_) => true,
        
    }
}

fn contains_unsafe(bid: &BodyId, hirmap: &Map) -> bool {
    let b = hirmap.body(*bid);
    println!("{:?}",b);
    node_contains_unsafe(&b.value.node)
}
impl <'a, 'hir: 'a> ItemLikeVisitor<'hir> for IdCollector<'a,'hir> {
    fn visit_item(&mut self, item: &'hir Item) {
        if let ItemKind::Fn(_,hdr,_,bid) = item.node {
            if hdr.unsafety == Unsafety::Normal && contains_unsafe(&bid, &self.comp_ctx) {
                self.ids.insert(item.hir_id);
            }
        } 

    }

    fn visit_trait_item(&mut self, _trait_item: &'hir TraitItem) {

    }

    fn visit_impl_item(&mut self, impl_item: &'hir ImplItem) {
            if let ImplItemKind::Method(sig,bid) = &impl_item.node {
            if sig.header.unsafety == Unsafety::Normal && contains_unsafe(&bid, &self.comp_ctx) {
                self.ids.insert(impl_item.hir_id);
            }
        } 
    }
}

impl <'a,'hir> IdCollector<'a, 'hir> {
    fn new(m: &'a Map<'hir>) -> IdCollector<'a,'hir> {
        IdCollector {
            ids: HashSet::new(),
            comp_ctx: m
        }
    }

    fn get_ids(mut self) -> Vec<DefId> {
        let mut hids: Vec<HirId> = self.ids.drain().collect();
        hids.drain(..).map(|x|self.comp_ctx.local_def_id_from_hir_id(x)).collect()
    } 
}

fn collect_target_func_ids(code: &Map) -> Vec<DefId> {
    let mut v =  IdCollector::new(code);
    code.krate().visit_all_item_likes(&mut v);
    v.get_ids()
}

fn find_sysroot() -> String {
    if let Ok(sysroot) = std::env::var("MIRI_SYSROOT") {
        return sysroot;
    }

    // Taken from PR <https://github.com/Manishearth/rust-clippy/pull/911>.
    let home = option_env!("RUSTUP_HOME").or(option_env!("MULTIRUST_HOME"));
    let toolchain = option_env!("RUSTUP_TOOLCHAIN").or(option_env!("MULTIRUST_TOOLCHAIN"));
    match (home, toolchain) {
        (Some(home), Some(toolchain)) => format!("{}/toolchains/{}", home, toolchain),
        _ => {
            option_env!("RUST_SYSROOT")
            .expect(
                "could not find sysroot. Either set `MIRI_SYSROOT` at run-time, or at \
                build-time specify `RUST_SYSROOT` env var or use rustup or multirust",
            )
            .to_owned()
        }
    }
}

pub fn run_executor(mut rustc_args: Vec<String>) {
    let sysroot_flag = String::from("--sysroot");
    if !rustc_args.contains(&sysroot_flag) {
        rustc_args.push(sysroot_flag);
        rustc_args.push(find_sysroot());
    }
    rustc_driver::run_compiler(&rustc_args,&mut GetTcntx{},None,None);
}
