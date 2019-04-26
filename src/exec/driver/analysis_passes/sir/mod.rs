

use rustc::hir::def_id::DefId;
use rustc::mir::Local;
use std::io::Write;
use rsmt2::errors::SmtRes;
use rustc::ty::{Ty, TyKind};

use rustc::mir;

mod structures;
use std::io::Cursor;

use rsmt2::print::Expr2Smt;
use structures::NodeVec;
use structures::NameVec;
pub use structures::Name;
pub use structures::NodeId;

use rsmt2::print::Sort2Smt;

#[derive(Debug)]
pub struct Node {
	statements: Vec<Expr>,
}

impl Node {
	fn new() -> Node {
		Node {
			statements: Vec::new()
		}
	}

	fn add_expr(&mut self, ex: Expr) {
		self.statements.push(ex);
	}


	pub fn is_empty(&self) ->bool {
		self.statements.is_empty()
	}

	fn to_smt(&self) -> String {
		let mut res = String::new();
		for state in self.statements.iter() {
			res.push_str(&state.to_smt());
		}
		res
	}
}



#[derive(Debug,Clone)]
pub struct Edge(Option<Expr>, NodeId);

impl Edge {
	fn get_precondition(&self) -> Option<Expr> {
		self.0.clone()
	}

	fn get_target(&self) -> NodeId {
		self.1
	}

	pub fn new(precondition: Option<Expr>, nid: NodeId) -> Edge {
		Edge(precondition,nid)
	}
	

	fn to_smt(&self, sir: &Sir) -> String {
		let mut res = String::new();
		res.push_str("(=> ");
		if let Some(x) = &self.0 {
			res.push_str(&x.to_smt());
			res.push(' ');
		} else {
			res.push_str("true ");
		}

		res.push_str(&sir.node_to_smt(self.1));
		res.push(')');
		res
	}
}




#[derive(Debug)]
pub struct Sir {
	declarations: NameVec<Declaration>,
	nodes: NodeVec<Node>,
	forward_edges: NodeVec<Vec<Edge>>,
	backward_edges: NodeVec<Vec<Edge>>

}

impl Sir {
	pub fn to_smt(&self, start: NodeId) -> String {
		self.node_to_smt(start)
	}


	fn node_to_smt(&self, nid: NodeId) -> String {
		let mut total = String::new();

		total.push_str("(and true ");
		let curr_node = &self.nodes[nid];
		total.push_str(&curr_node.to_smt());
		total.push_str(&self.process_all_children(nid));
		total.push_str(")");

		total
	}


	fn process_all_children(&self, nid: NodeId) -> String {
		let n_edges = &self.forward_edges[nid];
		let mut total = String::new();
		for edge in n_edges {
			total.push_str(&edge.to_smt(&self));
		}
		total
	}

	pub fn new() -> Sir {
		Sir {
			declarations: NameVec::new(),
			nodes: NodeVec::new(),
			forward_edges: NodeVec::new(),
			backward_edges: NodeVec::new(),
		}
	}

	pub fn add_declaration(&mut self,decl: Declaration) -> Name {
		self.declarations.push(decl)
	}

	pub fn get_declaration(&self, nm: Name) -> &Declaration {
		&self.declarations[nm]
	}

	pub fn add_property_to_declaration(&mut self, nm: Name, prop: MirVariableProp) {
		self.declarations[nm].add_property(prop);
	}


	pub fn get_all_names(&self) -> impl Iterator<Item = Name> {
		self.declarations.into_iter()
	}


	pub fn get_node(&self, nid: NodeId) -> &Node {
		&self.nodes[nid]
	}

	pub fn get_node_mut(&mut self, nid: NodeId) -> &mut Node {
		&mut self.nodes[nid]
	}

	pub fn get_out_edges(&self, nid: NodeId) -> &Vec<Edge> {
		&self.forward_edges[nid]
	}

	pub fn get_in_edges(&self, nid: NodeId) -> &Vec<Edge> {
		&self.backward_edges[nid]
	}

	pub fn add_node(&mut self) -> NodeId {
		let nid = self.nodes.push(Node::new());
		let oid = self.forward_edges.push(vec![]);
		let fid = self.backward_edges.push(vec![]);
		assert_eq!(nid,oid);
		assert_eq!(nid, fid);
		nid
	}

	pub fn add_expr_to_node(&mut self, nid: NodeId, expr: Expr) {
		self.nodes[nid].add_expr(expr);
	}

	pub fn add_edge(&mut self, nid: NodeId, edge: Edge) {
		self.backward_edges[edge.get_target()].push(Edge::new(edge.get_precondition(),nid));
		self.forward_edges[nid].push(edge);
		
	}

	pub fn get_path_constraint(&self, nid: NodeId) -> Expr {
		let mut total_exp = Vec::new();
		let mut cid = nid;
		let mut pred = self.get_in_edges(cid).clone().pop();
		while let Some(before) = pred {
			total_exp.push(before.get_precondition());
			cid = before.get_target();
			pred = self.get_in_edges(cid).clone().pop();
		}

		total_exp.into_iter().filter_map(|x|x)
		.fold(Expr::Value(SymTy::Bool(true)), |x, y| Expr::BinOp(Rator::And, Box::new(x), Box::new(y)))
	}

}

#[derive(Debug)]
pub struct Declaration(SymTy, Vec<MirVariableProp>, Option<(DefId,Local)>);

impl Declaration {
	fn add_property(&mut self, prop: MirVariableProp) {
		self.1.push(prop)
	}

	pub fn new_declaration(&self) -> Declaration {
		Declaration(self.0.clone(), vec![], self.2.clone())
	}

	pub fn get_property(&self) -> &Vec<MirVariableProp> {
		&self.1
	}

	pub fn get_location(&self) -> &Option<(DefId,Local)> {
		&self.2
	}
}


#[derive(Debug)]
pub enum MirVariableProp {
	IsDerefed(NodeId)
}

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
	pub fn decl_from(ty: Ty, arg_loc: Option<(DefId,Local)>) -> Declaration {	
		Declaration(match ty.sty {
			TyKind::Bool => SymTy::Bool(false),
			TyKind::Int(_) => SymTy::Integer(0),
			TyKind::Uint(_) => SymTy::Integer(0),
			TyKind::RawPtr(_) => SymTy::Integer(0),
			_ => unimplemented!()}, vec![],arg_loc)
	}
}

impl Sort2Smt for Declaration {
	fn sort_to_smt2<T: Write>(&self, w: &mut T) -> SmtRes<()> {
		write!(w,"{}",match self.0 {
			SymTy::Integer(_) => "Int",
			SymTy::Bool(_) => "Bool"
		})?;
		Ok(())
	} 
}

#[derive(Debug,Clone,Eq,PartialEq,Hash)]
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

	pub fn from_boolean(b: bool) -> SymTy {
		SymTy::Bool(b)
	}	
}


impl Expr {
	fn to_smt(&self) -> String {
		let mut x: Vec<u8> = Vec::new();
		self.expr_to_smt2(&mut Cursor::new(&mut x),()).unwrap();
		std::str::from_utf8(&x).unwrap().to_owned()
	}
}

impl rsmt2::print::Expr2Smt<()> for Expr {
	fn expr_to_smt2<T: std::io::Write>(&self, w: &mut T,_:()) -> SmtRes<()> {
		match self {
			Expr::Ref(nm) => {write!(w,"{} ",nm.to_id());},
			Expr::UnOp(op,rand) => {
				write!(w,"(");
				op.expr_to_smt2(w, ());
				rand.expr_to_smt2(w,());
				write!(w,")");
			},
			Expr::BinOp(op,rand1,rand2) => {
				write!(w,"(");
				op.expr_to_smt2(w, ());
				rand1.expr_to_smt2(w,());
				rand2.expr_to_smt2(w,());

				if let Rator::NotEqual = op {
					write!(w,")");
				}
				write!(w,")");
			},
			Expr::Value(val) => {
				val.expr_to_smt2(w,());
			}
		}

		Ok(())
	} 
}

impl rsmt2::print::Expr2Smt<()> for Rator {
	fn expr_to_smt2<T: std::io::Write>(&self, w: &mut T,_:()) -> SmtRes<()> {
		write!(w,"{}", match self {
			Rator::Eq => "=",
			Rator::Add => "+",
			Rator::And => "and",
			Rator::Div => "div",
			Rator::GreaterEqual => ">=",
			Rator::GreaterThan => ">",
			Rator::LessEqual => "<=",
			Rator::LessThan => "<",
			Rator::Mod => "mod",
			Rator::Mul => "*",
			Rator::Neg => "-",
			Rator::Not => "not",
			Rator::NotEqual => "(not (=",
			Rator::Sub => "-",
		});
		write!(w," ");
		Ok(())
	} 
}

impl rsmt2::print::Expr2Smt<()> for SymTy {
	fn expr_to_smt2<T: std::io::Write>(&self, w: &mut T,_:()) -> SmtRes<()> {
		match self {
			SymTy::Bool(bl) => {
				if *bl {
					write!(w,"true ");
				} else {
					write!(w,"false ");
				}
			},
			SymTy::Integer(num) => {
				write!(w,"{} ",num);
			}
		}

		Ok(())
	} 
}