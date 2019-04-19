use crate::exec::driver::analysis_passes::sir::Sir;
use rsmt2::Solver;



use super::sir::{Rator,Expr,SymTy};


use rsmt2::print::Sort2Smt;
use rsmt2::print::Sym2Smt;
use rsmt2::print::Expr2Smt;
use std::io::Write;

pub fn solve_sir(sir: Sir) -> bool {
	let mut solver =  Solver::default(()).unwrap();
	let mut out = std::io::stdout();
	for decl in sir.get_decls() {
		write!(&mut out,"(declare-fun ");
		decl.sym_to_smt2(&mut out, ());
		write!(&mut out," () ");
		decl.sort_to_smt2(&mut out);
		write!(&mut out," )");
		write!(&mut out, "\n");
		solver.declare_const(decl,decl).unwrap();

		if let Some(props) = sir.get_decl_prop(decl) {
			let n_expr = Expr::BinOp(Rator::Eq,Box::new(decl.to_expr()),Box::new(Expr::Value(SymTy::Integer(0))));
		}
	}

	let nd = sir.get_parent();
	nd.expr_to_smt2(&mut out, ());
	out.flush();
	solver.assert(&nd).unwrap();


	solver.check_sat().unwrap()
}