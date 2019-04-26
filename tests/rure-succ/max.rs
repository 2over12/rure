#![crate_type="lib"]

fn simple_max(a: usize, b: usize) -> usize {
    if a > b {
        unsafe {
            a
        }
    } else {
        b
    }
}
