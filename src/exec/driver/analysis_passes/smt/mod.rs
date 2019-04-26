
use rustc::mir::Local;
use rustc::hir::def_id::DefId;
use rustc::session::config::Input;
use rsmt2::parse::IdentParser;
use rsmt2::parse::ModelParser;
use crate::exec::driver::analysis_passes::sir::Sir;
use rsmt2::Solver;
use super::sir::NodeId;
use std::collections::HashMap;
use rsmt2::errors::SmtRes;
use rsmt2::print::Expr2Smt;

use super::sir::{Rator,Expr,SymTy,Name};




use super::sir::MirVariableProp;


pub fn solve_sir(sir: &Sir, entry: NodeId, additional_constraints: Vec<Expr>) -> Option<HashMap<(DefId,Local),SymTy>> {
	let mut solver =  Solver::default(SirParser).unwrap();

	for name in sir.get_all_names() {
		solver.declare_const(&name,sir.get_declaration(name)).unwrap();
	}

	for additional in additional_constraints.iter() {
		solver.assert(&additional).unwrap();
	}

	let res:String = sir.to_smt(entry);
	solver.assert(&res).unwrap();
	
	if solver.check_sat().unwrap() {
		Some(solver.get_model().unwrap().into_iter().filter_map(|(name,_,_,val)| {
			if let Some(loc) = sir.get_declaration(name).get_location() {
				Some((loc.clone(),val))
			} else {
				None
			}
		}).collect())
	} else {
		None
	}
}

#[derive(Clone,Copy)]
struct SirParser;

impl <'a> ModelParser<Name,SymTy,SymTy,&'a str> for SirParser {
	fn parse_value(self, i: &'a str, _id: &Name, _pair: &[(Name, SymTy)], ty: &SymTy) -> SmtRes<SymTy> {
		Ok(match ty {
			SymTy::Integer(_) => SymTy::Integer(i.parse().unwrap()),
			SymTy::Bool(_) => SymTy::Bool(i.parse().unwrap())

		})
	}
}

impl <'a> IdentParser<Name,SymTy,&'a str> for SirParser {
	fn parse_ident(self, i: &'a str)  -> SmtRes<Name> {
		Ok(Name::from_str(i))
	}

	fn parse_type(self, i: &'a str)  -> SmtRes<SymTy> {
		Ok(match i {
			"Int" => SymTy::Integer(0),
			"Bool" => SymTy::Bool(false),
			_ => panic!("oops")
		})
	}
}

