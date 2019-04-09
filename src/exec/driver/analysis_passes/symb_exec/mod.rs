
use rustc::mir::Rvalue;
use super::sir::{Node, Declaration};
use rustc::ty::{Ty,TyKind};
use rustc::mir::Mir;
use std::collections::HashMap;
use super::sir::Expr;
use rustc::mir::{TerminatorKind,Operand,PlaceBase,Place, BasicBlock, BasicBlockData,Statement,Terminator,StatementKind};
use rustc::mir::interpret;
use super::sir::SymTy;
use super::sir::Rator;
use rustc::mir;

struct ExecutionContext<'tcx> {
	mir: &'tcx Mir<'tcx>,
	memory: HashMap<Place<'tcx>,NameHolder>,
	allocator: Rc<RefCell<NameAlloc>>
}

use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug)]
struct NameHolder {
	name: Rc<RefCell<Name>>
}


impl Clone for NameHolder {
	fn clone(&self) -> Self {
		NameHolder {
			name: Rc::new(RefCell::new(self.to_id()))
		}
	}
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

#[derive(Debug,Copy,Clone)]
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

impl <'tcx> Clone for ExecutionContext<'tcx> {
	fn clone(&self) -> Self{
		ExecutionContext {
			memory: self.memory.clone(),
			mir: self.mir,
			allocator: Rc::clone(&self.allocator)
		}
	}
}

impl <'tcx> ExecutionContext<'tcx> {
	fn new(mir: &'tcx rustc::mir::Mir<'tcx>) -> (ExecutionContext<'tcx>, Vec<Declaration>) {
		let mut memory = HashMap::new();
		let mut allocator = NameAlloc::new();

		let mut decls = Vec::new();
		let f_id = allocator.alloc();
		let lcl = mir::Local::from_usize(0);
		let decl = Declaration::decl_from(mir.local_decls[lcl].ty, f_id);
		let loc = Place::Base(PlaceBase::Local(lcl));
		decls.push(decl);
		memory.insert(loc, NameHolder::new(f_id));

		for lcl_id in 1..mir.arg_count+1 {
			let n_id = allocator.alloc();
			let lcl = mir::Local::from_usize(lcl_id);
			let decl = Declaration::decl_from(mir.local_decls[lcl].ty, n_id);
			let loc = Place::Base(PlaceBase::Local(lcl));
			decls.push(decl);
			memory.insert(loc, NameHolder::new(n_id));
		}
		
		(ExecutionContext {
			mir, 
			memory,
			allocator: Rc::new(RefCell::new(allocator))
		},decls)
	}


	fn prepare_assign(&mut self) {
			for val in self.memory.values_mut() {
				val.update(self.allocator.borrow_mut().alloc())
			}

	}

	fn memory_intersection(&self, other: &ExecutionContext) -> Vec<(Name,Name)> {
		self.memory.keys().filter_map(|k| {
			if let Some(val) = other.memory.get(k) {
				let main_id = self.memory.get(k).unwrap().to_id();
				let o_id = val.to_id();
				Some((main_id,o_id))
			} else {
				None
			}
		}).collect()
	}


	fn alloc(&mut self) -> Name {
		self.allocator.borrow_mut().alloc()
	}


	fn get_ty_from_plc(&self,plc: &Place<'tcx>) -> Ty {
		match plc {
			Place::Base(bs) => match bs {
				PlaceBase::Local(lid) => self.mir.local_decls[*lid].ty ,
				PlaceBase::Static(st) => st.ty,
				_  => unimplemented!()
			}
			Place::Projection(proj) => {
				match proj.elem {
					mir::ProjectionElem::Deref => match self.get_ty_from_plc(&proj.base).sty {
						TyKind::RawPtr(tam) => tam.ty,
						TyKind::Ref(_,ty,_) => ty,
						_ => unimplemented!()
					},
					_ => unimplemented!()
				}
			},
		}
	}
}

pub fn eval_mir(mir: &Mir) -> (Node, Vec<Declaration>) {
	let (mut ctx, mut init_decls) = ExecutionContext::new(mir);
	let (node, mut resultant_decls) = process_block_as_node(&mir.basic_blocks()[BasicBlock::from_usize(0)], &mut ctx, None);
	init_decls.append(&mut resultant_decls);
	(node,init_decls)
}


fn process_block_as_node<'ctx>(blk: &BasicBlockData<'ctx>,ctx: &mut ExecutionContext<'ctx>,  precondition: Option<Expr>) -> (Node, Vec<Declaration>) {
	let (stats,mut dec1) = convert_statements(&blk.statements,ctx);
	let (sucessors, mut dec2) = process_terminator(ctx,&blk.terminator());
	dec1.append(&mut dec2);
	(Node::new(precondition, stats, sucessors), dec1)
}

fn process_terminator<'ctx>(mut ctx: &mut ExecutionContext<'ctx>,term:&Terminator<'ctx>) -> (Vec<Node>, Vec<Declaration>) {
	match &term.kind {
		TerminatorKind::Goto{
			target
		} => {
			let (node, decs) = process_block_as_node(&ctx.mir.basic_blocks()[*target], ctx, None);
			(vec![node], decs) 
		} TerminatorKind::SwitchInt{
			discr,
			switch_ty,
			values,
			targets
		} => {
			let mut decls: Vec<Declaration> = Vec::new();
			let mut nodes: Vec<(ExecutionContext,Node)> = Vec::new();
			let mut total_prec = Expr::Value(SymTy::Bool(true));
			let rand = apply_rand(&mut ctx, &discr, &mut decls);
			for (i,val) in values.into_iter().enumerate() {
				let prec = Expr::BinOp(Rator::Eq,Box::new(rand.clone()),Box::new(Expr::Value(SymTy::from_scalar(*val, switch_ty))));
				total_prec = Expr::BinOp(Rator::And, Box::new(prec.clone()), Box::new(total_prec));
				let curr_target = targets[i];
				let mut branch_ctx = ctx.clone();
				let (node,mut new_decls) = process_block_as_node(&ctx.mir.basic_blocks()[curr_target],&mut branch_ctx, Some(prec));
				nodes.push((branch_ctx,node));
				decls.append(&mut new_decls);
			}

			total_prec = Expr::UnOp(Rator::Not, Box::new(total_prec));
			let mut branch_ctx = ctx.clone();
			let (node,mut new_decls) = process_block_as_node(&ctx.mir.basic_blocks()[targets[targets.len() - 1]],&mut branch_ctx, Some(total_prec));
			nodes.push((branch_ctx,node));
			decls.append(&mut new_decls);
			(rendevue_nodes_at_ctx(nodes,&mut ctx),decls)
		},
		// TODO inline function Calls.
		TerminatorKind::Call {
			func: _,
			args: _,
			destination: dest,
			cleanup: _,
			from_hir_call: _,
		} =>  {
			match dest {
				None => unimplemented!(),
				Some((ret,next)) => {
					let n_id = ctx.alloc();
					let ty = ctx.get_ty_from_plc(ret);
					let decl = Declaration::decl_from(ty,n_id);
					let hand = ctx.memory.get_mut(ret).unwrap();
					hand.update(n_id);
					let (node, mut decs) = process_block_as_node(&ctx.mir.basic_blocks()[*next], ctx, None);
					decs.push(decl);
					(vec![node],decs)
				}
			}
		},
		_ => (vec![],vec![])

	}
}

// TODO Rendevue Local vars not initilaized in the first execution context
fn rendevue_nodes_at_ctx(mut nodes: Vec<(ExecutionContext,Node)>, ctx: &mut ExecutionContext) -> Vec<Node> {
	ctx.prepare_assign();


	   nodes.drain(..).map(|(res_ctx,nd)| {
		let mut bindings: Vec<(Name,Name)> = ctx.memory_intersection(&res_ctx);
		let exprs: Vec<Expr> = bindings.drain(..).map(|(n1,n2)| 
			Expr::BinOp(Rator::Eq, Box::new(Expr::Ref(n1)), Box::new(Expr::Ref(n2)))).collect();
		nd.insert_at_leaves(exprs)
	}).collect()
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
		StatementKind::StorageLive(lcl) => {
			let n_name = ctx.alloc();
			ctx.memory.insert(Place::Base(PlaceBase::Local(*lcl)),NameHolder::new(n_name));
			(vec![],vec![Declaration::decl_from(ctx.mir.local_decls[*lcl].ty,n_name)])
		} ,
		StatementKind::StorageDead(lcl) => {
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
	let new_id = ctx.alloc();
	let new_dec = Declaration::decl_from(ctx.get_ty_from_plc(loc), new_id);
	ctx.memory.get_mut(loc).unwrap().update(new_id);
	let mut declerations = Vec::new();
	declerations.push(new_dec);
	
	(vec![Expr::BinOp(Rator::Eq, Box::new(Expr::Ref(new_id)),Box::new(match val {
		Rvalue::Use(rand1) => {
			let val = apply_rand(ctx,rand1, &mut declerations);
			val
		},
		Rvalue::BinaryOp(op,rand1,rand2) => assign_bin_op(ctx,op, rand1, rand2, &mut declerations),
		Rvalue::CheckedBinaryOp(op,rand1,rand2) => assign_bin_op(ctx,op, rand1, rand2, &mut declerations),
		Rvalue::Cast(_castknd,rand,_ty) => {
			let rand = apply_rand(ctx, rand, &mut declerations);
			//TODO dont just call it here
			rand
		},
		Rvalue::UnaryOp(op,rand) => {
			let rand = Box::new(apply_rand(ctx,rand, &mut declerations));
			let op = Rator::from_mir_un(op);
			Expr::UnOp(op,rand)
		},
		Rvalue::Ref(_reg,_brw_kind,plc) => {
			Expr::Ref(ctx.memory.get(plc).unwrap().to_id())
			//TODO Handle fixing up Cells so that edits propogate. 
		},
		_ => unimplemented!()
	}))],declerations)
}


fn assign_bin_op<'ctx>(ctx: &mut ExecutionContext<'ctx>,op: &mir::BinOp, rand1: &Operand<'ctx>, rand2: &Operand<'ctx>, decls: &mut Vec<Declaration>) -> Expr {
			let rand1 = Box::new(apply_rand(ctx, rand1,decls));
			let rand2 = Box::new(apply_rand(ctx, rand2 ,decls));
			let n_op = Rator::from_mir_bin(op);
			Expr::BinOp(n_op,rand1,rand2)
}


fn apply_rand<'ctx>(ctx: &mut ExecutionContext<'ctx>, rand: &Operand<'ctx>, decls: &mut Vec<Declaration>) -> Expr {
	match rand {
		Operand::Copy(Place::Base(loc)) => {
		 let loc = Place::Base(loc.clone());
		 Expr::Ref(ctx.memory.get(&loc).unwrap().to_id())},
		Operand::Move(Place::Base(loc)) => { 
			let loc = Place::Base(loc.clone());
			Expr::Ref(ctx.memory.remove(&loc).unwrap().to_id())},
		Operand::Constant(val) => {
			match val.literal.val {
				interpret::ConstValue::Scalar(scl) => Expr::Value(SymTy::from_scalar(match scl {
					mir::interpret::Scalar::Bits {
						size: _,
						bits,
					} => bits,
					interpret::Scalar::Ptr(_) => unimplemented!()
				},val.literal.ty)),
				_ => unimplemented!()
			}
		},
		Operand::Copy(plcproj) => {
			deref_unknown(ctx, plcproj, decls)
		},
		Operand::Move(plcproj) => {
			deref_unknown(ctx, plcproj, decls)
		}
	}
}

fn deref_unknown(ctx: &mut ExecutionContext,  plc: &Place,decls: &mut Vec<Declaration>) -> Expr {
	let n_id = ctx.alloc();
	let ty = ctx.get_ty_from_plc(plc);
	let decl = Declaration::decl_from(ty, n_id);
	decls.push(decl);
	Expr::Ref(n_id)
}