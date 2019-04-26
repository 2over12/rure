
use rustc::mir::Terminator;
use std::borrow::Cow;
use rustc::mir::Rvalue;
use rustc::mir::interpret::ConstValue;
use rustc::mir::interpret::Scalar;
use rustc::ty::Ty;
use rustc::mir::LocalDecl;
use rustc::mir::Local;
use rustc::mir::BasicBlock;
use rustc::mir::Place;
use std::collections::HashMap;
use rustc::mir::Mir;
use crate::exec::driver::analysis_passes::sir::Sir;
use rustc::hir::def_id::DefId;
use rustc::mir::BasicBlockData;
use super::sir::Name;
use super::sir::NodeId;
use rustc::mir::Statement;
use rustc::mir::PlaceBase;
use super::sir::Declaration;
use rustc::mir::StatementKind;
use rustc::mir::ProjectionElem;
use super::sir::Expr;
use super::sir::MirVariableProp;
use super::sir::Rator;
use rustc::mir::Operand;
use super::sir::SymTy;
use rustc::mir::TerminatorKind;
use super::sir::Edge;


const MAX_UNROLL: usize = 5;

#[derive(Clone)]
struct Memory<'tcx> {
	assignments: HashMap<Place<'tcx>,Name>,
}

impl <'tcx> Memory <'tcx> {

	fn process_operand(&mut self, rand: Operand<'tcx>, nid: NodeId, sir: &mut Sir) -> Expr {
		match rand {
			Operand::Copy(plc) => Expr::Ref(self.copy_val(&plc,nid,sir)),
			Operand::Move(plc) => Expr::Ref(self.move_val(&plc,nid,sir)),
			Operand::Constant(cst) => Expr::Value(SymTy::from_scalar(match cst.literal.val {
				ConstValue::Scalar(sc) => match sc {
					Scalar::Bits{bits, size:_} => bits,
					_ => unimplemented!(),
				},
				_ => unimplemented!()
			}, cst.ty))
			
		}
	}

	fn new_assignment(&mut self, to: Place<'tcx>, nid: NodeId, sir: &mut Sir) -> Name {
		let old_name = self.process_plc(&to, nid, sir);
		let old_decl = sir.get_declaration(old_name);
		let new_name = sir.add_declaration(old_decl.new_declaration());
		self.assignments.insert(to, new_name);
		new_name
	}

	fn move_val(&mut self, plc:  &Place<'tcx>,nid: NodeId, sir: &mut Sir) -> Name {
		let _ = self.process_plc(plc, nid, sir);
		self.assignments.remove(plc).unwrap()
	}

	fn copy_val(&mut self, plc:  &Place<'tcx>,nid: NodeId, sir: &mut Sir) -> Name {
		let _ = self.process_plc(plc, nid, sir);
		*self.assignments.get(plc).unwrap()
	}

	fn process_plc(&mut self, plc: &Place<'tcx>, nid: NodeId, sir: &mut Sir) -> Name  {
		match plc {
			Place::Base(_) => *self.assignments.get(plc).unwrap(),
			Place::Projection(proj) => match proj.elem {
				ProjectionElem::Deref => {
					let name_of_current_deref = *self.assignments.get(&proj.base).unwrap();
					sir.add_property_to_declaration(name_of_current_deref, MirVariableProp::IsDerefed(nid));
					let name = sir.add_declaration(sir.get_declaration(name_of_current_deref).new_declaration());
					self.assignments.insert(plc.clone(), name);
					name
				},
				_ => unimplemented!()
			}
		}
	}

	fn from_args(args: impl Iterator<Item = Local>, did: DefId, mir: &Mir, sir: &mut Sir) -> Memory<'tcx> {
		
		let mut assignments = HashMap::new();

		let return_plc = Place::Base(PlaceBase::Local(Local::from(0 as usize)));
		let return_decl = Declaration::decl_from(mir.local_decls[Local::from(0 as usize)].ty,Some((did,Local::from(0 as usize))));
		let ret_name = sir.add_declaration(return_decl);
		assignments.insert(return_plc, ret_name);

		for arg in args {
			let lcl = &mir.local_decls[arg];
			let plc = Place::Base(PlaceBase::Local(arg));
			let declaration = Declaration::decl_from(lcl.ty,Some((did,arg)));
			let nm = sir.add_declaration(declaration);
			assignments.insert(plc, nm);
		}

		Memory {
			assignments
		}

	}

	fn add_new_var(&mut self, plc: Place<'tcx>, ty: Ty<'tcx>,sir: &mut Sir, did: Option<(DefId,Local)>) {
		let decl = Declaration::decl_from(ty,did);
		self.assignments.insert(plc, sir.add_declaration(decl));
	}

	fn remove_var(&mut self, plc: &Place<'tcx>) {
		self.assignments.remove(&plc);
	}
}


#[derive(Clone,Hash,PartialEq,Eq)]
struct Location {
	mir_def: DefId,
	block: BasicBlock
}

impl Location {
	fn new(mir_def: DefId,  block: BasicBlock) -> Location {
		Location {
			mir_def,
			block
		}
	}

	fn get_def_id(&self) -> DefId {
		self.mir_def
	}

	fn get_block_data<'tcx>(&self, mirs: &HashMap<DefId,&'tcx Mir<'tcx>>) -> &'tcx BasicBlockData<'tcx> {
		&mirs.get(&self.mir_def).unwrap().basic_blocks()[self.block]	
	}

	fn get_statements<'tcx>(&self,mirs: &HashMap<DefId,&'tcx Mir<'tcx>>) -> &Vec<Statement<'tcx>> {
		&self.get_block_data(mirs).statements
	}

	fn get_local_decl<'tcx>(&self, lcl: Local, mir: &HashMap<DefId,&'tcx Mir<'tcx>>) -> &LocalDecl<'tcx> {
		&mir.get(&self.mir_def).unwrap().local_decls[lcl]
	}


	fn from_block(&self, block: BasicBlock) -> Location {
		Location {
			block,
			mir_def: self.mir_def
		}
	}


}

struct Frame<'tcx> {
	generator: Option<NodeId>,
	precondition: Option<Expr>,
	current_memory: Memory<'tcx>,
	seen_counts: HashMap<Location, usize>,
	current_loc: Location,
	return_to: Vec<(Location)>
} 

impl <'tcx> Frame<'tcx> {

	fn from_new_loc(&self, new_generator: NodeId,new_loc: Location, precondition: Option<Expr>, memory: Memory<'tcx>) -> Frame<'tcx> {
		let mut seen_counts = self.seen_counts.clone();
		seen_counts.insert(new_loc.clone(), seen_counts.get(&new_loc).unwrap_or(&0) + 1);
		Frame {
			generator: Some(new_generator),
			current_loc: new_loc,
			return_to: self.return_to.clone(),
			seen_counts,
			current_memory: memory,
			precondition
		}
	}

	fn derive_next_frames(&mut self, nid: NodeId, term: &Terminator<'tcx>, sir: &mut Sir) -> impl Iterator<Item = Frame<'tcx>> {
		match &term.kind {
			TerminatorKind::Goto {target} => if let Some(conv) = self.derive_goto(nid,*target) {
				vec![conv]
			} else {
				vec![]
			}.into_iter(),
			TerminatorKind::Call {func:_,args:_,destination:_,cleanup:_,from_hir_call:_} => unimplemented!(),
			TerminatorKind::SwitchInt{discr, switch_ty,values,targets} => self.derive_switch_int(nid,discr, switch_ty, values,targets.clone(),sir).into_iter(),
			TerminatorKind::Assert{expected,cond,msg:_,target, cleanup:_} => {
				let test_val = SymTy::from_boolean(*expected);
				let rand = self.current_memory.process_operand(cond.clone(), nid, sir);
				let assert_expr = Expr::BinOp(Rator::Eq, Box::new(Expr::Value(test_val)), Box::new(rand));
				sir.add_expr_to_node(nid,assert_expr);
				if let Some(conv) = self.derive_goto(nid,*target) {
				vec![conv]
			} else {
				vec![]
			}.into_iter()},
			TerminatorKind::Return => vec![].into_iter(),
			_ => unimplemented!(),
		}
	}

	fn derive_switch_int(&self, generator: NodeId, discr: &Operand<'tcx>, switch_ty: Ty, values: &Cow<'tcx,[u128]>, mut targets: Vec<BasicBlock>, sir: &mut Sir) -> Vec<Frame<'tcx>> {
		let mut new_mem = self.current_memory.clone();
		let compare_to = new_mem.process_operand(discr.clone(), generator, sir);
		let otherwise_target = targets.pop().unwrap();
		let all_other = targets.iter(); 
		let (mut expressions, mut frames): (Vec<Expr>, Vec<Option<Frame>>) = values.iter().zip(all_other).map(|(desired_val, target)| {
			let comp = Expr::Value(SymTy::from_scalar(*desired_val, switch_ty));
			let prec = Expr::BinOp(Rator::Eq, Box::new(comp), Box::new(compare_to.clone()));
			(prec.clone(), self.block_to_frame(generator, *target, Some(prec), new_mem.clone()))
		}).unzip();


		let init_val = expressions.pop().unwrap();

		let otherwise_expr = Expr::UnOp(Rator::Not, Box::new(expressions.into_iter().fold(init_val, |x,y| Expr::BinOp(Rator::And, Box::new(x), Box::new(y)))));

		frames.push(self.block_to_frame(generator, otherwise_target, Some(otherwise_expr),new_mem));

		frames.into_iter().filter_map(|x| if let Some(val) = x {
			Some(val)
		} else {
			None
		}).collect()
	}


	fn derive_goto(&self, generator: NodeId, target: BasicBlock ) -> Option<Frame<'tcx>> {
		self.block_to_frame(generator,target, None,self.current_memory.clone())
	}

	fn block_to_frame(&self, generator: NodeId, target: BasicBlock, precondition: Option<Expr>,memory: Memory<'tcx>) -> Option<Frame<'tcx>> {
		let n_loc = self.current_loc.from_block(target);
		if !self.should_examine(&n_loc) {
			None
		} else {
			Some(self.from_new_loc(generator, n_loc, precondition, memory))
		}
	}


	fn should_examine(&self, target_loc: &Location) -> bool {
		*self.seen_counts.get(target_loc).unwrap_or(&0) < MAX_UNROLL
	}


	fn remove_var(&mut self, lcl: Local) {
		self.current_memory.remove_var(&Place::Base(PlaceBase::Local(lcl)));
	}

	fn get_block_data(&self, mirs: &HashMap<DefId,&'tcx Mir<'tcx>>) -> &BasicBlockData<'tcx> {
		self.current_loc.get_block_data(mirs)
	}

	fn get_statements(&self,mirs: &HashMap<DefId,&'tcx Mir<'tcx>>) -> &Vec<Statement<'tcx>> {
		self.current_loc.get_statements(mirs)
	}


	fn get_local_decl(&self, lcl: Local, mir: &HashMap<DefId,&'tcx Mir<'tcx>>) -> &LocalDecl<'tcx> {
		self.current_loc.get_local_decl(lcl, mir)
	}

	fn add_var(&mut self, lcl: Local, mir: &HashMap<DefId,&'tcx Mir<'tcx>>, sir: &mut Sir) {
		let plc = PlaceBase::Local(lcl);
		let dcl = self.get_local_decl(lcl, mir);
		self.current_memory.add_new_var(Place::Base(plc), dcl.ty,sir, Some((self.current_loc.get_def_id(),lcl)));
	}

	fn assign(&mut self, to: &Place<'tcx>, from: &Box<Rvalue<'tcx>>, nid: NodeId, sir: &mut Sir) { 
		let expr = self.evaluate_rvalue(from.clone(),nid,sir);
		let new_name = self.current_memory.new_assignment(to.clone(),nid,sir);
		let set = Expr::BinOp(Rator::Eq, Box::new(Expr::Ref(new_name)), Box::new(expr));
		sir.add_expr_to_node(nid,set);
	}

	fn evaluate_rvalue(&mut self,rval: Box<Rvalue<'tcx>>, nid: NodeId, sir: &mut Sir) -> Expr {
		match *rval {
			Rvalue::Use(rand) => self.current_memory.process_operand(rand,nid,sir),
			Rvalue::BinaryOp(binop, rand1, rand2) => Expr::BinOp(Rator::from_mir_bin(&binop),
				Box::new(self.current_memory.process_operand(rand1,nid,sir)),Box::new(self.current_memory.process_operand(rand2,nid,sir))),
			Rvalue::UnaryOp(unop, rand) => Expr::UnOp(Rator::from_mir_un(&unop), Box::new(self.current_memory.process_operand(rand,nid,sir))),
			Rvalue::Cast(_,rand,_) => self.current_memory.process_operand(rand,nid,sir),
			Rvalue::Ref(_,_,plc) => Expr::Ref(self.current_memory.process_plc(&plc,nid, sir)),
			Rvalue::CheckedBinaryOp(binop,rand1,rand2) => Expr::BinOp(Rator::from_mir_bin(&binop),
				Box::new(self.current_memory.process_operand(rand1,nid,sir)),Box::new(self.current_memory.process_operand(rand2,nid,sir))),
			_ => unimplemented!(),
		}
	}

	fn create_entry(def_id: DefId, mir: &Mir<'tcx>, sir: &mut Sir) -> Frame<'tcx> {
		let bid = BasicBlock::from(0 as usize);
		let args = mir.args_iter();
		let frm = Frame {
			seen_counts: HashMap::new(),
			generator: None,
			precondition: None,
			current_memory: Memory::from_args(args, def_id, mir, sir),
			current_loc: Location::new(def_id,bid),
			return_to: Vec::new()
		};

		frm
	}


	fn add_edge_to(&self, my_id: NodeId, sir: &mut Sir) {
		if let Some(gen) = self.generator {
			sir.add_edge(gen,Edge::new(self.precondition.clone(),my_id ))
		}
	}
}

pub struct ExecutionContext<'tcx> {
	mirs: HashMap<DefId,&'tcx Mir<'tcx>>,
	stack: Vec<Frame<'tcx>>,
	result: Sir
}

impl <'tcx> ExecutionContext<'tcx> {
	pub fn evaluate(mut self) -> (Sir, NodeId) {
		let mut entry = None;
		while let Some(mut curr_frame) =  self.stack.pop() {
			let cid = self.process_frame(&mut curr_frame);
			if entry.is_none() {
				entry = Some(cid);
			}
		}

		(self.result, entry.unwrap())
	}

	pub fn create_from_entry(entry: DefId, mirs: HashMap<DefId,&'tcx Mir<'tcx>>) -> ExecutionContext {
		let mut stack = Vec::new();
		let mut result = Sir::new();
		let frm = Frame::create_entry(entry, mirs.get(&entry).unwrap(), &mut result);

		stack.push(frm);

		ExecutionContext {
			mirs,
			stack,
			result
		}
	}


	fn process_frame(&mut self, curr_frame: &mut Frame<'tcx>) -> NodeId {
		let stats:Vec<Statement<'tcx>> = curr_frame.get_statements(&self.mirs).clone().drain(..).collect();
		let blk:BasicBlockData<'tcx> = curr_frame.get_block_data(&self.mirs).clone();

		let nid = self.perform_statements(curr_frame, &stats);
		self.push_next_frames(blk,curr_frame,nid);
		curr_frame.add_edge_to(nid, &mut self.result);
		nid
	}


	fn push_next_frames(&mut self, blk: BasicBlockData<'tcx>, curr_frame: &mut Frame<'tcx>, nid: NodeId) {
		let term = blk.terminator();
		for fr in curr_frame.derive_next_frames(nid, term, &mut self.result) {
			self.stack.push(fr);
		}
	}

	fn perform_statements(&mut self, curr_frame: &mut Frame<'tcx>, statements: &Vec<Statement<'tcx>>) -> NodeId {
		let nid = self.result.add_node();
		for stat in statements {
			match &stat.kind {
				StatementKind::Assign(to,from) => curr_frame.assign(to,from, nid, &mut self.result),
				StatementKind::StorageLive(lcl) => {curr_frame.add_var(*lcl,&self.mirs, &mut self.result)},
				StatementKind::StorageDead(lcl) => {curr_frame.remove_var(*lcl)},
				StatementKind::Nop => (),
				_ => unimplemented!(),
			}
		}
		nid
	} 
}

