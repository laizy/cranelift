//! Emitting binary ARM32 machine code.

use crate::binemit::{bad_encoding, CodeSink};
use crate::ir::{Function, Inst};
use crate::regalloc::RegDiversions;

/// Emit binary machine code for `inst` for the arm32 ISA.
pub fn emit_inst<CS: CodeSink + ?Sized>(
    func: &Function,
    inst: Inst,
    _divert: &mut RegDiversions,
    _sink: &mut CS,
) {
    bad_encoding(func, inst)
}

//clude!(concat!(env!("OUT_DIR"), "/binemit-arm32.rs"));
