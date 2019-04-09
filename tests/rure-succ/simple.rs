#![crate_type="lib"]

fn safe_deref(p: *const u32) -> u32 {
    if pointer_is_null(p) {
        unsafe {
            *p
        }
    } else {
        0
    } 
}

#[inline(always)] 
fn pointer_is_null(p: *const u32) -> bool {
	p as usize == 0
}