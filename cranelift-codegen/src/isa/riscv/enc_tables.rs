//! Encoding tables for RISC-V.

use super::registers::*;
use crate::ir;
use crate::isa;
use crate::isa::constraints::*;
use crate::isa::enc_tables::*;
use crate::isa::encoding::{base_size, RecipeSizing};
use crate::predicates;

// Include the generated encoding tables:
// - `LEVEL1_RV32`
// - `LEVEL1_RV64`
// - `LEVEL2`
// - `ENCLIST`
// - `INFO`
 
// riscv recipe predicates.
fn recipe_predicate_ii(_: crate::settings::PredicateView, inst: &ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::BinaryImm { imm, .. } = *inst {
        return predicates::is_signed_int(imm, 12, 0);
    }
    unreachable!();
}
fn recipe_predicate_iz(_: crate::settings::PredicateView, inst: &ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::UnaryImm { imm, .. } = *inst {
        return predicates::is_signed_int(imm, 12, 0);
    }
    unreachable!();
}
fn recipe_predicate_iicmp(_: crate::settings::PredicateView, inst: &ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::IntCompareImm { imm, .. } = *inst {
        return predicates::is_signed_int(imm, 12, 0);
    }
    unreachable!();
}
fn recipe_predicate_u(_: crate::settings::PredicateView, inst: &ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::UnaryImm { imm, .. } = *inst {
        return predicates::is_signed_int(imm, 32, 12);
    }
    unreachable!();
}

/// riscv recipe predicate table.
///
/// One entry per recipe, set to Some only when the recipe is guarded by a predicate.
pub static RECIPE_PREDICATES: [RecipePredicate; 20] = [
    None,
    None,
    None,
    Some(recipe_predicate_ii),
    Some(recipe_predicate_iz),
    Some(recipe_predicate_iicmp),
    None,
    None,
    None,
    None,
    None,
    Some(recipe_predicate_u),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
];

// riscv instruction predicates.
fn inst_predicate_0(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    let args = inst.arguments(&func.dfg.value_lists);
    func.dfg.value_type(args[1]) == ir::types::I32
}
fn inst_predicate_1(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    let args = inst.arguments(&func.dfg.value_lists);
    func.dfg.value_type(args[1]) == ir::types::I64
}
fn inst_predicate_2(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::IntCompare { cond, .. } = *inst {
        let _ = func;
        return predicates::is_equal(cond, ir::condcodes::IntCC::SignedLessThan);
    }
    unreachable!();
}
fn inst_predicate_3(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::IntCompare { cond, .. } = *inst {
        let _ = func;
        return predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedLessThan);
    }
    unreachable!();
}
fn inst_predicate_4(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::IntCompareImm { cond, .. } = *inst {
        let _ = func;
        return predicates::is_equal(cond, ir::condcodes::IntCC::SignedLessThan);
    }
    unreachable!();
}
fn inst_predicate_5(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::IntCompareImm { cond, .. } = *inst {
        let _ = func;
        return predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedLessThan);
    }
    unreachable!();
}
fn inst_predicate_6(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::BranchIcmp { cond, .. } = *inst {
        let _ = func;
        return predicates::is_equal(cond, ir::condcodes::IntCC::Equal);
    }
    unreachable!();
}
fn inst_predicate_7(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::BranchIcmp { cond, .. } = *inst {
        let _ = func;
        return predicates::is_equal(cond, ir::condcodes::IntCC::NotEqual);
    }
    unreachable!();
}
fn inst_predicate_8(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::BranchIcmp { cond, .. } = *inst {
        let _ = func;
        return predicates::is_equal(cond, ir::condcodes::IntCC::SignedLessThan);
    }
    unreachable!();
}
fn inst_predicate_9(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::BranchIcmp { cond, .. } = *inst {
        let _ = func;
        return predicates::is_equal(cond, ir::condcodes::IntCC::SignedGreaterThanOrEqual);
    }
    unreachable!();
}
fn inst_predicate_10(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::BranchIcmp { cond, .. } = *inst {
        let _ = func;
        return predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedLessThan);
    }
    unreachable!();
}
fn inst_predicate_11(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::BranchIcmp { cond, .. } = *inst {
        let _ = func;
        return predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedGreaterThanOrEqual);
    }
    unreachable!();
}

/// riscv instruction predicate table.
///
/// One entry per instruction predicate, so the encoding bytecode can embed indexes into this
/// table.
pub static INST_PREDICATES: [InstPredicate; 12] = [
    inst_predicate_0,
    inst_predicate_1,
    inst_predicate_2,
    inst_predicate_3,
    inst_predicate_4,
    inst_predicate_5,
    inst_predicate_6,
    inst_predicate_7,
    inst_predicate_8,
    inst_predicate_9,
    inst_predicate_10,
    inst_predicate_11,
];

/// riscv encoding lists.
///
/// This contains the entire encodings bytecode for every single instruction; the encodings
/// interpreter knows where to start from thanks to the initial lookup in the level 1 and level 2
/// table entries below.
pub static ENCLISTS: [u16; 157] = [
    // 000000: band.i32 (RV32)
    // --> [R#ec] and stop
    // 000000: band.i64 (RV64)
    // --> [R#ec] and stop
    0x0001, 0x00ec,
    // end of band.i64 (RV64)
    // end of band.i32 (RV32)
    // 000002: band_imm.i32 (RV32)
    // --> [Ii#e4] and stop
    // 000002: band_imm.i64 (RV64)
    // --> [Ii#e4] and stop
    0x0007, 0x00e4,
    // end of band_imm.i64 (RV64)
    // end of band_imm.i32 (RV32)
    // 000004: bor.i32 (RV32)
    // --> [R#cc] and stop
    // 000004: bor.i64 (RV64)
    // --> [R#cc] and stop
    0x0001, 0x00cc,
    // end of bor.i64 (RV64)
    // end of bor.i32 (RV32)
    // 000006: bor_imm.i32 (RV32)
    // --> [Ii#c4] and stop
    // 000006: bor_imm.i64 (RV64)
    // --> [Ii#c4] and stop
    0x0007, 0x00c4,
    // end of bor_imm.i64 (RV64)
    // end of bor_imm.i32 (RV32)
    // 000008: br_icmp.i32 (RV32)
    // skip 2 unless inst_predicate_6
    // 000008: br_icmp.i64 (RV64)
    // skip 2 unless inst_predicate_6
    0x3006,
    // --> [SB#18]
    // --> [SB#18]
    0x001c, 0x0018,
    // skip 2 unless inst_predicate_7
    // skip 2 unless inst_predicate_7
    0x3007,
    // --> [SB#38]
    // --> [SB#38]
    0x001c, 0x0038,
    // skip 2 unless inst_predicate_8
    // skip 2 unless inst_predicate_8
    0x3008,
    // --> [SB#98]
    // --> [SB#98]
    0x001c, 0x0098,
    // skip 2 unless inst_predicate_9
    // skip 2 unless inst_predicate_9
    0x3009,
    // --> [SB#b8]
    // --> [SB#b8]
    0x001c, 0x00b8,
    // skip 2 unless inst_predicate_10
    // skip 2 unless inst_predicate_10
    0x300a,
    // --> [SB#d8]
    // --> [SB#d8]
    0x001c, 0x00d8,
    // stop unless inst_predicate_11
    // stop unless inst_predicate_11
    0x100b,
    // --> [SB#f8] and stop
    // --> [SB#f8] and stop
    0x001d, 0x00f8,
    // end of br_icmp.i64 (RV64)
    // end of br_icmp.i32 (RV32)
    // 00001a: brnz.i32 (RV32)
    // --> [SBzero#38] and stop
    // 00001a: brnz.b1 (RV32)
    // --> [SBzero#38] and stop
    // 00001a: brnz.i64 (RV64)
    // --> [SBzero#38] and stop
    // 00001a: brnz.b1 (RV64)
    // --> [SBzero#38] and stop
    0x001f, 0x0038,
    // end of brnz.b1 (RV64)
    // end of brnz.i64 (RV64)
    // end of brnz.b1 (RV32)
    // end of brnz.i32 (RV32)
    // 00001c: brz.i32 (RV32)
    // --> [SBzero#18] and stop
    // 00001c: brz.b1 (RV32)
    // --> [SBzero#18] and stop
    // 00001c: brz.i64 (RV64)
    // --> [SBzero#18] and stop
    // 00001c: brz.b1 (RV64)
    // --> [SBzero#18] and stop
    0x001f, 0x0018,
    // end of brz.b1 (RV64)
    // end of brz.i64 (RV64)
    // end of brz.b1 (RV32)
    // end of brz.i32 (RV32)
    // 00001e: bxor.i32 (RV32)
    // --> [R#8c] and stop
    // 00001e: bxor.i64 (RV64)
    // --> [R#8c] and stop
    0x0001, 0x008c,
    // end of bxor.i64 (RV64)
    // end of bxor.i32 (RV32)
    // 000020: bxor_imm.i32 (RV32)
    // --> [Ii#84] and stop
    // 000020: bxor_imm.i64 (RV64)
    // --> [Ii#84] and stop
    0x0007, 0x0084,
    // end of bxor_imm.i64 (RV64)
    // end of bxor_imm.i32 (RV32)
    // 000022: call_indirect.i32 (RV32)
    // --> [Icall#19] and stop
    // 000022: call_indirect.i64 (RV64)
    // --> [Icall#19] and stop
    0x000f, 0x0019,
    // end of call_indirect.i64 (RV64)
    // end of call_indirect.i32 (RV32)
    // 000024: copy.i32 (RV32)
    // --> [Icopy#04] and stop
    // 000024: copy.b1 (RV32)
    // --> [Icopy#04] and stop
    // 000024: copy.i64 (RV64)
    // --> [Icopy#04] and stop
    // 000024: copy.b1 (RV64)
    // --> [Icopy#04] and stop
    0x0011, 0x0004,
    // end of copy.b1 (RV64)
    // end of copy.i64 (RV64)
    // end of copy.b1 (RV32)
    // end of copy.i32 (RV32)
    // 000026: copy_nop.i32 (RV32)
    // --> [stacknull#00] and stop
    // 000026: copy_nop.i64 (RV32)
    // --> [stacknull#00] and stop
    // 000026: copy_nop.i16 (RV32)
    // --> [stacknull#00] and stop
    // 000026: copy_nop.i8 (RV32)
    // --> [stacknull#00] and stop
    // 000026: copy_nop.f64 (RV32)
    // --> [stacknull#00] and stop
    // 000026: copy_nop.f32 (RV32)
    // --> [stacknull#00] and stop
    // 000026: copy_nop.i64 (RV64)
    // --> [stacknull#00] and stop
    // 000026: copy_nop.i32 (RV64)
    // --> [stacknull#00] and stop
    // 000026: copy_nop.i16 (RV64)
    // --> [stacknull#00] and stop
    // 000026: copy_nop.i8 (RV64)
    // --> [stacknull#00] and stop
    // 000026: copy_nop.f64 (RV64)
    // --> [stacknull#00] and stop
    // 000026: copy_nop.f32 (RV64)
    // --> [stacknull#00] and stop
    0x0025, 0x0000,
    // end of copy_nop.f32 (RV64)
    // end of copy_nop.f64 (RV64)
    // end of copy_nop.i8 (RV64)
    // end of copy_nop.i16 (RV64)
    // end of copy_nop.i32 (RV64)
    // end of copy_nop.i64 (RV64)
    // end of copy_nop.f32 (RV32)
    // end of copy_nop.f64 (RV32)
    // end of copy_nop.i8 (RV32)
    // end of copy_nop.i16 (RV32)
    // end of copy_nop.i64 (RV32)
    // end of copy_nop.i32 (RV32)
    // 000028: copy_to_ssa.i32 (RV32)
    // --> [copytossa#04] and stop
    // 000028: copy_to_ssa.b1 (RV32)
    // --> [copytossa#04] and stop
    // 000028: copy_to_ssa.r32 (RV32)
    // --> [copytossa#04] and stop
    // 000028: copy_to_ssa.i64 (RV64)
    // --> [copytossa#04] and stop
    // 000028: copy_to_ssa.b1 (RV64)
    // --> [copytossa#04] and stop
    // 000028: copy_to_ssa.r64 (RV64)
    // --> [copytossa#04] and stop
    0x0015, 0x0004,
    // end of copy_to_ssa.r64 (RV64)
    // end of copy_to_ssa.b1 (RV64)
    // end of copy_to_ssa.i64 (RV64)
    // end of copy_to_ssa.r32 (RV32)
    // end of copy_to_ssa.b1 (RV32)
    // end of copy_to_ssa.i32 (RV32)
    // 00002a: fill.i32 (RV32)
    // --> [GPfi#40] and stop
    // 00002a: fill.i32 (RV64)
    // --> [GPfi#40] and stop
    0x0023, 0x0040,
    // end of fill.i32 (RV64)
    // end of fill.i32 (RV32)
    // 00002c: fill_nop.i32 (RV32)
    // --> [fillnull#00] and stop
    // 00002c: fill_nop.b1 (RV32)
    // --> [fillnull#00] and stop
    // 00002c: fill_nop.i64 (RV32)
    // --> [fillnull#00] and stop
    // 00002c: fill_nop.i64 (RV64)
    // --> [fillnull#00] and stop
    // 00002c: fill_nop.i32 (RV64)
    // --> [fillnull#00] and stop
    // 00002c: fill_nop.b1 (RV64)
    // --> [fillnull#00] and stop
    0x0027, 0x0000,
    // end of fill_nop.b1 (RV64)
    // end of fill_nop.i32 (RV64)
    // end of fill_nop.i64 (RV64)
    // end of fill_nop.i64 (RV32)
    // end of fill_nop.b1 (RV32)
    // end of fill_nop.i32 (RV32)
    // 00002e: iadd.i32 (RV32)
    // --> [R#0c] and stop
    // 00002e: iadd.i64 (RV64)
    // --> [R#0c] and stop
    0x0001, 0x000c,
    // end of iadd.i64 (RV64)
    // end of iadd.i32 (RV32)
    // 000030: iadd_imm.i32 (RV32)
    // --> [Ii#04] and stop
    // 000030: iadd_imm.i64 (RV64)
    // --> [Ii#04] and stop
    0x0007, 0x0004,
    // end of iadd_imm.i64 (RV64)
    // end of iadd_imm.i32 (RV32)
    // 000032: icmp.i32 (RV32)
    // skip 2 unless inst_predicate_2
    // 000032: icmp.i64 (RV64)
    // skip 2 unless inst_predicate_2
    0x3002,
    // --> [Ricmp#4c]
    // --> [Ricmp#4c]
    0x0004, 0x004c,
    // stop unless inst_predicate_3
    // stop unless inst_predicate_3
    0x1003,
    // --> [Ricmp#6c] and stop
    // --> [Ricmp#6c] and stop
    0x0005, 0x006c,
    // end of icmp.i64 (RV64)
    // end of icmp.i32 (RV32)
    // 000038: icmp_imm.i32 (RV32)
    // skip 2 unless inst_predicate_4
    // 000038: icmp_imm.i64 (RV64)
    // skip 2 unless inst_predicate_4
    0x3004,
    // --> [Iicmp#44]
    // --> [Iicmp#44]
    0x000a, 0x0044,
    // stop unless inst_predicate_5
    // stop unless inst_predicate_5
    0x1005,
    // --> [Iicmp#64] and stop
    // --> [Iicmp#64] and stop
    0x000b, 0x0064,
    // end of icmp_imm.i64 (RV64)
    // end of icmp_imm.i32 (RV32)
    // 00003e: iconst.i32 (RV32)
    // --> [Iz#04]
    // 00003e: iconst.i64 (RV64)
    // --> [Iz#04]
    // 00003e: iconst.i32 (RV64)
    // --> [Iz#04]
    0x0008, 0x0004,
    // --> [U#0d] and stop
    // --> [U#0d] and stop
    // --> [U#0d] and stop
    0x0017, 0x000d,
    // end of iconst.i32 (RV64)
    // end of iconst.i64 (RV64)
    // end of iconst.i32 (RV32)
    // 000042: imul.i32 (RV32)
    // stop unless PredicateView(10)
    // 000042: imul.i64 (RV64)
    // stop unless PredicateView(10)
    0x1016,
    // --> [R#10c] and stop
    // --> [R#10c] and stop
    0x0001, 0x010c,
    // end of imul.i64 (RV64)
    // end of imul.i32 (RV32)
    // 000045: ishl.i32 (RV32)
    // stop unless inst_predicate_0
    0x1000,
    // --> [R#2c] and stop
    0x0001, 0x002c,
    // end of ishl.i32 (RV32)
    // 000048: ishl_imm.i32 (RV32)
    // --> [Rshamt#24] and stop
    // 000048: ishl_imm.i64 (RV64)
    // --> [Rshamt#24] and stop
    0x0003, 0x0024,
    // end of ishl_imm.i64 (RV64)
    // end of ishl_imm.i32 (RV32)
    // 00004a: isub.i32 (RV32)
    // --> [R#200c] and stop
    // 00004a: isub.i64 (RV64)
    // --> [R#200c] and stop
    0x0001, 0x200c,
    // end of isub.i64 (RV64)
    // end of isub.i32 (RV32)
    // 00004c: regmove.i32 (RV32)
    // --> [Irmov#04] and stop
    // 00004c: regmove.b1 (RV32)
    // --> [Irmov#04] and stop
    // 00004c: regmove.i64 (RV64)
    // --> [Irmov#04] and stop
    // 00004c: regmove.b1 (RV64)
    // --> [Irmov#04] and stop
    0x0013, 0x0004,
    // end of regmove.b1 (RV64)
    // end of regmove.i64 (RV64)
    // end of regmove.b1 (RV32)
    // end of regmove.i32 (RV32)
    // 00004e: spill.i32 (RV32)
    // --> [GPsp#48] and stop
    // 00004e: spill.i32 (RV64)
    // --> [GPsp#48] and stop
    0x0021, 0x0048,
    // end of spill.i32 (RV64)
    // end of spill.i32 (RV32)
    // 000050: sshr.i32 (RV32)
    // stop unless inst_predicate_0
    0x1000,
    // --> [R#20ac] and stop
    0x0001, 0x20ac,
    // end of sshr.i32 (RV32)
    // 000053: sshr_imm.i32 (RV32)
    // --> [Rshamt#20a4] and stop
    // 000053: sshr_imm.i64 (RV64)
    // --> [Rshamt#20a4] and stop
    0x0003, 0x20a4,
    // end of sshr_imm.i64 (RV64)
    // end of sshr_imm.i32 (RV32)
    // 000055: ushr.i32 (RV32)
    // stop unless inst_predicate_0
    0x1000,
    // --> [R#ac] and stop
    0x0001, 0x00ac,
    // end of ushr.i32 (RV32)
    // 000058: ushr_imm.i32 (RV32)
    // --> [Rshamt#a4] and stop
    // 000058: ushr_imm.i64 (RV64)
    // --> [Rshamt#a4] and stop
    0x0003, 0x00a4,
    // end of ushr_imm.i64 (RV64)
    // end of ushr_imm.i32 (RV32)
    // 00005a: call (RV32)
    // --> [UJcall#1b] and stop
    // 00005a: call (RV64)
    // --> [UJcall#1b] and stop
    0x001b, 0x001b,
    // end of call (RV64)
    // end of call (RV32)
    // 00005c: jump (RV32)
    // --> [UJ#1b] and stop
    // 00005c: jump (RV64)
    // --> [UJ#1b] and stop
    0x0019, 0x001b,
    // end of jump (RV64)
    // end of jump (RV32)
    // 00005e: return (RV32)
    // --> [Iret#19] and stop
    // 00005e: return (RV64)
    // --> [Iret#19] and stop
    0x000d, 0x0019,
    // end of return (RV64)
    // end of return (RV32)
    // 000060: fill.i64 (RV64)
    // --> [GPfi#60] and stop
    0x0023, 0x0060,
    // end of fill.i64 (RV64)
    // 000062: ishl.i64 (RV64)
    // skip 2 unless inst_predicate_1
    0x3001,
    // --> [R#2c]
    0x0000, 0x002c,
    // stop unless inst_predicate_0
    0x1000,
    // --> [R#2c] and stop
    0x0001, 0x002c,
    // end of ishl.i64 (RV64)
    // 000068: spill.i64 (RV64)
    // --> [GPsp#68] and stop
    0x0021, 0x0068,
    // end of spill.i64 (RV64)
    // 00006a: sshr.i64 (RV64)
    // skip 2 unless inst_predicate_1
    0x3001,
    // --> [R#20ac]
    0x0000, 0x20ac,
    // stop unless inst_predicate_0
    0x1000,
    // --> [R#20ac] and stop
    0x0001, 0x20ac,
    // end of sshr.i64 (RV64)
    // 000070: ushr.i64 (RV64)
    // skip 2 unless inst_predicate_1
    0x3001,
    // --> [R#ac]
    0x0000, 0x00ac,
    // stop unless inst_predicate_0
    0x1000,
    // --> [R#ac] and stop
    0x0001, 0x00ac,
    // end of ushr.i64 (RV64)
    // 000076: copy.i32 (RV64)
    // --> [Icopy#06] and stop
    0x0011, 0x0006,
    // end of copy.i32 (RV64)
    // 000078: copy_to_ssa.i32 (RV64)
    // --> [copytossa#06] and stop
    0x0015, 0x0006,
    // end of copy_to_ssa.i32 (RV64)
    // 00007a: iadd.i32 (RV64)
    // --> [R#0e] and stop
    0x0001, 0x000e,
    // end of iadd.i32 (RV64)
    // 00007c: iadd_imm.i32 (RV64)
    // --> [Ii#06] and stop
    0x0007, 0x0006,
    // end of iadd_imm.i32 (RV64)
    // 00007e: imul.i32 (RV64)
    // stop unless PredicateView(10)
    0x1016,
    // --> [R#10e] and stop
    0x0001, 0x010e,
    // end of imul.i32 (RV64)
    // 000081: ishl.i32 (RV64)
    // skip 2 unless inst_predicate_0
    0x3000,
    // --> [R#2e]
    0x0000, 0x002e,
    // stop unless inst_predicate_1
    0x1001,
    // --> [R#2e] and stop
    0x0001, 0x002e,
    // end of ishl.i32 (RV64)
    // 000087: ishl_imm.i32 (RV64)
    // --> [Rshamt#26] and stop
    0x0003, 0x0026,
    // end of ishl_imm.i32 (RV64)
    // 000089: isub.i32 (RV64)
    // --> [R#200e] and stop
    0x0001, 0x200e,
    // end of isub.i32 (RV64)
    // 00008b: regmove.i32 (RV64)
    // --> [Irmov#06] and stop
    0x0013, 0x0006,
    // end of regmove.i32 (RV64)
    // 00008d: sshr.i32 (RV64)
    // skip 2 unless inst_predicate_0
    0x3000,
    // --> [R#20ae]
    0x0000, 0x20ae,
    // stop unless inst_predicate_1
    0x1001,
    // --> [R#20ae] and stop
    0x0001, 0x20ae,
    // end of sshr.i32 (RV64)
    // 000093: sshr_imm.i32 (RV64)
    // --> [Rshamt#20a6] and stop
    0x0003, 0x20a6,
    // end of sshr_imm.i32 (RV64)
    // 000095: ushr.i32 (RV64)
    // skip 2 unless inst_predicate_0
    0x3000,
    // --> [R#ae]
    0x0000, 0x00ae,
    // stop unless inst_predicate_1
    0x1001,
    // --> [R#ae] and stop
    0x0001, 0x00ae,
    // end of ushr.i32 (RV64)
    // 00009b: ushr_imm.i32 (RV64)
    // --> [Rshamt#a6] and stop
    0x0003, 0x00a6,
];

/// riscv level 2 hash tables.
///
/// This hash table, keyed by instruction opcode, contains all the starting offsets for the
/// encodings interpreter, for all the CPU modes. It is jumped to after a lookup on the
/// instruction's controlling type in the level 1 hash table.
pub static LEVEL2: [Level2Entry<u16>; 208] = [
    // RV32
    // 000000: i32, 64 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x00001e },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x000024 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Brz), offset: 0x00001c },
    Level2Entry { opcode: Some(crate::ir::Opcode::Brnz), offset: 0x00001a },
    Level2Entry { opcode: Some(crate::ir::Opcode::BandImm), offset: 0x000002 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BorImm), offset: 0x000006 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BxorImm), offset: 0x000020 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BrIcmp), offset: 0x000008 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000026 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x00002a },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyToSsa), offset: 0x000028 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Ishl), offset: 0x000045 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x00004e },
    Level2Entry { opcode: Some(crate::ir::Opcode::Sshr), offset: 0x000050 },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x00002c },
    Level2Entry { opcode: Some(crate::ir::Opcode::IshlImm), offset: 0x000048 },
    Level2Entry { opcode: Some(crate::ir::Opcode::SshrImm), offset: 0x000053 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Ushr), offset: 0x000055 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x00004c },
    Level2Entry { opcode: Some(crate::ir::Opcode::CallIndirect), offset: 0x000022 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Icmp), offset: 0x000032 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IcmpImm), offset: 0x000038 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Iadd), offset: 0x00002e },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::UshrImm), offset: 0x000058 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Isub), offset: 0x00004a },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Imul), offset: 0x000042 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IaddImm), offset: 0x000030 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Iconst), offset: 0x00003e },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x000000 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x000004 },
    // 000040: typeless, 4 entries
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Call), offset: 0x00005a },
    Level2Entry { opcode: Some(crate::ir::Opcode::Jump), offset: 0x00005c },
    Level2Entry { opcode: Some(crate::ir::Opcode::Return), offset: 0x00005e },
    // 000044: b1, 8 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyToSsa), offset: 0x000028 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x000024 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Brz), offset: 0x00001c },
    Level2Entry { opcode: Some(crate::ir::Opcode::Brnz), offset: 0x00001a },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x00002c },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x00004c },
    Level2Entry { opcode: None, offset: 0 },
    // 00004c: i64, 4 entries
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000026 },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x00002c },
    Level2Entry { opcode: None, offset: 0 },
    // 000050: i16, 2 entries
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000026 },
    // 000052: i8, 2 entries
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000026 },
    // 000054: f64, 2 entries
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000026 },
    // 000056: f32, 2 entries
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000026 },
    // 000058: r32, 2 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyToSsa), offset: 0x000028 },
    Level2Entry { opcode: None, offset: 0 },
    // RV64
    // 00005a: i64, 64 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x00001e },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x000024 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Brz), offset: 0x00001c },
    Level2Entry { opcode: Some(crate::ir::Opcode::Brnz), offset: 0x00001a },
    Level2Entry { opcode: Some(crate::ir::Opcode::BandImm), offset: 0x000002 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BorImm), offset: 0x000006 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BxorImm), offset: 0x000020 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BrIcmp), offset: 0x000008 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000026 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x000060 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyToSsa), offset: 0x000028 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Ishl), offset: 0x000062 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x000068 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Sshr), offset: 0x00006a },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x00002c },
    Level2Entry { opcode: Some(crate::ir::Opcode::IshlImm), offset: 0x000048 },
    Level2Entry { opcode: Some(crate::ir::Opcode::SshrImm), offset: 0x000053 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Ushr), offset: 0x000070 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x00004c },
    Level2Entry { opcode: Some(crate::ir::Opcode::CallIndirect), offset: 0x000022 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Icmp), offset: 0x000032 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IcmpImm), offset: 0x000038 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Iadd), offset: 0x00002e },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::UshrImm), offset: 0x000058 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Isub), offset: 0x00004a },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Imul), offset: 0x000042 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IaddImm), offset: 0x000030 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Iconst), offset: 0x00003e },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x000000 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x000004 },
    // 00009a: i32, 32 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Isub), offset: 0x000089 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x000076 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x00004e },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x00002a },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x00002c },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x00008b },
    Level2Entry { opcode: Some(crate::ir::Opcode::Imul), offset: 0x00007e },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyToSsa), offset: 0x000078 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000026 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IaddImm), offset: 0x00007c },
    Level2Entry { opcode: Some(crate::ir::Opcode::Ishl), offset: 0x000081 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Ushr), offset: 0x000095 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Sshr), offset: 0x00008d },
    Level2Entry { opcode: Some(crate::ir::Opcode::IshlImm), offset: 0x000087 },
    Level2Entry { opcode: Some(crate::ir::Opcode::UshrImm), offset: 0x00009b },
    Level2Entry { opcode: Some(crate::ir::Opcode::SshrImm), offset: 0x000093 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Iconst), offset: 0x00003e },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Iadd), offset: 0x00007a },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    // 0000ba: typeless, 4 entries
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Call), offset: 0x00005a },
    Level2Entry { opcode: Some(crate::ir::Opcode::Jump), offset: 0x00005c },
    Level2Entry { opcode: Some(crate::ir::Opcode::Return), offset: 0x00005e },
    // 0000be: b1, 8 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyToSsa), offset: 0x000028 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x000024 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Brz), offset: 0x00001c },
    Level2Entry { opcode: Some(crate::ir::Opcode::Brnz), offset: 0x00001a },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x00002c },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x00004c },
    Level2Entry { opcode: None, offset: 0 },
    // 0000c6: i16, 2 entries
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000026 },
    // 0000c8: i8, 2 entries
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000026 },
    // 0000ca: f64, 2 entries
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000026 },
    // 0000cc: f32, 2 entries
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000026 },
    // 0000ce: r64, 2 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyToSsa), offset: 0x000028 },
    Level2Entry { opcode: None, offset: 0 },
];

/// riscv level 1 hash table for the CPU mode RV32.
///
/// This hash table, keyed by instruction controlling type, contains all the level 2
/// hash-tables offsets for the given CPU mode, as well as a legalization identifier indicating
/// which legalization scheme to apply when the instruction doesn't have any valid encoding for
/// this CPU mode.
pub static LEVEL1_RV32: [Level1Entry<u16>; 16] = [
    Level1Entry { ty: ir::types::INVALID, log2len: 2, offset: 0x000040, legalize: 0 }, // expand
    Level1Entry { ty: ir::types::B1, log2len: 3, offset: 0x000044, legalize: 1 }, // narrow_no_flags
    Level1Entry { ty: ir::types::INVALID, log2len: !0, offset: 0, legalize: 1 },
    Level1Entry { ty: ir::types::INVALID, log2len: !0, offset: 0, legalize: 1 },
    Level1Entry { ty: ir::types::INVALID, log2len: !0, offset: 0, legalize: 1 },
    Level1Entry { ty: ir::types::INVALID, log2len: !0, offset: 0, legalize: 1 },
    Level1Entry { ty: ir::types::I8, log2len: 1, offset: 0x000052, legalize: 1 }, // narrow_no_flags
    Level1Entry { ty: ir::types::I16, log2len: 1, offset: 0x000050, legalize: 1 }, // narrow_no_flags
    Level1Entry { ty: ir::types::I32, log2len: 6, offset: 0x000000, legalize: 0 }, // expand
    Level1Entry { ty: ir::types::I64, log2len: 2, offset: 0x00004c, legalize: 1 }, // narrow_no_flags
    Level1Entry { ty: ir::types::INVALID, log2len: !0, offset: 0, legalize: 1 },
    Level1Entry { ty: ir::types::F32, log2len: 1, offset: 0x000056, legalize: 0 }, // expand
    Level1Entry { ty: ir::types::F64, log2len: 1, offset: 0x000054, legalize: 0 }, // expand
    Level1Entry { ty: ir::types::INVALID, log2len: !0, offset: 0, legalize: 1 },
    Level1Entry { ty: ir::types::R32, log2len: 1, offset: 0x000058, legalize: 1 }, // narrow_no_flags
    Level1Entry { ty: ir::types::INVALID, log2len: !0, offset: 0, legalize: 1 },
];

/// riscv level 1 hash table for the CPU mode RV64.
///
/// This hash table, keyed by instruction controlling type, contains all the level 2
/// hash-tables offsets for the given CPU mode, as well as a legalization identifier indicating
/// which legalization scheme to apply when the instruction doesn't have any valid encoding for
/// this CPU mode.
pub static LEVEL1_RV64: [Level1Entry<u16>; 16] = [
    Level1Entry { ty: ir::types::INVALID, log2len: 2, offset: 0x0000ba, legalize: 0 }, // expand
    Level1Entry { ty: ir::types::B1, log2len: 3, offset: 0x0000be, legalize: 1 }, // narrow_no_flags
    Level1Entry { ty: ir::types::INVALID, log2len: !0, offset: 0, legalize: 1 },
    Level1Entry { ty: ir::types::INVALID, log2len: !0, offset: 0, legalize: 1 },
    Level1Entry { ty: ir::types::INVALID, log2len: !0, offset: 0, legalize: 1 },
    Level1Entry { ty: ir::types::INVALID, log2len: !0, offset: 0, legalize: 1 },
    Level1Entry { ty: ir::types::I8, log2len: 1, offset: 0x0000c8, legalize: 1 }, // narrow_no_flags
    Level1Entry { ty: ir::types::I16, log2len: 1, offset: 0x0000c6, legalize: 1 }, // narrow_no_flags
    Level1Entry { ty: ir::types::I32, log2len: 5, offset: 0x00009a, legalize: 0 }, // expand
    Level1Entry { ty: ir::types::I64, log2len: 6, offset: 0x00005a, legalize: 0 }, // expand
    Level1Entry { ty: ir::types::INVALID, log2len: !0, offset: 0, legalize: 1 },
    Level1Entry { ty: ir::types::F32, log2len: 1, offset: 0x0000cc, legalize: 0 }, // expand
    Level1Entry { ty: ir::types::F64, log2len: 1, offset: 0x0000ca, legalize: 0 }, // expand
    Level1Entry { ty: ir::types::INVALID, log2len: !0, offset: 0, legalize: 1 },
    Level1Entry { ty: ir::types::INVALID, log2len: !0, offset: 0, legalize: 1 },
    Level1Entry { ty: ir::types::R64, log2len: 1, offset: 0x0000ce, legalize: 1 }, // narrow_no_flags
];

/// riscv recipe names, using the same recipe index spaces as the one specified by the
/// corresponding binemit file.
static RECIPE_NAMES: [&str; 20] = [
    "R",
    "Rshamt",
    "Ricmp",
    "Ii",
    "Iz",
    "Iicmp",
    "Iret",
    "Icall",
    "Icopy",
    "Irmov",
    "copytossa",
    "U",
    "UJ",
    "UJcall",
    "SB",
    "SBzero",
    "GPsp",
    "GPfi",
    "stacknull",
    "fillnull",
];

/// riscv recipe constraints list, using the same recipe index spaces as the one
/// specified by the corresponding binemit file. These constraints are used by register
/// allocation to select the right location to use for input and output values.
static RECIPE_CONSTRAINTS: [RecipeConstraints; 20] = [
    // Constraints for recipe R:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Rshamt:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Ricmp:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Ii:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Iz:
    RecipeConstraints {
        ins: &[],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Iicmp:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Iret:
    RecipeConstraints {
        ins: &[],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Icall:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Icopy:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Irmov:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe copytossa:
    RecipeConstraints {
        ins: &[],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe U:
    RecipeConstraints {
        ins: &[],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe UJ:
    RecipeConstraints {
        ins: &[],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe UJcall:
    RecipeConstraints {
        ins: &[],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe SB:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe SBzero:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe GPsp:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Stack,
                regclass: &GPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe GPfi:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Stack,
                regclass: &GPR_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe stacknull:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Stack,
                regclass: &GPR_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Stack,
                regclass: &GPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe fillnull:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Stack,
                regclass: &GPR_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
];

/// riscv recipe sizing descriptors, using the same recipe index spaces as the one
/// specified by the corresponding binemit file. These are used to compute the final size of an
/// instruction, as well as to compute the range of branches.
static RECIPE_SIZING: [RecipeSizing; 20] = [
    // Code size information for recipe R:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Rshamt:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Ricmp:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Ii:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Iz:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Iicmp:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Iret:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Icall:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Icopy:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Irmov:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe copytossa:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe U:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe UJ:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: Some(BranchRange { origin: 0, bits: 21 }),
    },
    // Code size information for recipe UJcall:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe SB:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: Some(BranchRange { origin: 0, bits: 13 }),
    },
    // Code size information for recipe SBzero:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: Some(BranchRange { origin: 0, bits: 13 }),
    },
    // Code size information for recipe GPsp:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe GPfi:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe stacknull:
    RecipeSizing {
        base_size: 0,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe fillnull:
    RecipeSizing {
        base_size: 0,
        compute_size: base_size,
        branch_range: None,
    },
];

pub static INFO: isa::EncInfo = isa::EncInfo {
    constraints: &RECIPE_CONSTRAINTS,
    sizing: &RECIPE_SIZING,
    names: &RECIPE_NAMES,
};

 //clude!(concat!(env!("OUT_DIR"), "/encoding-riscv.rs"));
 
pub static LEGALIZE_ACTIONS: [isa::Legalize; 2] = [
    crate::legalizer::expand,
    crate::legalizer::narrow_no_flags,
];

 //clude!(concat!(env!("OUT_DIR"), "/legalize-riscv.rs"));
