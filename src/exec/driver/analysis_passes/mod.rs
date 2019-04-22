use crate::exec::driver::analysis_passes::smt::solve_sir;
use rustc::mir::Mir;
use rustc::ty::TyCtxt;
use rustc::hir::def_id::DefId;
use rustc_mir::transform::inline::Inline;
use rustc_mir::transform::MirSource;
use rustc::ty::InstanceDef;
use rustc_mir::transform::MirPass;
use std::collections::HashMap;
use self::sir::{MirVariableProp,Rator,Expr,SymTy};

mod symb_exec;
mod sir;
mod smt;

use symb_exec::ExecutionContext;

#[derive(PartialEq)]
pub struct ErrorInfo;

#[derive(PartialEq)]
pub enum PassResult {
	AssertiveOk,
	AssertiveError(ErrorInfo),
	Nondefinitive,
}


pub struct AnalysisHandler<'a,'tcx,'gcx> {
	start: DefId,
	code: Mir<'tcx>,
	ctx: &'a TyCtxt<'a,'gcx,'tcx>,
}

impl  <'a,'tcx,'gcx >AnalysisHandler<'a,'tcx, 'gcx> {
	pub fn new(start: DefId, ctx: &'a rustc::ty::TyCtxt<'_, 'tcx, 'gcx>) -> AnalysisHandler<'a,'tcx,'gcx> {
		let mut code = ctx.optimized_mir(start).clone();
		let liner = Inline {

		};

		let source = MirSource {
			instance: InstanceDef::Item(start),
			promoted: None,
		};
		liner.run_pass(*ctx,source, &mut code);
		AnalysisHandler {
			start,
			code,
			ctx,
		}
	}

	
	pub fn run_all_analyses(&self) -> Vec<ErrorInfo> {
			let mut mirs = HashMap::new();
			mirs.insert(self.start, &self.code);
			let (sir, entryid) = ExecutionContext::create_from_entry(self.start, mirs).evaluate();
			
			let vals = sir.get_all_names().filter_map(|x| if let Some(prop) = sir.get_declaration(x).get_property().first() {
				Some((x, prop))
			} else {
				None
			});

			for (interested_name, MirVariableProp::IsDerefed(nid) ) in vals {
				let assign = Expr::BinOp(Rator::Eq, Box::new(Expr::Ref(interested_name)), Box::new(Expr::Value(SymTy::Integer(0))));
				let pc = sir.get_path_constraint(*nid);
				let add = vec![pc,assign];
				println!("{}", solve_sir(&sir,entryid,add))
			}

			vec![]
	}
}