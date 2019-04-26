#![crate_type="lib"]

fn simple_max(a: usize, b: usize, c: *const usize) -> usize {
    if a + 1 > b {
        unsafe {
           *c
        }
    } else {
        b
    }
}