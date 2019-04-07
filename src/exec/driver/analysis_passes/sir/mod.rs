use super::symb_exec::Name;
use rustc::ty::{Ty, TyKind};
use rustc::mir::interpret::Scalar;
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


pub enum Expr {
	True,
	Value(SymTy),
	Ref(Name),
	BinOp(Rator, Box<Expr>, Box<Expr>)
}

pub enum Rator {
	Eq
}

impl Declaration {
	pub fn decl_from(ty: Ty, nm: Name) -> Declaration {
		
		Declaration(nm, match ty.sty {
			TyKind::Bool => SymTy::Bool(false),
			TyKind::Char => SymTy::Integer(0),
			TyKind::Int(_) => SymTy::Integer(0),
			TyKind::Uint(_) => SymTy::Integer(0),
			TyKind::RawPtr(_) => SymTy::Integer(0),
			_ => unimplemented!()})
	}
				
}


pub enum SymTy {
	Integer(usize),
	Bool(bool),	
}


impl SymTy {
	pub fn from_scalar(sc: Scalar, ty: Ty) -> SymTy {
		unimplemented!()
	}
}
