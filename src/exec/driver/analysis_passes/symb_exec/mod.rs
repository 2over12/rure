
use rustc::mir::Rvalue;
use super::sir::{Node, Declaration};
use rustc::ty::Ty;
use rustc::mir::Mir;
use std::collections::HashMap;
use super::sir::Expr;
use rustc::mir::{Operand,PlaceBase,Place, BasicBlock, BasicBlockData,Statement,Terminator,StatementKind};
use rustc::mir::interpret;
use super::sir::SymTy;
use super::sir::Rator;

struct ExecutionContext<'tcx> {
	mir: &'tcx Mir<'tcx>,
	memory: HashMap<Place<'tcx>,NameHolder>,
	allocator: NameAlloc
}
use std::rc::Rc;
use std::cell::RefCell;

struct NameHolder {
	name: Rc<RefCell<Name>>
}

impl NameHolder {
	fn new(n: Name) -> NameHolder {
		NameHolder {
			name: Rc::new(RefCell::new(n))
		}
	}

	fn update(&mut self, n: Name) {
		self.name.replace(n);
	}

	fn to_id(&self) -> Name {
		self.name.borrow().clone()
	}
}

#[derive(Copy,Clone)]
pub struct Name(usize);

impl Name {
	fn to_id(&self) -> String {
		format!("x{}",&self.0)
	}
}


struct NameAlloc {
	n_id: usize
}

impl NameAlloc {
	fn new() -> NameAlloc {
		NameAlloc {
			n_id: 0
		}
	}

	fn alloc(&mut self) -> Name {
		let n = self.n_id;
		self.n_id = self.n_id + 1;
		Name(n)
	}
}

impl <'tcx> ExecutionContext<'tcx> {
	fn new(mir: &'tcx rustc::mir::Mir<'tcx>) -> ExecutionContext<'tcx> {
		ExecutionContext {
			mir, 
			memory: HashMap::new(),
			allocator: NameAlloc::new()
		}
	}

	fn get_ty_from_plc(&self,plc: &Place<'tcx>) -> Ty {
		match plc {
			Place::Base(bs) => match bs {
				PlaceBase::Local(lid) => self.mir.local_decls[*lid].ty ,
				PlaceBase::Static(st) => st.ty,
				_  => unimplemented!()
			}
			Place::Projection(_proj) => unimplemented!(),
		}
	}
}

pub fn eval_mir(mir: &Mir) -> (Node, Vec<Declaration>) {
	let mut ctx = ExecutionContext::new(mir);
	process_block_as_node(&mir.basic_blocks()[BasicBlock::from_usize(0)],&mut ctx, None)
}


fn process_block_as_node<'ctx>(blk: &BasicBlockData<'ctx>,ctx: &mut ExecutionContext<'ctx>,  precondition: Option<Expr>) -> (Node, Vec<Declaration>) {
	let (stats,mut dec1) = convert_statements(&blk.statements,ctx);
	let (sucessors, mut dec2) = process_terminator(&blk.terminator());
	dec1.append(&mut dec2);
	(Node::new(precondition, stats, sucessors), dec1)
}

fn process_terminator(_term:&Terminator) -> (Vec<Node>, Vec<Declaration>) {
	(vec![],vec![])
}

fn convert_statements<'ctx>(stats: &Vec<Statement<'ctx>>, ctx: &mut ExecutionContext<'ctx>) -> (Vec<Expr>, Vec<Declaration>) {
	let v: Vec<(Vec<Expr>,Vec<Declaration>)> = stats.into_iter().map(|x|symbolize_statement(x,ctx)).collect();
	let mut exprs = Vec::new();
	let mut decs = Vec::new(); 
	for (mut xpr,mut dec) in v {
		exprs.append(&mut xpr);
		decs.append(&mut dec);
	}

	(exprs,decs)
}

fn symbolize_statement<'ctx>(stat: &Statement<'ctx>, ctx: &mut ExecutionContext<'ctx>) -> (Vec<Expr>, Vec<Declaration>) {
	match &stat.kind {
		StatementKind::StorageDead(lcl) => {
			let n_name = ctx.allocator.alloc();
			ctx.memory.insert(Place::Base(PlaceBase::Local(*lcl)),NameHolder::new(n_name));
			(vec![],vec![Declaration::decl_from(ctx.mir.local_decls[*lcl].ty,n_name)])
		} ,
		StatementKind::StorageLive(lcl) => {
			ctx.memory.remove(&Place::Base(PlaceBase::Local(*lcl)));
			(vec![],vec![])
		}
		StatementKind::Assign(loc, val) => {
		 assign_into(ctx,loc,val)


		},
		_ => (vec![],vec![]),
	}
}

fn assign_into<'ctx>(ctx: &mut ExecutionContext<'ctx>, loc: &rustc::mir::Place<'ctx>, val: &Rvalue<'ctx>) -> (Vec<Expr>,Vec<Declaration> ) {
	let new_id = ctx.allocator.alloc();
	let new_dec = Declaration::decl_from(ctx.get_ty_from_plc(loc), new_id);
	ctx.memory.get_mut(loc).unwrap().update(new_id);
	match val {
		Rvalue::Use(rand1) => {
			let val = apply_rand(ctx,rand1);
			let exp = Expr::BinOp(Rator::Eq, Box::new(Expr::Ref(new_id)),Box::new(val));
			(vec![exp], vec![new_dec])
		},
		_ => unimplemented!()
	}
}



fn apply_rand<'ctx>(ctx: &mut ExecutionContext<'ctx>, rand: &Operand<'ctx>) -> Expr {
	match rand {
		Operand::Copy(loc) => Expr::Ref(ctx.memory.get(&loc).unwrap().to_id()),
		Operand::Move(loc) =>  Expr::Ref(ctx.memory.remove(&loc).unwrap().to_id()),
		Operand::Constant(val) => {
			match val.literal.val {
				interpret::ConstValue::Scalar(scl) => Expr::Value(SymTy::from_scalar(scl,val.literal.ty)),
				_ => unimplemented!()
			}
		}
	}
}