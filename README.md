# RURE (Reliable Unsafe Rust Engine)

RURE's goal is to allow rust programmers to have confidence in their unsafe code. RURE accomplishes this by applying symbolic verification to safe functions that contain unsafe blocks. Symbolic verification allows rure to model the function's behavior accross all inputs to discover witnesses for the undefined behaviors listed here: https://doc.rust-lang.org/reference/behavior-considered-undefined.html

## Current State:
Recently finished extremely basic symbolic execution foir booleans and integers that as an MVP can detect if a pointer could be dereferenced as null.

## Next Steps:
* Reimplement function inlining (This should be relatively simple most of the work is already done)
* Return results of analysis to the user with error info and the witness.
* Proper handling for projections such as references and structs
* Analysis passes for other undefined behavior. Invalid values in primitive types might be a good next step. 
