use std::ops;
use rsmt2::print::Sym2Smt;
use rsmt2::errors::SmtRes;
use std::io::Write;

#[derive(Debug)]
pub struct NodeVec<V>(Vec<V>);

#[derive(Clone, Copy,Eq,PartialEq, Debug)]
pub struct NodeId(usize);

impl<V> ops::Index<NodeId> for NodeVec<V> {
    type Output = V;

    fn index(&self, idx: NodeId) -> &V {
        &self.0[idx.0]
    }
}

impl<V> ops::IndexMut<NodeId> for NodeVec<V> {

    fn index_mut(&mut self, idx: NodeId) -> &mut V {
        &mut self.0[idx.0]
    }
}

impl<V> NodeVec<V> {
	pub fn push(&mut self, item: V) -> NodeId {
		let ind = self.0.len();
		self.0.push(item);

		NodeId(ind)
	}

	pub fn new() -> NodeVec<V> {
		NodeVec(vec![])
	}
}

#[derive(Debug)]
pub struct NameVec<V>(Vec<V>);


pub struct NameIter {
	curr_index: usize,
	num_items: usize
}

impl NameIter {
	fn new(num_items: usize) -> NameIter {
		NameIter {
			curr_index: 0,
			num_items
		}
	}
}

impl Iterator for NameIter {
	type Item = Name;

	fn next(&mut self) -> Option<Name> {
		if self.curr_index < self.num_items {
			let item = Some(Name(self.curr_index));
			self.curr_index += 1; 
			item
		} else {
			None
		}
	}
}

impl <V> IntoIterator for &NameVec<V> {
	type Item = Name;
	type IntoIter = NameIter;
	fn into_iter(self) -> NameIter {
		NameIter::new(self.0.len())
	}
}


#[derive(Hash,Clone, Copy,Eq,PartialEq, Debug)]
pub struct Name(usize);


impl Name {
	pub fn to_id(&self) -> String {
		format!("x{}",self.0)
	}


	pub fn from_str(val: &str) -> Name {
		let id: usize;
		scan!(val.bytes() => "x{}", id);
		Name(id)
	}
}

impl Sym2Smt<()> for Name {

	fn sym_to_smt2<T: Write>(&self, wtr:&mut T, _: ()) -> SmtRes<()> {
		write!(wtr,"{}", self.to_id())?;
		Ok(())
	}
}

impl<V> ops::Index<Name> for NameVec<V> {
    type Output = V;

    fn index(&self, idx: Name) -> &V {
        &self.0[idx.0]
    }
}

impl<V> ops::IndexMut<Name> for NameVec<V> {

    fn index_mut(&mut self, idx: Name) -> &mut V {
        &mut self.0[idx.0]
    }
}

impl<V> NameVec<V> {
	pub fn push(&mut self, item: V) -> Name {
		let ind = self.0.len();
		self.0.push(item);

		Name(ind)
	}

	pub fn new() -> NameVec<V> {
		NameVec(vec![])
	}
}

