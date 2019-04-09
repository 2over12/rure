#![crate_type="lib"]

fn safe_deref(p: *const u32) -> u32 {
    if ! (p as usize == 0) {
        unsafe {
            *p
        }
    } else {
        0
    } 
}

/*
(declare-fun x0 () Int )
(declare-fun x1 () Int )
(declare-fun x2 () Bool )
(declare-fun x3 () Bool )
(declare-fun x4 () Int )
(declare-fun x5 () Int )
(declare-fun x6 () Int )
(declare-fun x7 () Int )
(declare-fun x8 () Bool )
(declare-fun x9 () Bool )
(declare-fun x10 () Int )
(declare-fun x11 () Int )
(declare-fun x12 () Int )
(declare-fun x13 () Int )
(declare-fun x14 () Int )
(assert (not (= x9 false )))
(assert (= x1 0))
(assert (and true (= x6 x1 )(= x7 x6 )(= x8 (= x7 0 ))(= x9 (not x8 ))(=> (= x9 false )(and true (= x10 0 )(and true (= x13 x10 )(= x14 x1 ))))(=> (not (and (= x9 false )true ))(and true (= x11 x12 )(and true (= x13 x11 )(= x14 x1 ))))))
(check-sat)
*/