
use crate::exec::driver::analysis_passes::sir::Sir;
use std::hash::Hash;
use rustc::mir::Rvalue;
use super::sir::{Node, Declaration};
use rustc::ty::{Ty,TyKind};
use rustc::mir::Mir;
use std::collections::HashMap;
use super::sir::MirVariableProps;
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

#[derive(Debug,Copy,Clone,Eq,PartialEq,Hash)]
pub struct Name(usize);

impl Name {
	pub fn to_id(&self) -> String {
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
	fn new(mir: &'tcx rustc::mir::Mir<'tcx>) -> (ExecutionContext<'tcx>, HashMap<Declaration,Vec<MirVariableProps>>) {
		let mut memory = HashMap::new();
		let mut allocator = NameAlloc::new();

		let mut decls = HashMap::new();
		let f_id = allocator.alloc();
		println!("init f {}",f_id.to_id());
		let lcl = mir::Local::from_usize(0);
		let decl = Declaration::decl_from(mir.local_decls[lcl].ty, f_id);
		let loc = Place::Base(PlaceBase::Local(lcl));
		decls.insert(decl, Vec::new());
		memory.insert(loc, NameHolder::new(f_id));

		for lcl_id in 1..mir.arg_count+1 {
			let n_id = allocator.alloc();
			println!("arg {}",n_id.to_id());
			let lcl = mir::Local::from_usize(lcl_id);
			let decl = Declaration::decl_from(mir.local_decls[lcl].ty, n_id);
			let loc = Place::Base(PlaceBase::Local(lcl));
			decls.insert(decl,Vec::new());
			memory.insert(loc, NameHolder::new(n_id));
		}
		
		(ExecutionContext {
			mir, 
			memory,
			allocator: Rc::new(RefCell::new(allocator))
		},decls)
	}


	fn prepare_assign(&mut self) -> Vec<Declaration> {
		let mut decls = Vec::new();
			for (key,val) in self.memory.iter_mut() {
				let t = ty_form_plc_mir(key,self.mir);
				let n_id = self.allocator.borrow_mut().alloc();
				let decl = Declaration::decl_from(t, n_id);
				decls.push(decl);
				val.update(n_id);
			}
			decls

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


	fn get_place_id(&self,plc: &Place<'tcx>) -> Name {
		match plc {
			Place::Base(bs) => match bs {
				PlaceBase::Local(lid) => self.memory.get(&Place::Base(PlaceBase::Local(*lid))).unwrap().to_id() ,
				_  => unimplemented!()
			}
			Place::Projection(proj) => {
				match proj.elem {
					mir::ProjectionElem::Deref => self.get_place_id(&proj.base),
					_ => unimplemented!()

			}
		}
	}
}
	fn get_ty_from_plc(&self,plc: &Place<'tcx>) -> Ty {
		ty_form_plc_mir(plc, self.mir)
	}


	fn get_declaration(&self,plc: &Place<'tcx>) -> Declaration {
		match plc {
			Place::Base(bs) => match bs {
				PlaceBase::Local(lid) => Declaration::decl_from(self.mir.local_decls[*lid].ty,self.memory.get(plc).unwrap().to_id()) ,
				PlaceBase::Static(_st) => unimplemented!(),
				_  => unimplemented!()
			}
			Place::Projection(proj) => {
					self.get_declaration(&proj.base) 
				
			},
		}
	}
}

fn ty_form_plc_mir<'tcx>(plc: &Place<'tcx>, mir: &Mir<'tcx>) -> Ty<'tcx> {
		match plc {
			Place::Base(bs) => match bs {
				PlaceBase::Local(lid) => mir.local_decls[*lid].ty ,
				PlaceBase::Static(st) => st.ty,
				_  => unimplemented!()
			}
			Place::Projection(proj) => {
				match proj.elem {
					mir::ProjectionElem::Deref => match ty_form_plc_mir(&proj.base,mir).sty {
						TyKind::RawPtr(tam) => tam.ty,
						TyKind::Ref(_,ty,_) => ty,
						_ => unimplemented!()
					},
					_ => unimplemented!()
				}
			},
		}
	}

pub fn eval_mir(mir: &Mir) -> Sir {
	let (mut ctx, mut init_decls) = ExecutionContext::new(mir);
	let (node, mut resultant_decls) = process_block_as_node(&mir.basic_blocks()[BasicBlock::from_usize(0)], &mut ctx, None);
	insert_all(&mut init_decls, &mut resultant_decls);
	Sir::new(node,init_decls)
}


fn insert_all<K: Eq + Hash,V>(into: &mut HashMap<K,V>, from: &mut HashMap<K,V>) {
	for (k,v) in from.drain() {
		into.insert(k, v);
	}
}

fn process_block_as_node<'ctx>(blk: &BasicBlockData<'ctx>,ctx: &mut ExecutionContext<'ctx>,  precondition: Option<Expr>) -> (Node, HashMap<Declaration,Vec<MirVariableProps>>) {
	let (stats,mut dec1) = convert_statements(&blk.statements,ctx);
	let (sucessors, mut dec2) = process_terminator(ctx,&blk.terminator());
	insert_all(&mut dec1, &mut dec2);
	(Node::new(precondition, stats, sucessors), dec1)
	
}

fn process_terminator<'ctx>(mut ctx: &mut ExecutionContext<'ctx>,term:&Terminator<'ctx>) -> (Vec<Node>, HashMap<Declaration,Vec<MirVariableProps>>) {
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
			let mut decls: HashMap<_,_> = HashMap::new();
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
				insert_all(&mut decls,&mut new_decls);
			}

			total_prec = Expr::UnOp(Rator::Not, Box::new(total_prec));
			let mut branch_ctx = ctx.clone();
			let (node,mut new_decls) = process_block_as_node(&ctx.mir.basic_blocks()[targets[targets.len() - 1]],&mut branch_ctx, Some(total_prec));
			nodes.push((branch_ctx,node));
			insert_all(&mut decls,&mut new_decls);
			let (new_nodes, mut additional_decls) = rendevue_nodes_at_ctx(nodes,&mut ctx);
			insert_all(&mut decls,&mut additional_decls);
			(new_nodes,decls)
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
					decs.insert(decl, Vec::new());
					(vec![node],decs)
				}
			}
		},
		_ => {(vec![],HashMap::new())}

	}
}

// TODO Rendevue Local vars not initilaized in the first execution context
fn rendevue_nodes_at_ctx(mut nodes: Vec<(ExecutionContext,Node)>, ctx: &mut ExecutionContext) -> (Vec<Node>,HashMap<Declaration,Vec<MirVariableProps>>) {
	

	let decls = ctx.prepare_assign().drain(..).map(|x|(x, Vec::new())).collect();


	   (nodes.drain(..).map(|(res_ctx,nd)| {
		let mut bindings: Vec<(Name,Name)> = ctx.memory_intersection(&res_ctx);
		let exprs: Vec<Expr> = bindings.drain(..).map(|(n1,n2)| 
			Expr::BinOp(Rator::Eq, Box::new(Expr::Ref(n1)), Box::new(Expr::Ref(n2)))).collect();
		nd.insert_at_leaves(exprs)
	}).collect(),decls)
}

fn convert_statements<'ctx>(stats: &Vec<Statement<'ctx>>, ctx: &mut ExecutionContext<'ctx>) -> (Vec<Expr>, HashMap<Declaration,Vec<MirVariableProps>>) {
	let mut v: Vec<(Vec<Expr>,HashMap<Declaration,Vec<MirVariableProps>>)> = stats.into_iter().map(|x|symbolize_statement(x,ctx)).collect();
	let mut exprs = Vec::new();
	let mut total_decls = HashMap::new();

	for (mut ex, mut dec) in v.drain(..) {
		exprs.append(&mut ex);
		for (key,val) in dec.drain() {
			total_decls.insert(key, val);
		}

	}

	(exprs,total_decls)
}

fn symbolize_statement<'ctx>(stat: &Statement<'ctx>, ctx: &mut ExecutionContext<'ctx>) -> (Vec<Expr>, HashMap<Declaration,Vec<MirVariableProps>>) {
	let mut decls = HashMap::new();
	match &stat.kind {
		StatementKind::StorageLive(lcl) => {
			let n_name = ctx.alloc();
			ctx.memory.insert(Place::Base(PlaceBase::Local(*lcl)),NameHolder::new(n_name));
			decls.insert(Declaration::decl_from(ctx.mir.local_decls[*lcl].ty,n_name),Vec::new());
			(vec![],decls)
		} ,
		StatementKind::StorageDead(lcl) => {
			ctx.memory.remove(&Place::Base(PlaceBase::Local(*lcl)));
			(vec![],decls)
		}
		StatementKind::Assign(loc, val) => {
		 assign_into(ctx,loc,val)


		},
		_ => (vec![],decls),
	}
}

fn assign_into<'ctx>(ctx: &mut ExecutionContext<'ctx>, loc: &rustc::mir::Place<'ctx>, val: &Rvalue<'ctx>) -> (Vec<Expr>,HashMap<Declaration,Vec<MirVariableProps>> ) {
	let new_id = ctx.alloc();
	let new_dec = Declaration::decl_from(ctx.get_ty_from_plc(loc), new_id);
	ctx.memory.get_mut(loc).unwrap().update(new_id);
	let mut declerations = HashMap::new();
	declerations.insert(new_dec,Vec::new());
	
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


fn assign_bin_op<'ctx>(ctx: &mut ExecutionContext<'ctx>,op: &mir::BinOp, rand1: &Operand<'ctx>, rand2: &Operand<'ctx>, decls: &mut HashMap<Declaration,Vec<MirVariableProps>>) -> Expr {
			let rand1 = Box::new(apply_rand(ctx, rand1,decls));
			let rand2 = Box::new(apply_rand(ctx, rand2 ,decls));
			let n_op = Rator::from_mir_bin(op);
			Expr::BinOp(n_op,rand1,rand2)
}


fn apply_rand<'ctx>(ctx: &mut ExecutionContext<'ctx>, rand: &Operand<'ctx>, decls: &mut HashMap<Declaration,Vec<MirVariableProps>>) -> Expr {
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

fn deref_unknown(ctx: &mut ExecutionContext,  plc: &Place,decls: &mut HashMap<Declaration,Vec<MirVariableProps>>) -> Expr {
	let n_id = ctx.alloc();
	println!("deref {}",ctx.get_place_id(plc).to_id());
	let dec = ctx.get_declaration(plc);
	println!("{:?}", decls);
	println!("{:?}",dec);
	let props: &mut Vec<_> = decls.get_mut(&dec).unwrap();
	props.push(MirVariableProps::IsDerefed);


	let ty = ctx.get_ty_from_plc(plc);
	let decl = Declaration::decl_from(ty, n_id);
	decls.insert(decl,Vec::new());
	Expr::Ref(n_id)
}