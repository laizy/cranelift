//! Legalize instructions.
//!
//! A legal instruction is one that can be mapped directly to a machine code instruction for the
//! target ISA. The `legalize_function()` function takes as input any function and transforms it
//! into an equivalent function using only legal instructions.
//!
//! The characteristics of legal instructions depend on the target ISA, so any given instruction
//! can be legal for one ISA and illegal for another.
//!
//! Besides transforming instructions, the legalizer also fills out the `function.encodings` map
//! which provides a legal encoding recipe for every instruction.
//!
//! The legalizer does not deal with register allocation constraints. These constraints are derived
//! from the encoding recipes, and solved later by the register allocator.

use crate::bitset::BitSet;
use crate::cursor::{Cursor, FuncCursor};
use crate::flowgraph::ControlFlowGraph;
use crate::ir::types::{I32, I64};
use crate::ir::{self, InstBuilder, MemFlags};
use crate::isa::TargetIsa;
use crate::predicates;
use crate::timing;
use alloc::collections::BTreeSet;
use alloc::vec::Vec;

mod boundary;
mod call;
mod globalvalue;
mod heap;
mod libcall;
mod split;
mod table;

use self::call::expand_call;
use self::globalvalue::expand_global_value;
use self::heap::expand_heap_addr;
use self::libcall::expand_as_libcall;
use self::table::expand_table_addr;

enum LegalizeInstResult {
    Done,
    Legalized,
    SplitLegalizePending,
}

/// Legalize `inst` for `isa`.
fn legalize_inst(
    inst: ir::Inst,
    pos: &mut FuncCursor,
    cfg: &mut ControlFlowGraph,
    isa: &dyn TargetIsa,
) -> LegalizeInstResult {
    let opcode = pos.func.dfg[inst].opcode();

    // Check for ABI boundaries that need to be converted to the legalized signature.
    if opcode.is_call() {
        if boundary::handle_call_abi(isa, inst, pos.func, cfg) {
            return LegalizeInstResult::Legalized;
        }
    } else if opcode.is_return() {
        if boundary::handle_return_abi(inst, pos.func, cfg) {
            return LegalizeInstResult::Legalized;
        }
    } else if opcode.is_branch() {
        split::simplify_branch_arguments(&mut pos.func.dfg, inst);
    } else if opcode == ir::Opcode::Isplit {
        pos.use_srcloc(inst);

        let arg = match pos.func.dfg[inst] {
            ir::InstructionData::Unary { arg, .. } => pos.func.dfg.resolve_aliases(arg),
            _ => panic!("Expected isplit: {}", pos.func.dfg.display_inst(inst, None)),
        };

        match pos.func.dfg.value_def(arg) {
            ir::ValueDef::Result(inst, _num) => {
                if let ir::InstructionData::Binary {
                    opcode: ir::Opcode::Iconcat,
                    ..
                } = pos.func.dfg[inst]
                {
                    // `arg` was created by an `iconcat` instruction.
                } else {
                    // `arg` was not created by an `iconcat` instruction. Don't try to resolve it,
                    // as otherwise `split::isplit` will re-insert the original `isplit`, causing
                    // an endless loop.
                    return LegalizeInstResult::SplitLegalizePending;
                }
            }
            ir::ValueDef::Param(_ebb, _num) => {}
        }

        let res = pos.func.dfg.inst_results(inst).to_vec();
        assert_eq!(res.len(), 2);
        let (resl, resh) = (res[0], res[1]); // Prevent borrowck error

        // Remove old isplit
        pos.func.dfg.clear_results(inst);
        pos.remove_inst();

        let curpos = pos.position();
        let srcloc = pos.srcloc();
        let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, arg);

        pos.func.dfg.change_to_alias(resl, xl);
        pos.func.dfg.change_to_alias(resh, xh);

        return LegalizeInstResult::Legalized;
    }

    match pos.func.update_encoding(inst, isa) {
        Ok(()) => LegalizeInstResult::Done,
        Err(action) => {
            // We should transform the instruction into legal equivalents.
            // If the current instruction was replaced, we need to double back and revisit
            // the expanded sequence. This is both to assign encodings and possible to
            // expand further.
            // There's a risk of infinite looping here if the legalization patterns are
            // unsound. Should we attempt to detect that?
            if action(inst, pos.func, cfg, isa) {
                return LegalizeInstResult::Legalized;
            }

            // We don't have any pattern expansion for this instruction either.
            // Try converting it to a library call as a last resort.
            if expand_as_libcall(inst, pos.func, isa) {
                LegalizeInstResult::Legalized
            } else {
                LegalizeInstResult::Done
            }
        }
    }
}

/// Legalize `func` for `isa`.
///
/// - Transform any instructions that don't have a legal representation in `isa`.
/// - Fill out `func.encodings`.
///
pub fn legalize_function(func: &mut ir::Function, cfg: &mut ControlFlowGraph, isa: &dyn TargetIsa) {
    let _tt = timing::legalize();
    debug_assert!(cfg.is_valid());

    boundary::legalize_signatures(func, isa);

    func.encodings.resize(func.dfg.num_insts());

    let mut pos = FuncCursor::new(func);
    let func_begin = pos.position();

    // Split ebb params before trying to legalize instructions, so that the newly introduced
    // isplit instructions get legalized.
    while let Some(ebb) = pos.next_ebb() {
        split::split_ebb_params(pos.func, cfg, ebb);
    }

    pos.set_position(func_begin);

    // This must be a set to prevent trying to legalize `isplit` and `vsplit` twice in certain cases.
    let mut pending_splits = BTreeSet::new();

    // Process EBBs in layout order. Some legalization actions may split the current EBB or append
    // new ones to the end. We need to make sure we visit those new EBBs too.
    while let Some(_ebb) = pos.next_ebb() {
        // Keep track of the cursor position before the instruction being processed, so we can
        // double back when replacing instructions.
        let mut prev_pos = pos.position();

        while let Some(inst) = pos.next_inst() {
            match legalize_inst(inst, &mut pos, cfg, isa) {
                // Remember this position in case we need to double back.
                LegalizeInstResult::Done => prev_pos = pos.position(),

                // Go back and legalize the inserted return value conversion instructions.
                LegalizeInstResult::Legalized => pos.set_position(prev_pos),

                // The argument of a `isplit` or `vsplit` instruction didn't resolve to a
                // `iconcat` or `vconcat` instruction. Try again after legalizing the rest of
                // the instructions.
                LegalizeInstResult::SplitLegalizePending => {
                    pending_splits.insert(inst);
                }
            }
        }
    }

    // Try legalizing `isplit` and `vsplit` instructions, which could not previously be legalized.
    for inst in pending_splits {
        pos.goto_inst(inst);
        legalize_inst(inst, &mut pos, cfg, isa);
    }

    // Now that we've lowered all br_tables, we don't need the jump tables anymore.
    if !isa.flags().jump_tables_enabled() {
        pos.func.jump_tables.clear();
    }
}

// Include legalization patterns that were generated by `gen_legalizer.rs` from the
// `TransformGroup` in `cranelift-codegen/meta/shared/legalize.rs`.
//
// Concretely, this defines private functions `narrow()`, and `expand()`.
 
/// Legalize instructions by expansion.
///
/// Rewrite instructions in terms of other instructions, generally
/// operating on the same types as the original instructions.
#[allow(unused_variables,unused_assignments,non_snake_case)]
pub fn expand(
    inst: crate::ir::Inst,
    func: &mut crate::ir::Function,
    cfg: &mut crate::flowgraph::ControlFlowGraph,
    isa: &dyn crate::isa::TargetIsa,
) -> bool {
    use crate::ir::InstBuilder;
    use crate::cursor::{Cursor, FuncCursor};
    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);
    {
        match pos.func.dfg[inst].opcode() {
            ir::Opcode::BandImm => {
                // Unwrap fields from instruction format a := band_imm(x, y)
                let (x, y, args) = if let ir::InstructionData::BinaryImm {
                    imm,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        imm,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := band(x, a1).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let a1 = pos.ins().iconst(typeof_x, y);
                let a = pos.func.dfg.replace(inst).band(x, a1);
                if pos.current_inst() == Some(inst) {
                    pos.next_inst();
                }
                return true;
            }

            ir::Opcode::BandNot => {
                // Unwrap fields from instruction format a := band_not(x, y)
                let (x, y, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := band(x, a1).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let a1 = pos.ins().bnot(y);
                let a = pos.func.dfg.replace(inst).band(x, a1);
                if pos.current_inst() == Some(inst) {
                    pos.next_inst();
                }
                return true;
            }

            ir::Opcode::Bitrev => {
                // Unwrap fields from instruction format a := bitrev.i32(x)
                let (x, args) = if let ir::InstructionData::Unary {
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := bor(e1, e2).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I32 {
                    let a1 = pos.ins().band_imm(x, 2863311530);
                    let a2 = pos.ins().ushr_imm(a1, 1);
                    let a3 = pos.ins().band_imm(x, 1431655765);
                    let a4 = pos.ins().ishl_imm(a3, 1);
                    let b = pos.ins().bor(a2, a4);
                    let b1 = pos.ins().band_imm(b, 3435973836);
                    let b2 = pos.ins().ushr_imm(b1, 2);
                    let b3 = pos.ins().band_imm(b, 858993459);
                    let b4 = pos.ins().ishl_imm(b3, 2);
                    let c = pos.ins().bor(b2, b4);
                    let c1 = pos.ins().band_imm(c, 4042322160);
                    let c2 = pos.ins().ushr_imm(c1, 4);
                    let c3 = pos.ins().band_imm(c, 252645135);
                    let c4 = pos.ins().ishl_imm(c3, 4);
                    let d = pos.ins().bor(c2, c4);
                    let d1 = pos.ins().band_imm(d, 4278255360);
                    let d2 = pos.ins().ushr_imm(d1, 8);
                    let d3 = pos.ins().band_imm(d, 16711935);
                    let d4 = pos.ins().ishl_imm(d3, 8);
                    let e = pos.ins().bor(d2, d4);
                    let e1 = pos.ins().ushr_imm(e, 16);
                    let e2 = pos.ins().ishl_imm(e, 16);
                    let a = pos.func.dfg.replace(inst).bor(e1, e2);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I64 {
                    let a1 = pos.ins().band_imm(x, -6148914691236517206);
                    let a2 = pos.ins().ushr_imm(a1, 1);
                    let a3 = pos.ins().band_imm(x, 6148914691236517205);
                    let a4 = pos.ins().ishl_imm(a3, 1);
                    let b = pos.ins().bor(a2, a4);
                    let b1 = pos.ins().band_imm(b, -3689348814741910324);
                    let b2 = pos.ins().ushr_imm(b1, 2);
                    let b3 = pos.ins().band_imm(b, 3689348814741910323);
                    let b4 = pos.ins().ishl_imm(b3, 2);
                    let c = pos.ins().bor(b2, b4);
                    let c1 = pos.ins().band_imm(c, -1085102592571150096);
                    let c2 = pos.ins().ushr_imm(c1, 4);
                    let c3 = pos.ins().band_imm(c, 1085102592571150095);
                    let c4 = pos.ins().ishl_imm(c3, 4);
                    let d = pos.ins().bor(c2, c4);
                    let d1 = pos.ins().band_imm(d, -71777214294589696);
                    let d2 = pos.ins().ushr_imm(d1, 8);
                    let d3 = pos.ins().band_imm(d, 71777214294589695);
                    let d4 = pos.ins().ishl_imm(d3, 8);
                    let e = pos.ins().bor(d2, d4);
                    let e1 = pos.ins().band_imm(e, -281470681808896);
                    let e2 = pos.ins().ushr_imm(e1, 16);
                    let e3 = pos.ins().band_imm(e, 281470681808895);
                    let e4 = pos.ins().ishl_imm(e3, 16);
                    let f = pos.ins().bor(e2, e4);
                    let f1 = pos.ins().ushr_imm(f, 32);
                    let f2 = pos.ins().ishl_imm(f, 32);
                    let a = pos.func.dfg.replace(inst).bor(f1, f2);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Bnot => {
                // Unwrap fields from instruction format a := bnot(x)
                let (x, args) = if let ir::InstructionData::Unary {
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := bxor(x, y).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let predicate = true;
                // typeof_x must belong to TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
                let predicate = predicate && TYPE_SETS[0].contains(typeof_x);
                if predicate {
                    let y = pos.ins().iconst(typeof_x, -1);
                    let a = pos.func.dfg.replace(inst).bxor(x, y);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::BorImm => {
                // Unwrap fields from instruction format a := bor_imm(x, y)
                let (x, y, args) = if let ir::InstructionData::BinaryImm {
                    imm,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        imm,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := bor(x, a1).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let a1 = pos.ins().iconst(typeof_x, y);
                let a = pos.func.dfg.replace(inst).bor(x, a1);
                if pos.current_inst() == Some(inst) {
                    pos.next_inst();
                }
                return true;
            }

            ir::Opcode::BorNot => {
                // Unwrap fields from instruction format a := bor_not(x, y)
                let (x, y, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := bor(x, a1).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let a1 = pos.ins().bnot(y);
                let a = pos.func.dfg.replace(inst).bor(x, a1);
                if pos.current_inst() == Some(inst) {
                    pos.next_inst();
                }
                return true;
            }

            ir::Opcode::BxorImm => {
                // Unwrap fields from instruction format a := bxor_imm(x, y)
                let (x, y, args) = if let ir::InstructionData::BinaryImm {
                    imm,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        imm,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := bxor(x, a1).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let a1 = pos.ins().iconst(typeof_x, y);
                let a = pos.func.dfg.replace(inst).bxor(x, a1);
                if pos.current_inst() == Some(inst) {
                    pos.next_inst();
                }
                return true;
            }

            ir::Opcode::BxorNot => {
                // Unwrap fields from instruction format a := bxor_not(x, y)
                let (x, y, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := bxor(x, a1).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let a1 = pos.ins().bnot(y);
                let a = pos.func.dfg.replace(inst).bxor(x, a1);
                if pos.current_inst() == Some(inst) {
                    pos.next_inst();
                }
                return true;
            }

            ir::Opcode::Fabs => {
                // Unwrap fields from instruction format a := fabs.f32(x)
                let (x, args) = if let ir::InstructionData::Unary {
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := band_not(x, b).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::F32 {
                    let b = pos.ins().f32const(ir::immediates::Ieee32::with_bits(0x80000000));
                    let a = pos.func.dfg.replace(inst).band_not(x, b);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::F64 {
                    let b = pos.ins().f64const(ir::immediates::Ieee64::with_bits(0x8000000000000000));
                    let a = pos.func.dfg.replace(inst).band_not(x, b);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Fcopysign => {
                // Unwrap fields from instruction format a := fcopysign.f32(x, y)
                let (x, y, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := bor(a1, a2).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::F32 {
                    let b = pos.ins().f32const(ir::immediates::Ieee32::with_bits(0x80000000));
                    let a1 = pos.ins().band_not(x, b);
                    let a2 = pos.ins().band(y, b);
                    let a = pos.func.dfg.replace(inst).bor(a1, a2);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::F64 {
                    let b = pos.ins().f64const(ir::immediates::Ieee64::with_bits(0x8000000000000000));
                    let a1 = pos.ins().band_not(x, b);
                    let a2 = pos.ins().band(y, b);
                    let a = pos.func.dfg.replace(inst).bor(a1, a2);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::FcvtFromSint => {
                // Unwrap fields from instruction format a := fcvt_from_sint.f32.i8(b)
                let (b, args) = if let ir::InstructionData::Unary {
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := fcvt_from_sint.f32(x).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 && pos.func.dfg.ctrl_typevar(inst) == ir::types::F32 {
                    let x = pos.ins().sextend(ir::types::I32, b);
                    let a = pos.func.dfg.replace(inst).fcvt_from_sint(ir::types::F32, x);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 && pos.func.dfg.ctrl_typevar(inst) == ir::types::F32 {
                    let x = pos.ins().sextend(ir::types::I32, b);
                    let a = pos.func.dfg.replace(inst).fcvt_from_sint(ir::types::F32, x);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 && pos.func.dfg.ctrl_typevar(inst) == ir::types::F64 {
                    let x = pos.ins().sextend(ir::types::I32, b);
                    let a = pos.func.dfg.replace(inst).fcvt_from_sint(ir::types::F64, x);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 && pos.func.dfg.ctrl_typevar(inst) == ir::types::F64 {
                    let x = pos.ins().sextend(ir::types::I32, b);
                    let a = pos.func.dfg.replace(inst).fcvt_from_sint(ir::types::F64, x);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Fneg => {
                // Unwrap fields from instruction format a := fneg.f32(x)
                let (x, args) = if let ir::InstructionData::Unary {
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := bxor(x, b).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::F32 {
                    let b = pos.ins().f32const(ir::immediates::Ieee32::with_bits(0x80000000));
                    let a = pos.func.dfg.replace(inst).bxor(x, b);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::F64 {
                    let b = pos.ins().f64const(ir::immediates::Ieee64::with_bits(0x8000000000000000));
                    let a = pos.func.dfg.replace(inst).bxor(x, b);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::IaddCarry => {
                // Unwrap fields from instruction format (a, c) := iadd_carry(x, y, c_in)
                let (x, y, c_in, args) = if let ir::InstructionData::Ternary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        pos.func.dfg.resolve_aliases(args[2]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                let a;
                let c;
                {
                    let r = pos.func.dfg.inst_results(inst);
                    a = r[0];
                    c = r[1];
                }

                pos.func.dfg.clear_results(inst);
                let (a1, c1) = pos.ins().iadd_cout(x, y);
                let c_int = pos.ins().bint(typeof_x, c_in);
                let (a, c2) = pos.ins().with_results([Some(a), None]).iadd_cout(a1, c_int);
                let c = pos.ins().with_result(c).bor(c1, c2);
                let removed = pos.remove_inst();
                debug_assert_eq!(removed, inst);
                return true;
            }

            ir::Opcode::IaddCin => {
                // Unwrap fields from instruction format a := iadd_cin(x, y, c)
                let (x, y, c, args) = if let ir::InstructionData::Ternary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        pos.func.dfg.resolve_aliases(args[2]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := iadd(a1, c_int).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let a1 = pos.ins().iadd(x, y);
                let c_int = pos.ins().bint(typeof_x, c);
                let a = pos.func.dfg.replace(inst).iadd(a1, c_int);
                if pos.current_inst() == Some(inst) {
                    pos.next_inst();
                }
                return true;
            }

            ir::Opcode::IaddCout => {
                // Unwrap fields from instruction format (a, c) := iadd_cout(x, y)
                let (x, y, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                let a;
                let c;
                {
                    let r = pos.func.dfg.inst_results(inst);
                    a = r[0];
                    c = r[1];
                }

                pos.func.dfg.clear_results(inst);
                let a = pos.ins().with_result(a).iadd(x, y);
                let c = pos.ins().with_result(c).icmp(ir::condcodes::IntCC::UnsignedLessThan, a, x);
                let removed = pos.remove_inst();
                debug_assert_eq!(removed, inst);
                return true;
            }

            ir::Opcode::IaddImm => {
                // Unwrap fields from instruction format a := iadd_imm(x, y)
                let (x, y, args) = if let ir::InstructionData::BinaryImm {
                    imm,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        imm,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := iadd(x, a1).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let a1 = pos.ins().iconst(typeof_x, y);
                let a = pos.func.dfg.replace(inst).iadd(x, a1);
                if pos.current_inst() == Some(inst) {
                    pos.next_inst();
                }
                return true;
            }

            ir::Opcode::IcmpImm => {
                // Unwrap fields from instruction format a := icmp_imm(cc, x, y)
                let (cc, x, y, args) = if let ir::InstructionData::IntCompareImm {
                    cond,
                    imm,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        cond,
                        pos.func.dfg.resolve_aliases(args[0]),
                        imm,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := icmp(cc, x, a1).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let a1 = pos.ins().iconst(typeof_x, y);
                let a = pos.func.dfg.replace(inst).icmp(cc, x, a1);
                if pos.current_inst() == Some(inst) {
                    pos.next_inst();
                }
                return true;
            }

            ir::Opcode::IfcmpImm => {
                // Unwrap fields from instruction format a := ifcmp_imm(x, y)
                let (x, y, args) = if let ir::InstructionData::BinaryImm {
                    imm,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        imm,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := ifcmp(x, a1).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let a1 = pos.ins().iconst(typeof_x, y);
                let a = pos.func.dfg.replace(inst).ifcmp(x, a1);
                if pos.current_inst() == Some(inst) {
                    pos.next_inst();
                }
                return true;
            }

            ir::Opcode::ImulImm => {
                // Unwrap fields from instruction format a := imul_imm(x, y)
                let (x, y, args) = if let ir::InstructionData::BinaryImm {
                    imm,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        imm,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := imul(x, a1).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let a1 = pos.ins().iconst(typeof_x, y);
                let a = pos.func.dfg.replace(inst).imul(x, a1);
                if pos.current_inst() == Some(inst) {
                    pos.next_inst();
                }
                return true;
            }

            ir::Opcode::IrsubImm => {
                // Unwrap fields from instruction format a := irsub_imm(y, x)
                let (y, x, args) = if let ir::InstructionData::BinaryImm {
                    imm,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        imm,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_y = pos.func.dfg.value_type(y);
                // Results handled by a := isub(a1, y).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let a1 = pos.ins().iconst(typeof_y, x);
                let a = pos.func.dfg.replace(inst).isub(a1, y);
                if pos.current_inst() == Some(inst) {
                    pos.next_inst();
                }
                return true;
            }

            ir::Opcode::IshlImm => {
                // Unwrap fields from instruction format a := ishl_imm(x, y)
                let (x, y, args) = if let ir::InstructionData::BinaryImm {
                    imm,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        imm,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := ishl(x, a1).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let a1 = pos.ins().iconst(ir::types::I32, y);
                let a = pos.func.dfg.replace(inst).ishl(x, a1);
                if pos.current_inst() == Some(inst) {
                    pos.next_inst();
                }
                return true;
            }

            ir::Opcode::IsubBin => {
                // Unwrap fields from instruction format a := isub_bin(x, y, b)
                let (x, y, b, args) = if let ir::InstructionData::Ternary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        pos.func.dfg.resolve_aliases(args[2]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := isub(a1, b_int).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let a1 = pos.ins().isub(x, y);
                let b_int = pos.ins().bint(typeof_x, b);
                let a = pos.func.dfg.replace(inst).isub(a1, b_int);
                if pos.current_inst() == Some(inst) {
                    pos.next_inst();
                }
                return true;
            }

            ir::Opcode::IsubBorrow => {
                // Unwrap fields from instruction format (a, b) := isub_borrow(x, y, b_in)
                let (x, y, b_in, args) = if let ir::InstructionData::Ternary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        pos.func.dfg.resolve_aliases(args[2]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                let a;
                let b;
                {
                    let r = pos.func.dfg.inst_results(inst);
                    a = r[0];
                    b = r[1];
                }

                pos.func.dfg.clear_results(inst);
                let (a1, b1) = pos.ins().isub_bout(x, y);
                let b_int = pos.ins().bint(typeof_x, b_in);
                let (a, b2) = pos.ins().with_results([Some(a), None]).isub_bout(a1, b_int);
                let b = pos.ins().with_result(b).bor(b1, b2);
                let removed = pos.remove_inst();
                debug_assert_eq!(removed, inst);
                return true;
            }

            ir::Opcode::IsubBout => {
                // Unwrap fields from instruction format (a, b) := isub_bout(x, y)
                let (x, y, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                let a;
                let b;
                {
                    let r = pos.func.dfg.inst_results(inst);
                    a = r[0];
                    b = r[1];
                }

                pos.func.dfg.clear_results(inst);
                let a = pos.ins().with_result(a).isub(x, y);
                let b = pos.ins().with_result(b).icmp(ir::condcodes::IntCC::UnsignedGreaterThan, a, x);
                let removed = pos.remove_inst();
                debug_assert_eq!(removed, inst);
                return true;
            }

            ir::Opcode::RotlImm => {
                // Unwrap fields from instruction format a := rotl_imm(x, y)
                let (x, y, args) = if let ir::InstructionData::BinaryImm {
                    imm,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        imm,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := rotl(x, a1).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let a1 = pos.ins().iconst(ir::types::I32, y);
                let a = pos.func.dfg.replace(inst).rotl(x, a1);
                if pos.current_inst() == Some(inst) {
                    pos.next_inst();
                }
                return true;
            }

            ir::Opcode::RotrImm => {
                // Unwrap fields from instruction format a := rotr_imm(x, y)
                let (x, y, args) = if let ir::InstructionData::BinaryImm {
                    imm,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        imm,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := rotr(x, a1).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let a1 = pos.ins().iconst(ir::types::I32, y);
                let a = pos.func.dfg.replace(inst).rotr(x, a1);
                if pos.current_inst() == Some(inst) {
                    pos.next_inst();
                }
                return true;
            }

            ir::Opcode::SdivImm => {
                // Unwrap fields from instruction format a := sdiv_imm(x, y)
                let (x, y, args) = if let ir::InstructionData::BinaryImm {
                    imm,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        imm,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := sdiv(x, a1).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let a1 = pos.ins().iconst(typeof_x, y);
                let a = pos.func.dfg.replace(inst).sdiv(x, a1);
                if pos.current_inst() == Some(inst) {
                    pos.next_inst();
                }
                return true;
            }

            ir::Opcode::SremImm => {
                // Unwrap fields from instruction format a := srem_imm(x, y)
                let (x, y, args) = if let ir::InstructionData::BinaryImm {
                    imm,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        imm,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := srem(x, a1).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let a1 = pos.ins().iconst(typeof_x, y);
                let a = pos.func.dfg.replace(inst).srem(x, a1);
                if pos.current_inst() == Some(inst) {
                    pos.next_inst();
                }
                return true;
            }

            ir::Opcode::SshrImm => {
                // Unwrap fields from instruction format a := sshr_imm(x, y)
                let (x, y, args) = if let ir::InstructionData::BinaryImm {
                    imm,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        imm,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := sshr(x, a1).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let a1 = pos.ins().iconst(ir::types::I32, y);
                let a = pos.func.dfg.replace(inst).sshr(x, a1);
                if pos.current_inst() == Some(inst) {
                    pos.next_inst();
                }
                return true;
            }

            ir::Opcode::UdivImm => {
                // Unwrap fields from instruction format a := udiv_imm(x, y)
                let (x, y, args) = if let ir::InstructionData::BinaryImm {
                    imm,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        imm,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := udiv(x, a1).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let a1 = pos.ins().iconst(typeof_x, y);
                let a = pos.func.dfg.replace(inst).udiv(x, a1);
                if pos.current_inst() == Some(inst) {
                    pos.next_inst();
                }
                return true;
            }

            ir::Opcode::UremImm => {
                // Unwrap fields from instruction format a := urem_imm(x, y)
                let (x, y, args) = if let ir::InstructionData::BinaryImm {
                    imm,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        imm,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := urem(x, a1).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let a1 = pos.ins().iconst(typeof_x, y);
                let a = pos.func.dfg.replace(inst).urem(x, a1);
                if pos.current_inst() == Some(inst) {
                    pos.next_inst();
                }
                return true;
            }

            ir::Opcode::UshrImm => {
                // Unwrap fields from instruction format a := ushr_imm(x, y)
                let (x, y, args) = if let ir::InstructionData::BinaryImm {
                    imm,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        imm,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := ushr(x, a1).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let a1 = pos.ins().iconst(ir::types::I32, y);
                let a = pos.func.dfg.replace(inst).ushr(x, a1);
                if pos.current_inst() == Some(inst) {
                    pos.next_inst();
                }
                return true;
            }

            ir::Opcode::BrIcmp => {
                expand_br_icmp(inst, func, cfg, isa);
                return true;
            }

            ir::Opcode::BrTable => {
                expand_br_table(inst, func, cfg, isa);
                return true;
            }

            ir::Opcode::Call => {
                expand_call(inst, func, cfg, isa);
                return true;
            }

            ir::Opcode::F32const => {
                expand_fconst(inst, func, cfg, isa);
                return true;
            }

            ir::Opcode::F64const => {
                expand_fconst(inst, func, cfg, isa);
                return true;
            }

            ir::Opcode::GlobalValue => {
                expand_global_value(inst, func, cfg, isa);
                return true;
            }

            ir::Opcode::HeapAddr => {
                expand_heap_addr(inst, func, cfg, isa);
                return true;
            }

            ir::Opcode::Select => {
                expand_select(inst, func, cfg, isa);
                return true;
            }

            ir::Opcode::StackLoad => {
                expand_stack_load(inst, func, cfg, isa);
                return true;
            }

            ir::Opcode::StackStore => {
                expand_stack_store(inst, func, cfg, isa);
                return true;
            }

            ir::Opcode::TableAddr => {
                expand_table_addr(inst, func, cfg, isa);
                return true;
            }

            ir::Opcode::Trapnz => {
                expand_cond_trap(inst, func, cfg, isa);
                return true;
            }

            ir::Opcode::Trapz => {
                expand_cond_trap(inst, func, cfg, isa);
                return true;
            }

            _ => {},
        }
    }
    false
}

/// Instruction expansions for architectures with flags.
///
/// Expand some instructions using CPU flags, then fall back to the normal
/// expansions. Not all architectures support CPU flags, so these patterns
/// are kept separate.
#[allow(unused_variables,unused_assignments,non_snake_case)]
pub fn expand_flags(
    inst: crate::ir::Inst,
    func: &mut crate::ir::Function,
    cfg: &mut crate::flowgraph::ControlFlowGraph,
    isa: &dyn crate::isa::TargetIsa,
) -> bool {
    use crate::ir::InstBuilder;
    use crate::cursor::{Cursor, FuncCursor};
    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);
    {
        match pos.func.dfg[inst].opcode() {
            ir::Opcode::Trapnz => {
                // Unwrap fields from instruction format () := trapnz(x, c)
                let (x, c, args) = if let ir::InstructionData::CondTrap {
                    code,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        code,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);

                let predicate = true;
                // typeof_x must belong to TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
                let predicate = predicate && TYPE_SETS[1].contains(typeof_x);
                if predicate {
                    pos.func.dfg.clear_results(inst);
                    let a = pos.ins().ifcmp_imm(x, 0);
                    pos.ins().trapif(ir::condcodes::IntCC::NotEqual, a, c);
                    let removed = pos.remove_inst();
                    debug_assert_eq!(removed, inst);
                    return true;
                }
            }

            ir::Opcode::Trapz => {
                // Unwrap fields from instruction format () := trapz(x, c)
                let (x, c, args) = if let ir::InstructionData::CondTrap {
                    code,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        code,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);

                let predicate = true;
                // typeof_x must belong to TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
                let predicate = predicate && TYPE_SETS[1].contains(typeof_x);
                if predicate {
                    pos.func.dfg.clear_results(inst);
                    let a = pos.ins().ifcmp_imm(x, 0);
                    pos.ins().trapif(ir::condcodes::IntCC::Equal, a, c);
                    let removed = pos.remove_inst();
                    debug_assert_eq!(removed, inst);
                    return true;
                }
            }

            _ => {},
        }
    }
    crate::legalizer::expand(inst, func, cfg, isa)
}

/// Legalize instructions by narrowing.
///
/// The transformations in the 'narrow' group work by expressing
/// instructions in terms of smaller types. Operations on vector types are
/// expressed in terms of vector types with fewer lanes, and integer
/// operations are expressed in terms of smaller integer types.
#[allow(unused_variables,unused_assignments,non_snake_case)]
pub fn narrow(
    inst: crate::ir::Inst,
    func: &mut crate::ir::Function,
    cfg: &mut crate::flowgraph::ControlFlowGraph,
    isa: &dyn crate::isa::TargetIsa,
) -> bool {
    use crate::ir::InstBuilder;
    use crate::cursor::{Cursor, FuncCursor};
    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);
    {
        match pos.func.dfg[inst].opcode() {
            ir::Opcode::Band => {
                // Unwrap fields from instruction format a := band(x, y)
                let (x, y, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := iconcat(al, ah).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let predicate = true;
                // typeof_x must belong to TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={16, 32, 64, 128})
                let predicate = predicate && TYPE_SETS[2].contains(typeof_x);
                if predicate {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let al = pos.ins().band(xl, yl);
                    let ah = pos.ins().band(xh, yh);
                    let a = pos.func.dfg.replace(inst).iconcat(al, ah);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::BandNot => {
                // Unwrap fields from instruction format a := band_not(x, y)
                let (x, y, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := iconcat(al, ah).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let predicate = true;
                // typeof_x must belong to TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={16, 32, 64, 128})
                let predicate = predicate && TYPE_SETS[2].contains(typeof_x);
                if predicate {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let al = pos.ins().band_not(xl, yl);
                    let ah = pos.ins().band_not(xh, yh);
                    let a = pos.func.dfg.replace(inst).iconcat(al, ah);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Bnot => {
                // Unwrap fields from instruction format a := bnot(x)
                let (x, args) = if let ir::InstructionData::Unary {
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := iconcat(al, ah).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let predicate = true;
                // typeof_x must belong to TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={16, 32, 64, 128})
                let predicate = predicate && TYPE_SETS[2].contains(typeof_x);
                if predicate {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let al = pos.ins().bnot(xl);
                    let ah = pos.ins().bnot(xh);
                    let a = pos.func.dfg.replace(inst).iconcat(al, ah);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Bor => {
                // Unwrap fields from instruction format a := bor(x, y)
                let (x, y, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := iconcat(al, ah).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let predicate = true;
                // typeof_x must belong to TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={16, 32, 64, 128})
                let predicate = predicate && TYPE_SETS[2].contains(typeof_x);
                if predicate {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let al = pos.ins().bor(xl, yl);
                    let ah = pos.ins().bor(xh, yh);
                    let a = pos.func.dfg.replace(inst).iconcat(al, ah);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::BorNot => {
                // Unwrap fields from instruction format a := bor_not(x, y)
                let (x, y, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := iconcat(al, ah).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let predicate = true;
                // typeof_x must belong to TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={16, 32, 64, 128})
                let predicate = predicate && TYPE_SETS[2].contains(typeof_x);
                if predicate {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let al = pos.ins().bor_not(xl, yl);
                    let ah = pos.ins().bor_not(xh, yh);
                    let a = pos.func.dfg.replace(inst).iconcat(al, ah);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Brnz => {
                // Unwrap fields from instruction format () := brnz.i128(x, ebb1, vararg)
                let (x, ebb1, args) = if let ir::InstructionData::Branch {
                    destination,
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    let args = args.as_slice(&pos.func.dfg.value_lists);
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        destination,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let vararg = &Vec::from(&args[1..]);

                if pos.func.dfg.value_type(args[0]) == ir::types::I128 {
                    let orig_ebb = pos.current_ebb().unwrap();
                    pos.func.dfg.clear_results(inst);
                    let ebb2 = pos.func.dfg.make_ebb();
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    pos.ins().brnz(xl, ebb1, vararg);
                    pos.ins().jump(ebb2, &[]);
                    pos.insert_ebb(ebb2);
                    pos.ins().brnz(xh, ebb1, vararg);
                    let removed = pos.remove_inst();
                    debug_assert_eq!(removed, inst);
                    cfg.recompute_ebb(pos.func, orig_ebb);
                    cfg.recompute_ebb(pos.func, ebb2);
                    return true;
                }
            }

            ir::Opcode::Brz => {
                // Unwrap fields from instruction format () := brz.i128(x, ebb, vararg)
                let (x, ebb, args) = if let ir::InstructionData::Branch {
                    destination,
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    let args = args.as_slice(&pos.func.dfg.value_lists);
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        destination,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let vararg = &Vec::from(&args[1..]);

                if pos.func.dfg.value_type(args[0]) == ir::types::I128 {
                    pos.func.dfg.clear_results(inst);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let a = pos.ins().icmp_imm(ir::condcodes::IntCC::Equal, xl, 0);
                    let b = pos.ins().icmp_imm(ir::condcodes::IntCC::Equal, xh, 0);
                    let c = pos.ins().band(a, b);
                    pos.ins().brnz(c, ebb, vararg);
                    let removed = pos.remove_inst();
                    debug_assert_eq!(removed, inst);
                    cfg.recompute_ebb(pos.func, pos.current_ebb().unwrap());
                    return true;
                }
            }

            ir::Opcode::Bxor => {
                // Unwrap fields from instruction format a := bxor(x, y)
                let (x, y, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := iconcat(al, ah).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let predicate = true;
                // typeof_x must belong to TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={16, 32, 64, 128})
                let predicate = predicate && TYPE_SETS[2].contains(typeof_x);
                if predicate {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let al = pos.ins().bxor(xl, yl);
                    let ah = pos.ins().bxor(xh, yh);
                    let a = pos.func.dfg.replace(inst).iconcat(al, ah);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::BxorNot => {
                // Unwrap fields from instruction format a := bxor_not(x, y)
                let (x, y, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := iconcat(al, ah).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let predicate = true;
                // typeof_x must belong to TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={16, 32, 64, 128})
                let predicate = predicate && TYPE_SETS[2].contains(typeof_x);
                if predicate {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let al = pos.ins().bxor_not(xl, yl);
                    let ah = pos.ins().bxor_not(xh, yh);
                    let a = pos.func.dfg.replace(inst).iconcat(al, ah);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Icmp => {
                // Unwrap fields from instruction format b := icmp.i64(ir::condcodes::IntCC::Equal, x, y)
                let (cond, x, y, args) = if let ir::InstructionData::IntCompare {
                    cond,
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        cond,
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by b := band(b1, b2).
                let r = pos.func.dfg.inst_results(inst);
                let b = &r[0];
                let typeof_b = pos.func.dfg.value_type(*b);

                if predicates::is_equal(cond, ir::condcodes::IntCC::Equal) && pos.func.dfg.value_type(args[0]) == ir::types::I64 {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let b1 = pos.ins().icmp(ir::condcodes::IntCC::Equal, xl, yl);
                    let b2 = pos.ins().icmp(ir::condcodes::IntCC::Equal, xh, yh);
                    let b = pos.func.dfg.replace(inst).band(b1, b2);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::NotEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I64 {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let b1 = pos.ins().icmp(ir::condcodes::IntCC::NotEqual, xl, yl);
                    let b2 = pos.ins().icmp(ir::condcodes::IntCC::NotEqual, xh, yh);
                    let b = pos.func.dfg.replace(inst).bor(b1, b2);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedGreaterThan) && pos.func.dfg.value_type(args[0]) == ir::types::I64 {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let b1 = pos.ins().icmp(ir::condcodes::IntCC::SignedGreaterThan, xh, yh);
                    let b2 = pos.ins().icmp(ir::condcodes::IntCC::SignedLessThan, xh, yh);
                    let b3 = pos.ins().icmp(ir::condcodes::IntCC::UnsignedGreaterThan, xl, yl);
                    let c1 = pos.ins().bnot(b2);
                    let c2 = pos.ins().band(c1, b3);
                    let b = pos.func.dfg.replace(inst).bor(b1, c2);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedGreaterThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I64 {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let b1 = pos.ins().icmp(ir::condcodes::IntCC::SignedGreaterThan, xh, yh);
                    let b2 = pos.ins().icmp(ir::condcodes::IntCC::SignedLessThan, xh, yh);
                    let b3 = pos.ins().icmp(ir::condcodes::IntCC::UnsignedGreaterThanOrEqual, xl, yl);
                    let c1 = pos.ins().bnot(b2);
                    let c2 = pos.ins().band(c1, b3);
                    let b = pos.func.dfg.replace(inst).bor(b1, c2);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedLessThan) && pos.func.dfg.value_type(args[0]) == ir::types::I64 {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let b1 = pos.ins().icmp(ir::condcodes::IntCC::SignedLessThan, xh, yh);
                    let b2 = pos.ins().icmp(ir::condcodes::IntCC::SignedGreaterThan, xh, yh);
                    let b3 = pos.ins().icmp(ir::condcodes::IntCC::UnsignedLessThan, xl, yl);
                    let c1 = pos.ins().bnot(b2);
                    let c2 = pos.ins().band(c1, b3);
                    let b = pos.func.dfg.replace(inst).bor(b1, c2);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedLessThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I64 {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let b1 = pos.ins().icmp(ir::condcodes::IntCC::SignedLessThan, xh, yh);
                    let b2 = pos.ins().icmp(ir::condcodes::IntCC::SignedGreaterThan, xh, yh);
                    let b3 = pos.ins().icmp(ir::condcodes::IntCC::UnsignedLessThanOrEqual, xl, yl);
                    let c1 = pos.ins().bnot(b2);
                    let c2 = pos.ins().band(c1, b3);
                    let b = pos.func.dfg.replace(inst).bor(b1, c2);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedGreaterThan) && pos.func.dfg.value_type(args[0]) == ir::types::I64 {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let b1 = pos.ins().icmp(ir::condcodes::IntCC::UnsignedGreaterThan, xh, yh);
                    let b2 = pos.ins().icmp(ir::condcodes::IntCC::UnsignedLessThan, xh, yh);
                    let b3 = pos.ins().icmp(ir::condcodes::IntCC::UnsignedGreaterThan, xl, yl);
                    let c1 = pos.ins().bnot(b2);
                    let c2 = pos.ins().band(c1, b3);
                    let b = pos.func.dfg.replace(inst).bor(b1, c2);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedGreaterThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I64 {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let b1 = pos.ins().icmp(ir::condcodes::IntCC::UnsignedGreaterThan, xh, yh);
                    let b2 = pos.ins().icmp(ir::condcodes::IntCC::UnsignedLessThan, xh, yh);
                    let b3 = pos.ins().icmp(ir::condcodes::IntCC::UnsignedGreaterThanOrEqual, xl, yl);
                    let c1 = pos.ins().bnot(b2);
                    let c2 = pos.ins().band(c1, b3);
                    let b = pos.func.dfg.replace(inst).bor(b1, c2);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedLessThan) && pos.func.dfg.value_type(args[0]) == ir::types::I64 {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let b1 = pos.ins().icmp(ir::condcodes::IntCC::UnsignedLessThan, xh, yh);
                    let b2 = pos.ins().icmp(ir::condcodes::IntCC::UnsignedGreaterThan, xh, yh);
                    let b3 = pos.ins().icmp(ir::condcodes::IntCC::UnsignedLessThan, xl, yl);
                    let c1 = pos.ins().bnot(b2);
                    let c2 = pos.ins().band(c1, b3);
                    let b = pos.func.dfg.replace(inst).bor(b1, c2);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedLessThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I64 {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let b1 = pos.ins().icmp(ir::condcodes::IntCC::UnsignedLessThan, xh, yh);
                    let b2 = pos.ins().icmp(ir::condcodes::IntCC::UnsignedGreaterThan, xh, yh);
                    let b3 = pos.ins().icmp(ir::condcodes::IntCC::UnsignedLessThanOrEqual, xl, yl);
                    let c1 = pos.ins().bnot(b2);
                    let c2 = pos.ins().band(c1, b3);
                    let b = pos.func.dfg.replace(inst).bor(b1, c2);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::Equal) && pos.func.dfg.value_type(args[0]) == ir::types::I128 {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let b1 = pos.ins().icmp(ir::condcodes::IntCC::Equal, xl, yl);
                    let b2 = pos.ins().icmp(ir::condcodes::IntCC::Equal, xh, yh);
                    let b = pos.func.dfg.replace(inst).band(b1, b2);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::NotEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I128 {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let b1 = pos.ins().icmp(ir::condcodes::IntCC::NotEqual, xl, yl);
                    let b2 = pos.ins().icmp(ir::condcodes::IntCC::NotEqual, xh, yh);
                    let b = pos.func.dfg.replace(inst).bor(b1, b2);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedGreaterThan) && pos.func.dfg.value_type(args[0]) == ir::types::I128 {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let b1 = pos.ins().icmp(ir::condcodes::IntCC::SignedGreaterThan, xh, yh);
                    let b2 = pos.ins().icmp(ir::condcodes::IntCC::SignedLessThan, xh, yh);
                    let b3 = pos.ins().icmp(ir::condcodes::IntCC::UnsignedGreaterThan, xl, yl);
                    let c1 = pos.ins().bnot(b2);
                    let c2 = pos.ins().band(c1, b3);
                    let b = pos.func.dfg.replace(inst).bor(b1, c2);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedGreaterThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I128 {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let b1 = pos.ins().icmp(ir::condcodes::IntCC::SignedGreaterThan, xh, yh);
                    let b2 = pos.ins().icmp(ir::condcodes::IntCC::SignedLessThan, xh, yh);
                    let b3 = pos.ins().icmp(ir::condcodes::IntCC::UnsignedGreaterThanOrEqual, xl, yl);
                    let c1 = pos.ins().bnot(b2);
                    let c2 = pos.ins().band(c1, b3);
                    let b = pos.func.dfg.replace(inst).bor(b1, c2);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedLessThan) && pos.func.dfg.value_type(args[0]) == ir::types::I128 {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let b1 = pos.ins().icmp(ir::condcodes::IntCC::SignedLessThan, xh, yh);
                    let b2 = pos.ins().icmp(ir::condcodes::IntCC::SignedGreaterThan, xh, yh);
                    let b3 = pos.ins().icmp(ir::condcodes::IntCC::UnsignedLessThan, xl, yl);
                    let c1 = pos.ins().bnot(b2);
                    let c2 = pos.ins().band(c1, b3);
                    let b = pos.func.dfg.replace(inst).bor(b1, c2);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedLessThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I128 {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let b1 = pos.ins().icmp(ir::condcodes::IntCC::SignedLessThan, xh, yh);
                    let b2 = pos.ins().icmp(ir::condcodes::IntCC::SignedGreaterThan, xh, yh);
                    let b3 = pos.ins().icmp(ir::condcodes::IntCC::UnsignedLessThanOrEqual, xl, yl);
                    let c1 = pos.ins().bnot(b2);
                    let c2 = pos.ins().band(c1, b3);
                    let b = pos.func.dfg.replace(inst).bor(b1, c2);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedGreaterThan) && pos.func.dfg.value_type(args[0]) == ir::types::I128 {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let b1 = pos.ins().icmp(ir::condcodes::IntCC::UnsignedGreaterThan, xh, yh);
                    let b2 = pos.ins().icmp(ir::condcodes::IntCC::UnsignedLessThan, xh, yh);
                    let b3 = pos.ins().icmp(ir::condcodes::IntCC::UnsignedGreaterThan, xl, yl);
                    let c1 = pos.ins().bnot(b2);
                    let c2 = pos.ins().band(c1, b3);
                    let b = pos.func.dfg.replace(inst).bor(b1, c2);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedGreaterThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I128 {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let b1 = pos.ins().icmp(ir::condcodes::IntCC::UnsignedGreaterThan, xh, yh);
                    let b2 = pos.ins().icmp(ir::condcodes::IntCC::UnsignedLessThan, xh, yh);
                    let b3 = pos.ins().icmp(ir::condcodes::IntCC::UnsignedGreaterThanOrEqual, xl, yl);
                    let c1 = pos.ins().bnot(b2);
                    let c2 = pos.ins().band(c1, b3);
                    let b = pos.func.dfg.replace(inst).bor(b1, c2);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedLessThan) && pos.func.dfg.value_type(args[0]) == ir::types::I128 {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let b1 = pos.ins().icmp(ir::condcodes::IntCC::UnsignedLessThan, xh, yh);
                    let b2 = pos.ins().icmp(ir::condcodes::IntCC::UnsignedGreaterThan, xh, yh);
                    let b3 = pos.ins().icmp(ir::condcodes::IntCC::UnsignedLessThan, xl, yl);
                    let c1 = pos.ins().bnot(b2);
                    let c2 = pos.ins().band(c1, b3);
                    let b = pos.func.dfg.replace(inst).bor(b1, c2);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedLessThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I128 {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let b1 = pos.ins().icmp(ir::condcodes::IntCC::UnsignedLessThan, xh, yh);
                    let b2 = pos.ins().icmp(ir::condcodes::IntCC::UnsignedGreaterThan, xh, yh);
                    let b3 = pos.ins().icmp(ir::condcodes::IntCC::UnsignedLessThanOrEqual, xl, yl);
                    let c1 = pos.ins().bnot(b2);
                    let c2 = pos.ins().band(c1, b3);
                    let b = pos.func.dfg.replace(inst).bor(b1, c2);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Imul => {
                // Unwrap fields from instruction format a := imul.i64(x, y)
                let (x, y, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := iconcat(al, ah).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I64 {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let a1 = pos.ins().imul(xh, yl);
                    let a2 = pos.ins().imul(xl, yh);
                    let a3 = pos.ins().iadd(a1, a2);
                    let a4 = pos.ins().umulhi(xl, yl);
                    let ah = pos.ins().iadd(a3, a4);
                    let al = pos.ins().imul(xl, yl);
                    let a = pos.func.dfg.replace(inst).iconcat(al, ah);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I128 {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let a1 = pos.ins().imul(xh, yl);
                    let a2 = pos.ins().imul(xl, yh);
                    let a3 = pos.ins().iadd(a1, a2);
                    let a4 = pos.ins().umulhi(xl, yl);
                    let ah = pos.ins().iadd(a3, a4);
                    let al = pos.ins().imul(xl, yl);
                    let a = pos.func.dfg.replace(inst).iconcat(al, ah);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Select => {
                // Unwrap fields from instruction format a := select(c, x, y)
                let (c, x, y, args) = if let ir::InstructionData::Ternary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        pos.func.dfg.resolve_aliases(args[2]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_c = pos.func.dfg.value_type(c);
                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := iconcat(al, ah).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let predicate = true;
                // typeof_x must belong to TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={16, 32, 64, 128})
                let predicate = predicate && TYPE_SETS[2].contains(typeof_x);
                if predicate {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let al = pos.ins().select(c, xl, yl);
                    let ah = pos.ins().select(c, xh, yh);
                    let a = pos.func.dfg.replace(inst).iconcat(al, ah);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::IcmpImm => {
                narrow_icmp_imm(inst, func, cfg, isa);
                return true;
            }

            ir::Opcode::Iconst => {
                narrow_iconst(inst, func, cfg, isa);
                return true;
            }

            ir::Opcode::Load => {
                narrow_load(inst, func, cfg, isa);
                return true;
            }

            ir::Opcode::Store => {
                narrow_store(inst, func, cfg, isa);
                return true;
            }

            _ => {},
        }
    }
    false
}

/// Narrow instructions for architectures with flags.
///
/// Narrow some instructions using CPU flags, then fall back to the normal
/// legalizations. Not all architectures support CPU flags, so these
/// patterns are kept separate.
#[allow(unused_variables,unused_assignments,non_snake_case)]
pub fn narrow_flags(
    inst: crate::ir::Inst,
    func: &mut crate::ir::Function,
    cfg: &mut crate::flowgraph::ControlFlowGraph,
    isa: &dyn crate::isa::TargetIsa,
) -> bool {
    use crate::ir::InstBuilder;
    use crate::cursor::{Cursor, FuncCursor};
    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);
    {
        match pos.func.dfg[inst].opcode() {
            ir::Opcode::Iadd => {
                // Unwrap fields from instruction format a := iadd(x, y)
                let (x, y, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := iconcat(al, ah).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let predicate = true;
                // typeof_x must belong to TypeSet(lanes={1}, ints={16, 32, 64, 128})
                let predicate = predicate && TYPE_SETS[3].contains(typeof_x);
                if predicate {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let (al, c) = pos.ins().iadd_ifcout(xl, yl);
                    let ah = pos.ins().iadd_ifcin(xh, yh, c);
                    let a = pos.func.dfg.replace(inst).iconcat(al, ah);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Isub => {
                // Unwrap fields from instruction format a := isub(x, y)
                let (x, y, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := iconcat(al, ah).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let predicate = true;
                // typeof_x must belong to TypeSet(lanes={1}, ints={16, 32, 64, 128})
                let predicate = predicate && TYPE_SETS[3].contains(typeof_x);
                if predicate {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let (al, b) = pos.ins().isub_ifbout(xl, yl);
                    let ah = pos.ins().isub_ifbin(xh, yh, b);
                    let a = pos.func.dfg.replace(inst).iconcat(al, ah);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            _ => {},
        }
    }
    crate::legalizer::narrow(inst, func, cfg, isa)
}

/// Narrow instructions for architectures without flags.
///
/// Narrow some instructions avoiding the use of CPU flags, then fall back
/// to the normal legalizations. Not all architectures support CPU flags,
/// so these patterns are kept separate.
#[allow(unused_variables,unused_assignments,non_snake_case)]
pub fn narrow_no_flags(
    inst: crate::ir::Inst,
    func: &mut crate::ir::Function,
    cfg: &mut crate::flowgraph::ControlFlowGraph,
    isa: &dyn crate::isa::TargetIsa,
) -> bool {
    use crate::ir::InstBuilder;
    use crate::cursor::{Cursor, FuncCursor};
    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);
    {
        match pos.func.dfg[inst].opcode() {
            ir::Opcode::Iadd => {
                // Unwrap fields from instruction format a := iadd(x, y)
                let (x, y, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := iconcat(al, ah).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let predicate = true;
                // typeof_x must belong to TypeSet(lanes={1}, ints={16, 32, 64, 128})
                let predicate = predicate && TYPE_SETS[3].contains(typeof_x);
                if predicate {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let (al, c) = pos.ins().iadd_cout(xl, yl);
                    let ah = pos.ins().iadd_cin(xh, yh, c);
                    let a = pos.func.dfg.replace(inst).iconcat(al, ah);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Isub => {
                // Unwrap fields from instruction format a := isub(x, y)
                let (x, y, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := iconcat(al, ah).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                let predicate = true;
                // typeof_x must belong to TypeSet(lanes={1}, ints={16, 32, 64, 128})
                let predicate = predicate && TYPE_SETS[3].contains(typeof_x);
                if predicate {
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (xl, xh) = split::isplit(pos.func, cfg, curpos, srcloc, x);
                    let curpos = pos.position();
                    let srcloc = pos.srcloc();
                    let (yl, yh) = split::isplit(pos.func, cfg, curpos, srcloc, y);
                    let (al, b) = pos.ins().isub_bout(xl, yl);
                    let ah = pos.ins().isub_bin(xh, yh, b);
                    let a = pos.func.dfg.replace(inst).iconcat(al, ah);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            _ => {},
        }
    }
    crate::legalizer::narrow(inst, func, cfg, isa)
}

/// Legalize instructions by widening.
///
/// The transformations in the 'widen' group work by expressing
/// instructions in terms of larger types.
#[allow(unused_variables,unused_assignments,non_snake_case)]
pub fn widen(
    inst: crate::ir::Inst,
    func: &mut crate::ir::Function,
    cfg: &mut crate::flowgraph::ControlFlowGraph,
    isa: &dyn crate::isa::TargetIsa,
) -> bool {
    use crate::ir::InstBuilder;
    use crate::cursor::{Cursor, FuncCursor};
    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);
    {
        match pos.func.dfg[inst].opcode() {
            ir::Opcode::Band => {
                // Unwrap fields from instruction format a := band.i8(b, c)
                let (b, c, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce.i8(z).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let z = pos.ins().band(x, y);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let z = pos.ins().band(x, y);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::BandImm => {
                // Unwrap fields from instruction format a := band_imm.i8(b, c)
                let (b, c, args) = if let ir::InstructionData::BinaryImm {
                    imm,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        imm,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce.i8(z).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let z = pos.ins().band_imm(x, c);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let z = pos.ins().band_imm(x, c);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::BandNot => {
                // Unwrap fields from instruction format a := band_not.i8(b, c)
                let (b, c, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce.i8(z).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let z = pos.ins().band_not(x, y);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let z = pos.ins().band_not(x, y);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Bint => {
                // Unwrap fields from instruction format a := bint.i8(b)
                let (b, args) = if let ir::InstructionData::Unary {
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_b = pos.func.dfg.value_type(b);
                // Results handled by a := ireduce.i8(x).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.ctrl_typevar(inst) == ir::types::I8 {
                    let x = pos.ins().bint(ir::types::I32, b);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, x);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.ctrl_typevar(inst) == ir::types::I16 {
                    let x = pos.ins().bint(ir::types::I32, b);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, x);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Bitrev => {
                // Unwrap fields from instruction format a := bitrev.i8(x)
                let (x, args) = if let ir::InstructionData::Unary {
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := bor(c2, c4).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let a1 = pos.ins().band_imm(x, 170);
                    let a2 = pos.ins().ushr_imm(a1, 1);
                    let a3 = pos.ins().band_imm(x, 85);
                    let a4 = pos.ins().ishl_imm(a3, 1);
                    let b = pos.ins().bor(a2, a4);
                    let b1 = pos.ins().band_imm(b, 204);
                    let b2 = pos.ins().ushr_imm(b1, 2);
                    let b3 = pos.ins().band_imm(b, 51);
                    let b4 = pos.ins().ishl_imm(b3, 2);
                    let c = pos.ins().bor(b2, b4);
                    let c1 = pos.ins().band_imm(c, 240);
                    let c2 = pos.ins().ushr_imm(c1, 4);
                    let c3 = pos.ins().band_imm(c, 15);
                    let c4 = pos.ins().ishl_imm(c3, 4);
                    let a = pos.func.dfg.replace(inst).bor(c2, c4);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let a1 = pos.ins().band_imm(x, 43690);
                    let a2 = pos.ins().ushr_imm(a1, 1);
                    let a3 = pos.ins().band_imm(x, 21845);
                    let a4 = pos.ins().ishl_imm(a3, 1);
                    let b = pos.ins().bor(a2, a4);
                    let b1 = pos.ins().band_imm(b, 52428);
                    let b2 = pos.ins().ushr_imm(b1, 2);
                    let b3 = pos.ins().band_imm(b, 13107);
                    let b4 = pos.ins().ishl_imm(b3, 2);
                    let c = pos.ins().bor(b2, b4);
                    let c1 = pos.ins().band_imm(c, 61680);
                    let c2 = pos.ins().ushr_imm(c1, 4);
                    let c3 = pos.ins().band_imm(c, 3855);
                    let c4 = pos.ins().ishl_imm(c3, 4);
                    let d = pos.ins().bor(c2, c4);
                    let d1 = pos.ins().band_imm(d, 65280);
                    let d2 = pos.ins().ushr_imm(d1, 8);
                    let d3 = pos.ins().band_imm(d, 255);
                    let d4 = pos.ins().ishl_imm(d3, 8);
                    let a = pos.func.dfg.replace(inst).bor(d2, d4);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Bnot => {
                // Unwrap fields from instruction format a := bnot.i8(b)
                let (b, args) = if let ir::InstructionData::Unary {
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce.i8(z).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let z = pos.ins().bnot(x);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let z = pos.ins().bnot(x);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Bor => {
                // Unwrap fields from instruction format a := bor.i8(b, c)
                let (b, c, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce.i8(z).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let z = pos.ins().bor(x, y);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let z = pos.ins().bor(x, y);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::BorImm => {
                // Unwrap fields from instruction format a := bor_imm.i8(b, c)
                let (b, c, args) = if let ir::InstructionData::BinaryImm {
                    imm,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        imm,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce.i8(z).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let z = pos.ins().bor_imm(x, c);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let z = pos.ins().bor_imm(x, c);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::BorNot => {
                // Unwrap fields from instruction format a := bor_not.i8(b, c)
                let (b, c, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce.i8(z).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let z = pos.ins().bor_not(x, y);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let z = pos.ins().bor_not(x, y);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::BrTable => {
                // Unwrap fields from instruction format () := br_table.i8(x, y, z)
                let (x, y, z, args) = if let ir::InstructionData::BranchTable {
                    destination,
                    table,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        destination,
                        table,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };


                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    pos.func.dfg.clear_results(inst);
                    let b = pos.ins().uextend(ir::types::I32, x);
                    pos.ins().br_table(b, y, z);
                    let removed = pos.remove_inst();
                    debug_assert_eq!(removed, inst);
                    cfg.recompute_ebb(pos.func, pos.current_ebb().unwrap());
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    pos.func.dfg.clear_results(inst);
                    let b = pos.ins().uextend(ir::types::I32, x);
                    pos.ins().br_table(b, y, z);
                    let removed = pos.remove_inst();
                    debug_assert_eq!(removed, inst);
                    cfg.recompute_ebb(pos.func, pos.current_ebb().unwrap());
                    return true;
                }
            }

            ir::Opcode::Bxor => {
                // Unwrap fields from instruction format a := bxor.i8(b, c)
                let (b, c, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce.i8(z).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let z = pos.ins().bxor(x, y);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let z = pos.ins().bxor(x, y);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::BxorImm => {
                // Unwrap fields from instruction format a := bxor_imm.i8(b, c)
                let (b, c, args) = if let ir::InstructionData::BinaryImm {
                    imm,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        imm,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce.i8(z).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let z = pos.ins().bxor_imm(x, c);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let z = pos.ins().bxor_imm(x, c);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::BxorNot => {
                // Unwrap fields from instruction format a := bxor_not.i8(b, c)
                let (b, c, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce.i8(z).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let z = pos.ins().bxor_not(x, y);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let z = pos.ins().bxor_not(x, y);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Cls => {
                // Unwrap fields from instruction format a := cls.i8(b)
                let (b, args) = if let ir::InstructionData::Unary {
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce.i8(e).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let c = pos.ins().sextend(ir::types::I32, b);
                    let d = pos.ins().cls(c);
                    let e = pos.ins().iadd_imm(d, -24);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, e);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let c = pos.ins().sextend(ir::types::I32, b);
                    let d = pos.ins().cls(c);
                    let e = pos.ins().iadd_imm(d, -16);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, e);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Clz => {
                // Unwrap fields from instruction format a := clz.i8(b)
                let (b, args) = if let ir::InstructionData::Unary {
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce.i8(e).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let c = pos.ins().uextend(ir::types::I32, b);
                    let d = pos.ins().clz(c);
                    let e = pos.ins().iadd_imm(d, -24);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, e);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let c = pos.ins().uextend(ir::types::I32, b);
                    let d = pos.ins().clz(c);
                    let e = pos.ins().iadd_imm(d, -16);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, e);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Ctz => {
                // Unwrap fields from instruction format a := ctz.i8(b)
                let (b, args) = if let ir::InstructionData::Unary {
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce.i8(e).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let c = pos.ins().uextend(ir::types::I32, b);
                    let d = pos.ins().bor_imm(c, 256);
                    let e = pos.ins().ctz(d);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, e);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let c = pos.ins().uextend(ir::types::I32, b);
                    let d = pos.ins().bor_imm(c, 65536);
                    let e = pos.ins().ctz(d);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, e);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Iadd => {
                // Unwrap fields from instruction format a := iadd.i8(b, c)
                let (b, c, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce.i8(z).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let z = pos.ins().iadd(x, y);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let z = pos.ins().iadd(x, y);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::IaddImm => {
                // Unwrap fields from instruction format a := iadd_imm.i8(b, c)
                let (b, c, args) = if let ir::InstructionData::BinaryImm {
                    imm,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        imm,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce.i8(z).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let z = pos.ins().iadd_imm(x, c);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let z = pos.ins().iadd_imm(x, c);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Icmp => {
                // Unwrap fields from instruction format a := icmp.i8(ir::condcodes::IntCC::Equal, b, c)
                let (cond, b, c, args) = if let ir::InstructionData::IntCompare {
                    cond,
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        cond,
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := icmp.i32(ir::condcodes::IntCC::Equal, x, y).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if predicates::is_equal(cond, ir::condcodes::IntCC::Equal) && pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let a = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::Equal, x, y);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::NotEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let a = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::NotEqual, x, y);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedGreaterThan) && pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let a = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::UnsignedGreaterThan, x, y);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedLessThan) && pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let a = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::UnsignedLessThan, x, y);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedGreaterThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let a = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::UnsignedGreaterThanOrEqual, x, y);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedLessThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let a = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::UnsignedLessThanOrEqual, x, y);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedGreaterThan) && pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().sextend(ir::types::I32, b);
                    let y = pos.ins().sextend(ir::types::I32, c);
                    let a = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::SignedGreaterThan, x, y);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedLessThan) && pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().sextend(ir::types::I32, b);
                    let y = pos.ins().sextend(ir::types::I32, c);
                    let a = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::SignedLessThan, x, y);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedGreaterThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().sextend(ir::types::I32, b);
                    let y = pos.ins().sextend(ir::types::I32, c);
                    let a = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::SignedGreaterThanOrEqual, x, y);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedLessThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().sextend(ir::types::I32, b);
                    let y = pos.ins().sextend(ir::types::I32, c);
                    let a = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::SignedLessThanOrEqual, x, y);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::Equal) && pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let a = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::Equal, x, y);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::NotEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let a = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::NotEqual, x, y);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedGreaterThan) && pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let a = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::UnsignedGreaterThan, x, y);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedLessThan) && pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let a = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::UnsignedLessThan, x, y);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedGreaterThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let a = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::UnsignedGreaterThanOrEqual, x, y);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedLessThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let a = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::UnsignedLessThanOrEqual, x, y);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedGreaterThan) && pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().sextend(ir::types::I32, b);
                    let y = pos.ins().sextend(ir::types::I32, c);
                    let a = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::SignedGreaterThan, x, y);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedLessThan) && pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().sextend(ir::types::I32, b);
                    let y = pos.ins().sextend(ir::types::I32, c);
                    let a = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::SignedLessThan, x, y);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedGreaterThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().sextend(ir::types::I32, b);
                    let y = pos.ins().sextend(ir::types::I32, c);
                    let a = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::SignedGreaterThanOrEqual, x, y);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedLessThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().sextend(ir::types::I32, b);
                    let y = pos.ins().sextend(ir::types::I32, c);
                    let a = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::SignedLessThanOrEqual, x, y);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::IcmpImm => {
                // Unwrap fields from instruction format a := icmp_imm.i8(ir::condcodes::IntCC::Equal, b, c)
                let (cond, b, c, args) = if let ir::InstructionData::IntCompareImm {
                    cond,
                    imm,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        cond,
                        pos.func.dfg.resolve_aliases(args[0]),
                        imm,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := icmp_imm(ir::condcodes::IntCC::Equal, x, c).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if predicates::is_equal(cond, ir::condcodes::IntCC::Equal) && pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let a = pos.func.dfg.replace(inst).icmp_imm(ir::condcodes::IntCC::Equal, x, c);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::NotEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let a = pos.func.dfg.replace(inst).icmp_imm(ir::condcodes::IntCC::NotEqual, x, c);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedGreaterThan) && pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let a = pos.func.dfg.replace(inst).icmp_imm(ir::condcodes::IntCC::UnsignedGreaterThan, x, c);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedLessThan) && pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let a = pos.func.dfg.replace(inst).icmp_imm(ir::condcodes::IntCC::UnsignedLessThan, x, c);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedGreaterThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let a = pos.func.dfg.replace(inst).icmp_imm(ir::condcodes::IntCC::UnsignedGreaterThanOrEqual, x, c);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedLessThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let a = pos.func.dfg.replace(inst).icmp_imm(ir::condcodes::IntCC::UnsignedLessThanOrEqual, x, c);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedGreaterThan) && pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().sextend(ir::types::I32, b);
                    let a = pos.func.dfg.replace(inst).icmp_imm(ir::condcodes::IntCC::SignedGreaterThan, x, c);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedLessThan) && pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().sextend(ir::types::I32, b);
                    let a = pos.func.dfg.replace(inst).icmp_imm(ir::condcodes::IntCC::SignedLessThan, x, c);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedGreaterThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().sextend(ir::types::I32, b);
                    let a = pos.func.dfg.replace(inst).icmp_imm(ir::condcodes::IntCC::SignedGreaterThanOrEqual, x, c);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedLessThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().sextend(ir::types::I32, b);
                    let a = pos.func.dfg.replace(inst).icmp_imm(ir::condcodes::IntCC::SignedLessThanOrEqual, x, c);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::Equal) && pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let a = pos.func.dfg.replace(inst).icmp_imm(ir::condcodes::IntCC::Equal, x, c);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::NotEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let a = pos.func.dfg.replace(inst).icmp_imm(ir::condcodes::IntCC::NotEqual, x, c);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedGreaterThan) && pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let a = pos.func.dfg.replace(inst).icmp_imm(ir::condcodes::IntCC::UnsignedGreaterThan, x, c);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedLessThan) && pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let a = pos.func.dfg.replace(inst).icmp_imm(ir::condcodes::IntCC::UnsignedLessThan, x, c);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedGreaterThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let a = pos.func.dfg.replace(inst).icmp_imm(ir::condcodes::IntCC::UnsignedGreaterThanOrEqual, x, c);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedLessThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let a = pos.func.dfg.replace(inst).icmp_imm(ir::condcodes::IntCC::UnsignedLessThanOrEqual, x, c);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedGreaterThan) && pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().sextend(ir::types::I32, b);
                    let a = pos.func.dfg.replace(inst).icmp_imm(ir::condcodes::IntCC::SignedGreaterThan, x, c);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedLessThan) && pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().sextend(ir::types::I32, b);
                    let a = pos.func.dfg.replace(inst).icmp_imm(ir::condcodes::IntCC::SignedLessThan, x, c);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedGreaterThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().sextend(ir::types::I32, b);
                    let a = pos.func.dfg.replace(inst).icmp_imm(ir::condcodes::IntCC::SignedGreaterThanOrEqual, x, c);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedLessThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().sextend(ir::types::I32, b);
                    let a = pos.func.dfg.replace(inst).icmp_imm(ir::condcodes::IntCC::SignedLessThanOrEqual, x, c);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Iconst => {
                // Unwrap fields from instruction format a := iconst.i8(b)
                let b = if let ir::InstructionData::UnaryImm {
                    imm,
                    ..
                } = pos.func.dfg[inst] {
                    imm
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce.i8(c).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.ctrl_typevar(inst) == ir::types::I8 {
                    let c = pos.ins().iconst(ir::types::I32, b);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, c);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.ctrl_typevar(inst) == ir::types::I16 {
                    let c = pos.ins().iconst(ir::types::I32, b);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, c);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Imul => {
                // Unwrap fields from instruction format a := imul.i8(b, c)
                let (b, c, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce.i8(z).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let z = pos.ins().imul(x, y);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let z = pos.ins().imul(x, y);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::ImulImm => {
                // Unwrap fields from instruction format a := imul_imm.i8(b, c)
                let (b, c, args) = if let ir::InstructionData::BinaryImm {
                    imm,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        imm,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce.i8(z).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let z = pos.ins().imul_imm(x, c);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let z = pos.ins().imul_imm(x, c);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::IrsubImm => {
                // Unwrap fields from instruction format a := irsub_imm.i8(b, c)
                let (b, c, args) = if let ir::InstructionData::BinaryImm {
                    imm,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        imm,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce.i8(z).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let z = pos.ins().irsub_imm(x, c);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let z = pos.ins().irsub_imm(x, c);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Ishl => {
                // Unwrap fields from instruction format a := ishl.i8(b, c)
                let (b, c, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_c = pos.func.dfg.value_type(c);
                // Results handled by a := ireduce.i8(z).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let z = pos.ins().ishl(x, c);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let z = pos.ins().ishl(x, c);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::IshlImm => {
                // Unwrap fields from instruction format a := ishl_imm.i8(b, c)
                let (b, c, args) = if let ir::InstructionData::BinaryImm {
                    imm,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        imm,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce.i8(z).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let z = pos.ins().ishl_imm(x, c);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let z = pos.ins().ishl_imm(x, c);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Isub => {
                // Unwrap fields from instruction format a := isub.i8(b, c)
                let (b, c, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce.i8(z).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let z = pos.ins().isub(x, y);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let z = pos.ins().isub(x, y);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Load => {
                // Unwrap fields from instruction format a := load.i8(flags, ptr, off)
                let (flags, ptr, off, args) = if let ir::InstructionData::Load {
                    flags,
                    offset,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        flags,
                        pos.func.dfg.resolve_aliases(args[0]),
                        offset,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_ptr = pos.func.dfg.value_type(ptr);
                // Results handled by a := ireduce(b).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.ctrl_typevar(inst) == ir::types::I8 {
                    let b = pos.ins().uload8(ir::types::I32, flags, ptr, off);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, b);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.ctrl_typevar(inst) == ir::types::I16 {
                    let b = pos.ins().uload16(ir::types::I32, flags, ptr, off);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, b);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Popcnt => {
                // Unwrap fields from instruction format a := popcnt.i8(b)
                let (b, args) = if let ir::InstructionData::Unary {
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce.i8(z).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let z = pos.ins().popcnt(x);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let z = pos.ins().popcnt(x);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Sdiv => {
                // Unwrap fields from instruction format a := sdiv.i8(b, c)
                let (b, c, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce.i8(z).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().sextend(ir::types::I32, b);
                    let y = pos.ins().sextend(ir::types::I32, c);
                    let z = pos.ins().sdiv(x, y);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().sextend(ir::types::I32, b);
                    let y = pos.ins().sextend(ir::types::I32, c);
                    let z = pos.ins().sdiv(x, y);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::SdivImm => {
                // Unwrap fields from instruction format a := sdiv_imm.i8(b, c)
                let (b, c, args) = if let ir::InstructionData::BinaryImm {
                    imm,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        imm,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce.i8(z).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().sextend(ir::types::I32, b);
                    let z = pos.ins().sdiv_imm(x, c);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().sextend(ir::types::I32, b);
                    let z = pos.ins().sdiv_imm(x, c);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Sextend => {
                // Unwrap fields from instruction format a := sextend.i16.i8(b)
                let (b, args) = if let ir::InstructionData::Unary {
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce(c).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 && pos.func.dfg.ctrl_typevar(inst) == ir::types::I16 {
                    let c = pos.ins().sextend(ir::types::I32, b);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, c);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Srem => {
                // Unwrap fields from instruction format a := srem.i8(b, c)
                let (b, c, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce.i8(z).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().sextend(ir::types::I32, b);
                    let y = pos.ins().sextend(ir::types::I32, c);
                    let z = pos.ins().srem(x, y);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().sextend(ir::types::I32, b);
                    let y = pos.ins().sextend(ir::types::I32, c);
                    let z = pos.ins().srem(x, y);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::SremImm => {
                // Unwrap fields from instruction format a := srem_imm.i8(b, c)
                let (b, c, args) = if let ir::InstructionData::BinaryImm {
                    imm,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        imm,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce.i8(z).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().sextend(ir::types::I32, b);
                    let z = pos.ins().srem_imm(x, c);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().sextend(ir::types::I32, b);
                    let z = pos.ins().srem_imm(x, c);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Sshr => {
                // Unwrap fields from instruction format a := sshr.i8(b, c)
                let (b, c, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_c = pos.func.dfg.value_type(c);
                // Results handled by a := ireduce.i8(z).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().sextend(ir::types::I32, b);
                    let z = pos.ins().sshr(x, c);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().sextend(ir::types::I32, b);
                    let z = pos.ins().sshr(x, c);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::SshrImm => {
                // Unwrap fields from instruction format a := sshr_imm.i8(b, c)
                let (b, c, args) = if let ir::InstructionData::BinaryImm {
                    imm,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        imm,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce.i8(z).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().sextend(ir::types::I32, b);
                    let z = pos.ins().sshr_imm(x, c);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().sextend(ir::types::I32, b);
                    let z = pos.ins().sshr_imm(x, c);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Store => {
                // Unwrap fields from instruction format () := store.i8(flags, a, ptr, off)
                let (flags, a, ptr, off, args) = if let ir::InstructionData::Store {
                    flags,
                    offset,
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        flags,
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        offset,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_ptr = pos.func.dfg.value_type(ptr);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    pos.func.dfg.clear_results(inst);
                    let b = pos.ins().uextend(ir::types::I32, a);
                    pos.ins().istore8(flags, b, ptr, off);
                    let removed = pos.remove_inst();
                    debug_assert_eq!(removed, inst);
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    pos.func.dfg.clear_results(inst);
                    let b = pos.ins().uextend(ir::types::I32, a);
                    pos.ins().istore16(flags, b, ptr, off);
                    let removed = pos.remove_inst();
                    debug_assert_eq!(removed, inst);
                    return true;
                }
            }

            ir::Opcode::Udiv => {
                // Unwrap fields from instruction format a := udiv.i8(b, c)
                let (b, c, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce.i8(z).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let z = pos.ins().udiv(x, y);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let z = pos.ins().udiv(x, y);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::UdivImm => {
                // Unwrap fields from instruction format a := udiv_imm.i8(b, c)
                let (b, c, args) = if let ir::InstructionData::BinaryImm {
                    imm,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        imm,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce.i8(z).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let z = pos.ins().udiv_imm(x, c);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let z = pos.ins().udiv_imm(x, c);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Uextend => {
                // Unwrap fields from instruction format a := uextend.i16.i8(b)
                let (b, args) = if let ir::InstructionData::Unary {
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce(c).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 && pos.func.dfg.ctrl_typevar(inst) == ir::types::I16 {
                    let c = pos.ins().uextend(ir::types::I32, b);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, c);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Urem => {
                // Unwrap fields from instruction format a := urem.i8(b, c)
                let (b, c, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce.i8(z).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let z = pos.ins().urem(x, y);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let y = pos.ins().uextend(ir::types::I32, c);
                    let z = pos.ins().urem(x, y);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::UremImm => {
                // Unwrap fields from instruction format a := urem_imm.i8(b, c)
                let (b, c, args) = if let ir::InstructionData::BinaryImm {
                    imm,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        imm,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce.i8(z).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let z = pos.ins().urem_imm(x, c);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let z = pos.ins().urem_imm(x, c);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Ushr => {
                // Unwrap fields from instruction format a := ushr.i8(b, c)
                let (b, c, args) = if let ir::InstructionData::Binary {
                    ref args,
                    ..
                } = pos.func.dfg[inst] {
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        pos.func.dfg.resolve_aliases(args[1]),
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                let typeof_c = pos.func.dfg.value_type(c);
                // Results handled by a := ireduce.i8(z).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let z = pos.ins().ushr(x, c);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let z = pos.ins().ushr(x, c);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::UshrImm => {
                // Unwrap fields from instruction format a := ushr_imm.i8(b, c)
                let (b, c, args) = if let ir::InstructionData::BinaryImm {
                    imm,
                    arg,
                    ..
                } = pos.func.dfg[inst] {
                    let args = [arg];
                    (
                        pos.func.dfg.resolve_aliases(args[0]),
                        imm,
                        args
                    )
                } else {
                    unreachable!("bad instruction format")
                };

                // Results handled by a := ireduce.i8(z).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I8 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let z = pos.ins().ushr_imm(x, c);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I8, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16 {
                    let x = pos.ins().uextend(ir::types::I32, b);
                    let z = pos.ins().ushr_imm(x, c);
                    let a = pos.func.dfg.replace(inst).ireduce(ir::types::I16, z);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            _ => {},
        }
    }
    false
}

// Table of value type sets.
const TYPE_SETS: [ir::instructions::ValueTypeSet; 4] = [
    ir::instructions::ValueTypeSet {
        // TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={8, 16, 32, 64, 128})
        lanes: BitSet::<u16>(511),
        ints: BitSet::<u8>(248),
        floats: BitSet::<u8>(0),
        bools: BitSet::<u8>(0),
        refs: BitSet::<u8>(0),
    },
    ir::instructions::ValueTypeSet {
        // TypeSet(lanes={1}, ints={8, 16, 32, 64, 128})
        lanes: BitSet::<u16>(1),
        ints: BitSet::<u8>(248),
        floats: BitSet::<u8>(0),
        bools: BitSet::<u8>(0),
        refs: BitSet::<u8>(0),
    },
    ir::instructions::ValueTypeSet {
        // TypeSet(lanes={1, 2, 4, 8, 16, 32, 64, 128, 256}, ints={16, 32, 64, 128})
        lanes: BitSet::<u16>(511),
        ints: BitSet::<u8>(240),
        floats: BitSet::<u8>(0),
        bools: BitSet::<u8>(0),
        refs: BitSet::<u8>(0),
    },
    ir::instructions::ValueTypeSet {
        // TypeSet(lanes={1}, ints={16, 32, 64, 128})
        lanes: BitSet::<u16>(1),
        ints: BitSet::<u8>(240),
        floats: BitSet::<u8>(0),
        bools: BitSet::<u8>(0),
        refs: BitSet::<u8>(0),
    },
];

 //clude!(concat!(env!("OUT_DIR"), "/legalizer.rs"));

/// Custom expansion for conditional trap instructions.
/// TODO: Add CFG support to the Rust DSL patterns so we won't have to do this.
fn expand_cond_trap(
    inst: ir::Inst,
    func: &mut ir::Function,
    cfg: &mut ControlFlowGraph,
    _isa: &dyn TargetIsa,
) {
    // Parse the instruction.
    let trapz;
    let (arg, code) = match func.dfg[inst] {
        ir::InstructionData::CondTrap { opcode, arg, code } => {
            // We want to branch *over* an unconditional trap.
            trapz = match opcode {
                ir::Opcode::Trapz => true,
                ir::Opcode::Trapnz => false,
                _ => panic!("Expected cond trap: {}", func.dfg.display_inst(inst, None)),
            };
            (arg, code)
        }
        _ => panic!("Expected cond trap: {}", func.dfg.display_inst(inst, None)),
    };

    // Split the EBB after `inst`:
    //
    //     trapnz arg
    //     ..
    //
    // Becomes:
    //
    //     brz arg, new_ebb_resume
    //     jump new_ebb_trap
    //
    //   new_ebb_trap:
    //     trap
    //
    //   new_ebb_resume:
    //     ..
    let old_ebb = func.layout.pp_ebb(inst);
    let new_ebb_trap = func.dfg.make_ebb();
    let new_ebb_resume = func.dfg.make_ebb();

    // Replace trap instruction by the inverted condition.
    if trapz {
        func.dfg.replace(inst).brnz(arg, new_ebb_resume, &[]);
    } else {
        func.dfg.replace(inst).brz(arg, new_ebb_resume, &[]);
    }

    // Add jump instruction after the inverted branch.
    let mut pos = FuncCursor::new(func).after_inst(inst);
    pos.use_srcloc(inst);
    pos.ins().jump(new_ebb_trap, &[]);

    // Insert the new label and the unconditional trap terminator.
    pos.insert_ebb(new_ebb_trap);
    pos.ins().trap(code);

    // Insert the new label and resume the execution when the trap fails.
    pos.insert_ebb(new_ebb_resume);

    // Finally update the CFG.
    cfg.recompute_ebb(pos.func, old_ebb);
    cfg.recompute_ebb(pos.func, new_ebb_resume);
    cfg.recompute_ebb(pos.func, new_ebb_trap);
}

/// Jump tables.
fn expand_br_table(
    inst: ir::Inst,
    func: &mut ir::Function,
    cfg: &mut ControlFlowGraph,
    isa: &dyn TargetIsa,
) {
    if isa.flags().jump_tables_enabled() {
        expand_br_table_jt(inst, func, cfg, isa);
    } else {
        expand_br_table_conds(inst, func, cfg, isa);
    }
}

/// Expand br_table to jump table.
fn expand_br_table_jt(
    inst: ir::Inst,
    func: &mut ir::Function,
    cfg: &mut ControlFlowGraph,
    isa: &dyn TargetIsa,
) {
    use crate::ir::condcodes::IntCC;

    let (arg, default_ebb, table) = match func.dfg[inst] {
        ir::InstructionData::BranchTable {
            opcode: ir::Opcode::BrTable,
            arg,
            destination,
            table,
        } => (arg, destination, table),
        _ => panic!("Expected br_table: {}", func.dfg.display_inst(inst, None)),
    };

    // Rewrite:
    //
    //     br_table $idx, default_ebb, $jt
    //
    // To:
    //
    //     $oob = ifcmp_imm $idx, len($jt)
    //     brif uge $oob, default_ebb
    //     jump fallthrough_ebb
    //
    //   fallthrough_ebb:
    //     $base = jump_table_base.i64 $jt
    //     $rel_addr = jump_table_entry.i64 $idx, $base, 4, $jt
    //     $addr = iadd $base, $rel_addr
    //     indirect_jump_table_br $addr, $jt

    let ebb = func.layout.pp_ebb(inst);
    let jump_table_ebb = func.dfg.make_ebb();

    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);

    // Bounds check.
    let table_size = pos.func.jump_tables[table].len() as i64;
    let oob = pos
        .ins()
        .icmp_imm(IntCC::UnsignedGreaterThanOrEqual, arg, table_size);

    pos.ins().brnz(oob, default_ebb, &[]);
    pos.ins().jump(jump_table_ebb, &[]);
    pos.insert_ebb(jump_table_ebb);

    let addr_ty = isa.pointer_type();

    let arg = if pos.func.dfg.value_type(arg) == addr_ty {
        arg
    } else {
        pos.ins().uextend(addr_ty, arg)
    };

    let base_addr = pos.ins().jump_table_base(addr_ty, table);
    let entry = pos
        .ins()
        .jump_table_entry(arg, base_addr, I32.bytes() as u8, table);

    let addr = pos.ins().iadd(base_addr, entry);
    pos.ins().indirect_jump_table_br(addr, table);

    pos.remove_inst();
    cfg.recompute_ebb(pos.func, ebb);
    cfg.recompute_ebb(pos.func, jump_table_ebb);
}

/// Expand br_table to series of conditionals.
fn expand_br_table_conds(
    inst: ir::Inst,
    func: &mut ir::Function,
    cfg: &mut ControlFlowGraph,
    _isa: &dyn TargetIsa,
) {
    use crate::ir::condcodes::IntCC;

    let (arg, default_ebb, table) = match func.dfg[inst] {
        ir::InstructionData::BranchTable {
            opcode: ir::Opcode::BrTable,
            arg,
            destination,
            table,
        } => (arg, destination, table),
        _ => panic!("Expected br_table: {}", func.dfg.display_inst(inst, None)),
    };

    let ebb = func.layout.pp_ebb(inst);

    // This is a poor man's jump table using just a sequence of conditional branches.
    let table_size = func.jump_tables[table].len();
    let mut cond_failed_ebb = vec![];
    if table_size >= 1 {
        cond_failed_ebb = alloc::vec::Vec::with_capacity(table_size - 1);
        for _ in 0..table_size - 1 {
            cond_failed_ebb.push(func.dfg.make_ebb());
        }
    }

    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);

    // Ignore the lint for this loop as the range needs to be 0 to table_size
    #[allow(clippy::needless_range_loop)]
    for i in 0..table_size {
        let dest = pos.func.jump_tables[table].as_slice()[i];
        let t = pos.ins().icmp_imm(IntCC::Equal, arg, i as i64);
        pos.ins().brnz(t, dest, &[]);
        // Jump to the next case.
        if i < table_size - 1 {
            let ebb = cond_failed_ebb[i];
            pos.ins().jump(ebb, &[]);
            pos.insert_ebb(ebb);
        }
    }

    // `br_table` jumps to the default destination if nothing matches
    pos.ins().jump(default_ebb, &[]);

    pos.remove_inst();
    cfg.recompute_ebb(pos.func, ebb);
    for failed_ebb in cond_failed_ebb.into_iter() {
        cfg.recompute_ebb(pos.func, failed_ebb);
    }
}

/// Expand the select instruction.
///
/// Conditional moves are available in some ISAs for some register classes. The remaining selects
/// are handled by a branch.
fn expand_select(
    inst: ir::Inst,
    func: &mut ir::Function,
    cfg: &mut ControlFlowGraph,
    _isa: &dyn TargetIsa,
) {
    let (ctrl, tval, fval) = match func.dfg[inst] {
        ir::InstructionData::Ternary {
            opcode: ir::Opcode::Select,
            args,
        } => (args[0], args[1], args[2]),
        _ => panic!("Expected select: {}", func.dfg.display_inst(inst, None)),
    };

    // Replace `result = select ctrl, tval, fval` with:
    //
    //   brnz ctrl, new_ebb(tval)
    //   jump new_ebb(fval)
    // new_ebb(result):
    let old_ebb = func.layout.pp_ebb(inst);
    let result = func.dfg.first_result(inst);
    func.dfg.clear_results(inst);
    let new_ebb = func.dfg.make_ebb();
    func.dfg.attach_ebb_param(new_ebb, result);

    func.dfg.replace(inst).brnz(ctrl, new_ebb, &[tval]);
    let mut pos = FuncCursor::new(func).after_inst(inst);
    pos.use_srcloc(inst);
    pos.ins().jump(new_ebb, &[fval]);
    pos.insert_ebb(new_ebb);

    cfg.recompute_ebb(pos.func, new_ebb);
    cfg.recompute_ebb(pos.func, old_ebb);
}

fn expand_br_icmp(
    inst: ir::Inst,
    func: &mut ir::Function,
    cfg: &mut ControlFlowGraph,
    _isa: &dyn TargetIsa,
) {
    let (cond, a, b, destination, ebb_args) = match func.dfg[inst] {
        ir::InstructionData::BranchIcmp {
            cond,
            destination,
            ref args,
            ..
        } => (
            cond,
            args.get(0, &func.dfg.value_lists).unwrap(),
            args.get(1, &func.dfg.value_lists).unwrap(),
            destination,
            args.as_slice(&func.dfg.value_lists)[2..].to_vec(),
        ),
        _ => panic!("Expected br_icmp {}", func.dfg.display_inst(inst, None)),
    };

    let old_ebb = func.layout.pp_ebb(inst);
    func.dfg.clear_results(inst);

    let icmp_res = func.dfg.replace(inst).icmp(cond, a, b);
    let mut pos = FuncCursor::new(func).after_inst(inst);
    pos.use_srcloc(inst);
    pos.ins().brnz(icmp_res, destination, &ebb_args);

    cfg.recompute_ebb(pos.func, destination);
    cfg.recompute_ebb(pos.func, old_ebb);
}

/// Expand illegal `f32const` and `f64const` instructions.
fn expand_fconst(
    inst: ir::Inst,
    func: &mut ir::Function,
    _cfg: &mut ControlFlowGraph,
    _isa: &dyn TargetIsa,
) {
    let ty = func.dfg.value_type(func.dfg.first_result(inst));
    debug_assert!(!ty.is_vector(), "Only scalar fconst supported: {}", ty);

    // In the future, we may want to generate constant pool entries for these constants, but for
    // now use an `iconst` and a bit cast.
    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);
    let ival = match pos.func.dfg[inst] {
        ir::InstructionData::UnaryIeee32 {
            opcode: ir::Opcode::F32const,
            imm,
        } => pos.ins().iconst(ir::types::I32, i64::from(imm.bits())),
        ir::InstructionData::UnaryIeee64 {
            opcode: ir::Opcode::F64const,
            imm,
        } => pos.ins().iconst(ir::types::I64, imm.bits() as i64),
        _ => panic!("Expected fconst: {}", pos.func.dfg.display_inst(inst, None)),
    };
    pos.func.dfg.replace(inst).bitcast(ty, ival);
}

/// Expand illegal `stack_load` instructions.
fn expand_stack_load(
    inst: ir::Inst,
    func: &mut ir::Function,
    _cfg: &mut ControlFlowGraph,
    isa: &dyn TargetIsa,
) {
    let ty = func.dfg.value_type(func.dfg.first_result(inst));
    let addr_ty = isa.pointer_type();

    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);

    let (stack_slot, offset) = match pos.func.dfg[inst] {
        ir::InstructionData::StackLoad {
            opcode: _opcode,
            stack_slot,
            offset,
        } => (stack_slot, offset),
        _ => panic!(
            "Expected stack_load: {}",
            pos.func.dfg.display_inst(inst, None)
        ),
    };

    let addr = pos.ins().stack_addr(addr_ty, stack_slot, offset);

    // Stack slots are required to be accessible and aligned.
    let mflags = MemFlags::trusted();
    pos.func.dfg.replace(inst).load(ty, mflags, addr, 0);
}

/// Expand illegal `stack_store` instructions.
fn expand_stack_store(
    inst: ir::Inst,
    func: &mut ir::Function,
    _cfg: &mut ControlFlowGraph,
    isa: &dyn TargetIsa,
) {
    let addr_ty = isa.pointer_type();

    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);

    let (val, stack_slot, offset) = match pos.func.dfg[inst] {
        ir::InstructionData::StackStore {
            opcode: _opcode,
            arg,
            stack_slot,
            offset,
        } => (arg, stack_slot, offset),
        _ => panic!(
            "Expected stack_store: {}",
            pos.func.dfg.display_inst(inst, None)
        ),
    };

    let addr = pos.ins().stack_addr(addr_ty, stack_slot, offset);

    let mut mflags = MemFlags::new();
    // Stack slots are required to be accessible and aligned.
    mflags.set_notrap();
    mflags.set_aligned();
    pos.func.dfg.replace(inst).store(mflags, val, addr, 0);
}

/// Split a load into two parts before `iconcat`ing the result together.
fn narrow_load(
    inst: ir::Inst,
    func: &mut ir::Function,
    _cfg: &mut ControlFlowGraph,
    _isa: &dyn TargetIsa,
) {
    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);

    let (ptr, offset, flags) = match pos.func.dfg[inst] {
        ir::InstructionData::Load {
            opcode: ir::Opcode::Load,
            arg,
            offset,
            flags,
        } => (arg, offset, flags),
        _ => panic!("Expected load: {}", pos.func.dfg.display_inst(inst, None)),
    };

    let res_ty = pos.func.dfg.ctrl_typevar(inst);
    let small_ty = res_ty.half_width().expect("Can't narrow load");

    let al = pos.ins().load(small_ty, flags, ptr, offset);
    let ah = pos.ins().load(
        small_ty,
        flags,
        ptr,
        offset.try_add_i64(8).expect("load offset overflow"),
    );
    pos.func.dfg.replace(inst).iconcat(al, ah);
}

/// Split a store into two parts after `isplit`ing the value.
fn narrow_store(
    inst: ir::Inst,
    func: &mut ir::Function,
    _cfg: &mut ControlFlowGraph,
    _isa: &dyn TargetIsa,
) {
    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);

    let (val, ptr, offset, flags) = match pos.func.dfg[inst] {
        ir::InstructionData::Store {
            opcode: ir::Opcode::Store,
            args,
            offset,
            flags,
        } => (args[0], args[1], offset, flags),
        _ => panic!("Expected store: {}", pos.func.dfg.display_inst(inst, None)),
    };

    let (al, ah) = pos.ins().isplit(val);
    pos.ins().store(flags, al, ptr, offset);
    pos.ins().store(
        flags,
        ah,
        ptr,
        offset.try_add_i64(8).expect("store offset overflow"),
    );
    pos.remove_inst();
}

/// Expands an illegal iconst value by splitting it into two.
fn narrow_iconst(
    inst: ir::Inst,
    func: &mut ir::Function,
    _cfg: &mut ControlFlowGraph,
    isa: &dyn TargetIsa,
) {
    let imm: i64 = if let ir::InstructionData::UnaryImm {
        opcode: ir::Opcode::Iconst,
        imm,
    } = &func.dfg[inst]
    {
        (*imm).into()
    } else {
        panic!("unexpected instruction in narrow_iconst");
    };

    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);

    let ty = pos.func.dfg.ctrl_typevar(inst);
    if isa.pointer_bits() == 32 && ty == I64 {
        let low = pos.ins().iconst(I32, imm & 0xffffffff);
        let high = pos.ins().iconst(I32, imm >> 32);
        // The instruction has as many results as iconcat, so no need to replace them.
        pos.func.dfg.replace(inst).iconcat(low, high);
        return;
    }

    unimplemented!("missing encoding or legalization for iconst.{:?}", ty);
}

fn narrow_icmp_imm(
    inst: ir::Inst,
    func: &mut ir::Function,
    _cfg: &mut ControlFlowGraph,
    _isa: &dyn TargetIsa,
) {
    use crate::ir::condcodes::{CondCode, IntCC};

    let (arg, cond, imm): (ir::Value, IntCC, i64) = match func.dfg[inst] {
        ir::InstructionData::IntCompareImm {
            opcode: ir::Opcode::IcmpImm,
            arg,
            cond,
            imm,
        } => (arg, cond, imm.into()),
        _ => panic!("unexpected instruction in narrow_icmp_imm"),
    };

    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);

    let ty = pos.func.dfg.ctrl_typevar(inst);
    let ty_half = ty.half_width().unwrap();

    let imm_low = pos
        .ins()
        .iconst(ty_half, imm & (1u128 << (ty_half.bits() - 1)) as i64);
    let imm_high = pos
        .ins()
        .iconst(ty_half, imm.wrapping_shr(ty_half.bits().into()));
    let (arg_low, arg_high) = pos.ins().isplit(arg);

    match cond {
        IntCC::Equal => {
            let res_low = pos.ins().icmp(cond, arg_low, imm_low);
            let res_high = pos.ins().icmp(cond, arg_high, imm_high);
            pos.func.dfg.replace(inst).band(res_low, res_high);
        }
        IntCC::NotEqual => {
            let res_low = pos.ins().icmp(cond, arg_low, imm_low);
            let res_high = pos.ins().icmp(cond, arg_high, imm_high);
            pos.func.dfg.replace(inst).bor(res_low, res_high);
        }
        IntCC::SignedGreaterThan
        | IntCC::SignedGreaterThanOrEqual
        | IntCC::SignedLessThan
        | IntCC::SignedLessThanOrEqual
        | IntCC::UnsignedGreaterThan
        | IntCC::UnsignedGreaterThanOrEqual
        | IntCC::UnsignedLessThan
        | IntCC::UnsignedLessThanOrEqual => {
            let b1 = pos.ins().icmp(cond.without_equal(), arg_high, imm_high);
            let b2 = pos
                .ins()
                .icmp(cond.inverse().without_equal(), arg_high, imm_high);
            let b3 = pos.ins().icmp(cond.unsigned(), arg_low, imm_low);
            let c1 = pos.ins().bnot(b2);
            let c2 = pos.ins().band(c1, b3);
            pos.func.dfg.replace(inst).bor(b1, c2);
        }
        _ => unimplemented!("missing legalization for condition {:?}", cond),
    }
}
