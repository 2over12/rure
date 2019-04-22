
use crate::exec::driver::analysis_passes::sir::Sir;
use rsmt2::Solver;
use super::sir::NodeId;


use super::sir::{Rator,Expr,SymTy};




use super::sir::MirVariableProp;


pub fn solve_sir(sir: &Sir, entry: NodeId, additional_constraints: Vec<Expr>) -> bool {
	let mut solver =  Solver::default(()).unwrap();

	for name in sir.get_all_names() {
		solver.declare_const(&name,sir.get_declaration(name)).unwrap();
	}

	for additional in additional_constraints.iter() {
		solver.assert(&additional).unwrap();
	}

	let res:String = sir.to_smt(entry);
	solver.assert(&res).unwrap();
	solver.check_sat().unwrap()
	
}
