use rustc::mir::Mir;
use rustc::ty::TyCtxt;
use rustc::hir::def_id::DefId;
use rustc_mir::transform::inline::Inline;
use rustc_mir::transform::MirSource;
use rustc::ty::InstanceDef;
use rustc_mir::transform::MirPass;

mod symb_exec;
mod sir;
mod smt;


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
			let sir = symb_exec::eval_mir(&self.code);
			
			println!("{:?}",smt::solve_sir(sir));

			vec![]
	}
}