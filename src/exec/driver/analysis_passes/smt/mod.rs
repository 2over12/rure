use rsmt2::Solver;



use super::sir::Node;
use super::sir::Declaration;

use rsmt2::print::Sort2Smt;
use rsmt2::print::Sym2Smt;
use rsmt2::print::Expr2Smt;
use std::io::Write;

pub fn solve_sir(decls: Vec<Declaration>, nd: Node) -> bool {
	let mut solver =  Solver::default(()).unwrap();
	let mut out = std::io::stdout();
	for decl in &decls {
		write!(&mut out,"(declare-fun ");
		decl.sort_to_smt2(&mut out);
		write!(&mut out,"() ");
		decl.sym_to_smt2(&mut out, ());
		write!(&mut out," )");
		write!(&mut out, "\n");
		solver.declare_const(decl,decl).unwrap();
	}

	nd.expr_to_smt2(&mut out, ());
	out.flush();
	solver.assert(&nd).unwrap();


	solver.check_sat().unwrap()
}