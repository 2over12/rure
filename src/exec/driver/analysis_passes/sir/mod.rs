use super::symb_exec::Name;
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

/*
struct FuncId(usize);

impl Into<usize> for FuncId {
	fn into(self) -> usize {
		self.0

	}
}

impl From<usize> for FuncId {
	fn from(t:usize) -> FuncId {
		FuncId(t)
	}
}

pub struct GuardedVec<T: From<usize> + Into<usize> ,V> {
	raw: Vec<V>,
	_marker: PhantomData<T>
}

impl  <T: From<usize> +Into<usize>,V> GuardedVec<T,V> {
	fn new() -> GuardedVec<T,V> {
		GuardedVec {
			raw: Vec::new(),
			_marker: PhantomData
		}
	}

	fn push(&mut self, item:V) -> T {
		self.raw.push(item);
		T::from(self.raw.len()-1)
	}

}

impl <T: From<usize> + Into<usize>,V> Index<T> for GuardedVec<T,V> {
	type Output = V;

	fn index(&self, idx: T) -> &V {
		&self.raw[idx.into()]

	}
}
*/



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
