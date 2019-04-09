#![crate_type="lib"]

fn stupid_func(p: usize) -> u32 {

    let v: *const u32 = p as *const u32;
    unsafe {
            return *v;
    }
}

fn nullish() {

}

struct Simple {

}

impl Simple {
	fn method(&self) {
		println!("hi");
	}

	unsafe fn bad(&self) {
		unsafe {
			println!("bad");
		}
	}

	unsafe fn bad_but_good(&self) -> u8 {
		unsafe {
			1+1
		}
	}
}