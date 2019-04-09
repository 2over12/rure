use rsmt2::SmtRes;
pub use super::symb_exec::Name;
use rustc::ty::{Ty, TyKind};

use rustc::mir;

#[derive(Debug)]
pub struct Node {
	precondition: Option<Expr>,
	statements: Vec<Expr>,
	sucessors: Vec<Node>
}

impl Node {
	pub fn new(precondition: Option<Expr>, statements: Vec<Expr>, sucessors: Vec<Node>) -> Node {
		Node {
			precondition: precondition,
			statements: statements,
			sucessors: sucessors
		}
	}

	pub fn insert_at_leaves(mut self, mut exprs: Vec<Expr>) -> Node {
		if self.sucessors.is_empty() {
			self.statements.append(&mut exprs);
			self
		} else {
			Node {
				precondition: self.precondition,
				statements: self.statements,
				sucessors: self.sucessors.drain(..).map(|n| n.insert_at_leaves(exprs.clone())).collect()
			}
		}
	}
}


pub struct Declaration(Name,SymTy);

#[derive(Debug,Clone)]
pub enum Expr {
	Value(SymTy),
	Ref(Name),
	BinOp(Rator, Box<Expr>, Box<Expr>),
	UnOp(Rator, Box<Expr>)
}

/*
 * Rators should be seperated out and type checked against Logics. 
 *
 */
 #[derive(Debug,Clone)]
pub enum Rator {
	Eq,
	Add,
	Sub,
	Mul,
	Div,
	Mod,
	NotEqual,
	GreaterEqual,
	GreaterThan,
	LessEqual,
	LessThan,
	Not,
	Neg,
	And
	/*
		BitVector stuff should go here later
	*/

}

impl Rator {
	pub fn from_mir_bin(op: &mir::BinOp) -> Rator {
		match op {
			mir::BinOp::Add => Rator::Add,
			mir::BinOp::Sub => Rator::Sub,
			mir::BinOp::Mul => Rator::Mul,
			mir::BinOp::Div => Rator::Div,
			mir::BinOp::Rem => Rator::Mod,
			mir::BinOp::Eq => Rator::Eq,
			mir::BinOp::Lt => Rator::LessThan,
			mir::BinOp::Le => Rator::LessEqual,
			mir::BinOp::Gt => Rator::GreaterThan,
			mir::BinOp::Ge => Rator::GreaterEqual,
			mir::BinOp::Ne => Rator::NotEqual,
			_ => unimplemented!()
		}
	}

	pub fn from_mir_un(op: &mir::UnOp) -> Rator {
		match op {
			mir::UnOp::Not => Rator::Not,
			mir::UnOp::Neg => Rator::Neg,
		}
	}
}

impl Declaration {
	pub fn decl_from(ty: Ty, nm: Name) -> Declaration {
		
		Declaration(nm, match ty.sty {
			TyKind::Bool => SymTy::Bool(false),
			TyKind::Int(_) => SymTy::Integer(0),
			TyKind::Uint(_) => SymTy::Integer(0),
			TyKind::RawPtr(_) => SymTy::Integer(0),
			_ => unimplemented!()})
	}
				
}

#[derive(Debug,Clone)]
pub enum SymTy {
	Integer(u128),
	Bool(bool),	
}


impl SymTy {
	pub fn from_scalar(sc: u128, ty: Ty) -> SymTy {
		match ty.sty {
			TyKind::Int(_) | TyKind::RawPtr(_) | TyKind::Uint(_) => SymTy::Integer(sc),
			TyKind::Bool => SymTy::Bool(if sc == 1 {
				true
			} else {
				false
			}),
			_ => unimplemented!()

		}
	}
}

impl rsmt2::print::Sym2Smt<()> for Declaration {
	fn sym_to_smt2<T: std::io::Write>(&self, w: &mut T,_:()) -> SmtRes<()> {
		write!(w, "{}", self.0.to_id());
		Ok(())

	}
}

impl rsmt2::print::Sort2Smt<> for Declaration {
	fn sort_to_smt2<T: std::io::Write>(&self, w: &mut T) -> SmtRes<()> {
		match self.1 {
			SymTy::Integer(_) => write!(w,"Int"),
			SymTy::Bool(_) => write!(w,"Bool")
		};
		Ok(())
	}
}

impl rsmt2::print::Expr2Smt<()> for Node {
	fn expr_to_smt2<T: std::io::Write>(&self, w: &mut T,_:()) -> SmtRes<()> {

	}
}