#![crate_type="lib"]

fn safe_deref(p: *const u32) -> u32 {
    if !p.is_null() {
        unsafe {
            *p
        }
    } else {
        0
    } 
}