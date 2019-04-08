

mod analysis_passes;
use analysis_passes::AnalysisHandler;

use rustc::hir::Block;


use rustc::hir::Unsafety;
use rustc::hir::def_id::DefId;
use rustc_driver::{Callbacks};
use rustc_interface::interface::Compiler;
use rustc::hir::map::Map;
use rustc::hir::itemlikevisit::{ItemLikeVisitor};
use rustc::hir::intravisit::{Visitor,FnKind,NestedVisitorMap};
use rustc::hir::{BlockCheckMode, ImplItemKind,Item, TraitItem, ImplItem,ItemKind, HirId};
use std::collections::HashSet;
use rustc::hir::intravisit;


struct GetTcntx {

}

impl Callbacks for GetTcntx {
    fn after_analysis(&mut self, compiler: &Compiler) -> bool {
        compiler.session().abort_if_errors();
        compiler.global_ctxt().unwrap().peek_mut().enter(|tcx| {
            let ids = collect_target_func_ids(tcx.hir());

            for id in ids {
                let pass_handler = analysis_passes::AnalysisHandler::new(id, &tcx);
                let errors = pass_handler.run_all_analyses();
                //compiler.session().abort();

            }
        });

        compiler.session().abort_if_errors();

        false
    }
}



struct ContainsUsafe<'v,'tcx> {
    state: bool,
    ctx: &'v Map<'tcx>
}

impl <'tcx,'v> intravisit::Visitor<'v> for ContainsUsafe<'v,'tcx> {

    fn nested_visit_map<'this>(&'this mut self) -> NestedVisitorMap<'this,'v> {
        NestedVisitorMap::OnlyBodies(self.ctx)
    }

    fn visit_block(&mut self, b: &Block) {
        if b.rules != BlockCheckMode::DefaultBlock {
            self.state = true;
        } else {
            let mut v = ContainsUsafe::new(self.ctx);
            intravisit::walk_block(&mut v, b);
            self.state = self.state || v.consume();
        }
    }
}

impl <'b,'y> ContainsUsafe<'b,'y> {
    fn new<'a,'ctx>(ctx: &'a Map<'ctx>) -> ContainsUsafe<'a,'ctx> {
        ContainsUsafe {
            ctx,
            state: false
        }
    }

    fn consume(self) -> bool {
        self.state
    }
}



struct IdCollector<'a,'hir: 'a> {
    ids: HashSet<HirId>,
    comp_ctx: &'a Map<'hir>
}


impl <'a, 'hir: 'a> ItemLikeVisitor<'hir> for IdCollector<'a,'hir> {
    fn visit_item(&mut self, item: &'hir Item) {
        if let ItemKind::Fn(decl,hdr,gen,bid) = &item.node {
            if hdr.unsafety == Unsafety::Normal {
                println!("{:?}",decl);
                let mut v =  ContainsUsafe::new(&self.comp_ctx);
                v.visit_fn(FnKind::ItemFn(item.ident, &gen,*hdr,&item.vis,&item.attrs),&decl,*bid,item.span,item.hir_id);
                if v.consume() {
                    self.ids.insert(item.hir_id);
                }
            }
        } 

    }

    fn visit_trait_item(&mut self, _trait_item: &'hir TraitItem) {

    }

    fn visit_impl_item(&mut self, impl_item: &'hir ImplItem) {
            if let ImplItemKind::Method(sig,bid) = &impl_item.node {
            if sig.header.unsafety == Unsafety::Normal {
                let mut v =  ContainsUsafe::new(&self.comp_ctx);
                v.visit_fn(FnKind::Method(impl_item.ident, &sig,Some(&impl_item.vis),&impl_item.attrs),&sig.decl,*bid,impl_item.span,impl_item.hir_id);
                if v.consume() {
                    self.ids.insert(impl_item.hir_id);
                }
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

    rustc_driver::run_compiler(&rustc_args,&mut GetTcntx{},None,None).unwrap();
}
