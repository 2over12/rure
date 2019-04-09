use rsmt2::Solver;
use rsmt2::print::{Sym2Smt,Sort2Smt};


use super::sir::Node;
use super::sir::Declaration;




fn solve_sir(decls: Vec<Declaration>, nd: Node) -> bool {
	let mut solver =  Solver::default(()).unwrap();
	for decl in &decls {
		solver.declare_const(decl,decl);
	}
}