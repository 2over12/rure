use rustc::mir::Mir;
use rustc::ty::TyCtxt;
use rustc::hir::def_id::DefId;


mod symb_exec;
mod sir;


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
	code: &'a Mir<'tcx>,
	ctx: &'a TyCtxt<'a,'gcx,'tcx>,
}

impl  <'a,'tcx,'gcx >AnalysisHandler<'a,'tcx, 'gcx> {
	pub fn new(start: DefId, ctx: &'a TyCtxt<'_, 'gcx, 'tcx>) -> AnalysisHandler<'a,'tcx,'gcx> {
		AnalysisHandler {
			start,
			code: ctx.optimized_mir(start),
			ctx,
		}
	}

	
	pub fn run_all_analyses(&self) -> Vec<ErrorInfo> {
			let (node, decls) = symb_exec::eval_mir(self.code);

			vec![]
	}
}