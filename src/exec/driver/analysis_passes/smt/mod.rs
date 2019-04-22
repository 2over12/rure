
use crate::exec::driver::analysis_passes::sir::Sir;
use rsmt2::Solver;
use super::sir::NodeId;


use super::sir::{Rator,Expr,SymTy};




use super::sir::MirVariableProp;


pub fn solve_sir(sir: &Sir, entry: NodeId, additional_constraints: Vec<Expr>) -> bool {
	let mut solver =  Solver::default(()).unwrap();
	//let mut out = std::io::stdout();
	
/*	let vals = sir.get_all_names().filter_map(|x| if let Some(prop) = sir.get_declaration(x).get_property().first() {
		Some((x, prop))
	} else {
		None
	});

	for (interested_name, MirVariableProp::IsDerefed(nid) ) in vals {
		let mut solver =  Solver::default(()).unwrap();
		for name in sir.get_all_names() {
			solver.declare_const(&name,sir.get_declaration(name)).unwrap();

		}


		let res:String = sir.to_smt(entry);
		solver.assert(&Expr::BinOp(Rator::Eq, Box::new(Expr::Ref(interested_name)), Box::new(Expr::Value(SymTy::Integer(0))))).unwrap();

		
		solver.assert(&sir.get_path_constraint(*nid)).unwrap();
		

		solver.assert(&res).unwrap();

		println!("{:?}",solver.check_sat().unwrap());
	};*/

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
