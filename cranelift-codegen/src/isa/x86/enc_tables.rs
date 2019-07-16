//! Encoding tables for x86 ISAs.

use super::registers::*;
use crate::bitset::BitSet;
use crate::cursor::{Cursor, FuncCursor};
use crate::flowgraph::ControlFlowGraph;
use crate::ir::condcodes::{FloatCC, IntCC};
use crate::ir::{self, Function, Inst, InstBuilder};
use crate::isa::constraints::*;
use crate::isa::enc_tables::*;
use crate::isa::encoding::base_size;
use crate::isa::encoding::RecipeSizing;
use crate::isa::RegUnit;
use crate::isa::{self, TargetIsa};
use crate::predicates;
use crate::regalloc::RegDiversions;

fn recipe_predicate_op1r_ib(_: crate::settings::PredicateView, inst: &ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::BinaryImm { imm, .. } = *inst {
        return predicates::is_signed_int(imm, 8, 0);
    }
    unreachable!();
}
fn recipe_predicate_op1r_id(_: crate::settings::PredicateView, inst: &ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::BinaryImm { imm, .. } = *inst {
        return predicates::is_signed_int(imm, 32, 0);
    }
    unreachable!();
}
fn recipe_predicate_rexop1u_id(
    _: crate::settings::PredicateView,
    inst: &ir::InstructionData,
) -> bool {
    if let crate::ir::InstructionData::UnaryImm { imm, .. } = *inst {
        return predicates::is_signed_int(imm, 32, 0);
    }
    unreachable!();
}
fn recipe_predicate_op1ldwithindex(
    _: crate::settings::PredicateView,
    inst: &ir::InstructionData,
) -> bool {
    if let crate::ir::InstructionData::LoadComplex { offset, .. } = *inst {
        return predicates::is_equal(offset, 0);
    }
    unreachable!();
}
fn recipe_predicate_op1ldwithindexdisp8(
    _: crate::settings::PredicateView,
    inst: &ir::InstructionData,
) -> bool {
    if let crate::ir::InstructionData::LoadComplex { offset, .. } = *inst {
        return predicates::is_signed_int(offset, 8, 0);
    }
    unreachable!();
}
fn recipe_predicate_op1ldwithindexdisp32(
    _: crate::settings::PredicateView,
    inst: &ir::InstructionData,
) -> bool {
    if let crate::ir::InstructionData::LoadComplex { offset, .. } = *inst {
        return predicates::is_signed_int(offset, 32, 0);
    }
    unreachable!();
}
fn recipe_predicate_op1stwithindex(
    _: crate::settings::PredicateView,
    inst: &ir::InstructionData,
) -> bool {
    if let crate::ir::InstructionData::StoreComplex { offset, .. } = *inst {
        return predicates::is_equal(offset, 0);
    }
    unreachable!();
}
fn recipe_predicate_op1stwithindexdisp8(
    _: crate::settings::PredicateView,
    inst: &ir::InstructionData,
) -> bool {
    if let crate::ir::InstructionData::StoreComplex { offset, .. } = *inst {
        return predicates::is_signed_int(offset, 8, 0);
    }
    unreachable!();
}
fn recipe_predicate_op1stwithindexdisp32(
    _: crate::settings::PredicateView,
    inst: &ir::InstructionData,
) -> bool {
    if let crate::ir::InstructionData::StoreComplex { offset, .. } = *inst {
        return predicates::is_signed_int(offset, 32, 0);
    }
    unreachable!();
}
fn recipe_predicate_op1st(_: crate::settings::PredicateView, inst: &ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::Store { offset, .. } = *inst {
        return predicates::is_equal(offset, 0);
    }
    unreachable!();
}
fn recipe_predicate_op1stdisp8(
    _: crate::settings::PredicateView,
    inst: &ir::InstructionData,
) -> bool {
    if let crate::ir::InstructionData::Store { offset, .. } = *inst {
        return predicates::is_signed_int(offset, 8, 0);
    }
    unreachable!();
}
fn recipe_predicate_op1ld(_: crate::settings::PredicateView, inst: &ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::Load { offset, .. } = *inst {
        return predicates::is_equal(offset, 0);
    }
    unreachable!();
}
fn recipe_predicate_op1lddisp8(
    _: crate::settings::PredicateView,
    inst: &ir::InstructionData,
) -> bool {
    if let crate::ir::InstructionData::Load { offset, .. } = *inst {
        return predicates::is_signed_int(offset, 8, 0);
    }
    unreachable!();
}
fn recipe_predicate_op1lddisp32(
    _: crate::settings::PredicateView,
    inst: &ir::InstructionData,
) -> bool {
    if let crate::ir::InstructionData::Load { offset, .. } = *inst {
        return predicates::is_signed_int(offset, 32, 0);
    }
    unreachable!();
}
fn recipe_predicate_op1adjustsp_ib(
    _: crate::settings::PredicateView,
    inst: &ir::InstructionData,
) -> bool {
    if let crate::ir::InstructionData::UnaryImm { imm, .. } = *inst {
        return predicates::is_signed_int(imm, 8, 0);
    }
    unreachable!();
}
fn recipe_predicate_op1brfb(_: crate::settings::PredicateView, inst: &ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::BranchFloat { cond, .. } = *inst {
        return predicates::is_equal(cond, ir::condcodes::FloatCC::Ordered)
            || predicates::is_equal(cond, ir::condcodes::FloatCC::Unordered)
            || predicates::is_equal(cond, ir::condcodes::FloatCC::OrderedNotEqual)
            || predicates::is_equal(cond, ir::condcodes::FloatCC::UnorderedOrEqual)
            || predicates::is_equal(cond, ir::condcodes::FloatCC::GreaterThan)
            || predicates::is_equal(cond, ir::condcodes::FloatCC::GreaterThanOrEqual)
            || predicates::is_equal(cond, ir::condcodes::FloatCC::UnorderedOrLessThan)
            || predicates::is_equal(cond, ir::condcodes::FloatCC::UnorderedOrLessThanOrEqual);
    }
    unreachable!();
}
fn recipe_predicate_rexop1jt_entry(
    _: crate::settings::PredicateView,
    inst: &ir::InstructionData,
) -> bool {
    if let crate::ir::InstructionData::BranchTableEntry { imm, .. } = *inst {
        return predicates::is_equal(imm, 1)
            || predicates::is_equal(imm, 2)
            || predicates::is_equal(imm, 4)
            || predicates::is_equal(imm, 8);
    }
    unreachable!();
}
fn recipe_predicate_trapff(_: crate::settings::PredicateView, inst: &ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::FloatCondTrap { cond, .. } = *inst {
        return predicates::is_equal(cond, ir::condcodes::FloatCC::Ordered)
            || predicates::is_equal(cond, ir::condcodes::FloatCC::Unordered)
            || predicates::is_equal(cond, ir::condcodes::FloatCC::OrderedNotEqual)
            || predicates::is_equal(cond, ir::condcodes::FloatCC::UnorderedOrEqual)
            || predicates::is_equal(cond, ir::condcodes::FloatCC::GreaterThan)
            || predicates::is_equal(cond, ir::condcodes::FloatCC::GreaterThanOrEqual)
            || predicates::is_equal(cond, ir::condcodes::FloatCC::UnorderedOrLessThan)
            || predicates::is_equal(cond, ir::condcodes::FloatCC::UnorderedOrLessThanOrEqual);
    }
    unreachable!();
}
fn recipe_predicate_op1icscc_ib(
    _: crate::settings::PredicateView,
    inst: &ir::InstructionData,
) -> bool {
    if let crate::ir::InstructionData::IntCompareImm { imm, .. } = *inst {
        return predicates::is_signed_int(imm, 8, 0);
    }
    unreachable!();
}
fn recipe_predicate_op1icscc_id(
    _: crate::settings::PredicateView,
    inst: &ir::InstructionData,
) -> bool {
    if let crate::ir::InstructionData::IntCompareImm { imm, .. } = *inst {
        return predicates::is_signed_int(imm, 32, 0);
    }
    unreachable!();
}
fn recipe_predicate_op2f32imm_z(
    _: crate::settings::PredicateView,
    inst: &ir::InstructionData,
) -> bool {
    if let crate::ir::InstructionData::UnaryIeee32 { imm, .. } = *inst {
        return predicates::is_zero_32_bit_float(imm);
    }
    unreachable!();
}
fn recipe_predicate_mp2f64imm_z(
    _: crate::settings::PredicateView,
    inst: &ir::InstructionData,
) -> bool {
    if let crate::ir::InstructionData::UnaryIeee64 { imm, .. } = *inst {
        return predicates::is_zero_64_bit_float(imm);
    }
    unreachable!();
}
fn recipe_predicate_mp3furmi_rnd(
    isap: crate::settings::PredicateView,
    _: &ir::InstructionData,
) -> bool {
    isap.test(16)
}
fn recipe_predicate_op2fcscc(
    _: crate::settings::PredicateView,
    inst: &ir::InstructionData,
) -> bool {
    if let crate::ir::InstructionData::FloatCompare { cond, .. } = *inst {
        return predicates::is_equal(cond, ir::condcodes::FloatCC::Ordered)
            || predicates::is_equal(cond, ir::condcodes::FloatCC::Unordered)
            || predicates::is_equal(cond, ir::condcodes::FloatCC::OrderedNotEqual)
            || predicates::is_equal(cond, ir::condcodes::FloatCC::UnorderedOrEqual)
            || predicates::is_equal(cond, ir::condcodes::FloatCC::GreaterThan)
            || predicates::is_equal(cond, ir::condcodes::FloatCC::GreaterThanOrEqual)
            || predicates::is_equal(cond, ir::condcodes::FloatCC::UnorderedOrLessThan)
            || predicates::is_equal(cond, ir::condcodes::FloatCC::UnorderedOrLessThanOrEqual);
    }
    unreachable!();
}
pub static RECIPE_PREDICATES: [RecipePredicate; 243] = [
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(recipe_predicate_op1r_ib),
    Some(recipe_predicate_op1r_ib),
    Some(recipe_predicate_op1r_id),
    Some(recipe_predicate_op1r_id),
    None,
    None,
    Some(recipe_predicate_rexop1u_id),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(recipe_predicate_op1ldwithindex),
    Some(recipe_predicate_op1ldwithindex),
    Some(recipe_predicate_op1ldwithindex),
    Some(recipe_predicate_op1ldwithindex),
    Some(recipe_predicate_op1ldwithindexdisp8),
    Some(recipe_predicate_op1ldwithindexdisp8),
    Some(recipe_predicate_op1ldwithindexdisp8),
    Some(recipe_predicate_op1ldwithindexdisp8),
    Some(recipe_predicate_op1ldwithindexdisp32),
    Some(recipe_predicate_op1ldwithindexdisp32),
    Some(recipe_predicate_op1ldwithindexdisp32),
    Some(recipe_predicate_op1ldwithindexdisp32),
    Some(recipe_predicate_op1stwithindex),
    Some(recipe_predicate_op1stwithindex),
    Some(recipe_predicate_op1stwithindex),
    Some(recipe_predicate_op1stwithindex),
    Some(recipe_predicate_op1stwithindexdisp8),
    Some(recipe_predicate_op1stwithindexdisp8),
    Some(recipe_predicate_op1stwithindexdisp8),
    Some(recipe_predicate_op1stwithindexdisp8),
    Some(recipe_predicate_op1stwithindexdisp32),
    Some(recipe_predicate_op1stwithindexdisp32),
    Some(recipe_predicate_op1stwithindexdisp32),
    Some(recipe_predicate_op1stwithindexdisp32),
    Some(recipe_predicate_op1stwithindex),
    Some(recipe_predicate_op1stwithindex),
    Some(recipe_predicate_op1stwithindexdisp8),
    Some(recipe_predicate_op1stwithindexdisp8),
    Some(recipe_predicate_op1stwithindexdisp32),
    Some(recipe_predicate_op1stwithindexdisp32),
    Some(recipe_predicate_op1st),
    Some(recipe_predicate_op1st),
    Some(recipe_predicate_op1st),
    Some(recipe_predicate_op1st),
    Some(recipe_predicate_op1stdisp8),
    Some(recipe_predicate_op1stdisp8),
    Some(recipe_predicate_op1stdisp8),
    Some(recipe_predicate_op1stdisp8),
    None,
    None,
    None,
    None,
    Some(recipe_predicate_op1st),
    Some(recipe_predicate_op1stdisp8),
    None,
    None,
    None,
    None,
    None,
    Some(recipe_predicate_op1ld),
    Some(recipe_predicate_op1ld),
    Some(recipe_predicate_op1ld),
    Some(recipe_predicate_op1ld),
    Some(recipe_predicate_op1lddisp8),
    Some(recipe_predicate_op1lddisp8),
    Some(recipe_predicate_op1lddisp8),
    Some(recipe_predicate_op1lddisp8),
    Some(recipe_predicate_op1lddisp32),
    Some(recipe_predicate_op1lddisp32),
    Some(recipe_predicate_op1lddisp32),
    Some(recipe_predicate_op1lddisp32),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(recipe_predicate_op1adjustsp_ib),
    Some(recipe_predicate_rexop1u_id),
    Some(recipe_predicate_op1adjustsp_ib),
    Some(recipe_predicate_rexop1u_id),
    Some(recipe_predicate_op1ld),
    Some(recipe_predicate_op1ld),
    Some(recipe_predicate_op1lddisp8),
    Some(recipe_predicate_op1lddisp8),
    Some(recipe_predicate_op1lddisp32),
    Some(recipe_predicate_op1lddisp32),
    Some(recipe_predicate_op1ldwithindex),
    Some(recipe_predicate_op1ldwithindex),
    Some(recipe_predicate_op1ldwithindexdisp8),
    Some(recipe_predicate_op1ldwithindexdisp8),
    Some(recipe_predicate_op1ldwithindexdisp32),
    Some(recipe_predicate_op1ldwithindexdisp32),
    Some(recipe_predicate_op1st),
    Some(recipe_predicate_op1st),
    Some(recipe_predicate_op1stdisp8),
    Some(recipe_predicate_op1stdisp8),
    None,
    None,
    Some(recipe_predicate_op1stwithindex),
    Some(recipe_predicate_op1stwithindex),
    Some(recipe_predicate_op1stwithindexdisp8),
    Some(recipe_predicate_op1stwithindexdisp8),
    Some(recipe_predicate_op1stwithindexdisp32),
    Some(recipe_predicate_op1stwithindexdisp32),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(recipe_predicate_op1brfb),
    Some(recipe_predicate_op1brfb),
    Some(recipe_predicate_op1brfb),
    Some(recipe_predicate_op1brfb),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(recipe_predicate_rexop1jt_entry),
    Some(recipe_predicate_rexop1jt_entry),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(recipe_predicate_trapff),
    None,
    None,
    Some(recipe_predicate_op1icscc_ib),
    Some(recipe_predicate_op1icscc_ib),
    Some(recipe_predicate_op1icscc_id),
    Some(recipe_predicate_op1icscc_id),
    None,
    None,
    Some(recipe_predicate_op1r_ib),
    Some(recipe_predicate_op1r_ib),
    Some(recipe_predicate_op1r_id),
    Some(recipe_predicate_op1r_id),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(recipe_predicate_op2f32imm_z),
    Some(recipe_predicate_mp2f64imm_z),
    Some(recipe_predicate_op2f32imm_z),
    Some(recipe_predicate_mp2f64imm_z),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(recipe_predicate_mp3furmi_rnd),
    Some(recipe_predicate_mp3furmi_rnd),
    None,
    None,
    None,
    None,
    None,
    None,
    Some(recipe_predicate_op2fcscc),
    Some(recipe_predicate_op2fcscc),
    Some(recipe_predicate_op2fcscc),
    Some(recipe_predicate_op2fcscc),
    None,
    None,
    None,
    None,
];
fn inst_predicate_0(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::UnaryImm { imm, .. } = *inst {
        let _ = func;
        return predicates::is_unsigned_int(imm, 32, 0);
    }
    unreachable!();
}
fn inst_predicate_1(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::LoadComplex { ref args, .. } = *inst {
        let _ = func;
        return predicates::has_length_of(args, 2, func);
    }
    unreachable!();
}
fn inst_predicate_2(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::StoreComplex { ref args, .. } = *inst {
        let _ = func;
        return predicates::has_length_of(args, 3, func);
    }
    unreachable!();
}
fn inst_predicate_3(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::FuncAddr { func_ref, .. } = *inst {
        let _ = func;
        return predicates::is_colocated_func(func_ref, func);
    }
    unreachable!();
}
fn inst_predicate_4(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::UnaryGlobalValue { global_value, .. } = *inst {
        let _ = func;
        return predicates::is_colocated_data(global_value, func);
    }
    unreachable!();
}
fn inst_predicate_5(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::Call { func_ref, .. } = *inst {
        let _ = func;
        return predicates::is_colocated_func(func_ref, func);
    }
    unreachable!();
}
fn inst_predicate_6(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    let args = inst.arguments(&func.dfg.value_lists);
    func.dfg.value_type(args[0]) == ir::types::B1
}
fn inst_predicate_7(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    let args = inst.arguments(&func.dfg.value_lists);
    func.dfg.value_type(args[0]) == ir::types::I16
}
fn inst_predicate_8(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    let args = inst.arguments(&func.dfg.value_lists);
    func.dfg.value_type(args[0]) == ir::types::I32
}
fn inst_predicate_9(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    let args = inst.arguments(&func.dfg.value_lists);
    func.dfg.value_type(args[0]) == ir::types::I64
}
fn inst_predicate_10(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    let args = inst.arguments(&func.dfg.value_lists);
    func.dfg.value_type(args[0]) == ir::types::I8
}
fn inst_predicate_11(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::UnaryIeee32 { imm, .. } = *inst {
        let _ = func;
        return predicates::is_zero_32_bit_float(imm);
    }
    unreachable!();
}
fn inst_predicate_12(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::UnaryIeee64 { imm, .. } = *inst {
        let _ = func;
        return predicates::is_zero_64_bit_float(imm);
    }
    unreachable!();
}
fn inst_predicate_13(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    let args = inst.arguments(&func.dfg.value_lists);
    func.dfg.value_type(args[0]) == ir::types::F32
}
fn inst_predicate_14(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    let args = inst.arguments(&func.dfg.value_lists);
    func.dfg.value_type(args[0]) == ir::types::F64
}
pub static INST_PREDICATES: [InstPredicate; 15] = [
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
    inst_predicate_12,
    inst_predicate_13,
    inst_predicate_14,
];
pub static ENCLISTS: [u16; 1416] = [
    // 000000: band.i32 (I64)
    // --> [RexOp1rr#21]
    // 000000: band.b1 (I64)
    // --> [RexOp1rr#21]
    0x0002, 0x0021,
    // --> [Op1rr#21] and stop
    // --> [Op1rr#21] and stop
    // 000002: band.i32 (I32)
    // --> [Op1rr#21] and stop
    // 000002: band.b1 (I32)
    // --> [Op1rr#21] and stop
    0x0001, 0x0021,
    // end of band.b1 (I32)
    // end of band.i32 (I32)
    // end of band.b1 (I64)
    // end of band.i32 (I64)
    // 000004: band_imm.i32 (I64)
    // --> [RexOp1r_ib#4083]
    0x001e, 0x4083, // --> [Op1r_ib#4083]
    0x001c, 0x4083, // --> [RexOp1r_id#4081]
    0x0022, 0x4081, // --> [Op1r_id#4081] and stop
    0x0021, 0x4081,
    // end of band_imm.i32 (I64)
    // 00000c: bint.i32 (I64)
    // stop unless inst_predicate_6
    // 00000c: bint.i64 (I64)
    // stop unless inst_predicate_6
    0x1006, // --> [RexOp2urm_noflags#4b6]
    // --> [RexOp2urm_noflags#4b6]
    0x019e, 0x04b6,
    // --> [Op2urm_noflags_abcd#4b6] and stop
    // --> [Op2urm_noflags_abcd#4b6] and stop
    0x019d, 0x04b6,
    // end of bint.i64 (I64)
    // end of bint.i32 (I64)
    // 000011: bitcast.i32 (I64)
    // stop unless inst_predicate_13
    0x100d, // --> [RexMp2rfumr#57e]
    0x01b4, 0x057e, // --> [Mp2rfumr#57e] and stop
    0x01b3, 0x057e,
    // end of bitcast.i32 (I64)
    // 000016: bnot.i32 (I64)
    // --> [RexOp1ur#20f7]
    0x0006, 0x20f7,
    // --> [Op1ur#20f7] and stop
    // 000018: bnot.i32 (I32)
    // --> [Op1ur#20f7] and stop
    0x0005, 0x20f7,
    // end of bnot.i32 (I32)
    // end of bnot.i32 (I64)
    // 00001a: bor.i32 (I64)
    // --> [RexOp1rr#09]
    // 00001a: bor.b1 (I64)
    // --> [RexOp1rr#09]
    0x0002, 0x0009,
    // --> [Op1rr#09] and stop
    // --> [Op1rr#09] and stop
    // 00001c: bor.i32 (I32)
    // --> [Op1rr#09] and stop
    // 00001c: bor.b1 (I32)
    // --> [Op1rr#09] and stop
    0x0001, 0x0009,
    // end of bor.b1 (I32)
    // end of bor.i32 (I32)
    // end of bor.b1 (I64)
    // end of bor.i32 (I64)
    // 00001e: bor_imm.i32 (I64)
    // --> [RexOp1r_ib#1083]
    0x001e, 0x1083, // --> [Op1r_ib#1083]
    0x001c, 0x1083, // --> [RexOp1r_id#1081]
    0x0022, 0x1081, // --> [Op1r_id#1081] and stop
    0x0021, 0x1081,
    // end of bor_imm.i32 (I64)
    // 000026: brnz.i32 (I64)
    // --> [RexOp1tjccb#75]
    0x014c, 0x0075, // --> [Op1tjccb#75]
    0x014a, 0x0075, // --> [RexOp1tjccd#85]
    0x0150, 0x0085, // --> [Op1tjccd#85] and stop
    0x014f, 0x0085,
    // end of brnz.i32 (I64)
    // 00002e: brz.i32 (I64)
    // --> [RexOp1tjccb#74]
    0x014c, 0x0074, // --> [Op1tjccb#74]
    0x014a, 0x0074, // --> [RexOp1tjccd#84]
    0x0150, 0x0084, // --> [Op1tjccd#84] and stop
    0x014f, 0x0084,
    // end of brz.i32 (I64)
    // 000036: bxor.i32 (I64)
    // --> [RexOp1rr#31]
    // 000036: bxor.b1 (I64)
    // --> [RexOp1rr#31]
    0x0002, 0x0031,
    // --> [Op1rr#31] and stop
    // --> [Op1rr#31] and stop
    // 000038: bxor.i32 (I32)
    // --> [Op1rr#31] and stop
    // 000038: bxor.b1 (I32)
    // --> [Op1rr#31] and stop
    0x0001, 0x0031,
    // end of bxor.b1 (I32)
    // end of bxor.i32 (I32)
    // end of bxor.b1 (I64)
    // end of bxor.i32 (I64)
    // 00003a: bxor_imm.i32 (I64)
    // --> [RexOp1r_ib#6083]
    0x001e, 0x6083, // --> [Op1r_ib#6083]
    0x001c, 0x6083, // --> [RexOp1r_id#6081]
    0x0022, 0x6081, // --> [Op1r_id#6081] and stop
    0x0021, 0x6081,
    // end of bxor_imm.i32 (I64)
    // 000042: clz.i32 (I64)
    // stop unless PredicateView(14)
    0x101d, // --> [RexMp2urm#6bd]
    0x0036, 0x06bd, // --> [Mp2urm#6bd] and stop
    0x0035, 0x06bd,
    // end of clz.i32 (I64)
    // 000047: copy.i32 (I64)
    // --> [RexOp1umr#89]
    // 000047: copy.b1 (I64)
    // --> [RexOp1umr#89]
    // 000047: copy.i8 (I64)
    // --> [RexOp1umr#89]
    // 000047: copy.i16 (I64)
    // --> [RexOp1umr#89]
    0x0016, 0x0089,
    // --> [Op1umr#89] and stop
    // --> [Op1umr#89] and stop
    // --> [Op1umr#89] and stop
    // --> [Op1umr#89] and stop
    // 000049: copy.i32 (I32)
    // --> [Op1umr#89] and stop
    // 000049: copy.b1 (I32)
    // --> [Op1umr#89] and stop
    // 000049: copy.i8 (I32)
    // --> [Op1umr#89] and stop
    // 000049: copy.i16 (I32)
    // --> [Op1umr#89] and stop
    0x0015, 0x0089,
    // end of copy.i16 (I32)
    // end of copy.i8 (I32)
    // end of copy.b1 (I32)
    // end of copy.i32 (I32)
    // end of copy.i16 (I64)
    // end of copy.i8 (I64)
    // end of copy.b1 (I64)
    // end of copy.i32 (I64)
    // 00004b: copy_nop.i32 (I64)
    // --> [stacknull#00] and stop
    // 00004b: copy_nop.i64 (I64)
    // --> [stacknull#00] and stop
    // 00004b: copy_nop.i8 (I64)
    // --> [stacknull#00] and stop
    // 00004b: copy_nop.i16 (I64)
    // --> [stacknull#00] and stop
    // 00004b: copy_nop.f64 (I64)
    // --> [stacknull#00] and stop
    // 00004b: copy_nop.f32 (I64)
    // --> [stacknull#00] and stop
    // 00004b: copy_nop.i32 (I32)
    // --> [stacknull#00] and stop
    // 00004b: copy_nop.i8 (I32)
    // --> [stacknull#00] and stop
    // 00004b: copy_nop.i16 (I32)
    // --> [stacknull#00] and stop
    // 00004b: copy_nop.i64 (I32)
    // --> [stacknull#00] and stop
    // 00004b: copy_nop.f64 (I32)
    // --> [stacknull#00] and stop
    // 00004b: copy_nop.f32 (I32)
    // --> [stacknull#00] and stop
    0x00c7, 0x0000,
    // end of copy_nop.f32 (I32)
    // end of copy_nop.f64 (I32)
    // end of copy_nop.i64 (I32)
    // end of copy_nop.i16 (I32)
    // end of copy_nop.i8 (I32)
    // end of copy_nop.i32 (I32)
    // end of copy_nop.f32 (I64)
    // end of copy_nop.f64 (I64)
    // end of copy_nop.i16 (I64)
    // end of copy_nop.i8 (I64)
    // end of copy_nop.i64 (I64)
    // end of copy_nop.i32 (I64)
    // 00004d: ctz.i32 (I64)
    // stop unless PredicateView(13)
    0x101c, // --> [RexMp2urm#6bc]
    0x0036, 0x06bc, // --> [Mp2urm#6bc] and stop
    0x0035, 0x06bc,
    // end of ctz.i32 (I64)
    // 000052: fill.i32 (I64)
    // --> [RexOp1fillSib32#8b]
    // 000052: fill.b1 (I64)
    // --> [RexOp1fillSib32#8b]
    // 000052: fill.i8 (I64)
    // --> [RexOp1fillSib32#8b]
    // 000052: fill.i16 (I64)
    // --> [RexOp1fillSib32#8b]
    0x00b4, 0x008b,
    // --> [Op1fillSib32#8b] and stop
    // --> [Op1fillSib32#8b] and stop
    // --> [Op1fillSib32#8b] and stop
    // --> [Op1fillSib32#8b] and stop
    // 000054: fill.i32 (I32)
    // --> [Op1fillSib32#8b] and stop
    // 000054: fill.b1 (I32)
    // --> [Op1fillSib32#8b] and stop
    // 000054: fill.i8 (I32)
    // --> [Op1fillSib32#8b] and stop
    // 000054: fill.i16 (I32)
    // --> [Op1fillSib32#8b] and stop
    0x00b3, 0x008b,
    // end of fill.i16 (I32)
    // end of fill.i8 (I32)
    // end of fill.b1 (I32)
    // end of fill.i32 (I32)
    // end of fill.i16 (I64)
    // end of fill.i8 (I64)
    // end of fill.b1 (I64)
    // end of fill.i32 (I64)
    // 000056: iadd.i32 (I64)
    // --> [RexOp1rr#01]
    0x0002, // 000057: iadd.i32 (I32)
    // --> [Op1rr#01] and stop
    0x0001, // --> [Op1rr#01] and stop
    0x0001, // end of iadd.i32 (I32)
    0x0001,
    // end of iadd.i32 (I64)
    // 00005a: iadd_imm.i32 (I64)
    // --> [RexOp1r_ib#83]
    0x001e, 0x0083, // --> [Op1r_ib#83]
    0x001c, 0x0083, // --> [RexOp1r_id#81]
    0x0022, 0x0081, // --> [Op1r_id#81] and stop
    0x0021, 0x0081,
    // end of iadd_imm.i32 (I64)
    // 000062: icmp.i32 (I64)
    // --> [RexOp1icscc#39]
    0x0172, 0x0039,
    // --> [Op1icscc#39] and stop
    // 000064: icmp.i32 (I32)
    // --> [Op1icscc#39] and stop
    0x0171, 0x0039,
    // end of icmp.i32 (I32)
    // end of icmp.i32 (I64)
    // 000066: icmp_imm.i32 (I64)
    // --> [RexOp1icscc_ib#7083]
    0x0176, 0x7083, // --> [Op1icscc_ib#7083]
    0x0174, 0x7083, // --> [RexOp1icscc_id#7081]
    0x017a, 0x7081, // --> [Op1icscc_id#7081] and stop
    0x0179, 0x7081,
    // end of icmp_imm.i32 (I64)
    // 00006e: iconst.i32 (I64)
    // --> [RexOp1pu_id#b8]
    0x0026, 0x00b8,
    // --> [Op1pu_id#b8] and stop
    // 000070: iconst.i32 (I32)
    // --> [Op1pu_id#b8] and stop
    0x0025, 0x00b8,
    // end of iconst.i32 (I32)
    // end of iconst.i32 (I64)
    // 000072: ifcmp.i32 (I64)
    // --> [RexOp1rcmp#39]
    0x017e, 0x0039,
    // --> [Op1rcmp#39] and stop
    // 000074: ifcmp.i32 (I32)
    // --> [Op1rcmp#39] and stop
    0x017d, 0x0039,
    // end of ifcmp.i32 (I32)
    // end of ifcmp.i32 (I64)
    // 000076: ifcmp_imm.i32 (I64)
    // --> [RexOp1rcmp_ib#7083]
    0x0182, 0x7083, // --> [Op1rcmp_ib#7083]
    0x0180, 0x7083, // --> [RexOp1rcmp_id#7081]
    0x0186, 0x7081, // --> [Op1rcmp_id#7081] and stop
    0x0185, 0x7081,
    // end of ifcmp_imm.i32 (I64)
    // 00007e: imul.i32 (I64)
    // --> [RexOp2rrx#4af]
    0x000a, 0x04af,
    // --> [Op2rrx#4af] and stop
    // 000080: imul.i32 (I32)
    // --> [Op2rrx#4af] and stop
    0x0009, 0x04af,
    // end of imul.i32 (I32)
    // end of imul.i32 (I64)
    // 000082: ireduce.i32 (I64)
    // stop unless inst_predicate_9
    0x1009, // --> [null#00] and stop
    0x01a1, 0x0000,
    // end of ireduce.i32 (I64)
    // 000085: ishl.i32 (I64)
    // --> [RexOp1rc#40d3]
    0x0032, 0x40d3,
    // --> [Op1rc#40d3] and stop
    // 000087: ishl.i32 (I32)
    // --> [Op1rc#40d3] and stop
    0x0031, 0x40d3,
    // end of ishl.i32 (I32)
    // end of ishl.i32 (I64)
    // 000089: ishl_imm.i32 (I64)
    // --> [RexOp1r_ib#40c1]
    0x001e, 0x40c1,
    // --> [Op1r_ib#40c1] and stop
    // 00008b: ishl_imm.i32 (I32)
    // --> [Op1r_ib#40c1] and stop
    0x001d, 0x40c1,
    // end of ishl_imm.i32 (I32)
    // end of ishl_imm.i32 (I64)
    // 00008d: istore16.i32 (I64)
    // --> [RexMp1st#189]
    // 00008d: istore16.i64 (I64)
    // --> [RexMp1st#189]
    0x007a, 0x0189, // --> [Mp1st#189]
    // --> [Mp1st#189]
    0x0078, 0x0189, // --> [RexMp1stDisp8#189]
    // --> [RexMp1stDisp8#189]
    0x0082, 0x0189, // --> [Mp1stDisp8#189]
    // --> [Mp1stDisp8#189]
    0x0080, 0x0189, // --> [RexMp1stDisp32#189]
    // --> [RexMp1stDisp32#189]
    0x008a, 0x0189,
    // --> [Mp1stDisp32#189] and stop
    // --> [Mp1stDisp32#189] and stop
    0x0089, 0x0189,
    // end of istore16.i64 (I64)
    // end of istore16.i32 (I64)
    // 000099: istore16_complex.i32 (I64)
    // stop unless inst_predicate_2
    // 000099: istore16_complex.i64 (I64)
    // stop unless inst_predicate_2
    0x1002, // --> [RexMp1stWithIndex#189]
    // --> [RexMp1stWithIndex#189]
    0x0056, 0x0189, // --> [Mp1stWithIndex#189]
    // --> [Mp1stWithIndex#189]
    0x0054, 0x0189,
    // --> [RexMp1stWithIndexDisp8#189]
    // --> [RexMp1stWithIndexDisp8#189]
    0x005e, 0x0189,
    // --> [Mp1stWithIndexDisp8#189]
    // --> [Mp1stWithIndexDisp8#189]
    0x005c, 0x0189,
    // --> [RexMp1stWithIndexDisp32#189]
    // --> [RexMp1stWithIndexDisp32#189]
    0x0066, 0x0189,
    // --> [Mp1stWithIndexDisp32#189] and stop
    // --> [Mp1stWithIndexDisp32#189] and stop
    0x0065, 0x0189,
    // end of istore16_complex.i64 (I64)
    // end of istore16_complex.i32 (I64)
    // 0000a6: istore8.i32 (I64)
    // --> [RexOp1st#88]
    // 0000a6: istore8.i64 (I64)
    // --> [RexOp1st#88]
    0x0076, 0x0088, // --> [Op1st_abcd#88]
    // --> [Op1st_abcd#88]
    0x008c, 0x0088, // --> [RexOp1stDisp8#88]
    // --> [RexOp1stDisp8#88]
    0x007e, 0x0088, // --> [Op1stDisp8_abcd#88]
    // --> [Op1stDisp8_abcd#88]
    0x008e, 0x0088, // --> [RexOp1stDisp32#88]
    // --> [RexOp1stDisp32#88]
    0x0086, 0x0088,
    // --> [Op1stDisp32_abcd#88] and stop
    // --> [Op1stDisp32_abcd#88] and stop
    0x0091, 0x0088,
    // end of istore8.i64 (I64)
    // end of istore8.i32 (I64)
    // 0000b2: istore8_complex.i32 (I64)
    // stop unless inst_predicate_2
    // 0000b2: istore8_complex.i64 (I64)
    // stop unless inst_predicate_2
    0x1002, // --> [RexOp1stWithIndex_abcd#88]
    // --> [RexOp1stWithIndex_abcd#88]
    0x006a, 0x0088, // --> [Op1stWithIndex_abcd#88]
    // --> [Op1stWithIndex_abcd#88]
    0x0068, 0x0088,
    // --> [RexOp1stWithIndexDisp8_abcd#88]
    // --> [RexOp1stWithIndexDisp8_abcd#88]
    0x006e, 0x0088,
    // --> [Op1stWithIndexDisp8_abcd#88]
    // --> [Op1stWithIndexDisp8_abcd#88]
    0x006c, 0x0088,
    // --> [RexOp1stWithIndexDisp32_abcd#88]
    // --> [RexOp1stWithIndexDisp32_abcd#88]
    0x0072, 0x0088,
    // --> [Op1stWithIndexDisp32_abcd#88] and stop
    // --> [Op1stWithIndexDisp32_abcd#88] and stop
    0x0071, 0x0088,
    // end of istore8_complex.i64 (I64)
    // end of istore8_complex.i32 (I64)
    // 0000bf: isub.i32 (I64)
    // --> [RexOp1rr#29]
    0x0002, 0x0029,
    // --> [Op1rr#29] and stop
    // 0000c1: isub.i32 (I32)
    // --> [Op1rr#29] and stop
    0x0001, 0x0029,
    // end of isub.i32 (I32)
    // end of isub.i32 (I64)
    // 0000c3: load.i32 (I64)
    // --> [RexOp1ld#8b]
    // 0000c3: uload32.i64 (I64)
    // --> [RexOp1ld#8b]
    0x009c, 0x008b, // --> [Op1ld#8b]
    // --> [Op1ld#8b]
    0x009a, 0x008b, // --> [RexOp1ldDisp8#8b]
    // --> [RexOp1ldDisp8#8b]
    0x00a4, 0x008b, // --> [Op1ldDisp8#8b]
    // --> [Op1ldDisp8#8b]
    0x00a2, 0x008b, // --> [RexOp1ldDisp32#8b]
    // --> [RexOp1ldDisp32#8b]
    0x00ac, 0x008b,
    // --> [Op1ldDisp32#8b] and stop
    // --> [Op1ldDisp32#8b] and stop
    0x00ab, 0x008b,
    // end of uload32.i64 (I64)
    // end of load.i32 (I64)
    // 0000cf: load_complex.i32 (I64)
    // stop unless inst_predicate_1
    // 0000cf: uload32_complex (I64)
    // stop unless inst_predicate_1
    0x1001, // --> [RexOp1ldWithIndex#8b]
    // --> [RexOp1ldWithIndex#8b]
    0x003a, 0x008b, // --> [Op1ldWithIndex#8b]
    // --> [Op1ldWithIndex#8b]
    0x0038, 0x008b,
    // --> [RexOp1ldWithIndexDisp8#8b]
    // --> [RexOp1ldWithIndexDisp8#8b]
    0x0042, 0x008b, // --> [Op1ldWithIndexDisp8#8b]
    // --> [Op1ldWithIndexDisp8#8b]
    0x0040, 0x008b,
    // --> [RexOp1ldWithIndexDisp32#8b]
    // --> [RexOp1ldWithIndexDisp32#8b]
    0x004a, 0x008b,
    // --> [Op1ldWithIndexDisp32#8b] and stop
    // --> [Op1ldWithIndexDisp32#8b] and stop
    0x0049, 0x008b,
    // end of uload32_complex (I64)
    // end of load_complex.i32 (I64)
    // 0000dc: popcnt.i32 (I64)
    // stop unless PredicateView(15)
    0x101e, // --> [RexMp2urm#6b8]
    0x0036, 0x06b8, // --> [Mp2urm#6b8] and stop
    0x0035, 0x06b8,
    // end of popcnt.i32 (I64)
    // 0000e1: regfill.i32 (I64)
    // --> [RexOp1regfill32#8b]
    // 0000e1: regfill.b1 (I64)
    // --> [RexOp1regfill32#8b]
    // 0000e1: regfill.i8 (I64)
    // --> [RexOp1regfill32#8b]
    // 0000e1: regfill.i16 (I64)
    // --> [RexOp1regfill32#8b]
    0x00b8, 0x008b,
    // --> [Op1regfill32#8b] and stop
    // --> [Op1regfill32#8b] and stop
    // --> [Op1regfill32#8b] and stop
    // --> [Op1regfill32#8b] and stop
    // 0000e3: regfill.i32 (I32)
    // --> [Op1regfill32#8b] and stop
    // 0000e3: regfill.b1 (I32)
    // --> [Op1regfill32#8b] and stop
    // 0000e3: regfill.i8 (I32)
    // --> [Op1regfill32#8b] and stop
    // 0000e3: regfill.i16 (I32)
    // --> [Op1regfill32#8b] and stop
    0x00b7, 0x008b,
    // end of regfill.i16 (I32)
    // end of regfill.i8 (I32)
    // end of regfill.b1 (I32)
    // end of regfill.i32 (I32)
    // end of regfill.i16 (I64)
    // end of regfill.i8 (I64)
    // end of regfill.b1 (I64)
    // end of regfill.i32 (I64)
    // 0000e5: regmove.i32 (I64)
    // --> [RexOp1rmov#89] and stop
    // 0000e5: regmove.i16 (I64)
    // --> [RexOp1rmov#89] and stop
    0x001b, 0x0089,
    // end of regmove.i16 (I64)
    // end of regmove.i32 (I64)
    // 0000e7: regspill.i32 (I64)
    // --> [RexOp1regspill32#89]
    // 0000e7: regspill.b1 (I64)
    // --> [RexOp1regspill32#89]
    // 0000e7: regspill.i8 (I64)
    // --> [RexOp1regspill32#89]
    // 0000e7: regspill.i16 (I64)
    // --> [RexOp1regspill32#89]
    0x0098, 0x0089,
    // --> [Op1regspill32#89] and stop
    // --> [Op1regspill32#89] and stop
    // --> [Op1regspill32#89] and stop
    // --> [Op1regspill32#89] and stop
    // 0000e9: regspill.i32 (I32)
    // --> [Op1regspill32#89] and stop
    // 0000e9: regspill.b1 (I32)
    // --> [Op1regspill32#89] and stop
    // 0000e9: regspill.i8 (I32)
    // --> [Op1regspill32#89] and stop
    // 0000e9: regspill.i16 (I32)
    // --> [Op1regspill32#89] and stop
    0x0097, 0x0089,
    // end of regspill.i16 (I32)
    // end of regspill.i8 (I32)
    // end of regspill.b1 (I32)
    // end of regspill.i32 (I32)
    // end of regspill.i16 (I64)
    // end of regspill.i8 (I64)
    // end of regspill.b1 (I64)
    // end of regspill.i32 (I64)
    // 0000eb: rotl.i32 (I64)
    // --> [RexOp1rc#d3]
    0x0032, 0x00d3,
    // --> [Op1rc#d3] and stop
    // 0000ed: rotl.i32 (I32)
    // --> [Op1rc#d3] and stop
    0x0031, 0x00d3,
    // end of rotl.i32 (I32)
    // end of rotl.i32 (I64)
    // 0000ef: rotl_imm.i32 (I64)
    // --> [RexOp1r_ib#c1]
    0x001e, 0x00c1,
    // --> [Op1r_ib#c1] and stop
    // 0000f1: rotl_imm.i32 (I32)
    // --> [Op1r_ib#c1] and stop
    0x001d, 0x00c1,
    // end of rotl_imm.i32 (I32)
    // end of rotl_imm.i32 (I64)
    // 0000f3: rotr.i32 (I64)
    // --> [RexOp1rc#10d3]
    0x0032, 0x10d3,
    // --> [Op1rc#10d3] and stop
    // 0000f5: rotr.i32 (I32)
    // --> [Op1rc#10d3] and stop
    0x0031, 0x10d3,
    // end of rotr.i32 (I32)
    // end of rotr.i32 (I64)
    // 0000f7: rotr_imm.i32 (I64)
    // --> [RexOp1r_ib#10c1]
    0x001e, 0x10c1,
    // --> [Op1r_ib#10c1] and stop
    // 0000f9: rotr_imm.i32 (I32)
    // --> [Op1r_ib#10c1] and stop
    0x001d, 0x10c1,
    // end of rotr_imm.i32 (I32)
    // end of rotr_imm.i32 (I64)
    // 0000fb: selectif.i32 (I64)
    // --> [RexOp2cmov#440]
    0x0196, 0x0440,
    // --> [Op2cmov#440] and stop
    // 0000fd: selectif.i32 (I32)
    // --> [Op2cmov#440] and stop
    0x0195, 0x0440,
    // end of selectif.i32 (I32)
    // end of selectif.i32 (I64)
    // 0000ff: sextend.i32 (I64)
    // skip 4 unless inst_predicate_10
    0x500a, // --> [RexOp2urm_noflags#4be]
    0x019e, 0x04be, // --> [Op2urm_noflags_abcd#4be]
    0x019c, 0x04be, // stop unless inst_predicate_7
    0x1007, // --> [RexOp2urm_noflags#4bf]
    0x019e, 0x04bf, // --> [Op2urm_noflags#4bf] and stop
    0x01a3, 0x04bf,
    // end of sextend.i32 (I64)
    // 000109: sload16.i32 (I64)
    // --> [RexOp2ld#4bf]
    0x00a0, 0x04bf, // --> [Op2ld#4bf]
    0x009e, 0x04bf, // --> [RexOp2ldDisp8#4bf]
    0x00a8, 0x04bf, // --> [Op2ldDisp8#4bf]
    0x00a6, 0x04bf, // --> [RexOp2ldDisp32#4bf]
    0x00b0, 0x04bf, // --> [Op2ldDisp32#4bf] and stop
    0x00af, 0x04bf,
    // end of sload16.i32 (I64)
    // 000115: sload16_complex.i32 (I64)
    // stop unless inst_predicate_1
    0x1001, // --> [RexOp2ldWithIndex#4bf]
    0x003e, 0x04bf, // --> [Op2ldWithIndex#4bf]
    0x003c, 0x04bf, // --> [RexOp2ldWithIndexDisp8#4bf]
    0x0046, 0x04bf, // --> [Op2ldWithIndexDisp8#4bf]
    0x0044, 0x04bf, // --> [RexOp2ldWithIndexDisp32#4bf]
    0x004e, 0x04bf, // --> [Op2ldWithIndexDisp32#4bf] and stop
    0x004d, 0x04bf,
    // end of sload16_complex.i32 (I64)
    // 000122: sload8.i32 (I64)
    // --> [RexOp2ld#4be]
    0x00a0, 0x04be, // --> [Op2ld#4be]
    0x009e, 0x04be, // --> [RexOp2ldDisp8#4be]
    0x00a8, 0x04be, // --> [Op2ldDisp8#4be]
    0x00a6, 0x04be, // --> [RexOp2ldDisp32#4be]
    0x00b0, 0x04be, // --> [Op2ldDisp32#4be] and stop
    0x00af, 0x04be,
    // end of sload8.i32 (I64)
    // 00012e: sload8_complex.i32 (I64)
    // stop unless inst_predicate_1
    0x1001, // --> [RexOp2ldWithIndex#4be]
    0x003e, 0x04be, // --> [Op2ldWithIndex#4be]
    0x003c, 0x04be, // --> [RexOp2ldWithIndexDisp8#4be]
    0x0046, 0x04be, // --> [Op2ldWithIndexDisp8#4be]
    0x0044, 0x04be, // --> [RexOp2ldWithIndexDisp32#4be]
    0x004e, 0x04be, // --> [Op2ldWithIndexDisp32#4be] and stop
    0x004d, 0x04be,
    // end of sload8_complex.i32 (I64)
    // 00013b: spill.i32 (I64)
    // --> [RexOp1spillSib32#89]
    // 00013b: spill.b1 (I64)
    // --> [RexOp1spillSib32#89]
    // 00013b: spill.i8 (I64)
    // --> [RexOp1spillSib32#89]
    // 00013b: spill.i16 (I64)
    // --> [RexOp1spillSib32#89]
    0x0094, 0x0089,
    // --> [Op1spillSib32#89] and stop
    // --> [Op1spillSib32#89] and stop
    // --> [Op1spillSib32#89] and stop
    // --> [Op1spillSib32#89] and stop
    // 00013d: spill.i32 (I32)
    // --> [Op1spillSib32#89] and stop
    // 00013d: spill.b1 (I32)
    // --> [Op1spillSib32#89] and stop
    // 00013d: spill.i8 (I32)
    // --> [Op1spillSib32#89] and stop
    // 00013d: spill.i16 (I32)
    // --> [Op1spillSib32#89] and stop
    0x0093, 0x0089,
    // end of spill.i16 (I32)
    // end of spill.i8 (I32)
    // end of spill.b1 (I32)
    // end of spill.i32 (I32)
    // end of spill.i16 (I64)
    // end of spill.i8 (I64)
    // end of spill.b1 (I64)
    // end of spill.i32 (I64)
    // 00013f: sshr.i32 (I64)
    // --> [RexOp1rc#70d3]
    0x0032, 0x70d3,
    // --> [Op1rc#70d3] and stop
    // 000141: sshr.i32 (I32)
    // --> [Op1rc#70d3] and stop
    0x0031, 0x70d3,
    // end of sshr.i32 (I32)
    // end of sshr.i32 (I64)
    // 000143: sshr_imm.i32 (I64)
    // --> [RexOp1r_ib#70c1]
    0x001e, 0x70c1,
    // --> [Op1r_ib#70c1] and stop
    // 000145: sshr_imm.i32 (I32)
    // --> [Op1r_ib#70c1] and stop
    0x001d, 0x70c1,
    // end of sshr_imm.i32 (I32)
    // end of sshr_imm.i32 (I64)
    // 000147: store.i32 (I64)
    // --> [RexOp1st#89]
    // 000147: istore32.i64 (I64)
    // --> [RexOp1st#89]
    0x0076, 0x0089, // --> [Op1st#89]
    // --> [Op1st#89]
    0x0074, 0x0089, // --> [RexOp1stDisp8#89]
    // --> [RexOp1stDisp8#89]
    0x007e, 0x0089, // --> [Op1stDisp8#89]
    // --> [Op1stDisp8#89]
    0x007c, 0x0089, // --> [RexOp1stDisp32#89]
    // --> [RexOp1stDisp32#89]
    0x0086, 0x0089,
    // --> [Op1stDisp32#89] and stop
    // --> [Op1stDisp32#89] and stop
    0x0085, 0x0089,
    // end of istore32.i64 (I64)
    // end of store.i32 (I64)
    // 000153: store_complex.i32 (I64)
    // stop unless inst_predicate_2
    // 000153: istore32_complex (I64)
    // stop unless inst_predicate_2
    0x1002, // --> [RexOp1stWithIndex#89]
    // --> [RexOp1stWithIndex#89]
    0x0052, 0x0089, // --> [Op1stWithIndex#89]
    // --> [Op1stWithIndex#89]
    0x0050, 0x0089,
    // --> [RexOp1stWithIndexDisp8#89]
    // --> [RexOp1stWithIndexDisp8#89]
    0x005a, 0x0089, // --> [Op1stWithIndexDisp8#89]
    // --> [Op1stWithIndexDisp8#89]
    0x0058, 0x0089,
    // --> [RexOp1stWithIndexDisp32#89]
    // --> [RexOp1stWithIndexDisp32#89]
    0x0062, 0x0089,
    // --> [Op1stWithIndexDisp32#89] and stop
    // --> [Op1stWithIndexDisp32#89] and stop
    0x0061, 0x0089,
    // end of istore32_complex (I64)
    // end of store_complex.i32 (I64)
    // 000160: uextend.i32 (I64)
    // skip 4 unless inst_predicate_10
    0x500a, // --> [RexOp2urm_noflags#4b6]
    0x019e, 0x04b6, // --> [Op2urm_noflags_abcd#4b6]
    0x019c, 0x04b6, // stop unless inst_predicate_7
    0x1007, // --> [RexOp2urm_noflags#4b7]
    0x019e, 0x04b7, // --> [Op2urm_noflags#4b7] and stop
    0x01a3, 0x04b7,
    // end of uextend.i32 (I64)
    // 00016a: uload16.i32 (I64)
    // --> [RexOp2ld#4b7]
    0x00a0, 0x04b7, // --> [Op2ld#4b7]
    0x009e, 0x04b7, // --> [RexOp2ldDisp8#4b7]
    0x00a8, 0x04b7, // --> [Op2ldDisp8#4b7]
    0x00a6, 0x04b7, // --> [RexOp2ldDisp32#4b7]
    0x00b0, 0x04b7, // --> [Op2ldDisp32#4b7] and stop
    0x00af, 0x04b7,
    // end of uload16.i32 (I64)
    // 000176: uload16_complex.i32 (I64)
    // stop unless inst_predicate_1
    0x1001, // --> [RexOp2ldWithIndex#4b7]
    0x003e, 0x04b7, // --> [Op2ldWithIndex#4b7]
    0x003c, 0x04b7, // --> [RexOp2ldWithIndexDisp8#4b7]
    0x0046, 0x04b7, // --> [Op2ldWithIndexDisp8#4b7]
    0x0044, 0x04b7, // --> [RexOp2ldWithIndexDisp32#4b7]
    0x004e, 0x04b7, // --> [Op2ldWithIndexDisp32#4b7] and stop
    0x004d, 0x04b7,
    // end of uload16_complex.i32 (I64)
    // 000183: uload8.i32 (I64)
    // --> [RexOp2ld#4b6]
    0x00a0, 0x04b6, // --> [Op2ld#4b6]
    0x009e, 0x04b6, // --> [RexOp2ldDisp8#4b6]
    0x00a8, 0x04b6, // --> [Op2ldDisp8#4b6]
    0x00a6, 0x04b6, // --> [RexOp2ldDisp32#4b6]
    0x00b0, 0x04b6, // --> [Op2ldDisp32#4b6] and stop
    0x00af, 0x04b6,
    // end of uload8.i32 (I64)
    // 00018f: uload8_complex.i32 (I64)
    // stop unless inst_predicate_1
    0x1001, // --> [RexOp2ldWithIndex#4b6]
    0x003e, 0x04b6, // --> [Op2ldWithIndex#4b6]
    0x003c, 0x04b6, // --> [RexOp2ldWithIndexDisp8#4b6]
    0x0046, 0x04b6, // --> [Op2ldWithIndexDisp8#4b6]
    0x0044, 0x04b6, // --> [RexOp2ldWithIndexDisp32#4b6]
    0x004e, 0x04b6, // --> [Op2ldWithIndexDisp32#4b6] and stop
    0x004d, 0x04b6,
    // end of uload8_complex.i32 (I64)
    // 00019c: ushr.i32 (I64)
    // --> [RexOp1rc#50d3]
    0x0032, 0x50d3,
    // --> [Op1rc#50d3] and stop
    // 00019e: ushr.i32 (I32)
    // --> [Op1rc#50d3] and stop
    0x0031, 0x50d3,
    // end of ushr.i32 (I32)
    // end of ushr.i32 (I64)
    // 0001a0: ushr_imm.i32 (I64)
    // --> [RexOp1r_ib#50c1]
    0x001e, 0x50c1,
    // --> [Op1r_ib#50c1] and stop
    // 0001a2: ushr_imm.i32 (I32)
    // --> [Op1r_ib#50c1] and stop
    0x001d, 0x50c1,
    // end of ushr_imm.i32 (I32)
    // end of ushr_imm.i32 (I64)
    // 0001a4: x86_bsf.i32 (I64)
    // --> [RexOp2bsf_and_bsr#4bc]
    0x019a, 0x04bc,
    // --> [Op2bsf_and_bsr#4bc] and stop
    // 0001a6: x86_bsf.i32 (I32)
    // --> [Op2bsf_and_bsr#4bc] and stop
    0x0199, 0x04bc,
    // end of x86_bsf.i32 (I32)
    // end of x86_bsf.i32 (I64)
    // 0001a8: x86_bsr.i32 (I64)
    // --> [RexOp2bsf_and_bsr#4bd]
    0x019a, 0x04bd,
    // --> [Op2bsf_and_bsr#4bd] and stop
    // 0001aa: x86_bsr.i32 (I32)
    // --> [Op2bsf_and_bsr#4bd] and stop
    0x0199, 0x04bd,
    // end of x86_bsr.i32 (I32)
    // end of x86_bsr.i32 (I64)
    // 0001ac: x86_cvtt2si.i32 (I64)
    // skip 4 unless inst_predicate_13
    0x500d, // --> [RexMp2rfurm#62c]
    0x01c4, 0x062c, // --> [Mp2rfurm#62c]
    0x01c2, 0x062c, // stop unless inst_predicate_14
    0x100e, // --> [RexMp2rfurm#72c]
    0x01c4, 0x072c, // --> [Mp2rfurm#72c] and stop
    0x01c3, 0x072c,
    // end of x86_cvtt2si.i32 (I64)
    // 0001b6: x86_sdivmodx.i32 (I64)
    // --> [RexOp1div#70f7]
    0x000e, 0x70f7,
    // --> [Op1div#70f7] and stop
    // 0001b8: x86_sdivmodx.i32 (I32)
    // --> [Op1div#70f7] and stop
    0x000d, 0x70f7,
    // end of x86_sdivmodx.i32 (I32)
    // end of x86_sdivmodx.i32 (I64)
    // 0001ba: x86_smulx.i32 (I64)
    // --> [RexOp1mulx#50f7]
    0x0012, 0x50f7,
    // --> [Op1mulx#50f7] and stop
    // 0001bc: x86_smulx.i32 (I32)
    // --> [Op1mulx#50f7] and stop
    0x0011, 0x50f7,
    // end of x86_smulx.i32 (I32)
    // end of x86_smulx.i32 (I64)
    // 0001be: x86_udivmodx.i32 (I64)
    // --> [RexOp1div#60f7]
    0x000e, 0x60f7,
    // --> [Op1div#60f7] and stop
    // 0001c0: x86_udivmodx.i32 (I32)
    // --> [Op1div#60f7] and stop
    0x000d, 0x60f7,
    // end of x86_udivmodx.i32 (I32)
    // end of x86_udivmodx.i32 (I64)
    // 0001c2: x86_umulx.i32 (I64)
    // --> [RexOp1mulx#40f7]
    0x0012, 0x40f7,
    // --> [Op1mulx#40f7] and stop
    // 0001c4: x86_umulx.i32 (I32)
    // --> [Op1mulx#40f7] and stop
    0x0011, 0x40f7,
    // end of x86_umulx.i32 (I32)
    // end of x86_umulx.i32 (I64)
    // 0001c6: adjust_sp_down.i64 (I64)
    // --> [RexOp1adjustsp#8029] and stop
    0x00cb, 0x8029,
    // end of adjust_sp_down.i64 (I64)
    // 0001c8: band.i64 (I64)
    // --> [RexOp1rr#8021] and stop
    0x0003, 0x8021,
    // end of band.i64 (I64)
    // 0001ca: band_imm.i64 (I64)
    // --> [RexOp1r_ib#c083]
    0x001e, 0xc083, // --> [RexOp1r_id#c081] and stop
    0x0023, 0xc081,
    // end of band_imm.i64 (I64)
    // 0001ce: bitcast.i64 (I64)
    // stop unless inst_predicate_14
    0x100e, // --> [RexMp2rfumr#857e] and stop
    0x01b5, 0x857e,
    // end of bitcast.i64 (I64)
    // 0001d1: bnot.i64 (I64)
    // --> [RexOp1ur#a0f7] and stop
    0x0007, 0xa0f7,
    // end of bnot.i64 (I64)
    // 0001d3: bor.i64 (I64)
    // --> [RexOp1rr#8009] and stop
    0x0003, 0x8009,
    // end of bor.i64 (I64)
    // 0001d5: bor_imm.i64 (I64)
    // --> [RexOp1r_ib#9083]
    0x001e, 0x9083, // --> [RexOp1r_id#9081] and stop
    0x0023, 0x9081,
    // end of bor_imm.i64 (I64)
    // 0001d9: brnz.i64 (I64)
    // --> [RexOp1tjccb#8075]
    0x014c, 0x8075, // --> [RexOp1tjccd#8085] and stop
    0x0151, 0x8085,
    // end of brnz.i64 (I64)
    // 0001dd: brz.i64 (I64)
    // --> [RexOp1tjccb#8074]
    0x014c, 0x8074, // --> [RexOp1tjccd#8084] and stop
    0x0151, 0x8084,
    // end of brz.i64 (I64)
    // 0001e1: bxor.i64 (I64)
    // --> [RexOp1rr#8031] and stop
    0x0003, 0x8031,
    // end of bxor.i64 (I64)
    // 0001e3: bxor_imm.i64 (I64)
    // --> [RexOp1r_ib#e083]
    0x001e, 0xe083, // --> [RexOp1r_id#e081] and stop
    0x0023, 0xe081,
    // end of bxor_imm.i64 (I64)
    // 0001e7: call_indirect.i64 (I64)
    // --> [RexOp1call_r#20ff]
    0x0132, 0x20ff,
    // --> [Op1call_r#20ff] and stop
    // 0001e9: call_indirect.i32 (I32)
    // --> [Op1call_r#20ff] and stop
    0x0131, 0x20ff,
    // end of call_indirect.i32 (I32)
    // end of call_indirect.i64 (I64)
    // 0001eb: clz.i64 (I64)
    // stop unless PredicateView(14)
    0x101d, // --> [RexMp2urm#86bd] and stop
    0x0037, 0x86bd,
    // end of clz.i64 (I64)
    // 0001ee: copy.i64 (I64)
    // --> [RexOp1umr#8089] and stop
    0x0017, 0x8089,
    // end of copy.i64 (I64)
    // 0001f0: ctz.i64 (I64)
    // stop unless PredicateView(13)
    0x101c, // --> [RexMp2urm#86bc] and stop
    0x0037, 0x86bc,
    // end of ctz.i64 (I64)
    // 0001f3: fill.i64 (I64)
    // --> [RexOp1fillSib32#808b] and stop
    0x00b5, 0x808b,
    // end of fill.i64 (I64)
    // 0001f5: func_addr.i64 (I64)
    // skip 2 unless PredicateView(11)
    0x301a, // --> [RexOp1fnaddr8#80b8]
    0x0116, 0x80b8, // skip 2 unless PredicateView(9)
    0x3018, // --> [RexOp1allones_fnaddr8#80b8]
    0x011a, 0x80b8, // skip 2 unless inst_predicate_3
    0x3003, // --> [RexOp1pcrel_fnaddr8#808d]
    0x011c, 0x808d, // stop unless PredicateView(10)
    0x1019, // --> [RexOp1got_fnaddr8#808b] and stop
    0x011f, 0x808b,
    // end of func_addr.i64 (I64)
    // 000201: iadd.i64 (I64)
    // --> [RexOp1rr#8001] and stop
    0x0003, 0x8001,
    // end of iadd.i64 (I64)
    // 000203: iadd_imm.i64 (I64)
    // --> [RexOp1r_ib#8083]
    0x001e, 0x8083, // --> [RexOp1r_id#8081] and stop
    0x0023, 0x8081,
    // end of iadd_imm.i64 (I64)
    // 000207: icmp.i64 (I64)
    // --> [RexOp1icscc#8039] and stop
    0x0173, 0x8039,
    // end of icmp.i64 (I64)
    // 000209: icmp_imm.i64 (I64)
    // --> [RexOp1icscc_ib#f083]
    0x0176, 0xf083, // --> [RexOp1icscc_id#f081] and stop
    0x017b, 0xf081,
    // end of icmp_imm.i64 (I64)
    // 00020d: iconst.i64 (I64)
    // skip 4 unless inst_predicate_0
    0x5000, // --> [RexOp1pu_id#b8]
    0x0026, 0x00b8, // --> [Op1pu_id#b8]
    0x0024, 0x00b8, // --> [RexOp1u_id#80c7]
    0x0028, 0x80c7, // --> [RexOp1pu_iq#80b8] and stop
    0x002b, 0x80b8,
    // end of iconst.i64 (I64)
    // 000216: ifcmp.i64 (I64)
    // --> [RexOp1rcmp#8039] and stop
    0x017f, 0x8039,
    // end of ifcmp.i64 (I64)
    // 000218: ifcmp_imm.i64 (I64)
    // --> [RexOp1rcmp_ib#f083]
    0x0182, 0xf083, // --> [RexOp1rcmp_id#f081] and stop
    0x0187, 0xf081,
    // end of ifcmp_imm.i64 (I64)
    // 00021c: ifcmp_sp.i64 (I64)
    // --> [RexOp1rcmp_sp#8039] and stop
    0x018b, 0x8039,
    // end of ifcmp_sp.i64 (I64)
    // 00021e: imul.i64 (I64)
    // --> [RexOp2rrx#84af] and stop
    0x000b, 0x84af,
    // end of imul.i64 (I64)
    // 000220: indirect_jump_table_br.i64 (I64)
    // --> [RexOp1indirect_jmp#40ff]
    0x0164, 0x40ff,
    // --> [Op1indirect_jmp#40ff] and stop
    // 000222: indirect_jump_table_br.i32 (I32)
    // --> [Op1indirect_jmp#40ff] and stop
    0x0167, 0x40ff,
    // end of indirect_jump_table_br.i32 (I32)
    // end of indirect_jump_table_br.i64 (I64)
    // 000224: ishl.i64 (I64)
    // --> [RexOp1rc#c0d3] and stop
    0x0033, 0xc0d3,
    // end of ishl.i64 (I64)
    // 000226: ishl_imm.i64 (I64)
    // --> [RexOp1r_ib#c0c1] and stop
    0x001f, 0xc0c1,
    // end of ishl_imm.i64 (I64)
    // 000228: isub.i64 (I64)
    // --> [RexOp1rr#8029] and stop
    0x0003, 0x8029,
    // end of isub.i64 (I64)
    // 00022a: jump_table_base.i64 (I64)
    // --> [RexOp1jt_base#808d] and stop
    0x0161, 0x808d,
    // end of jump_table_base.i64 (I64)
    // 00022c: jump_table_entry.i64 (I64)
    // --> [RexOp1jt_entry#8063] and stop
    0x015d, 0x8063,
    // end of jump_table_entry.i64 (I64)
    // 00022e: load.i64 (I64)
    // --> [RexOp1ld#808b]
    0x009c, 0x808b, // --> [RexOp1ldDisp8#808b]
    0x00a4, 0x808b, // --> [RexOp1ldDisp32#808b] and stop
    0x00ad, 0x808b,
    // end of load.i64 (I64)
    // 000234: load_complex.i64 (I64)
    // stop unless inst_predicate_1
    0x1001, // --> [RexOp1ldWithIndex#808b]
    0x003a, 0x808b, // --> [RexOp1ldWithIndexDisp8#808b]
    0x0042, 0x808b, // --> [RexOp1ldWithIndexDisp32#808b] and stop
    0x004b, 0x808b,
    // end of load_complex.i64 (I64)
    // 00023b: popcnt.i64 (I64)
    // stop unless PredicateView(15)
    0x101e, // --> [RexMp2urm#86b8] and stop
    0x0037, 0x86b8,
    // end of popcnt.i64 (I64)
    // 00023e: regfill.i64 (I64)
    // --> [RexOp1regfill32#808b] and stop
    0x00b9, 0x808b,
    // end of regfill.i64 (I64)
    // 000240: regmove.i64 (I64)
    // --> [RexOp1rmov#8089] and stop
    0x001b, 0x8089,
    // end of regmove.i64 (I64)
    // 000242: regspill.i64 (I64)
    // --> [RexOp1regspill32#8089] and stop
    0x0099, 0x8089,
    // end of regspill.i64 (I64)
    // 000244: rotl.i64 (I64)
    // --> [RexOp1rc#80d3] and stop
    0x0033, 0x80d3,
    // end of rotl.i64 (I64)
    // 000246: rotl_imm.i64 (I64)
    // --> [RexOp1r_ib#80c1] and stop
    0x001f, 0x80c1,
    // end of rotl_imm.i64 (I64)
    // 000248: rotr.i64 (I64)
    // --> [RexOp1rc#90d3] and stop
    0x0033, 0x90d3,
    // end of rotr.i64 (I64)
    // 00024a: rotr_imm.i64 (I64)
    // --> [RexOp1r_ib#90c1] and stop
    0x001f, 0x90c1,
    // end of rotr_imm.i64 (I64)
    // 00024c: selectif.i64 (I64)
    // --> [RexOp2cmov#8440] and stop
    0x0197, 0x8440,
    // end of selectif.i64 (I64)
    // 00024e: sextend.i64 (I64)
    // skip 2 unless inst_predicate_10
    0x300a, // --> [RexOp2urm_noflags#84be]
    0x019e, 0x84be, // skip 2 unless inst_predicate_7
    0x3007, // --> [RexOp2urm_noflags#84bf]
    0x019e, 0x84bf, // stop unless inst_predicate_8
    0x1008, // --> [RexOp1urm_noflags#8063] and stop
    0x01a5, 0x8063,
    // end of sextend.i64 (I64)
    // 000257: sload16.i64 (I64)
    // --> [RexOp2ld#84bf]
    0x00a0, 0x84bf, // --> [RexOp2ldDisp8#84bf]
    0x00a8, 0x84bf, // --> [RexOp2ldDisp32#84bf] and stop
    0x00b1, 0x84bf,
    // end of sload16.i64 (I64)
    // 00025d: sload16_complex.i64 (I64)
    // stop unless inst_predicate_1
    0x1001, // --> [RexOp2ldWithIndex#84bf]
    0x003e, 0x84bf, // --> [RexOp2ldWithIndexDisp8#84bf]
    0x0046, 0x84bf, // --> [RexOp2ldWithIndexDisp32#84bf] and stop
    0x004f, 0x84bf,
    // end of sload16_complex.i64 (I64)
    // 000264: sload32.i64 (I64)
    // --> [RexOp1ld#8063]
    0x009c, 0x8063, // --> [RexOp1ldDisp8#8063]
    0x00a4, 0x8063, // --> [RexOp1ldDisp32#8063] and stop
    0x00ad, 0x8063,
    // end of sload32.i64 (I64)
    // 00026a: sload8.i64 (I64)
    // --> [RexOp2ld#84be]
    0x00a0, 0x84be, // --> [RexOp2ldDisp8#84be]
    0x00a8, 0x84be, // --> [RexOp2ldDisp32#84be] and stop
    0x00b1, 0x84be,
    // end of sload8.i64 (I64)
    // 000270: sload8_complex.i64 (I64)
    // stop unless inst_predicate_1
    0x1001, // --> [RexOp2ldWithIndex#84be]
    0x003e, 0x84be, // --> [RexOp2ldWithIndexDisp8#84be]
    0x0046, 0x84be, // --> [RexOp2ldWithIndexDisp32#84be] and stop
    0x004f, 0x84be,
    // end of sload8_complex.i64 (I64)
    // 000277: spill.i64 (I64)
    // --> [RexOp1spillSib32#8089] and stop
    0x0095, 0x8089,
    // end of spill.i64 (I64)
    // 000279: sshr.i64 (I64)
    // --> [RexOp1rc#f0d3] and stop
    0x0033, 0xf0d3,
    // end of sshr.i64 (I64)
    // 00027b: sshr_imm.i64 (I64)
    // --> [RexOp1r_ib#f0c1] and stop
    0x001f, 0xf0c1,
    // end of sshr_imm.i64 (I64)
    // 00027d: stack_addr.i64 (I64)
    // --> [RexOp1spaddr8_id#808d] and stop
    0x012b, 0x808d,
    // end of stack_addr.i64 (I64)
    // 00027f: store.i64 (I64)
    // --> [RexOp1st#8089]
    0x0076, 0x8089, // --> [RexOp1stDisp8#8089]
    0x007e, 0x8089, // --> [RexOp1stDisp32#8089] and stop
    0x0087, 0x8089,
    // end of store.i64 (I64)
    // 000285: store_complex.i64 (I64)
    // stop unless inst_predicate_2
    0x1002, // --> [RexOp1stWithIndex#8089]
    0x0052, 0x8089, // --> [RexOp1stWithIndexDisp8#8089]
    0x005a, 0x8089, // --> [RexOp1stWithIndexDisp32#8089] and stop
    0x0063, 0x8089,
    // end of store_complex.i64 (I64)
    // 00028c: symbol_value.i64 (I64)
    // skip 2 unless PredicateView(12)
    0x301b, // --> [RexOp1gvaddr8#80b8]
    0x0122, 0x80b8, // skip 3 unless PredicateView(10)
    0x4019, // skip 2 unless inst_predicate_4
    0x3004, // --> [RexOp1pcrel_gvaddr8#808d]
    0x0124, 0x808d, // stop unless PredicateView(10)
    0x1019, // --> [RexOp1got_gvaddr8#808b] and stop
    0x0127, 0x808b,
    // end of symbol_value.i64 (I64)
    // 000296: uextend.i64 (I64)
    // skip 4 unless inst_predicate_10
    0x500a, // --> [RexOp2urm_noflags#4b6]
    0x019e, 0x04b6, // --> [Op2urm_noflags_abcd#4b6]
    0x019c, 0x04b6, // skip 4 unless inst_predicate_7
    0x5007, // --> [RexOp2urm_noflags#4b7]
    0x019e, 0x04b7, // --> [Op2urm_noflags#4b7]
    0x01a2, 0x04b7, // stop unless inst_predicate_8
    0x1008, // --> [RexOp1umr#89]
    0x0016, 0x0089, // --> [Op1umr#89] and stop
    0x0015, 0x0089,
    // end of uextend.i64 (I64)
    // 0002a5: uload16.i64 (I64)
    // --> [RexOp2ld#84b7]
    0x00a0, 0x84b7, // --> [RexOp2ldDisp8#84b7]
    0x00a8, 0x84b7, // --> [RexOp2ldDisp32#84b7] and stop
    0x00b1, 0x84b7,
    // end of uload16.i64 (I64)
    // 0002ab: uload16_complex.i64 (I64)
    // stop unless inst_predicate_1
    0x1001, // --> [RexOp2ldWithIndex#84b7]
    0x003e, 0x84b7, // --> [RexOp2ldWithIndexDisp8#84b7]
    0x0046, 0x84b7, // --> [RexOp2ldWithIndexDisp32#84b7] and stop
    0x004f, 0x84b7,
    // end of uload16_complex.i64 (I64)
    // 0002b2: uload8.i64 (I64)
    // --> [RexOp2ld#84b6]
    0x00a0, 0x84b6, // --> [RexOp2ldDisp8#84b6]
    0x00a8, 0x84b6, // --> [RexOp2ldDisp32#84b6] and stop
    0x00b1, 0x84b6,
    // end of uload8.i64 (I64)
    // 0002b8: uload8_complex.i64 (I64)
    // stop unless inst_predicate_1
    0x1001, // --> [RexOp2ldWithIndex#84b6]
    0x003e, 0x84b6, // --> [RexOp2ldWithIndexDisp8#84b6]
    0x0046, 0x84b6, // --> [RexOp2ldWithIndexDisp32#84b6] and stop
    0x004f, 0x84b6,
    // end of uload8_complex.i64 (I64)
    // 0002bf: ushr.i64 (I64)
    // --> [RexOp1rc#d0d3] and stop
    0x0033, 0xd0d3,
    // end of ushr.i64 (I64)
    // 0002c1: ushr_imm.i64 (I64)
    // --> [RexOp1r_ib#d0c1] and stop
    0x001f, 0xd0c1,
    // end of ushr_imm.i64 (I64)
    // 0002c3: x86_bsf.i64 (I64)
    // --> [RexOp2bsf_and_bsr#84bc] and stop
    0x019b, 0x84bc,
    // end of x86_bsf.i64 (I64)
    // 0002c5: x86_bsr.i64 (I64)
    // --> [RexOp2bsf_and_bsr#84bd] and stop
    0x019b, 0x84bd,
    // end of x86_bsr.i64 (I64)
    // 0002c7: x86_cvtt2si.i64 (I64)
    // skip 2 unless inst_predicate_13
    0x300d, // --> [RexMp2rfurm#862c]
    0x01c4, 0x862c, // stop unless inst_predicate_14
    0x100e, // --> [RexMp2rfurm#872c] and stop
    0x01c5, 0x872c,
    // end of x86_cvtt2si.i64 (I64)
    // 0002cd: x86_pop.i64 (I64)
    // --> [RexOp1popq#58]
    0x00c0, 0x0058,
    // --> [Op1popq#58] and stop
    // 0002cf: x86_pop.i32 (I32)
    // --> [Op1popq#58] and stop
    0x00bf, 0x0058,
    // end of x86_pop.i32 (I32)
    // end of x86_pop.i64 (I64)
    // 0002d1: x86_push.i64 (I64)
    // --> [RexOp1pushq#50]
    0x00bc, 0x0050,
    // --> [Op1pushq#50] and stop
    // 0002d3: x86_push.i32 (I32)
    // --> [Op1pushq#50] and stop
    0x00bb, 0x0050,
    // end of x86_push.i32 (I32)
    // end of x86_push.i64 (I64)
    // 0002d5: x86_sdivmodx.i64 (I64)
    // --> [RexOp1div#f0f7] and stop
    0x000f, 0xf0f7,
    // end of x86_sdivmodx.i64 (I64)
    // 0002d7: x86_smulx.i64 (I64)
    // --> [RexOp1mulx#d0f7] and stop
    0x0013, 0xd0f7,
    // end of x86_smulx.i64 (I64)
    // 0002d9: x86_udivmodx.i64 (I64)
    // --> [RexOp1div#e0f7] and stop
    0x000f, 0xe0f7,
    // end of x86_udivmodx.i64 (I64)
    // 0002db: x86_umulx.i64 (I64)
    // --> [RexOp1mulx#c0f7] and stop
    0x0013, 0xc0f7,
    // end of x86_umulx.i64 (I64)
    // 0002dd: bconst.b1 (I64)
    // --> [RexOp1pu_id_bool#b8]
    0x002e, 0x00b8,
    // --> [Op1pu_id_bool#b8] and stop
    // 0002df: bconst.b1 (I32)
    // --> [Op1pu_id_bool#b8] and stop
    0x002d, 0x00b8,
    // end of bconst.b1 (I32)
    // end of bconst.b1 (I64)
    // 0002e1: brnz.b1 (I64)
    // --> [RexOp1t8jccb#75]
    0x0156, 0x0075, // --> [Op1t8jccb_abcd#75]
    0x0154, 0x0075, // --> [RexOp1t8jccd#85]
    0x015a, 0x0085, // --> [Op1t8jccd_abcd#85] and stop
    0x0159, 0x0085,
    // end of brnz.b1 (I64)
    // 0002e9: brz.b1 (I64)
    // --> [RexOp1t8jccb#74]
    0x0156, 0x0074, // --> [Op1t8jccb_abcd#74]
    0x0154, 0x0074, // --> [RexOp1t8jccd#84]
    0x015a, 0x0084, // --> [Op1t8jccd_abcd#84] and stop
    0x0159, 0x0084,
    // end of brz.b1 (I64)
    // 0002f1: regmove.b1 (I64)
    // --> [RexOp1rmov#89]
    0x001a, 0x0089,
    // --> [Op1rmov#89] and stop
    // 0002f3: regmove.i32 (I32)
    // --> [Op1rmov#89] and stop
    // 0002f3: regmove.b1 (I32)
    // --> [Op1rmov#89] and stop
    // 0002f3: regmove.i16 (I32)
    // --> [Op1rmov#89] and stop
    0x0019, 0x0089,
    // end of regmove.i16 (I32)
    // end of regmove.b1 (I32)
    // end of regmove.i32 (I32)
    // end of regmove.b1 (I64)
    // 0002f5: ireduce.i8 (I64)
    // skip 2 unless inst_predicate_7
    0x3007, // --> [null#00]
    0x01a0, 0x0000,
    // skip 2 unless inst_predicate_8
    // 0002f8: ireduce.i16 (I64)
    // skip 2 unless inst_predicate_8
    0x3008, // --> [null#00]
    // --> [null#00]
    0x01a0, 0x0000, // stop unless inst_predicate_9
    // stop unless inst_predicate_9
    0x1009, // --> [null#00] and stop
    // --> [null#00] and stop
    0x01a1, 0x0000,
    // end of ireduce.i16 (I64)
    // end of ireduce.i8 (I64)
    // 0002fe: regmove.i8 (I64)
    // --> [RexOp1rmov#89]
    0x001a, 0x0089, // --> [RexOp1rmov#89]
    0x001a, 0x0089, // --> [Op1rmov#89] and stop
    0x0019, 0x0089,
    // end of regmove.i8 (I64)
    // 000304: adjust_sp_down_imm (I64)
    // --> [RexOp1adjustsp_ib#d083]
    0x00d0, 0xd083, // --> [RexOp1adjustsp_id#d081] and stop
    0x00d3, 0xd081,
    // end of adjust_sp_down_imm (I64)
    // 000308: adjust_sp_up_imm (I64)
    // --> [RexOp1adjustsp_ib#8083]
    0x00d0, 0x8083, // --> [RexOp1adjustsp_id#8081] and stop
    0x00d3, 0x8081,
    // end of adjust_sp_up_imm (I64)
    // 00030c: brff (I64)
    // --> [RexOp1brfb#70]
    0x0144, 0x0070, // --> [Op1brfb#70]
    0x0142, 0x0070, // --> [RexOp2brfd#480]
    0x0148, 0x0480, // --> [Op2brfd#480] and stop
    0x0147, 0x0480,
    // end of brff (I64)
    // 000314: brif (I64)
    // --> [RexOp1brib#70]
    0x013c, 0x0070, // --> [Op1brib#70]
    0x013a, 0x0070, // --> [RexOp2brid#480]
    0x0140, 0x0480, // --> [Op2brid#480] and stop
    0x013f, 0x0480,
    // end of brif (I64)
    // 00031c: call (I64)
    // skip 2 unless inst_predicate_5
    0x3005, // --> [Op1call_id#e8]
    0x012c, 0x00e8, // stop unless PredicateView(10)
    0x1019, // --> [Op1call_plt_id#e8] and stop
    0x012f, 0x00e8,
    // end of call (I64)
    // 000322: copy_special (I64)
    // --> [RexOp1copysp#8089] and stop
    0x00c3, 0x8089,
    // end of copy_special (I64)
    // 000324: debugtrap (I64)
    // --> [debugtrap#00] and stop
    // 000324: debugtrap (I32)
    // --> [debugtrap#00] and stop
    0x016b, 0x0000,
    // end of debugtrap (I32)
    // end of debugtrap (I64)
    // 000326: f32const (I64)
    // stop unless inst_predicate_11
    0x100b, // --> [RexOp2f32imm_z#457]
    0x01aa, 0x0457, // --> [Op2f32imm_z#457] and stop
    0x01a7, 0x0457,
    // end of f32const (I64)
    // 00032b: f64const (I64)
    // stop unless inst_predicate_12
    0x100c, // --> [RexMp2f64imm_z#557]
    0x01ac, 0x0557, // --> [Mp2f64imm_z#557] and stop
    0x01a9, 0x0557,
    // end of f64const (I64)
    // 000330: jump (I64)
    // --> [Op1jmpb#eb]
    // 000330: jump (I32)
    // --> [Op1jmpb#eb]
    0x0136, 0x00eb, // --> [Op1jmpd#e9] and stop
    // --> [Op1jmpd#e9] and stop
    0x0139, 0x00e9,
    // end of jump (I32)
    // end of jump (I64)
    // 000334: return (I64)
    // --> [Op1ret#c3] and stop
    // 000334: return (I32)
    // --> [Op1ret#c3] and stop
    0x0135, 0x00c3,
    // end of return (I32)
    // end of return (I64)
    // 000336: sload32_complex (I64)
    // stop unless inst_predicate_1
    0x1001, // --> [RexOp1ldWithIndex#8063]
    0x003a, 0x8063, // --> [RexOp1ldWithIndexDisp8#8063]
    0x0042, 0x8063, // --> [RexOp1ldWithIndexDisp32#8063] and stop
    0x004b, 0x8063,
    // end of sload32_complex (I64)
    // 00033d: trap (I64)
    // --> [Op2trap#40b] and stop
    // 00033d: trap (I32)
    // --> [Op2trap#40b] and stop
    0x0169, 0x040b,
    // end of trap (I32)
    // end of trap (I64)
    // 00033f: trapff (I64)
    // --> [trapff#00] and stop
    // 00033f: trapff (I32)
    // --> [trapff#00] and stop
    0x016f, 0x0000,
    // end of trapff (I32)
    // end of trapff (I64)
    // 000341: trapif (I64)
    // --> [trapif#00] and stop
    // 000341: trapif (I32)
    // --> [trapif#00] and stop
    0x016d, 0x0000,
    // end of trapif (I32)
    // end of trapif (I64)
    // 000343: trueff (I64)
    // --> [RexOp2setf#490]
    0x0192, 0x0490,
    // --> [Op2setf_abcd#490] and stop
    // 000345: trueff (I32)
    // --> [Op2setf_abcd#490] and stop
    0x0191, 0x0490,
    // end of trueff (I32)
    // end of trueff (I64)
    // 000347: trueif (I64)
    // --> [RexOp2seti#490]
    0x018e, 0x0490,
    // --> [Op2seti_abcd#490] and stop
    // 000349: trueif (I32)
    // --> [Op2seti_abcd#490] and stop
    0x018d, 0x0490,
    // end of trueif (I32)
    // end of trueif (I64)
    // 00034b: band.f64 (I64)
    // --> [RexOp2fa#454]
    // 00034b: band.f32 (I64)
    // --> [RexOp2fa#454]
    0x01d0, 0x0454,
    // --> [Op2fa#454] and stop
    // --> [Op2fa#454] and stop
    // 00034d: band.f64 (I32)
    // --> [Op2fa#454] and stop
    // 00034d: band.f32 (I32)
    // --> [Op2fa#454] and stop
    0x01cf, 0x0454,
    // end of band.f32 (I32)
    // end of band.f64 (I32)
    // end of band.f32 (I64)
    // end of band.f64 (I64)
    // 00034f: band_not.f64 (I64)
    // --> [RexOp2fax#455]
    // 00034f: band_not.f32 (I64)
    // --> [RexOp2fax#455]
    0x01d4, 0x0455,
    // --> [Op2fax#455] and stop
    // --> [Op2fax#455] and stop
    // 000351: band_not.f64 (I32)
    // --> [Op2fax#455] and stop
    // 000351: band_not.f32 (I32)
    // --> [Op2fax#455] and stop
    0x01d3, 0x0455,
    // end of band_not.f32 (I32)
    // end of band_not.f64 (I32)
    // end of band_not.f32 (I64)
    // end of band_not.f64 (I64)
    // 000353: bitcast.f64 (I64)
    // stop unless inst_predicate_9
    0x1009, // --> [RexMp2frurm#856e] and stop
    0x01b1, 0x856e,
    // end of bitcast.f64 (I64)
    // 000356: bor.f64 (I64)
    // --> [RexOp2fa#456]
    // 000356: bor.f32 (I64)
    // --> [RexOp2fa#456]
    0x01d0, 0x0456,
    // --> [Op2fa#456] and stop
    // --> [Op2fa#456] and stop
    // 000358: bor.f64 (I32)
    // --> [Op2fa#456] and stop
    // 000358: bor.f32 (I32)
    // --> [Op2fa#456] and stop
    0x01cf, 0x0456,
    // end of bor.f32 (I32)
    // end of bor.f64 (I32)
    // end of bor.f32 (I64)
    // end of bor.f64 (I64)
    // 00035a: bxor.f64 (I64)
    // --> [RexOp2fa#457]
    // 00035a: bxor.f32 (I64)
    // --> [RexOp2fa#457]
    0x01d0, 0x0457,
    // --> [Op2fa#457] and stop
    // --> [Op2fa#457] and stop
    // 00035c: bxor.f64 (I32)
    // --> [Op2fa#457] and stop
    // 00035c: bxor.f32 (I32)
    // --> [Op2fa#457] and stop
    0x01cf, 0x0457,
    // end of bxor.f32 (I32)
    // end of bxor.f64 (I32)
    // end of bxor.f32 (I64)
    // end of bxor.f64 (I64)
    // 00035e: ceil.f64 (I64)
    // stop unless PredicateView(16)
    // 00035e: floor.f64 (I64)
    // stop unless PredicateView(16)
    // 00035e: nearest.f64 (I64)
    // stop unless PredicateView(16)
    // 00035e: trunc.f64 (I64)
    // stop unless PredicateView(16)
    0x101f,
    // --> [RexMp3furmi_rnd#d0b]
    // --> [RexMp3furmi_rnd#d0b]
    // --> [RexMp3furmi_rnd#d0b]
    // --> [RexMp3furmi_rnd#d0b]
    0x01c8, 0x0d0b,
    // --> [Mp3furmi_rnd#d0b] and stop
    // --> [Mp3furmi_rnd#d0b] and stop
    // --> [Mp3furmi_rnd#d0b] and stop
    // --> [Mp3furmi_rnd#d0b] and stop
    0x01c7, 0x0d0b,
    // end of trunc.f64 (I64)
    // end of nearest.f64 (I64)
    // end of floor.f64 (I64)
    // end of ceil.f64 (I64)
    // 000363: copy.f64 (I64)
    // --> [RexOp2furm#428]
    // 000363: copy.f32 (I64)
    // --> [RexOp2furm#428]
    0x01b8, 0x0428,
    // --> [Op2furm#428] and stop
    // --> [Op2furm#428] and stop
    // 000365: copy.f64 (I32)
    // --> [Op2furm#428] and stop
    // 000365: copy.f32 (I32)
    // --> [Op2furm#428] and stop
    0x01b7, 0x0428,
    // end of copy.f32 (I32)
    // end of copy.f64 (I32)
    // end of copy.f32 (I64)
    // end of copy.f64 (I64)
    // 000367: fadd.f64 (I64)
    // --> [RexMp2fa#758]
    0x01cc, 0x0758,
    // --> [Mp2fa#758] and stop
    // 000369: fadd.f64 (I32)
    // --> [Mp2fa#758] and stop
    0x01cb, 0x0758,
    // end of fadd.f64 (I32)
    // end of fadd.f64 (I64)
    // 00036b: fcmp.f64 (I64)
    // --> [RexMp2fcscc#52e]
    0x01dc, 0x052e,
    // --> [Mp2fcscc#52e] and stop
    // 00036d: fcmp.f64 (I32)
    // --> [Mp2fcscc#52e] and stop
    0x01db, 0x052e,
    // end of fcmp.f64 (I32)
    // end of fcmp.f64 (I64)
    // 00036f: fcvt_from_sint.f64 (I64)
    // skip 4 unless inst_predicate_8
    0x5008, // --> [RexMp2frurm#72a]
    0x01b0, 0x072a, // --> [Mp2frurm#72a]
    0x01ae, 0x072a, // stop unless inst_predicate_9
    0x1009, // --> [RexMp2frurm#872a] and stop
    0x01b1, 0x872a,
    // end of fcvt_from_sint.f64 (I64)
    // 000377: fdiv.f64 (I64)
    // --> [RexMp2fa#75e]
    0x01cc, 0x075e,
    // --> [Mp2fa#75e] and stop
    // 000379: fdiv.f64 (I32)
    // --> [Mp2fa#75e] and stop
    0x01cb, 0x075e,
    // end of fdiv.f64 (I32)
    // end of fdiv.f64 (I64)
    // 00037b: ffcmp.f64 (I64)
    // --> [RexMp2fcmp#52e]
    0x01e4, 0x052e,
    // --> [Mp2fcmp#52e] and stop
    // 00037d: ffcmp.f64 (I32)
    // --> [Mp2fcmp#52e] and stop
    0x01e3, 0x052e,
    // end of ffcmp.f64 (I32)
    // end of ffcmp.f64 (I64)
    // 00037f: fill.f64 (I64)
    // --> [RexMp2ffillSib32#710]
    0x0106, 0x0710,
    // --> [Mp2ffillSib32#710] and stop
    // 000381: fill.f64 (I32)
    // --> [Mp2ffillSib32#710] and stop
    0x0105, 0x0710,
    // end of fill.f64 (I32)
    // end of fill.f64 (I64)
    // 000383: fmul.f64 (I64)
    // --> [RexMp2fa#759]
    0x01cc, 0x0759,
    // --> [Mp2fa#759] and stop
    // 000385: fmul.f64 (I32)
    // --> [Mp2fa#759] and stop
    0x01cb, 0x0759,
    // end of fmul.f64 (I32)
    // end of fmul.f64 (I64)
    // 000387: fpromote.f64 (I64)
    // stop unless inst_predicate_13
    0x100d, // --> [RexMp2furm#65a]
    0x01c0, 0x065a, // --> [Mp2furm#65a] and stop
    0x01bf, 0x065a,
    // end of fpromote.f64 (I64)
    // 00038c: fsub.f64 (I64)
    // --> [RexMp2fa#75c]
    0x01cc, 0x075c,
    // --> [Mp2fa#75c] and stop
    // 00038e: fsub.f64 (I32)
    // --> [Mp2fa#75c] and stop
    0x01cb, 0x075c,
    // end of fsub.f64 (I32)
    // end of fsub.f64 (I64)
    // 000390: load.f64 (I64)
    // --> [RexMp2fld#710]
    0x00d6, 0x0710, // --> [Mp2fld#710]
    0x00d4, 0x0710, // --> [RexMp2fldDisp8#710]
    0x00da, 0x0710, // --> [Mp2fldDisp8#710]
    0x00d8, 0x0710, // --> [RexMp2fldDisp32#710]
    0x00de, 0x0710, // --> [Mp2fldDisp32#710] and stop
    0x00dd, 0x0710,
    // end of load.f64 (I64)
    // 00039c: load_complex.f64 (I64)
    // --> [RexMp2fldWithIndex#710]
    0x00e2, 0x0710, // --> [Mp2fldWithIndex#710]
    0x00e0, 0x0710, // --> [RexMp2fldWithIndexDisp8#710]
    0x00e6, 0x0710, // --> [Mp2fldWithIndexDisp8#710]
    0x00e4, 0x0710, // --> [RexMp2fldWithIndexDisp32#710]
    0x00ea, 0x0710, // --> [Mp2fldWithIndexDisp32#710] and stop
    0x00e9, 0x0710,
    // end of load_complex.f64 (I64)
    // 0003a8: regfill.f64 (I64)
    // --> [RexMp2fregfill32#710]
    0x010a, 0x0710,
    // --> [Mp2fregfill32#710] and stop
    // 0003aa: regfill.f64 (I32)
    // --> [Mp2fregfill32#710] and stop
    0x0109, 0x0710,
    // end of regfill.f64 (I32)
    // end of regfill.f64 (I64)
    // 0003ac: regmove.f64 (I64)
    // --> [RexOp2frmov#428] and stop
    // 0003ac: regmove.f32 (I64)
    // --> [RexOp2frmov#428] and stop
    0x01bd, 0x0428,
    // end of regmove.f32 (I64)
    // end of regmove.f64 (I64)
    // 0003ae: regspill.f64 (I64)
    // --> [RexMp2fregspill32#711]
    0x0112, 0x0711,
    // --> [Mp2fregspill32#711] and stop
    // 0003b0: regspill.f64 (I32)
    // --> [Mp2fregspill32#711] and stop
    0x0111, 0x0711,
    // end of regspill.f64 (I32)
    // end of regspill.f64 (I64)
    // 0003b2: spill.f64 (I64)
    // --> [RexMp2fspillSib32#711]
    0x010e, 0x0711,
    // --> [Mp2fspillSib32#711] and stop
    // 0003b4: spill.f64 (I32)
    // --> [Mp2fspillSib32#711] and stop
    0x010d, 0x0711,
    // end of spill.f64 (I32)
    // end of spill.f64 (I64)
    // 0003b6: sqrt.f64 (I64)
    // --> [RexMp2furm#751]
    0x01c0, 0x0751,
    // --> [Mp2furm#751] and stop
    // 0003b8: sqrt.f64 (I32)
    // --> [Mp2furm#751] and stop
    0x01bf, 0x0751,
    // end of sqrt.f64 (I32)
    // end of sqrt.f64 (I64)
    // 0003ba: store.f64 (I64)
    // --> [RexMp2fst#711]
    0x00ee, 0x0711, // --> [Mp2fst#711]
    0x00ec, 0x0711, // --> [RexMp2fstDisp8#711]
    0x00f2, 0x0711, // --> [Mp2fstDisp8#711]
    0x00f0, 0x0711, // --> [RexMp2fstDisp32#711]
    0x00f6, 0x0711, // --> [Mp2fstDisp32#711] and stop
    0x00f5, 0x0711,
    // end of store.f64 (I64)
    // 0003c6: store_complex.f64 (I64)
    // --> [RexMp2fstWithIndex#711]
    0x00fa, 0x0711, // --> [Mp2fstWithIndex#711]
    0x00f8, 0x0711, // --> [RexMp2fstWithIndexDisp8#711]
    0x00fe, 0x0711, // --> [Mp2fstWithIndexDisp8#711]
    0x00fc, 0x0711, // --> [RexMp2fstWithIndexDisp32#711]
    0x0102, 0x0711, // --> [Mp2fstWithIndexDisp32#711] and stop
    0x0101, 0x0711,
    // end of store_complex.f64 (I64)
    // 0003d2: x86_fmax.f64 (I64)
    // --> [RexMp2fa#75f]
    0x01cc, 0x075f,
    // --> [Mp2fa#75f] and stop
    // 0003d4: x86_fmax.f64 (I32)
    // --> [Mp2fa#75f] and stop
    0x01cb, 0x075f,
    // end of x86_fmax.f64 (I32)
    // end of x86_fmax.f64 (I64)
    // 0003d6: x86_fmin.f64 (I64)
    // --> [RexMp2fa#75d]
    0x01cc, 0x075d,
    // --> [Mp2fa#75d] and stop
    // 0003d8: x86_fmin.f64 (I32)
    // --> [Mp2fa#75d] and stop
    0x01cb, 0x075d,
    // end of x86_fmin.f64 (I32)
    // end of x86_fmin.f64 (I64)
    // 0003da: bitcast.f32 (I64)
    // stop unless inst_predicate_8
    0x1008, // --> [RexMp2frurm#56e]
    0x01b0, 0x056e, // --> [Mp2frurm#56e] and stop
    0x01af, 0x056e,
    // end of bitcast.f32 (I64)
    // 0003df: ceil.f32 (I64)
    // stop unless PredicateView(16)
    // 0003df: floor.f32 (I64)
    // stop unless PredicateView(16)
    // 0003df: nearest.f32 (I64)
    // stop unless PredicateView(16)
    // 0003df: trunc.f32 (I64)
    // stop unless PredicateView(16)
    0x101f,
    // --> [RexMp3furmi_rnd#d0a]
    // --> [RexMp3furmi_rnd#d0a]
    // --> [RexMp3furmi_rnd#d0a]
    // --> [RexMp3furmi_rnd#d0a]
    0x01c8, 0x0d0a,
    // --> [Mp3furmi_rnd#d0a] and stop
    // --> [Mp3furmi_rnd#d0a] and stop
    // --> [Mp3furmi_rnd#d0a] and stop
    // --> [Mp3furmi_rnd#d0a] and stop
    0x01c7, 0x0d0a,
    // end of trunc.f32 (I64)
    // end of nearest.f32 (I64)
    // end of floor.f32 (I64)
    // end of ceil.f32 (I64)
    // 0003e4: fadd.f32 (I64)
    // --> [RexMp2fa#658]
    0x01cc, 0x0658,
    // --> [Mp2fa#658] and stop
    // 0003e6: fadd.f32 (I32)
    // --> [Mp2fa#658] and stop
    0x01cb, 0x0658,
    // end of fadd.f32 (I32)
    // end of fadd.f32 (I64)
    // 0003e8: fcmp.f32 (I64)
    // --> [RexOp2fcscc#42e]
    0x01d8, 0x042e,
    // --> [Op2fcscc#42e] and stop
    // 0003ea: fcmp.f32 (I32)
    // --> [Op2fcscc#42e] and stop
    0x01d7, 0x042e,
    // end of fcmp.f32 (I32)
    // end of fcmp.f32 (I64)
    // 0003ec: fcvt_from_sint.f32 (I64)
    // skip 4 unless inst_predicate_8
    0x5008, // --> [RexMp2frurm#62a]
    0x01b0, 0x062a, // --> [Mp2frurm#62a]
    0x01ae, 0x062a, // stop unless inst_predicate_9
    0x1009, // --> [RexMp2frurm#862a] and stop
    0x01b1, 0x862a,
    // end of fcvt_from_sint.f32 (I64)
    // 0003f4: fdemote.f32 (I64)
    // stop unless inst_predicate_14
    0x100e, // --> [RexMp2furm#75a]
    0x01c0, 0x075a, // --> [Mp2furm#75a] and stop
    0x01bf, 0x075a,
    // end of fdemote.f32 (I64)
    // 0003f9: fdiv.f32 (I64)
    // --> [RexMp2fa#65e]
    0x01cc, 0x065e,
    // --> [Mp2fa#65e] and stop
    // 0003fb: fdiv.f32 (I32)
    // --> [Mp2fa#65e] and stop
    0x01cb, 0x065e,
    // end of fdiv.f32 (I32)
    // end of fdiv.f32 (I64)
    // 0003fd: ffcmp.f32 (I64)
    // --> [RexOp2fcmp#42e]
    0x01e0, 0x042e,
    // --> [Op2fcmp#42e] and stop
    // 0003ff: ffcmp.f32 (I32)
    // --> [Op2fcmp#42e] and stop
    0x01df, 0x042e,
    // end of ffcmp.f32 (I32)
    // end of ffcmp.f32 (I64)
    // 000401: fill.f32 (I64)
    // --> [RexMp2ffillSib32#610]
    0x0106, 0x0610,
    // --> [Mp2ffillSib32#610] and stop
    // 000403: fill.f32 (I32)
    // --> [Mp2ffillSib32#610] and stop
    0x0105, 0x0610,
    // end of fill.f32 (I32)
    // end of fill.f32 (I64)
    // 000405: fmul.f32 (I64)
    // --> [RexMp2fa#659]
    0x01cc, 0x0659,
    // --> [Mp2fa#659] and stop
    // 000407: fmul.f32 (I32)
    // --> [Mp2fa#659] and stop
    0x01cb, 0x0659,
    // end of fmul.f32 (I32)
    // end of fmul.f32 (I64)
    // 000409: fsub.f32 (I64)
    // --> [RexMp2fa#65c]
    0x01cc, 0x065c,
    // --> [Mp2fa#65c] and stop
    // 00040b: fsub.f32 (I32)
    // --> [Mp2fa#65c] and stop
    0x01cb, 0x065c,
    // end of fsub.f32 (I32)
    // end of fsub.f32 (I64)
    // 00040d: load.f32 (I64)
    // --> [RexMp2fld#610]
    0x00d6, 0x0610, // --> [Mp2fld#610]
    0x00d4, 0x0610, // --> [RexMp2fldDisp8#610]
    0x00da, 0x0610, // --> [Mp2fldDisp8#610]
    0x00d8, 0x0610, // --> [RexMp2fldDisp32#610]
    0x00de, 0x0610, // --> [Mp2fldDisp32#610] and stop
    0x00dd, 0x0610,
    // end of load.f32 (I64)
    // 000419: load_complex.f32 (I64)
    // --> [RexMp2fldWithIndex#610]
    0x00e2, 0x0610, // --> [Mp2fldWithIndex#610]
    0x00e0, 0x0610, // --> [RexMp2fldWithIndexDisp8#610]
    0x00e6, 0x0610, // --> [Mp2fldWithIndexDisp8#610]
    0x00e4, 0x0610, // --> [RexMp2fldWithIndexDisp32#610]
    0x00ea, 0x0610, // --> [Mp2fldWithIndexDisp32#610] and stop
    0x00e9, 0x0610,
    // end of load_complex.f32 (I64)
    // 000425: regfill.f32 (I64)
    // --> [RexMp2fregfill32#610]
    0x010a, 0x0610,
    // --> [Mp2fregfill32#610] and stop
    // 000427: regfill.f32 (I32)
    // --> [Mp2fregfill32#610] and stop
    0x0109, 0x0610,
    // end of regfill.f32 (I32)
    // end of regfill.f32 (I64)
    // 000429: regspill.f32 (I64)
    // --> [RexMp2fregspill32#611]
    0x0112, 0x0611,
    // --> [Mp2fregspill32#611] and stop
    // 00042b: regspill.f32 (I32)
    // --> [Mp2fregspill32#611] and stop
    0x0111, 0x0611,
    // end of regspill.f32 (I32)
    // end of regspill.f32 (I64)
    // 00042d: spill.f32 (I64)
    // --> [RexMp2fspillSib32#611]
    0x010e, 0x0611,
    // --> [Mp2fspillSib32#611] and stop
    // 00042f: spill.f32 (I32)
    // --> [Mp2fspillSib32#611] and stop
    0x010d, 0x0611,
    // end of spill.f32 (I32)
    // end of spill.f32 (I64)
    // 000431: sqrt.f32 (I64)
    // --> [RexMp2furm#651]
    0x01c0, 0x0651,
    // --> [Mp2furm#651] and stop
    // 000433: sqrt.f32 (I32)
    // --> [Mp2furm#651] and stop
    0x01bf, 0x0651,
    // end of sqrt.f32 (I32)
    // end of sqrt.f32 (I64)
    // 000435: store.f32 (I64)
    // --> [RexMp2fst#611]
    0x00ee, 0x0611, // --> [Mp2fst#611]
    0x00ec, 0x0611, // --> [RexMp2fstDisp8#611]
    0x00f2, 0x0611, // --> [Mp2fstDisp8#611]
    0x00f0, 0x0611, // --> [RexMp2fstDisp32#611]
    0x00f6, 0x0611, // --> [Mp2fstDisp32#611] and stop
    0x00f5, 0x0611,
    // end of store.f32 (I64)
    // 000441: store_complex.f32 (I64)
    // --> [RexMp2fstWithIndex#611]
    0x00fa, 0x0611, // --> [Mp2fstWithIndex#611]
    0x00f8, 0x0611, // --> [RexMp2fstWithIndexDisp8#611]
    0x00fe, 0x0611, // --> [Mp2fstWithIndexDisp8#611]
    0x00fc, 0x0611, // --> [RexMp2fstWithIndexDisp32#611]
    0x0102, 0x0611, // --> [Mp2fstWithIndexDisp32#611] and stop
    0x0101, 0x0611,
    // end of store_complex.f32 (I64)
    // 00044d: x86_fmax.f32 (I64)
    // --> [RexMp2fa#65f]
    0x01cc, 0x065f,
    // --> [Mp2fa#65f] and stop
    // 00044f: x86_fmax.f32 (I32)
    // --> [Mp2fa#65f] and stop
    0x01cb, 0x065f,
    // end of x86_fmax.f32 (I32)
    // end of x86_fmax.f32 (I64)
    // 000451: x86_fmin.f32 (I64)
    // --> [RexMp2fa#65d]
    0x01cc, 0x065d,
    // --> [Mp2fa#65d] and stop
    // 000453: x86_fmin.f32 (I32)
    // --> [Mp2fa#65d] and stop
    0x01cb, 0x065d,
    // end of x86_fmin.f32 (I32)
    // end of x86_fmin.f32 (I64)
    // 000455: adjust_sp_down.i32 (I32)
    // --> [Op1adjustsp#29] and stop
    0x00c9, 0x0029,
    // end of adjust_sp_down.i32 (I32)
    // 000457: band_imm.i32 (I32)
    // --> [Op1r_ib#4083]
    0x001c, 0x4083, // --> [Op1r_id#4081] and stop
    0x0021, 0x4081,
    // end of band_imm.i32 (I32)
    // 00045b: bint.i32 (I32)
    // stop unless inst_predicate_6
    0x1006, // --> [Op2urm_noflags_abcd#4b6] and stop
    0x019d, 0x04b6,
    // end of bint.i32 (I32)
    // 00045e: bitcast.i32 (I32)
    // stop unless inst_predicate_13
    0x100d, // --> [Mp2rfumr#57e] and stop
    0x01b3, 0x057e,
    // end of bitcast.i32 (I32)
    // 000461: bor_imm.i32 (I32)
    // --> [Op1r_ib#1083]
    0x001c, 0x1083, // --> [Op1r_id#1081] and stop
    0x0021, 0x1081,
    // end of bor_imm.i32 (I32)
    // 000465: brnz.i32 (I32)
    // --> [Op1tjccb#75]
    0x014a, 0x0075, // --> [Op1tjccd#85] and stop
    0x014f, 0x0085,
    // end of brnz.i32 (I32)
    // 000469: brz.i32 (I32)
    // --> [Op1tjccb#74]
    0x014a, 0x0074, // --> [Op1tjccd#84] and stop
    0x014f, 0x0084,
    // end of brz.i32 (I32)
    // 00046d: bxor_imm.i32 (I32)
    // --> [Op1r_ib#6083]
    0x001c, 0x6083, // --> [Op1r_id#6081] and stop
    0x0021, 0x6081,
    // end of bxor_imm.i32 (I32)
    // 000471: clz.i32 (I32)
    // stop unless PredicateView(14)
    0x101d, // --> [Mp2urm#6bd] and stop
    0x0035, 0x06bd,
    // end of clz.i32 (I32)
    // 000474: ctz.i32 (I32)
    // stop unless PredicateView(13)
    0x101c, // --> [Mp2urm#6bc] and stop
    0x0035, 0x06bc,
    // end of ctz.i32 (I32)
    // 000477: func_addr.i32 (I32)
    // skip 2 unless PredicateView(11)
    0x301a, // --> [Op1fnaddr4#b8]
    0x0114, 0x00b8, // stop unless PredicateView(9)
    0x1018, // --> [Op1allones_fnaddr4#b8] and stop
    0x0119, 0x00b8,
    // end of func_addr.i32 (I32)
    // 00047d: iadd_imm.i32 (I32)
    // --> [Op1r_ib#83]
    0x001c, 0x0083, // --> [Op1r_id#81] and stop
    0x0021, 0x0081,
    // end of iadd_imm.i32 (I32)
    // 000481: icmp_imm.i32 (I32)
    // --> [Op1icscc_ib#7083]
    0x0174, 0x7083, // --> [Op1icscc_id#7081] and stop
    0x0179, 0x7081,
    // end of icmp_imm.i32 (I32)
    // 000485: ifcmp_imm.i32 (I32)
    // --> [Op1rcmp_ib#7083]
    0x0180, 0x7083, // --> [Op1rcmp_id#7081] and stop
    0x0185, 0x7081,
    // end of ifcmp_imm.i32 (I32)
    // 000489: ifcmp_sp.i32 (I32)
    // --> [Op1rcmp_sp#39] and stop
    0x0189, 0x0039,
    // end of ifcmp_sp.i32 (I32)
    // 00048b: istore16.i32 (I32)
    // --> [Mp1st#189]
    0x0078, 0x0189, // --> [Mp1stDisp8#189]
    0x0080, 0x0189, // --> [Mp1stDisp32#189] and stop
    0x0089, 0x0189,
    // end of istore16.i32 (I32)
    // 000491: istore16_complex.i32 (I32)
    // stop unless inst_predicate_2
    0x1002, // --> [Mp1stWithIndex#189]
    0x0054, 0x0189, // --> [Mp1stWithIndexDisp8#189]
    0x005c, 0x0189, // --> [Mp1stWithIndexDisp32#189] and stop
    0x0065, 0x0189,
    // end of istore16_complex.i32 (I32)
    // 000498: istore8.i32 (I32)
    // --> [Op1st_abcd#88]
    0x008c, 0x0088, // --> [Op1stDisp8_abcd#88]
    0x008e, 0x0088, // --> [Op1stDisp32_abcd#88] and stop
    0x0091, 0x0088,
    // end of istore8.i32 (I32)
    // 00049e: istore8_complex.i32 (I32)
    // stop unless inst_predicate_2
    0x1002, // --> [Op1stWithIndex_abcd#88]
    0x0068, 0x0088, // --> [Op1stWithIndexDisp8_abcd#88]
    0x006c, 0x0088, // --> [Op1stWithIndexDisp32_abcd#88] and stop
    0x0071, 0x0088,
    // end of istore8_complex.i32 (I32)
    // 0004a5: jump_table_base.i32 (I32)
    // --> [Op1jt_base#8d] and stop
    0x0163, 0x008d,
    // end of jump_table_base.i32 (I32)
    // 0004a7: jump_table_entry.i32 (I32)
    // --> [Op1jt_entry#8b] and stop
    0x015f, 0x008b,
    // end of jump_table_entry.i32 (I32)
    // 0004a9: load.i32 (I32)
    // --> [Op1ld#8b]
    0x009a, 0x008b, // --> [Op1ldDisp8#8b]
    0x00a2, 0x008b, // --> [Op1ldDisp32#8b] and stop
    0x00ab, 0x008b,
    // end of load.i32 (I32)
    // 0004af: load_complex.i32 (I32)
    // stop unless inst_predicate_1
    0x1001, // --> [Op1ldWithIndex#8b]
    0x0038, 0x008b, // --> [Op1ldWithIndexDisp8#8b]
    0x0040, 0x008b, // --> [Op1ldWithIndexDisp32#8b] and stop
    0x0049, 0x008b,
    // end of load_complex.i32 (I32)
    // 0004b6: popcnt.i32 (I32)
    // stop unless PredicateView(15)
    0x101e, // --> [Mp2urm#6b8] and stop
    0x0035, 0x06b8,
    // end of popcnt.i32 (I32)
    // 0004b9: sextend.i32 (I32)
    // skip 2 unless inst_predicate_10
    0x300a, // --> [Op2urm_noflags_abcd#4be]
    0x019c, 0x04be, // stop unless inst_predicate_7
    0x1007, // --> [Op2urm_noflags#4bf] and stop
    0x01a3, 0x04bf,
    // end of sextend.i32 (I32)
    // 0004bf: sload16.i32 (I32)
    // --> [Op2ld#4bf]
    0x009e, 0x04bf, // --> [Op2ldDisp8#4bf]
    0x00a6, 0x04bf, // --> [Op2ldDisp32#4bf] and stop
    0x00af, 0x04bf,
    // end of sload16.i32 (I32)
    // 0004c5: sload16_complex.i32 (I32)
    // stop unless inst_predicate_1
    0x1001, // --> [Op2ldWithIndex#4bf]
    0x003c, 0x04bf, // --> [Op2ldWithIndexDisp8#4bf]
    0x0044, 0x04bf, // --> [Op2ldWithIndexDisp32#4bf] and stop
    0x004d, 0x04bf,
    // end of sload16_complex.i32 (I32)
    // 0004cc: sload8.i32 (I32)
    // --> [Op2ld#4be]
    0x009e, 0x04be, // --> [Op2ldDisp8#4be]
    0x00a6, 0x04be, // --> [Op2ldDisp32#4be] and stop
    0x00af, 0x04be,
    // end of sload8.i32 (I32)
    // 0004d2: sload8_complex.i32 (I32)
    // stop unless inst_predicate_1
    0x1001, // --> [Op2ldWithIndex#4be]
    0x003c, 0x04be, // --> [Op2ldWithIndexDisp8#4be]
    0x0044, 0x04be, // --> [Op2ldWithIndexDisp32#4be] and stop
    0x004d, 0x04be,
    // end of sload8_complex.i32 (I32)
    // 0004d9: stack_addr.i32 (I32)
    // --> [Op1spaddr4_id#8d] and stop
    0x0129, 0x008d,
    // end of stack_addr.i32 (I32)
    // 0004db: store.i32 (I32)
    // --> [Op1st#89]
    0x0074, 0x0089, // --> [Op1stDisp8#89]
    0x007c, 0x0089, // --> [Op1stDisp32#89] and stop
    0x0085, 0x0089,
    // end of store.i32 (I32)
    // 0004e1: store_complex.i32 (I32)
    // stop unless inst_predicate_2
    0x1002, // --> [Op1stWithIndex#89]
    0x0050, 0x0089, // --> [Op1stWithIndexDisp8#89]
    0x0058, 0x0089, // --> [Op1stWithIndexDisp32#89] and stop
    0x0061, 0x0089,
    // end of store_complex.i32 (I32)
    // 0004e8: symbol_value.i32 (I32)
    // stop unless PredicateView(12)
    0x101b, // --> [Op1gvaddr4#b8] and stop
    0x0121, 0x00b8,
    // end of symbol_value.i32 (I32)
    // 0004eb: uextend.i32 (I32)
    // skip 2 unless inst_predicate_10
    0x300a, // --> [Op2urm_noflags_abcd#4b6]
    0x019c, 0x04b6, // stop unless inst_predicate_7
    0x1007, // --> [Op2urm_noflags#4b7] and stop
    0x01a3, 0x04b7,
    // end of uextend.i32 (I32)
    // 0004f1: uload16.i32 (I32)
    // --> [Op2ld#4b7]
    0x009e, 0x04b7, // --> [Op2ldDisp8#4b7]
    0x00a6, 0x04b7, // --> [Op2ldDisp32#4b7] and stop
    0x00af, 0x04b7,
    // end of uload16.i32 (I32)
    // 0004f7: uload16_complex.i32 (I32)
    // stop unless inst_predicate_1
    0x1001, // --> [Op2ldWithIndex#4b7]
    0x003c, 0x04b7, // --> [Op2ldWithIndexDisp8#4b7]
    0x0044, 0x04b7, // --> [Op2ldWithIndexDisp32#4b7] and stop
    0x004d, 0x04b7,
    // end of uload16_complex.i32 (I32)
    // 0004fe: uload8.i32 (I32)
    // --> [Op2ld#4b6]
    0x009e, 0x04b6, // --> [Op2ldDisp8#4b6]
    0x00a6, 0x04b6, // --> [Op2ldDisp32#4b6] and stop
    0x00af, 0x04b6,
    // end of uload8.i32 (I32)
    // 000504: uload8_complex.i32 (I32)
    // stop unless inst_predicate_1
    0x1001, // --> [Op2ldWithIndex#4b6]
    0x003c, 0x04b6, // --> [Op2ldWithIndexDisp8#4b6]
    0x0044, 0x04b6, // --> [Op2ldWithIndexDisp32#4b6] and stop
    0x004d, 0x04b6,
    // end of uload8_complex.i32 (I32)
    // 00050b: x86_cvtt2si.i32 (I32)
    // skip 2 unless inst_predicate_13
    0x300d, // --> [Mp2rfurm#62c]
    0x01c2, 0x062c, // stop unless inst_predicate_14
    0x100e, // --> [Mp2rfurm#72c] and stop
    0x01c3, 0x072c,
    // end of x86_cvtt2si.i32 (I32)
    // 000511: brnz.b1 (I32)
    // --> [Op1t8jccd_long#85]
    0x0152, 0x0085, // --> [Op1t8jccb_abcd#75]
    0x0154, 0x0075, // --> [Op1t8jccd_abcd#85] and stop
    0x0159, 0x0085,
    // end of brnz.b1 (I32)
    // 000517: brz.b1 (I32)
    // --> [Op1t8jccd_long#84]
    0x0152, 0x0084, // --> [Op1t8jccb_abcd#74]
    0x0154, 0x0074, // --> [Op1t8jccd_abcd#84] and stop
    0x0159, 0x0084,
    // end of brz.b1 (I32)
    // 00051d: ireduce.i8 (I32)
    // skip 2 unless inst_predicate_7
    0x3007, // --> [null#00]
    0x01a0, 0x0000,
    // stop unless inst_predicate_8
    // 000520: ireduce.i16 (I32)
    // stop unless inst_predicate_8
    0x1008, // --> [null#00] and stop
    // --> [null#00] and stop
    0x01a1, 0x0000,
    // end of ireduce.i16 (I32)
    // end of ireduce.i8 (I32)
    // 000523: regmove.i8 (I32)
    // --> [Op1rmov#89]
    0x0018, 0x0089, // --> [Op1rmov#89] and stop
    0x0019, 0x0089,
    // end of regmove.i8 (I32)
    // 000527: adjust_sp_down_imm (I32)
    // --> [Op1adjustsp_ib#5083]
    0x00cc, 0x5083, // --> [Op1adjustsp_id#5081] and stop
    0x00cf, 0x5081,
    // end of adjust_sp_down_imm (I32)
    // 00052b: adjust_sp_up_imm (I32)
    // --> [Op1adjustsp_ib#83]
    0x00cc, 0x0083, // --> [Op1adjustsp_id#81] and stop
    0x00cf, 0x0081,
    // end of adjust_sp_up_imm (I32)
    // 00052f: brff (I32)
    // --> [Op1brfb#70]
    0x0142, 0x0070, // --> [Op2brfd#480] and stop
    0x0147, 0x0480,
    // end of brff (I32)
    // 000533: brif (I32)
    // --> [Op1brib#70]
    0x013a, 0x0070, // --> [Op2brid#480] and stop
    0x013f, 0x0480,
    // end of brif (I32)
    // 000537: call (I32)
    // --> [Op1call_id#e8] and stop
    0x012d, 0x00e8,
    // end of call (I32)
    // 000539: copy_special (I32)
    // --> [Op1copysp#89] and stop
    0x00c5, 0x0089,
    // end of copy_special (I32)
    // 00053b: f32const (I32)
    // stop unless inst_predicate_11
    0x100b, // --> [Op2f32imm_z#457] and stop
    0x01a7, 0x0457,
    // end of f32const (I32)
    // 00053e: f64const (I32)
    // stop unless inst_predicate_12
    0x100c, // --> [Mp2f64imm_z#557] and stop
    0x01a9, 0x0557,
    // end of f64const (I32)
    // 000541: ceil.f64 (I32)
    // stop unless PredicateView(16)
    // 000541: floor.f64 (I32)
    // stop unless PredicateView(16)
    // 000541: nearest.f64 (I32)
    // stop unless PredicateView(16)
    // 000541: trunc.f64 (I32)
    // stop unless PredicateView(16)
    0x101f,
    // --> [Mp3furmi_rnd#d0b] and stop
    // --> [Mp3furmi_rnd#d0b] and stop
    // --> [Mp3furmi_rnd#d0b] and stop
    // --> [Mp3furmi_rnd#d0b] and stop
    0x01c7, 0x0d0b,
    // end of trunc.f64 (I32)
    // end of nearest.f64 (I32)
    // end of floor.f64 (I32)
    // end of ceil.f64 (I32)
    // 000544: fcvt_from_sint.f64 (I32)
    // stop unless inst_predicate_8
    0x1008, // --> [Mp2frurm#72a] and stop
    0x01af, 0x072a,
    // end of fcvt_from_sint.f64 (I32)
    // 000547: fpromote.f64 (I32)
    // stop unless inst_predicate_13
    0x100d, // --> [Mp2furm#65a] and stop
    0x01bf, 0x065a,
    // end of fpromote.f64 (I32)
    // 00054a: load.f64 (I32)
    // --> [Mp2fld#710]
    0x00d4, 0x0710, // --> [Mp2fldDisp8#710]
    0x00d8, 0x0710, // --> [Mp2fldDisp32#710] and stop
    0x00dd, 0x0710,
    // end of load.f64 (I32)
    // 000550: load_complex.f64 (I32)
    // --> [Mp2fldWithIndex#710]
    0x00e0, 0x0710, // --> [Mp2fldWithIndexDisp8#710]
    0x00e4, 0x0710, // --> [Mp2fldWithIndexDisp32#710] and stop
    0x00e9, 0x0710,
    // end of load_complex.f64 (I32)
    // 000556: regmove.f64 (I32)
    // --> [Op2frmov#428] and stop
    // 000556: regmove.f32 (I32)
    // --> [Op2frmov#428] and stop
    0x01bb, 0x0428,
    // end of regmove.f32 (I32)
    // end of regmove.f64 (I32)
    // 000558: store.f64 (I32)
    // --> [Mp2fst#711]
    0x00ec, 0x0711, // --> [Mp2fstDisp8#711]
    0x00f0, 0x0711, // --> [Mp2fstDisp32#711] and stop
    0x00f5, 0x0711,
    // end of store.f64 (I32)
    // 00055e: store_complex.f64 (I32)
    // --> [Mp2fstWithIndex#711]
    0x00f8, 0x0711, // --> [Mp2fstWithIndexDisp8#711]
    0x00fc, 0x0711, // --> [Mp2fstWithIndexDisp32#711] and stop
    0x0101, 0x0711,
    // end of store_complex.f64 (I32)
    // 000564: bitcast.f32 (I32)
    // stop unless inst_predicate_8
    0x1008, // --> [Mp2frurm#56e] and stop
    0x01af, 0x056e,
    // end of bitcast.f32 (I32)
    // 000567: ceil.f32 (I32)
    // stop unless PredicateView(16)
    // 000567: floor.f32 (I32)
    // stop unless PredicateView(16)
    // 000567: nearest.f32 (I32)
    // stop unless PredicateView(16)
    // 000567: trunc.f32 (I32)
    // stop unless PredicateView(16)
    0x101f,
    // --> [Mp3furmi_rnd#d0a] and stop
    // --> [Mp3furmi_rnd#d0a] and stop
    // --> [Mp3furmi_rnd#d0a] and stop
    // --> [Mp3furmi_rnd#d0a] and stop
    0x01c7, 0x0d0a,
    // end of trunc.f32 (I32)
    // end of nearest.f32 (I32)
    // end of floor.f32 (I32)
    // end of ceil.f32 (I32)
    // 00056a: fcvt_from_sint.f32 (I32)
    // stop unless inst_predicate_8
    0x1008, // --> [Mp2frurm#62a] and stop
    0x01af, 0x062a,
    // end of fcvt_from_sint.f32 (I32)
    // 00056d: fdemote.f32 (I32)
    // stop unless inst_predicate_14
    0x100e, // --> [Mp2furm#75a] and stop
    0x01bf, 0x075a,
    // end of fdemote.f32 (I32)
    // 000570: load.f32 (I32)
    // --> [Mp2fld#610]
    0x00d4, 0x0610, // --> [Mp2fldDisp8#610]
    0x00d8, 0x0610, // --> [Mp2fldDisp32#610] and stop
    0x00dd, 0x0610,
    // end of load.f32 (I32)
    // 000576: load_complex.f32 (I32)
    // --> [Mp2fldWithIndex#610]
    0x00e0, 0x0610, // --> [Mp2fldWithIndexDisp8#610]
    0x00e4, 0x0610, // --> [Mp2fldWithIndexDisp32#610] and stop
    0x00e9, 0x0610,
    // end of load_complex.f32 (I32)
    // 00057c: store.f32 (I32)
    // --> [Mp2fst#611]
    0x00ec, 0x0611, // --> [Mp2fstDisp8#611]
    0x00f0, 0x0611, // --> [Mp2fstDisp32#611] and stop
    0x00f5, 0x0611,
    // end of store.f32 (I32)
    // 000582: store_complex.f32 (I32)
    // --> [Mp2fstWithIndex#611]
    0x00f8, 0x0611, // --> [Mp2fstWithIndexDisp8#611]
    0x00fc, 0x0611, // --> [Mp2fstWithIndexDisp32#611] and stop
    0x0101, 0x0611,
];
pub static LEVEL2: [Level2Entry<u16>; 802] = [
    // I64
    // 000000: i32, 128 entries
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Brz),
        offset: 0x00002e,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Brnz),
        offset: 0x000026,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Bitcast),
        offset: 0x000011,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Bint),
        offset: 0x00000c,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Load),
        offset: 0x0000c3,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Ireduce),
        offset: 0x000082,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::LoadComplex),
        offset: 0x0000cf,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Sextend),
        offset: 0x0000ff,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::StoreComplex),
        offset: 0x000153,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Store),
        offset: 0x000147,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Sload8),
        offset: 0x000122,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Sload8Complex),
        offset: 0x00012e,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Istore8),
        offset: 0x0000a6,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Istore8Complex),
        offset: 0x0000b2,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Uload16),
        offset: 0x00016a,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Uload16Complex),
        offset: 0x000176,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Sload16),
        offset: 0x000109,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Sload16Complex),
        offset: 0x000115,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Istore16),
        offset: 0x00008d,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Istore16Complex),
        offset: 0x000099,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Sdivmodx),
        offset: 0x0001b6,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Uextend),
        offset: 0x000160,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Cvtt2si),
        offset: 0x0001ac,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Uload8),
        offset: 0x000183,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Uload8Complex),
        offset: 0x00018f,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Umulx),
        offset: 0x0001c2,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Bsr),
        offset: 0x0001a8,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Bsf),
        offset: 0x0001a4,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Smulx),
        offset: 0x0001ba,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Iconst),
        offset: 0x00006e,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Selectif),
        offset: 0x0000fb,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Copy),
        offset: 0x000047,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Spill),
        offset: 0x00013b,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fill),
        offset: 0x000052,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regmove),
        offset: 0x0000e5,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::CopyNop),
        offset: 0x00004b,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Udivmodx),
        offset: 0x0001be,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regspill),
        offset: 0x0000e7,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regfill),
        offset: 0x0000e1,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Icmp),
        offset: 0x000062,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::IcmpImm),
        offset: 0x000066,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Ifcmp),
        offset: 0x000072,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::IfcmpImm),
        offset: 0x000076,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Iadd),
        offset: 0x000056,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Isub),
        offset: 0x0000bf,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Imul),
        offset: 0x00007e,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::IaddImm),
        offset: 0x00005a,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Band),
        offset: 0x000000,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Bor),
        offset: 0x00001a,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Bxor),
        offset: 0x000036,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Bnot),
        offset: 0x000016,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::BandImm),
        offset: 0x000004,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::BorImm),
        offset: 0x00001e,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::BxorImm),
        offset: 0x00003a,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Rotl),
        offset: 0x0000eb,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Rotr),
        offset: 0x0000f3,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::RotlImm),
        offset: 0x0000ef,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::RotrImm),
        offset: 0x0000f7,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Ishl),
        offset: 0x000085,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Ushr),
        offset: 0x00019c,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Sshr),
        offset: 0x00013f,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::IshlImm),
        offset: 0x000089,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::UshrImm),
        offset: 0x0001a0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::SshrImm),
        offset: 0x000143,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Clz),
        offset: 0x000042,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Ctz),
        offset: 0x00004d,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Popcnt),
        offset: 0x0000dc,
    },
    // 000080: i64, 128 entries
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Brz),
        offset: 0x0001dd,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Brnz),
        offset: 0x0001d9,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::JumpTableEntry),
        offset: 0x00022c,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::JumpTableBase),
        offset: 0x00022a,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::IndirectJumpTableBr),
        offset: 0x000220,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Bitcast),
        offset: 0x0001ce,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::CallIndirect),
        offset: 0x0001e7,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Bint),
        offset: 0x00000c,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::FuncAddr),
        offset: 0x0001f5,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Load),
        offset: 0x00022e,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::LoadComplex),
        offset: 0x000234,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Sextend),
        offset: 0x00024e,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::StoreComplex),
        offset: 0x000285,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Store),
        offset: 0x00027f,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Sload8),
        offset: 0x00026a,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Sload8Complex),
        offset: 0x000270,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Istore8),
        offset: 0x0000a6,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Istore8Complex),
        offset: 0x0000b2,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Uload16),
        offset: 0x0002a5,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Uload16Complex),
        offset: 0x0002ab,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Sload16),
        offset: 0x000257,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Sload16Complex),
        offset: 0x00025d,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Istore16),
        offset: 0x00008d,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Istore16Complex),
        offset: 0x000099,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Uload32),
        offset: 0x0000c3,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Uextend),
        offset: 0x000296,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Sload32),
        offset: 0x000264,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Uload8),
        offset: 0x0002b2,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Istore32),
        offset: 0x000147,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Cvtt2si),
        offset: 0x0002c7,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Pop),
        offset: 0x0002cd,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Bsr),
        offset: 0x0002c5,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::StackAddr),
        offset: 0x00027d,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Bsf),
        offset: 0x0002c3,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::SymbolValue),
        offset: 0x00028c,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Push),
        offset: 0x0002d1,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Iconst),
        offset: 0x00020d,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Sdivmodx),
        offset: 0x0002d5,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Umulx),
        offset: 0x0002db,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Smulx),
        offset: 0x0002d7,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Uload8Complex),
        offset: 0x0002b8,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Selectif),
        offset: 0x00024c,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Copy),
        offset: 0x0001ee,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Spill),
        offset: 0x000277,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fill),
        offset: 0x0001f3,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regmove),
        offset: 0x000240,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::CopyNop),
        offset: 0x00004b,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::AdjustSpDown),
        offset: 0x0001c6,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::IfcmpSp),
        offset: 0x00021c,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regspill),
        offset: 0x000242,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regfill),
        offset: 0x00023e,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Udivmodx),
        offset: 0x0002d9,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Icmp),
        offset: 0x000207,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::IcmpImm),
        offset: 0x000209,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Ifcmp),
        offset: 0x000216,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::IfcmpImm),
        offset: 0x000218,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Iadd),
        offset: 0x000201,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Isub),
        offset: 0x000228,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Imul),
        offset: 0x00021e,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::IaddImm),
        offset: 0x000203,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Band),
        offset: 0x0001c8,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Bor),
        offset: 0x0001d3,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Bxor),
        offset: 0x0001e1,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Bnot),
        offset: 0x0001d1,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::BandImm),
        offset: 0x0001ca,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::BorImm),
        offset: 0x0001d5,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::BxorImm),
        offset: 0x0001e3,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Rotl),
        offset: 0x000244,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Rotr),
        offset: 0x000248,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::RotlImm),
        offset: 0x000246,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::RotrImm),
        offset: 0x00024a,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Ishl),
        offset: 0x000224,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Ushr),
        offset: 0x0002bf,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Sshr),
        offset: 0x000279,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::IshlImm),
        offset: 0x000226,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::UshrImm),
        offset: 0x0002c1,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::SshrImm),
        offset: 0x00027b,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Clz),
        offset: 0x0001eb,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Ctz),
        offset: 0x0001f0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Popcnt),
        offset: 0x00023b,
    },
    // 000100: b1, 16 entries
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Brz),
        offset: 0x0002e9,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Brnz),
        offset: 0x0002e1,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regspill),
        offset: 0x0000e7,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regfill),
        offset: 0x0000e1,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Band),
        offset: 0x000000,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Bconst),
        offset: 0x0002dd,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Bor),
        offset: 0x00001a,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Bxor),
        offset: 0x000036,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Copy),
        offset: 0x000047,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Spill),
        offset: 0x00013b,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fill),
        offset: 0x000052,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regmove),
        offset: 0x0002f1,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    // 000110: i8, 16 entries
    Level2Entry {
        opcode: Some(crate::ir::Opcode::CopyNop),
        offset: 0x00004b,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regspill),
        offset: 0x0000e7,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regfill),
        offset: 0x0000e1,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Ireduce),
        offset: 0x0002f5,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Copy),
        offset: 0x000047,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Spill),
        offset: 0x00013b,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fill),
        offset: 0x000052,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regmove),
        offset: 0x0002fe,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    // 000120: i16, 16 entries
    Level2Entry {
        opcode: Some(crate::ir::Opcode::CopyNop),
        offset: 0x00004b,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regspill),
        offset: 0x0000e7,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regfill),
        offset: 0x0000e1,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Ireduce),
        offset: 0x0002f8,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Copy),
        offset: 0x000047,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Spill),
        offset: 0x00013b,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fill),
        offset: 0x000052,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regmove),
        offset: 0x0000e5,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    // 000130: typeless, 32 entries
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Jump),
        offset: 0x000330,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::AdjustSpUpImm),
        offset: 0x000308,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::AdjustSpDownImm),
        offset: 0x000304,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Brif),
        offset: 0x000314,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Brff),
        offset: 0x00030c,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Uload32Complex),
        offset: 0x0000cf,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Sload32Complex),
        offset: 0x000336,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Debugtrap),
        offset: 0x000324,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Istore32Complex),
        offset: 0x000153,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Trap),
        offset: 0x00033d,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Trapif),
        offset: 0x000341,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Trapff),
        offset: 0x00033f,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Return),
        offset: 0x000334,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Trueff),
        offset: 0x000343,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Call),
        offset: 0x00031c,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::F32const),
        offset: 0x000326,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::F64const),
        offset: 0x00032b,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Trueif),
        offset: 0x000347,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::CopySpecial),
        offset: 0x000322,
    },
    // 000150: f64, 64 entries
    Level2Entry {
        opcode: Some(crate::ir::Opcode::CopyNop),
        offset: 0x00004b,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fcmp),
        offset: 0x00036b,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fadd),
        offset: 0x000367,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fsub),
        offset: 0x00038c,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Ffcmp),
        offset: 0x00037b,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fdiv),
        offset: 0x000377,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regfill),
        offset: 0x0003a8,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fmul),
        offset: 0x000383,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regspill),
        offset: 0x0003ae,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Sqrt),
        offset: 0x0003b6,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Ceil),
        offset: 0x00035e,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Floor),
        offset: 0x00035e,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Trunc),
        offset: 0x00035e,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Nearest),
        offset: 0x00035e,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Bitcast),
        offset: 0x000353,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Load),
        offset: 0x000390,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::LoadComplex),
        offset: 0x00039c,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Store),
        offset: 0x0003ba,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::StoreComplex),
        offset: 0x0003c6,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fpromote),
        offset: 0x000387,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::FcvtFromSint),
        offset: 0x00036f,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Band),
        offset: 0x00034b,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Bor),
        offset: 0x000356,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Bxor),
        offset: 0x00035a,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Fmin),
        offset: 0x0003d6,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::BandNot),
        offset: 0x00034f,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Fmax),
        offset: 0x0003d2,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Copy),
        offset: 0x000363,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Spill),
        offset: 0x0003b2,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fill),
        offset: 0x00037f,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regmove),
        offset: 0x0003ac,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    // 000190: f32, 64 entries
    Level2Entry {
        opcode: Some(crate::ir::Opcode::CopyNop),
        offset: 0x00004b,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fcmp),
        offset: 0x0003e8,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fadd),
        offset: 0x0003e4,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fsub),
        offset: 0x000409,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Ffcmp),
        offset: 0x0003fd,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fdiv),
        offset: 0x0003f9,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regfill),
        offset: 0x000425,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fmul),
        offset: 0x000405,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regspill),
        offset: 0x000429,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Sqrt),
        offset: 0x000431,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Ceil),
        offset: 0x0003df,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Floor),
        offset: 0x0003df,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Trunc),
        offset: 0x0003df,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Nearest),
        offset: 0x0003df,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Bitcast),
        offset: 0x0003da,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Load),
        offset: 0x00040d,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::LoadComplex),
        offset: 0x000419,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Store),
        offset: 0x000435,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::StoreComplex),
        offset: 0x000441,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fdemote),
        offset: 0x0003f4,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::FcvtFromSint),
        offset: 0x0003ec,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Band),
        offset: 0x00034b,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Bor),
        offset: 0x000356,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Bxor),
        offset: 0x00035a,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Fmin),
        offset: 0x000451,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::BandNot),
        offset: 0x00034f,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Fmax),
        offset: 0x00044d,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Copy),
        offset: 0x000363,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Spill),
        offset: 0x00042d,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fill),
        offset: 0x000401,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regmove),
        offset: 0x0003ac,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    // I32
    // 0001d0: i32, 128 entries
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Brz),
        offset: 0x000469,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Brnz),
        offset: 0x000465,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::JumpTableEntry),
        offset: 0x0004a7,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::JumpTableBase),
        offset: 0x0004a5,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::IndirectJumpTableBr),
        offset: 0x000222,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Bitcast),
        offset: 0x00045e,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::CallIndirect),
        offset: 0x0001e9,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Bint),
        offset: 0x00045b,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::FuncAddr),
        offset: 0x000477,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Load),
        offset: 0x0004a9,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::LoadComplex),
        offset: 0x0004af,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Sextend),
        offset: 0x0004b9,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::StoreComplex),
        offset: 0x0004e1,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Store),
        offset: 0x0004db,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Sload8),
        offset: 0x0004cc,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Sload8Complex),
        offset: 0x0004d2,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Istore8),
        offset: 0x000498,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Istore8Complex),
        offset: 0x00049e,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Uload16),
        offset: 0x0004f1,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Uload16Complex),
        offset: 0x0004f7,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Sload16),
        offset: 0x0004bf,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Sload16Complex),
        offset: 0x0004c5,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Istore16),
        offset: 0x00048b,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Istore16Complex),
        offset: 0x000491,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Sdivmodx),
        offset: 0x0001b8,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Uextend),
        offset: 0x0004eb,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Cvtt2si),
        offset: 0x00050b,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Uload8),
        offset: 0x0004fe,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Uload8Complex),
        offset: 0x000504,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Push),
        offset: 0x0002d3,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Pop),
        offset: 0x0002cf,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Bsr),
        offset: 0x0001aa,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::StackAddr),
        offset: 0x0004d9,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Bsf),
        offset: 0x0001a6,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::SymbolValue),
        offset: 0x0004e8,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Smulx),
        offset: 0x0001bc,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Iconst),
        offset: 0x000070,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Umulx),
        offset: 0x0001c4,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Selectif),
        offset: 0x0000fd,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Copy),
        offset: 0x000049,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Spill),
        offset: 0x00013d,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fill),
        offset: 0x000054,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regmove),
        offset: 0x0002f3,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::CopyNop),
        offset: 0x00004b,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::AdjustSpDown),
        offset: 0x000455,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::IfcmpSp),
        offset: 0x000489,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regspill),
        offset: 0x0000e9,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regfill),
        offset: 0x0000e3,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Udivmodx),
        offset: 0x0001c0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Icmp),
        offset: 0x000064,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::IcmpImm),
        offset: 0x000481,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Ifcmp),
        offset: 0x000074,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::IfcmpImm),
        offset: 0x000485,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Iadd),
        offset: 0x000057,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Isub),
        offset: 0x0000c1,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Imul),
        offset: 0x000080,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::IaddImm),
        offset: 0x00047d,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Band),
        offset: 0x000002,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Bor),
        offset: 0x00001c,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Bxor),
        offset: 0x000038,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Bnot),
        offset: 0x000018,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::BandImm),
        offset: 0x000457,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::BorImm),
        offset: 0x000461,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::BxorImm),
        offset: 0x00046d,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Rotl),
        offset: 0x0000ed,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Rotr),
        offset: 0x0000f5,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::RotlImm),
        offset: 0x0000f1,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::RotrImm),
        offset: 0x0000f9,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Ishl),
        offset: 0x000087,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Ushr),
        offset: 0x00019e,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Sshr),
        offset: 0x000141,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::IshlImm),
        offset: 0x00008b,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::UshrImm),
        offset: 0x0001a2,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::SshrImm),
        offset: 0x000145,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Clz),
        offset: 0x000471,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Ctz),
        offset: 0x000474,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Popcnt),
        offset: 0x0004b6,
    },
    // 000250: b1, 16 entries
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Brz),
        offset: 0x000517,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Brnz),
        offset: 0x000511,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regspill),
        offset: 0x0000e9,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regfill),
        offset: 0x0000e3,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Band),
        offset: 0x000002,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Bconst),
        offset: 0x0002df,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Bor),
        offset: 0x00001c,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Bxor),
        offset: 0x000038,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Copy),
        offset: 0x000049,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Spill),
        offset: 0x00013d,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fill),
        offset: 0x000054,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regmove),
        offset: 0x0002f3,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    // 000260: i8, 16 entries
    Level2Entry {
        opcode: Some(crate::ir::Opcode::CopyNop),
        offset: 0x00004b,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regspill),
        offset: 0x0000e9,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regfill),
        offset: 0x0000e3,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Ireduce),
        offset: 0x00051d,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Copy),
        offset: 0x000049,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Spill),
        offset: 0x00013d,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fill),
        offset: 0x000054,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regmove),
        offset: 0x000523,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    // 000270: i16, 16 entries
    Level2Entry {
        opcode: Some(crate::ir::Opcode::CopyNop),
        offset: 0x00004b,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regspill),
        offset: 0x0000e9,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regfill),
        offset: 0x0000e3,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Ireduce),
        offset: 0x000520,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Copy),
        offset: 0x000049,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Spill),
        offset: 0x00013d,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fill),
        offset: 0x000054,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regmove),
        offset: 0x0002f3,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    // 000280: typeless, 32 entries
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Jump),
        offset: 0x000330,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::AdjustSpUpImm),
        offset: 0x00052b,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::AdjustSpDownImm),
        offset: 0x000527,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Brif),
        offset: 0x000533,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Brff),
        offset: 0x00052f,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Debugtrap),
        offset: 0x000324,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Trap),
        offset: 0x00033d,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Trapif),
        offset: 0x000341,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Trapff),
        offset: 0x00033f,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Return),
        offset: 0x000334,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Trueff),
        offset: 0x000345,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Call),
        offset: 0x000537,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::F32const),
        offset: 0x00053b,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::F64const),
        offset: 0x00053e,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Trueif),
        offset: 0x000349,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::CopySpecial),
        offset: 0x000539,
    },
    // 0002a0: i64, 2 entries
    Level2Entry {
        opcode: Some(crate::ir::Opcode::CopyNop),
        offset: 0x00004b,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    // 0002a2: f64, 64 entries
    Level2Entry {
        opcode: Some(crate::ir::Opcode::CopyNop),
        offset: 0x00004b,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fcmp),
        offset: 0x00036d,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fadd),
        offset: 0x000369,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fsub),
        offset: 0x00038e,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Ffcmp),
        offset: 0x00037d,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fdiv),
        offset: 0x000379,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regfill),
        offset: 0x0003aa,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fmul),
        offset: 0x000385,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regspill),
        offset: 0x0003b0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Sqrt),
        offset: 0x0003b8,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Ceil),
        offset: 0x000541,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Floor),
        offset: 0x000541,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Trunc),
        offset: 0x000541,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Nearest),
        offset: 0x000541,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Load),
        offset: 0x00054a,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::LoadComplex),
        offset: 0x000550,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Store),
        offset: 0x000558,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::StoreComplex),
        offset: 0x00055e,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fpromote),
        offset: 0x000547,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::FcvtFromSint),
        offset: 0x000544,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Band),
        offset: 0x00034d,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Bor),
        offset: 0x000358,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Bxor),
        offset: 0x00035c,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Fmin),
        offset: 0x0003d8,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::BandNot),
        offset: 0x000351,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Fmax),
        offset: 0x0003d4,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Copy),
        offset: 0x000365,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Spill),
        offset: 0x0003b4,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fill),
        offset: 0x000381,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regmove),
        offset: 0x000556,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    // 0002e2: f32, 64 entries
    Level2Entry {
        opcode: Some(crate::ir::Opcode::CopyNop),
        offset: 0x00004b,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fcmp),
        offset: 0x0003ea,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fadd),
        offset: 0x0003e6,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fsub),
        offset: 0x00040b,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Ffcmp),
        offset: 0x0003ff,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fdiv),
        offset: 0x0003fb,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regfill),
        offset: 0x000427,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fmul),
        offset: 0x000407,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regspill),
        offset: 0x00042b,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Sqrt),
        offset: 0x000433,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Ceil),
        offset: 0x000567,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Floor),
        offset: 0x000567,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Trunc),
        offset: 0x000567,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Nearest),
        offset: 0x000567,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Bitcast),
        offset: 0x000564,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Load),
        offset: 0x000570,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::LoadComplex),
        offset: 0x000576,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Store),
        offset: 0x00057c,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::StoreComplex),
        offset: 0x000582,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fdemote),
        offset: 0x00056d,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::FcvtFromSint),
        offset: 0x00056a,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Band),
        offset: 0x00034d,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Bor),
        offset: 0x000358,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Bxor),
        offset: 0x00035c,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Fmin),
        offset: 0x000453,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::BandNot),
        offset: 0x000351,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::X86Fmax),
        offset: 0x00044f,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Copy),
        offset: 0x000365,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Spill),
        offset: 0x00042f,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Fill),
        offset: 0x000403,
    },
    Level2Entry {
        opcode: Some(crate::ir::Opcode::Regmove),
        offset: 0x000556,
    },
    Level2Entry {
        opcode: None,
        offset: 0,
    },
];
pub static LEVEL1_I64: [Level1Entry<u16>; 16] = [
    Level1Entry {
        ty: ir::types::B1,
        log2len: 4,
        offset: 0x000100,
        legalize: 1,
    }, // expand_flags
    Level1Entry {
        ty: ir::types::INVALID,
        log2len: 5,
        offset: 0x000130,
        legalize: 1,
    }, // expand_flags
    Level1Entry {
        ty: ir::types::INVALID,
        log2len: !0,
        offset: 0,
        legalize: 0,
    },
    Level1Entry {
        ty: ir::types::INVALID,
        log2len: !0,
        offset: 0,
        legalize: 0,
    },
    Level1Entry {
        ty: ir::types::INVALID,
        log2len: !0,
        offset: 0,
        legalize: 0,
    },
    Level1Entry {
        ty: ir::types::I8,
        log2len: 4,
        offset: 0x000110,
        legalize: 2,
    }, // widen
    Level1Entry {
        ty: ir::types::I16,
        log2len: 4,
        offset: 0x000120,
        legalize: 2,
    }, // widen
    Level1Entry {
        ty: ir::types::I32,
        log2len: 7,
        offset: 0x000000,
        legalize: 3,
    }, // x86_expand
    Level1Entry {
        ty: ir::types::I64,
        log2len: 7,
        offset: 0x000080,
        legalize: 3,
    }, // x86_expand
    Level1Entry {
        ty: ir::types::F32,
        log2len: 6,
        offset: 0x000190,
        legalize: 3,
    }, // x86_expand
    Level1Entry {
        ty: ir::types::F64,
        log2len: 6,
        offset: 0x000150,
        legalize: 3,
    }, // x86_expand
    Level1Entry {
        ty: ir::types::INVALID,
        log2len: !0,
        offset: 0,
        legalize: 0,
    },
    Level1Entry {
        ty: ir::types::INVALID,
        log2len: !0,
        offset: 0,
        legalize: 0,
    },
    Level1Entry {
        ty: ir::types::INVALID,
        log2len: !0,
        offset: 0,
        legalize: 0,
    },
    Level1Entry {
        ty: ir::types::INVALID,
        log2len: !0,
        offset: 0,
        legalize: 0,
    },
    Level1Entry {
        ty: ir::types::INVALID,
        log2len: !0,
        offset: 0,
        legalize: 0,
    },
];
pub static LEVEL1_I32: [Level1Entry<u16>; 16] = [
    Level1Entry {
        ty: ir::types::B1,
        log2len: 4,
        offset: 0x000250,
        legalize: 1,
    }, // expand_flags
    Level1Entry {
        ty: ir::types::INVALID,
        log2len: 5,
        offset: 0x000280,
        legalize: 1,
    }, // expand_flags
    Level1Entry {
        ty: ir::types::INVALID,
        log2len: !0,
        offset: 0,
        legalize: 0,
    },
    Level1Entry {
        ty: ir::types::INVALID,
        log2len: !0,
        offset: 0,
        legalize: 0,
    },
    Level1Entry {
        ty: ir::types::INVALID,
        log2len: !0,
        offset: 0,
        legalize: 0,
    },
    Level1Entry {
        ty: ir::types::I8,
        log2len: 4,
        offset: 0x000260,
        legalize: 2,
    }, // widen
    Level1Entry {
        ty: ir::types::I16,
        log2len: 4,
        offset: 0x000270,
        legalize: 2,
    }, // widen
    Level1Entry {
        ty: ir::types::I32,
        log2len: 7,
        offset: 0x0001d0,
        legalize: 3,
    }, // x86_expand
    Level1Entry {
        ty: ir::types::I64,
        log2len: 1,
        offset: 0x0002a0,
        legalize: 0,
    }, // narrow
    Level1Entry {
        ty: ir::types::F32,
        log2len: 6,
        offset: 0x0002e2,
        legalize: 3,
    }, // x86_expand
    Level1Entry {
        ty: ir::types::F64,
        log2len: 6,
        offset: 0x0002a2,
        legalize: 3,
    }, // x86_expand
    Level1Entry {
        ty: ir::types::INVALID,
        log2len: !0,
        offset: 0,
        legalize: 0,
    },
    Level1Entry {
        ty: ir::types::INVALID,
        log2len: !0,
        offset: 0,
        legalize: 0,
    },
    Level1Entry {
        ty: ir::types::INVALID,
        log2len: !0,
        offset: 0,
        legalize: 0,
    },
    Level1Entry {
        ty: ir::types::INVALID,
        log2len: !0,
        offset: 0,
        legalize: 0,
    },
    Level1Entry {
        ty: ir::types::INVALID,
        log2len: !0,
        offset: 0,
        legalize: 0,
    },
];
static RECIPE_NAMES: [&str; 243] = [
    "Op1rr",
    "RexOp1rr",
    "Op1ur",
    "RexOp1ur",
    "Op2rrx",
    "RexOp2rrx",
    "Op1div",
    "RexOp1div",
    "Op1mulx",
    "RexOp1mulx",
    "Op1umr",
    "RexOp1umr",
    "Op1rmov",
    "RexOp1rmov",
    "Op1r_ib",
    "RexOp1r_ib",
    "Op1r_id",
    "RexOp1r_id",
    "Op1pu_id",
    "RexOp1pu_id",
    "RexOp1u_id",
    "RexOp1pu_iq",
    "Op1pu_id_bool",
    "RexOp1pu_id_bool",
    "Op1rc",
    "RexOp1rc",
    "Mp2urm",
    "RexMp2urm",
    "Op1ldWithIndex",
    "RexOp1ldWithIndex",
    "Op2ldWithIndex",
    "RexOp2ldWithIndex",
    "Op1ldWithIndexDisp8",
    "RexOp1ldWithIndexDisp8",
    "Op2ldWithIndexDisp8",
    "RexOp2ldWithIndexDisp8",
    "Op1ldWithIndexDisp32",
    "RexOp1ldWithIndexDisp32",
    "Op2ldWithIndexDisp32",
    "RexOp2ldWithIndexDisp32",
    "Op1stWithIndex",
    "RexOp1stWithIndex",
    "Mp1stWithIndex",
    "RexMp1stWithIndex",
    "Op1stWithIndexDisp8",
    "RexOp1stWithIndexDisp8",
    "Mp1stWithIndexDisp8",
    "RexMp1stWithIndexDisp8",
    "Op1stWithIndexDisp32",
    "RexOp1stWithIndexDisp32",
    "Mp1stWithIndexDisp32",
    "RexMp1stWithIndexDisp32",
    "Op1stWithIndex_abcd",
    "RexOp1stWithIndex_abcd",
    "Op1stWithIndexDisp8_abcd",
    "RexOp1stWithIndexDisp8_abcd",
    "Op1stWithIndexDisp32_abcd",
    "RexOp1stWithIndexDisp32_abcd",
    "Op1st",
    "RexOp1st",
    "Mp1st",
    "RexMp1st",
    "Op1stDisp8",
    "RexOp1stDisp8",
    "Mp1stDisp8",
    "RexMp1stDisp8",
    "Op1stDisp32",
    "RexOp1stDisp32",
    "Mp1stDisp32",
    "RexMp1stDisp32",
    "Op1st_abcd",
    "Op1stDisp8_abcd",
    "Op1stDisp32_abcd",
    "Op1spillSib32",
    "RexOp1spillSib32",
    "Op1regspill32",
    "RexOp1regspill32",
    "Op1ld",
    "RexOp1ld",
    "Op2ld",
    "RexOp2ld",
    "Op1ldDisp8",
    "RexOp1ldDisp8",
    "Op2ldDisp8",
    "RexOp2ldDisp8",
    "Op1ldDisp32",
    "RexOp1ldDisp32",
    "Op2ldDisp32",
    "RexOp2ldDisp32",
    "Op1fillSib32",
    "RexOp1fillSib32",
    "Op1regfill32",
    "RexOp1regfill32",
    "Op1pushq",
    "RexOp1pushq",
    "Op1popq",
    "RexOp1popq",
    "RexOp1copysp",
    "Op1copysp",
    "stacknull",
    "Op1adjustsp",
    "RexOp1adjustsp",
    "Op1adjustsp_ib",
    "Op1adjustsp_id",
    "RexOp1adjustsp_ib",
    "RexOp1adjustsp_id",
    "Mp2fld",
    "RexMp2fld",
    "Mp2fldDisp8",
    "RexMp2fldDisp8",
    "Mp2fldDisp32",
    "RexMp2fldDisp32",
    "Mp2fldWithIndex",
    "RexMp2fldWithIndex",
    "Mp2fldWithIndexDisp8",
    "RexMp2fldWithIndexDisp8",
    "Mp2fldWithIndexDisp32",
    "RexMp2fldWithIndexDisp32",
    "Mp2fst",
    "RexMp2fst",
    "Mp2fstDisp8",
    "RexMp2fstDisp8",
    "Mp2fstDisp32",
    "RexMp2fstDisp32",
    "Mp2fstWithIndex",
    "RexMp2fstWithIndex",
    "Mp2fstWithIndexDisp8",
    "RexMp2fstWithIndexDisp8",
    "Mp2fstWithIndexDisp32",
    "RexMp2fstWithIndexDisp32",
    "Mp2ffillSib32",
    "RexMp2ffillSib32",
    "Mp2fregfill32",
    "RexMp2fregfill32",
    "Mp2fspillSib32",
    "RexMp2fspillSib32",
    "Mp2fregspill32",
    "RexMp2fregspill32",
    "Op1fnaddr4",
    "RexOp1fnaddr8",
    "Op1allones_fnaddr4",
    "RexOp1allones_fnaddr8",
    "RexOp1pcrel_fnaddr8",
    "RexOp1got_fnaddr8",
    "Op1gvaddr4",
    "RexOp1gvaddr8",
    "RexOp1pcrel_gvaddr8",
    "RexOp1got_gvaddr8",
    "Op1spaddr4_id",
    "RexOp1spaddr8_id",
    "Op1call_id",
    "Op1call_plt_id",
    "Op1call_r",
    "RexOp1call_r",
    "Op1ret",
    "Op1jmpb",
    "Op1jmpd",
    "Op1brib",
    "RexOp1brib",
    "Op2brid",
    "RexOp2brid",
    "Op1brfb",
    "RexOp1brfb",
    "Op2brfd",
    "RexOp2brfd",
    "Op1tjccb",
    "RexOp1tjccb",
    "Op1tjccd",
    "RexOp1tjccd",
    "Op1t8jccd_long",
    "Op1t8jccb_abcd",
    "RexOp1t8jccb",
    "Op1t8jccd_abcd",
    "RexOp1t8jccd",
    "RexOp1jt_entry",
    "Op1jt_entry",
    "RexOp1jt_base",
    "Op1jt_base",
    "RexOp1indirect_jmp",
    "Op1indirect_jmp",
    "Op2trap",
    "debugtrap",
    "trapif",
    "trapff",
    "Op1icscc",
    "RexOp1icscc",
    "Op1icscc_ib",
    "RexOp1icscc_ib",
    "Op1icscc_id",
    "RexOp1icscc_id",
    "Op1rcmp",
    "RexOp1rcmp",
    "Op1rcmp_ib",
    "RexOp1rcmp_ib",
    "Op1rcmp_id",
    "RexOp1rcmp_id",
    "Op1rcmp_sp",
    "RexOp1rcmp_sp",
    "Op2seti_abcd",
    "RexOp2seti",
    "Op2setf_abcd",
    "RexOp2setf",
    "Op2cmov",
    "RexOp2cmov",
    "Op2bsf_and_bsr",
    "RexOp2bsf_and_bsr",
    "Op2urm_noflags_abcd",
    "RexOp2urm_noflags",
    "null",
    "Op2urm_noflags",
    "RexOp1urm_noflags",
    "Op2f32imm_z",
    "Mp2f64imm_z",
    "RexOp2f32imm_z",
    "RexMp2f64imm_z",
    "Mp2frurm",
    "RexMp2frurm",
    "Mp2rfumr",
    "RexMp2rfumr",
    "Op2furm",
    "RexOp2furm",
    "Op2frmov",
    "RexOp2frmov",
    "Mp2furm",
    "RexMp2furm",
    "Mp2rfurm",
    "RexMp2rfurm",
    "Mp3furmi_rnd",
    "RexMp3furmi_rnd",
    "Mp2fa",
    "RexMp2fa",
    "Op2fa",
    "RexOp2fa",
    "Op2fax",
    "RexOp2fax",
    "Op2fcscc",
    "RexOp2fcscc",
    "Mp2fcscc",
    "RexMp2fcscc",
    "Op2fcmp",
    "RexOp2fcmp",
    "Mp2fcmp",
    "RexMp2fcmp",
];
static RECIPE_CONSTRAINTS: [RecipeConstraints; 243] = [
    // Constraints for recipe Op1rr:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Tied(0),
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1rr:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Tied(0),
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1ur:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Tied(0),
            regclass: &GPR8_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Tied(0),
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1ur:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Tied(0),
            regclass: &GPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Tied(0),
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe Op2rrx:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Tied(0),
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp2rrx:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Tied(0),
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1div:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::FixedTied(0),
                regclass: &GPR_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::FixedTied(2),
                regclass: &GPR_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::FixedTied(0),
                regclass: &GPR_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::FixedTied(2),
                regclass: &GPR_DATA,
            },
        ],
        fixed_ins: true,
        fixed_outs: true,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1div:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::FixedTied(0),
                regclass: &GPR_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::FixedTied(2),
                regclass: &GPR_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::FixedTied(0),
                regclass: &GPR_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::FixedTied(2),
                regclass: &GPR_DATA,
            },
        ],
        fixed_ins: true,
        fixed_outs: true,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1mulx:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::FixedTied(0),
                regclass: &GPR_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::FixedTied(0),
                regclass: &GPR_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::FixedReg(2),
                regclass: &GPR_DATA,
            },
        ],
        fixed_ins: true,
        fixed_outs: true,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1mulx:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::FixedTied(0),
                regclass: &GPR_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::FixedTied(0),
                regclass: &GPR_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::FixedReg(2),
                regclass: &GPR_DATA,
            },
        ],
        fixed_ins: true,
        fixed_outs: true,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1umr:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1umr:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op1rmov:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1rmov:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op1r_ib:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Tied(0),
            regclass: &GPR8_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Tied(0),
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1r_ib:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Tied(0),
            regclass: &GPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Tied(0),
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1r_id:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Tied(0),
            regclass: &GPR8_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Tied(0),
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1r_id:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Tied(0),
            regclass: &GPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Tied(0),
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1pu_id:
    RecipeConstraints {
        ins: &[],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1pu_id:
    RecipeConstraints {
        ins: &[],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1u_id:
    RecipeConstraints {
        ins: &[],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1pu_iq:
    RecipeConstraints {
        ins: &[],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1pu_id_bool:
    RecipeConstraints {
        ins: &[],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1pu_id_bool:
    RecipeConstraints {
        ins: &[],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1rc:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::FixedReg(1),
                regclass: &GPR_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Tied(0),
            regclass: &GPR8_DATA,
        }],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1rc:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::FixedReg(1),
                regclass: &GPR_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Tied(0),
            regclass: &GPR_DATA,
        }],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe Mp2urm:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexMp2urm:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1ldWithIndex:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1ldWithIndex:
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
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op2ldWithIndex:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp2ldWithIndex:
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
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op1ldWithIndexDisp8:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1ldWithIndexDisp8:
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
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op2ldWithIndexDisp8:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp2ldWithIndexDisp8:
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
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op1ldWithIndexDisp32:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1ldWithIndexDisp32:
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
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op2ldWithIndexDisp32:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp2ldWithIndexDisp32:
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
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op1stWithIndex:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1stWithIndex:
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
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Mp1stWithIndex:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp1stWithIndex:
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
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op1stWithIndexDisp8:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1stWithIndexDisp8:
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
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Mp1stWithIndexDisp8:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp1stWithIndexDisp8:
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
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op1stWithIndexDisp32:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1stWithIndexDisp32:
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
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Mp1stWithIndexDisp32:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp1stWithIndexDisp32:
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
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op1stWithIndex_abcd:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &ABCD_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1stWithIndex_abcd:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &ABCD_DATA,
            },
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
        clobbers_flags: false,
    },
    // Constraints for recipe Op1stWithIndexDisp8_abcd:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &ABCD_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1stWithIndexDisp8_abcd:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &ABCD_DATA,
            },
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
        clobbers_flags: false,
    },
    // Constraints for recipe Op1stWithIndexDisp32_abcd:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &ABCD_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1stWithIndexDisp32_abcd:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &ABCD_DATA,
            },
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
        clobbers_flags: false,
    },
    // Constraints for recipe Op1st:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1st:
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
        clobbers_flags: false,
    },
    // Constraints for recipe Mp1st:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp1st:
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
        clobbers_flags: false,
    },
    // Constraints for recipe Op1stDisp8:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1stDisp8:
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
        clobbers_flags: false,
    },
    // Constraints for recipe Mp1stDisp8:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp1stDisp8:
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
        clobbers_flags: false,
    },
    // Constraints for recipe Op1stDisp32:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1stDisp32:
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
        clobbers_flags: false,
    },
    // Constraints for recipe Mp1stDisp32:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp1stDisp32:
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
        clobbers_flags: false,
    },
    // Constraints for recipe Op1st_abcd:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &ABCD_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op1stDisp8_abcd:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &ABCD_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op1stDisp32_abcd:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &ABCD_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op1spillSib32:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Stack,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1spillSib32:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Stack,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op1regspill32:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1regspill32:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op1ld:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1ld:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op2ld:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp2ld:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op1ldDisp8:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1ldDisp8:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op2ldDisp8:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp2ldDisp8:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op1ldDisp32:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1ldDisp32:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op2ldDisp32:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp2ldDisp32:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op1fillSib32:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Stack,
            regclass: &GPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1fillSib32:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Stack,
            regclass: &GPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op1regfill32:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Stack,
            regclass: &GPR_DATA,
        }],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1regfill32:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Stack,
            regclass: &GPR_DATA,
        }],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op1pushq:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1pushq:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1popq:
    RecipeConstraints {
        ins: &[],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1popq:
    RecipeConstraints {
        ins: &[],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1copysp:
    RecipeConstraints {
        ins: &[],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op1copysp:
    RecipeConstraints {
        ins: &[],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe stacknull:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Stack,
            regclass: &GPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Stack,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1adjustsp:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1adjustsp:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1adjustsp_ib:
    RecipeConstraints {
        ins: &[],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1adjustsp_id:
    RecipeConstraints {
        ins: &[],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1adjustsp_ib:
    RecipeConstraints {
        ins: &[],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1adjustsp_id:
    RecipeConstraints {
        ins: &[],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Mp2fld:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp2fld:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Mp2fldDisp8:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp2fldDisp8:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Mp2fldDisp32:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp2fldDisp32:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Mp2fldWithIndex:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp2fldWithIndex:
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
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Mp2fldWithIndexDisp8:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp2fldWithIndexDisp8:
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
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Mp2fldWithIndexDisp32:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp2fldWithIndexDisp32:
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
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Mp2fst:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp2fst:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
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
        clobbers_flags: false,
    },
    // Constraints for recipe Mp2fstDisp8:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp2fstDisp8:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
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
        clobbers_flags: false,
    },
    // Constraints for recipe Mp2fstDisp32:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp2fstDisp32:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
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
        clobbers_flags: false,
    },
    // Constraints for recipe Mp2fstWithIndex:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp2fstWithIndex:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
            },
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
        clobbers_flags: false,
    },
    // Constraints for recipe Mp2fstWithIndexDisp8:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp2fstWithIndexDisp8:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
            },
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
        clobbers_flags: false,
    },
    // Constraints for recipe Mp2fstWithIndexDisp32:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp2fstWithIndexDisp32:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
            },
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
        clobbers_flags: false,
    },
    // Constraints for recipe Mp2ffillSib32:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Stack,
            regclass: &FPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp2ffillSib32:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Stack,
            regclass: &FPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Mp2fregfill32:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Stack,
            regclass: &FPR_DATA,
        }],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp2fregfill32:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Stack,
            regclass: &FPR_DATA,
        }],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Mp2fspillSib32:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR8_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Stack,
            regclass: &FPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp2fspillSib32:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Stack,
            regclass: &FPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Mp2fregspill32:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR8_DATA,
        }],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp2fregspill32:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR_DATA,
        }],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op1fnaddr4:
    RecipeConstraints {
        ins: &[],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1fnaddr8:
    RecipeConstraints {
        ins: &[],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1allones_fnaddr4:
    RecipeConstraints {
        ins: &[],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1allones_fnaddr8:
    RecipeConstraints {
        ins: &[],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1pcrel_fnaddr8:
    RecipeConstraints {
        ins: &[],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1got_fnaddr8:
    RecipeConstraints {
        ins: &[],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1gvaddr4:
    RecipeConstraints {
        ins: &[],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1gvaddr8:
    RecipeConstraints {
        ins: &[],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1pcrel_gvaddr8:
    RecipeConstraints {
        ins: &[],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1got_gvaddr8:
    RecipeConstraints {
        ins: &[],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1spaddr4_id:
    RecipeConstraints {
        ins: &[],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1spaddr8_id:
    RecipeConstraints {
        ins: &[],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1call_id:
    RecipeConstraints {
        ins: &[],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1call_plt_id:
    RecipeConstraints {
        ins: &[],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1call_r:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1call_r:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1ret:
    RecipeConstraints {
        ins: &[],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1jmpb:
    RecipeConstraints {
        ins: &[],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op1jmpd:
    RecipeConstraints {
        ins: &[],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op1brib:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::FixedReg(32),
            regclass: &FLAG_DATA,
        }],
        outs: &[],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1brib:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::FixedReg(32),
            regclass: &FLAG_DATA,
        }],
        outs: &[],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op2brid:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::FixedReg(32),
            regclass: &FLAG_DATA,
        }],
        outs: &[],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp2brid:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::FixedReg(32),
            regclass: &FLAG_DATA,
        }],
        outs: &[],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op1brfb:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::FixedReg(32),
            regclass: &FLAG_DATA,
        }],
        outs: &[],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1brfb:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::FixedReg(32),
            regclass: &FLAG_DATA,
        }],
        outs: &[],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op2brfd:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::FixedReg(32),
            regclass: &FLAG_DATA,
        }],
        outs: &[],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp2brfd:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::FixedReg(32),
            regclass: &FLAG_DATA,
        }],
        outs: &[],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op1tjccb:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1tjccb:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1tjccd:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1tjccd:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1t8jccd_long:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1t8jccb_abcd:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &ABCD_DATA,
        }],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1t8jccb:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1t8jccd_abcd:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &ABCD_DATA,
        }],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1t8jccd:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1jt_entry:
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
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op1jt_entry:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1jt_base:
    RecipeConstraints {
        ins: &[],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op1jt_base:
    RecipeConstraints {
        ins: &[],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1indirect_jmp:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op1indirect_jmp:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op2trap:
    RecipeConstraints {
        ins: &[],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe debugtrap:
    RecipeConstraints {
        ins: &[],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe trapif:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::FixedReg(32),
            regclass: &FLAG_DATA,
        }],
        outs: &[],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe trapff:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::FixedReg(32),
            regclass: &FLAG_DATA,
        }],
        outs: &[],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op1icscc:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &ABCD_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1icscc:
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
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &ABCD_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1icscc_ib:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &ABCD_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1icscc_ib:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &ABCD_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1icscc_id:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &ABCD_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1icscc_id:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &ABCD_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1rcmp:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::FixedReg(32),
            regclass: &FLAG_DATA,
        }],
        fixed_ins: false,
        fixed_outs: true,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1rcmp:
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
        outs: &[OperandConstraint {
            kind: ConstraintKind::FixedReg(32),
            regclass: &FLAG_DATA,
        }],
        fixed_ins: false,
        fixed_outs: true,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1rcmp_ib:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::FixedReg(32),
            regclass: &FLAG_DATA,
        }],
        fixed_ins: false,
        fixed_outs: true,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1rcmp_ib:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::FixedReg(32),
            regclass: &FLAG_DATA,
        }],
        fixed_ins: false,
        fixed_outs: true,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1rcmp_id:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::FixedReg(32),
            regclass: &FLAG_DATA,
        }],
        fixed_ins: false,
        fixed_outs: true,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1rcmp_id:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::FixedReg(32),
            regclass: &FLAG_DATA,
        }],
        fixed_ins: false,
        fixed_outs: true,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1rcmp_sp:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::FixedReg(32),
            regclass: &FLAG_DATA,
        }],
        fixed_ins: false,
        fixed_outs: true,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1rcmp_sp:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::FixedReg(32),
            regclass: &FLAG_DATA,
        }],
        fixed_ins: false,
        fixed_outs: true,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op2seti_abcd:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::FixedReg(32),
            regclass: &FLAG_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &ABCD_DATA,
        }],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp2seti:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::FixedReg(32),
            regclass: &FLAG_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op2setf_abcd:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::FixedReg(32),
            regclass: &FLAG_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &ABCD_DATA,
        }],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp2setf:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::FixedReg(32),
            regclass: &FLAG_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op2cmov:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::FixedReg(32),
                regclass: &FLAG_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Tied(2),
            regclass: &GPR8_DATA,
        }],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp2cmov:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::FixedReg(32),
                regclass: &FLAG_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Tied(2),
            regclass: &GPR_DATA,
        }],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: false,
    },
    // Constraints for recipe Op2bsf_and_bsr:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::FixedReg(32),
                regclass: &FLAG_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: true,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp2bsf_and_bsr:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::FixedReg(32),
                regclass: &FLAG_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: true,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op2urm_noflags_abcd:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &ABCD_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp2urm_noflags:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe null:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Tied(0),
            regclass: &GPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Tied(0),
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe Op2urm_noflags:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1urm_noflags:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op2f32imm_z:
    RecipeConstraints {
        ins: &[],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Mp2f64imm_z:
    RecipeConstraints {
        ins: &[],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp2f32imm_z:
    RecipeConstraints {
        ins: &[],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexMp2f64imm_z:
    RecipeConstraints {
        ins: &[],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Mp2frurm:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp2frurm:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Mp2rfumr:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR8_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp2rfumr:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op2furm:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR8_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp2furm:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op2frmov:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR8_DATA,
        }],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp2frmov:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR_DATA,
        }],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Mp2furm:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR8_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp2furm:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Mp2rfurm:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR8_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp2rfurm:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &GPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Mp3furmi_rnd:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR8_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexMp3furmi_rnd:
    RecipeConstraints {
        ins: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR_DATA,
        }],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &FPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Mp2fa:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &FPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Tied(0),
            regclass: &FPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe RexMp2fa:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &FPR_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Tied(0),
            regclass: &FPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe Op2fa:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &FPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Tied(0),
            regclass: &FPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp2fa:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &FPR_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Tied(0),
            regclass: &FPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe Op2fax:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &FPR8_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Tied(1),
            regclass: &FPR8_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp2fax:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &FPR_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Tied(1),
            regclass: &FPR_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe Op2fcscc:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &ABCD_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp2fcscc:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &ABCD_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Mp2fcscc:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &ABCD_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexMp2fcscc:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::Reg,
            regclass: &ABCD_DATA,
        }],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op2fcmp:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::FixedReg(32),
            regclass: &FLAG_DATA,
        }],
        fixed_ins: false,
        fixed_outs: true,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp2fcmp:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::FixedReg(32),
            regclass: &FLAG_DATA,
        }],
        fixed_ins: false,
        fixed_outs: true,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Mp2fcmp:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::FixedReg(32),
            regclass: &FLAG_DATA,
        }],
        fixed_ins: false,
        fixed_outs: true,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexMp2fcmp:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
            },
        ],
        outs: &[OperandConstraint {
            kind: ConstraintKind::FixedReg(32),
            regclass: &FLAG_DATA,
        }],
        fixed_ins: false,
        fixed_outs: true,
        tied_ops: false,
        clobbers_flags: true,
    },
];
static RECIPE_SIZING: [RecipeSizing; 243] = [
    // Code size information for recipe Op1rr:
    RecipeSizing {
        base_size: 2,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1rr:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1ur:
    RecipeSizing {
        base_size: 2,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1ur:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op2rrx:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp2rrx:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1div:
    RecipeSizing {
        base_size: 2,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1div:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1mulx:
    RecipeSizing {
        base_size: 2,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1mulx:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1umr:
    RecipeSizing {
        base_size: 2,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1umr:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1rmov:
    RecipeSizing {
        base_size: 2,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1rmov:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1r_ib:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1r_ib:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1r_id:
    RecipeSizing {
        base_size: 6,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1r_id:
    RecipeSizing {
        base_size: 7,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1pu_id:
    RecipeSizing {
        base_size: 5,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1pu_id:
    RecipeSizing {
        base_size: 6,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1u_id:
    RecipeSizing {
        base_size: 7,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1pu_iq:
    RecipeSizing {
        base_size: 10,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1pu_id_bool:
    RecipeSizing {
        base_size: 5,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1pu_id_bool:
    RecipeSizing {
        base_size: 6,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1rc:
    RecipeSizing {
        base_size: 2,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1rc:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Mp2urm:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexMp2urm:
    RecipeSizing {
        base_size: 5,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1ldWithIndex:
    RecipeSizing {
        base_size: 3,
        compute_size: size_plus_maybe_offset_for_in_reg_0,
        branch_range: None,
    },
    // Code size information for recipe RexOp1ldWithIndex:
    RecipeSizing {
        base_size: 4,
        compute_size: size_plus_maybe_offset_for_in_reg_0,
        branch_range: None,
    },
    // Code size information for recipe Op2ldWithIndex:
    RecipeSizing {
        base_size: 4,
        compute_size: size_plus_maybe_offset_for_in_reg_0,
        branch_range: None,
    },
    // Code size information for recipe RexOp2ldWithIndex:
    RecipeSizing {
        base_size: 5,
        compute_size: size_plus_maybe_offset_for_in_reg_0,
        branch_range: None,
    },
    // Code size information for recipe Op1ldWithIndexDisp8:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1ldWithIndexDisp8:
    RecipeSizing {
        base_size: 5,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op2ldWithIndexDisp8:
    RecipeSizing {
        base_size: 5,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp2ldWithIndexDisp8:
    RecipeSizing {
        base_size: 6,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1ldWithIndexDisp32:
    RecipeSizing {
        base_size: 7,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1ldWithIndexDisp32:
    RecipeSizing {
        base_size: 8,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op2ldWithIndexDisp32:
    RecipeSizing {
        base_size: 8,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp2ldWithIndexDisp32:
    RecipeSizing {
        base_size: 9,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1stWithIndex:
    RecipeSizing {
        base_size: 3,
        compute_size: size_plus_maybe_offset_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe RexOp1stWithIndex:
    RecipeSizing {
        base_size: 4,
        compute_size: size_plus_maybe_offset_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe Mp1stWithIndex:
    RecipeSizing {
        base_size: 4,
        compute_size: size_plus_maybe_offset_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe RexMp1stWithIndex:
    RecipeSizing {
        base_size: 5,
        compute_size: size_plus_maybe_offset_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe Op1stWithIndexDisp8:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1stWithIndexDisp8:
    RecipeSizing {
        base_size: 5,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Mp1stWithIndexDisp8:
    RecipeSizing {
        base_size: 5,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexMp1stWithIndexDisp8:
    RecipeSizing {
        base_size: 6,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1stWithIndexDisp32:
    RecipeSizing {
        base_size: 7,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1stWithIndexDisp32:
    RecipeSizing {
        base_size: 8,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Mp1stWithIndexDisp32:
    RecipeSizing {
        base_size: 8,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexMp1stWithIndexDisp32:
    RecipeSizing {
        base_size: 9,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1stWithIndex_abcd:
    RecipeSizing {
        base_size: 3,
        compute_size: size_plus_maybe_offset_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe RexOp1stWithIndex_abcd:
    RecipeSizing {
        base_size: 4,
        compute_size: size_plus_maybe_offset_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe Op1stWithIndexDisp8_abcd:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1stWithIndexDisp8_abcd:
    RecipeSizing {
        base_size: 5,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1stWithIndexDisp32_abcd:
    RecipeSizing {
        base_size: 7,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1stWithIndexDisp32_abcd:
    RecipeSizing {
        base_size: 8,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1st:
    RecipeSizing {
        base_size: 2,
        compute_size: size_plus_maybe_sib_or_offset_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe RexOp1st:
    RecipeSizing {
        base_size: 3,
        compute_size: size_plus_maybe_sib_or_offset_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe Mp1st:
    RecipeSizing {
        base_size: 3,
        compute_size: size_plus_maybe_sib_or_offset_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe RexMp1st:
    RecipeSizing {
        base_size: 4,
        compute_size: size_plus_maybe_sib_or_offset_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe Op1stDisp8:
    RecipeSizing {
        base_size: 3,
        compute_size: size_plus_maybe_sib_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe RexOp1stDisp8:
    RecipeSizing {
        base_size: 4,
        compute_size: size_plus_maybe_sib_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe Mp1stDisp8:
    RecipeSizing {
        base_size: 4,
        compute_size: size_plus_maybe_sib_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe RexMp1stDisp8:
    RecipeSizing {
        base_size: 5,
        compute_size: size_plus_maybe_sib_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe Op1stDisp32:
    RecipeSizing {
        base_size: 6,
        compute_size: size_plus_maybe_sib_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe RexOp1stDisp32:
    RecipeSizing {
        base_size: 7,
        compute_size: size_plus_maybe_sib_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe Mp1stDisp32:
    RecipeSizing {
        base_size: 7,
        compute_size: size_plus_maybe_sib_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe RexMp1stDisp32:
    RecipeSizing {
        base_size: 8,
        compute_size: size_plus_maybe_sib_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe Op1st_abcd:
    RecipeSizing {
        base_size: 2,
        compute_size: size_plus_maybe_sib_or_offset_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe Op1stDisp8_abcd:
    RecipeSizing {
        base_size: 3,
        compute_size: size_plus_maybe_sib_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe Op1stDisp32_abcd:
    RecipeSizing {
        base_size: 6,
        compute_size: size_plus_maybe_sib_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe Op1spillSib32:
    RecipeSizing {
        base_size: 7,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1spillSib32:
    RecipeSizing {
        base_size: 8,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1regspill32:
    RecipeSizing {
        base_size: 7,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1regspill32:
    RecipeSizing {
        base_size: 8,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1ld:
    RecipeSizing {
        base_size: 2,
        compute_size: size_plus_maybe_sib_or_offset_for_in_reg_0,
        branch_range: None,
    },
    // Code size information for recipe RexOp1ld:
    RecipeSizing {
        base_size: 3,
        compute_size: size_plus_maybe_sib_or_offset_for_in_reg_0,
        branch_range: None,
    },
    // Code size information for recipe Op2ld:
    RecipeSizing {
        base_size: 3,
        compute_size: size_plus_maybe_sib_or_offset_for_in_reg_0,
        branch_range: None,
    },
    // Code size information for recipe RexOp2ld:
    RecipeSizing {
        base_size: 4,
        compute_size: size_plus_maybe_sib_or_offset_for_in_reg_0,
        branch_range: None,
    },
    // Code size information for recipe Op1ldDisp8:
    RecipeSizing {
        base_size: 3,
        compute_size: size_plus_maybe_sib_for_in_reg_0,
        branch_range: None,
    },
    // Code size information for recipe RexOp1ldDisp8:
    RecipeSizing {
        base_size: 4,
        compute_size: size_plus_maybe_sib_for_in_reg_0,
        branch_range: None,
    },
    // Code size information for recipe Op2ldDisp8:
    RecipeSizing {
        base_size: 4,
        compute_size: size_plus_maybe_sib_for_in_reg_0,
        branch_range: None,
    },
    // Code size information for recipe RexOp2ldDisp8:
    RecipeSizing {
        base_size: 5,
        compute_size: size_plus_maybe_sib_for_in_reg_0,
        branch_range: None,
    },
    // Code size information for recipe Op1ldDisp32:
    RecipeSizing {
        base_size: 6,
        compute_size: size_plus_maybe_sib_for_in_reg_0,
        branch_range: None,
    },
    // Code size information for recipe RexOp1ldDisp32:
    RecipeSizing {
        base_size: 7,
        compute_size: size_plus_maybe_sib_for_in_reg_0,
        branch_range: None,
    },
    // Code size information for recipe Op2ldDisp32:
    RecipeSizing {
        base_size: 7,
        compute_size: size_plus_maybe_sib_for_in_reg_0,
        branch_range: None,
    },
    // Code size information for recipe RexOp2ldDisp32:
    RecipeSizing {
        base_size: 8,
        compute_size: size_plus_maybe_sib_for_in_reg_0,
        branch_range: None,
    },
    // Code size information for recipe Op1fillSib32:
    RecipeSizing {
        base_size: 7,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1fillSib32:
    RecipeSizing {
        base_size: 8,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1regfill32:
    RecipeSizing {
        base_size: 7,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1regfill32:
    RecipeSizing {
        base_size: 8,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1pushq:
    RecipeSizing {
        base_size: 1,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1pushq:
    RecipeSizing {
        base_size: 2,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1popq:
    RecipeSizing {
        base_size: 1,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1popq:
    RecipeSizing {
        base_size: 2,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1copysp:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1copysp:
    RecipeSizing {
        base_size: 2,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe stacknull:
    RecipeSizing {
        base_size: 0,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1adjustsp:
    RecipeSizing {
        base_size: 2,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1adjustsp:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1adjustsp_ib:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1adjustsp_id:
    RecipeSizing {
        base_size: 6,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1adjustsp_ib:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1adjustsp_id:
    RecipeSizing {
        base_size: 7,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Mp2fld:
    RecipeSizing {
        base_size: 4,
        compute_size: size_plus_maybe_sib_or_offset_for_in_reg_0,
        branch_range: None,
    },
    // Code size information for recipe RexMp2fld:
    RecipeSizing {
        base_size: 5,
        compute_size: size_plus_maybe_sib_or_offset_for_in_reg_0,
        branch_range: None,
    },
    // Code size information for recipe Mp2fldDisp8:
    RecipeSizing {
        base_size: 5,
        compute_size: size_plus_maybe_sib_for_in_reg_0,
        branch_range: None,
    },
    // Code size information for recipe RexMp2fldDisp8:
    RecipeSizing {
        base_size: 6,
        compute_size: size_plus_maybe_sib_for_in_reg_0,
        branch_range: None,
    },
    // Code size information for recipe Mp2fldDisp32:
    RecipeSizing {
        base_size: 8,
        compute_size: size_plus_maybe_sib_for_in_reg_0,
        branch_range: None,
    },
    // Code size information for recipe RexMp2fldDisp32:
    RecipeSizing {
        base_size: 9,
        compute_size: size_plus_maybe_sib_for_in_reg_0,
        branch_range: None,
    },
    // Code size information for recipe Mp2fldWithIndex:
    RecipeSizing {
        base_size: 5,
        compute_size: size_plus_maybe_offset_for_in_reg_0,
        branch_range: None,
    },
    // Code size information for recipe RexMp2fldWithIndex:
    RecipeSizing {
        base_size: 6,
        compute_size: size_plus_maybe_offset_for_in_reg_0,
        branch_range: None,
    },
    // Code size information for recipe Mp2fldWithIndexDisp8:
    RecipeSizing {
        base_size: 6,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexMp2fldWithIndexDisp8:
    RecipeSizing {
        base_size: 7,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Mp2fldWithIndexDisp32:
    RecipeSizing {
        base_size: 9,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexMp2fldWithIndexDisp32:
    RecipeSizing {
        base_size: 10,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Mp2fst:
    RecipeSizing {
        base_size: 4,
        compute_size: size_plus_maybe_sib_or_offset_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe RexMp2fst:
    RecipeSizing {
        base_size: 5,
        compute_size: size_plus_maybe_sib_or_offset_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe Mp2fstDisp8:
    RecipeSizing {
        base_size: 5,
        compute_size: size_plus_maybe_sib_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe RexMp2fstDisp8:
    RecipeSizing {
        base_size: 6,
        compute_size: size_plus_maybe_sib_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe Mp2fstDisp32:
    RecipeSizing {
        base_size: 8,
        compute_size: size_plus_maybe_sib_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe RexMp2fstDisp32:
    RecipeSizing {
        base_size: 9,
        compute_size: size_plus_maybe_sib_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe Mp2fstWithIndex:
    RecipeSizing {
        base_size: 5,
        compute_size: size_plus_maybe_offset_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe RexMp2fstWithIndex:
    RecipeSizing {
        base_size: 6,
        compute_size: size_plus_maybe_offset_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe Mp2fstWithIndexDisp8:
    RecipeSizing {
        base_size: 6,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexMp2fstWithIndexDisp8:
    RecipeSizing {
        base_size: 7,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Mp2fstWithIndexDisp32:
    RecipeSizing {
        base_size: 9,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexMp2fstWithIndexDisp32:
    RecipeSizing {
        base_size: 10,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Mp2ffillSib32:
    RecipeSizing {
        base_size: 9,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexMp2ffillSib32:
    RecipeSizing {
        base_size: 10,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Mp2fregfill32:
    RecipeSizing {
        base_size: 9,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexMp2fregfill32:
    RecipeSizing {
        base_size: 10,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Mp2fspillSib32:
    RecipeSizing {
        base_size: 9,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexMp2fspillSib32:
    RecipeSizing {
        base_size: 10,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Mp2fregspill32:
    RecipeSizing {
        base_size: 9,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexMp2fregspill32:
    RecipeSizing {
        base_size: 10,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1fnaddr4:
    RecipeSizing {
        base_size: 5,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1fnaddr8:
    RecipeSizing {
        base_size: 10,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1allones_fnaddr4:
    RecipeSizing {
        base_size: 5,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1allones_fnaddr8:
    RecipeSizing {
        base_size: 10,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1pcrel_fnaddr8:
    RecipeSizing {
        base_size: 7,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1got_fnaddr8:
    RecipeSizing {
        base_size: 7,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1gvaddr4:
    RecipeSizing {
        base_size: 5,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1gvaddr8:
    RecipeSizing {
        base_size: 10,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1pcrel_gvaddr8:
    RecipeSizing {
        base_size: 7,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1got_gvaddr8:
    RecipeSizing {
        base_size: 7,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1spaddr4_id:
    RecipeSizing {
        base_size: 7,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1spaddr8_id:
    RecipeSizing {
        base_size: 8,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1call_id:
    RecipeSizing {
        base_size: 5,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1call_plt_id:
    RecipeSizing {
        base_size: 5,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1call_r:
    RecipeSizing {
        base_size: 2,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1call_r:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1ret:
    RecipeSizing {
        base_size: 1,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1jmpb:
    RecipeSizing {
        base_size: 2,
        compute_size: base_size,
        branch_range: Some(BranchRange { origin: 2, bits: 8 }),
    },
    // Code size information for recipe Op1jmpd:
    RecipeSizing {
        base_size: 5,
        compute_size: base_size,
        branch_range: Some(BranchRange {
            origin: 5,
            bits: 32,
        }),
    },
    // Code size information for recipe Op1brib:
    RecipeSizing {
        base_size: 2,
        compute_size: base_size,
        branch_range: Some(BranchRange { origin: 2, bits: 8 }),
    },
    // Code size information for recipe RexOp1brib:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: Some(BranchRange { origin: 3, bits: 8 }),
    },
    // Code size information for recipe Op2brid:
    RecipeSizing {
        base_size: 6,
        compute_size: base_size,
        branch_range: Some(BranchRange {
            origin: 6,
            bits: 32,
        }),
    },
    // Code size information for recipe RexOp2brid:
    RecipeSizing {
        base_size: 7,
        compute_size: base_size,
        branch_range: Some(BranchRange {
            origin: 7,
            bits: 32,
        }),
    },
    // Code size information for recipe Op1brfb:
    RecipeSizing {
        base_size: 2,
        compute_size: base_size,
        branch_range: Some(BranchRange { origin: 2, bits: 8 }),
    },
    // Code size information for recipe RexOp1brfb:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: Some(BranchRange { origin: 3, bits: 8 }),
    },
    // Code size information for recipe Op2brfd:
    RecipeSizing {
        base_size: 6,
        compute_size: base_size,
        branch_range: Some(BranchRange {
            origin: 6,
            bits: 32,
        }),
    },
    // Code size information for recipe RexOp2brfd:
    RecipeSizing {
        base_size: 7,
        compute_size: base_size,
        branch_range: Some(BranchRange {
            origin: 7,
            bits: 32,
        }),
    },
    // Code size information for recipe Op1tjccb:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: Some(BranchRange { origin: 4, bits: 8 }),
    },
    // Code size information for recipe RexOp1tjccb:
    RecipeSizing {
        base_size: 5,
        compute_size: base_size,
        branch_range: Some(BranchRange { origin: 5, bits: 8 }),
    },
    // Code size information for recipe Op1tjccd:
    RecipeSizing {
        base_size: 8,
        compute_size: base_size,
        branch_range: Some(BranchRange {
            origin: 8,
            bits: 32,
        }),
    },
    // Code size information for recipe RexOp1tjccd:
    RecipeSizing {
        base_size: 9,
        compute_size: base_size,
        branch_range: Some(BranchRange {
            origin: 9,
            bits: 32,
        }),
    },
    // Code size information for recipe Op1t8jccd_long:
    RecipeSizing {
        base_size: 12,
        compute_size: base_size,
        branch_range: Some(BranchRange {
            origin: 12,
            bits: 32,
        }),
    },
    // Code size information for recipe Op1t8jccb_abcd:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: Some(BranchRange { origin: 4, bits: 8 }),
    },
    // Code size information for recipe RexOp1t8jccb:
    RecipeSizing {
        base_size: 5,
        compute_size: base_size,
        branch_range: Some(BranchRange { origin: 5, bits: 8 }),
    },
    // Code size information for recipe Op1t8jccd_abcd:
    RecipeSizing {
        base_size: 8,
        compute_size: base_size,
        branch_range: Some(BranchRange {
            origin: 8,
            bits: 32,
        }),
    },
    // Code size information for recipe RexOp1t8jccd:
    RecipeSizing {
        base_size: 9,
        compute_size: base_size,
        branch_range: Some(BranchRange {
            origin: 9,
            bits: 32,
        }),
    },
    // Code size information for recipe RexOp1jt_entry:
    RecipeSizing {
        base_size: 4,
        compute_size: size_plus_maybe_offset_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe Op1jt_entry:
    RecipeSizing {
        base_size: 3,
        compute_size: size_plus_maybe_offset_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe RexOp1jt_base:
    RecipeSizing {
        base_size: 7,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1jt_base:
    RecipeSizing {
        base_size: 6,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1indirect_jmp:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1indirect_jmp:
    RecipeSizing {
        base_size: 2,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op2trap:
    RecipeSizing {
        base_size: 2,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe debugtrap:
    RecipeSizing {
        base_size: 1,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe trapif:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe trapff:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1icscc:
    RecipeSizing {
        base_size: 5,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1icscc:
    RecipeSizing {
        base_size: 6,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1icscc_ib:
    RecipeSizing {
        base_size: 6,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1icscc_ib:
    RecipeSizing {
        base_size: 7,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1icscc_id:
    RecipeSizing {
        base_size: 9,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1icscc_id:
    RecipeSizing {
        base_size: 10,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1rcmp:
    RecipeSizing {
        base_size: 2,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1rcmp:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1rcmp_ib:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1rcmp_ib:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1rcmp_id:
    RecipeSizing {
        base_size: 6,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1rcmp_id:
    RecipeSizing {
        base_size: 7,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1rcmp_sp:
    RecipeSizing {
        base_size: 2,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1rcmp_sp:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op2seti_abcd:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp2seti:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op2setf_abcd:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp2setf:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op2cmov:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp2cmov:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op2bsf_and_bsr:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp2bsf_and_bsr:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op2urm_noflags_abcd:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp2urm_noflags:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe null:
    RecipeSizing {
        base_size: 0,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op2urm_noflags:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1urm_noflags:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op2f32imm_z:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Mp2f64imm_z:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp2f32imm_z:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexMp2f64imm_z:
    RecipeSizing {
        base_size: 5,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Mp2frurm:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexMp2frurm:
    RecipeSizing {
        base_size: 5,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Mp2rfumr:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexMp2rfumr:
    RecipeSizing {
        base_size: 5,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op2furm:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp2furm:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op2frmov:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp2frmov:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Mp2furm:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexMp2furm:
    RecipeSizing {
        base_size: 5,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Mp2rfurm:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexMp2rfurm:
    RecipeSizing {
        base_size: 5,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Mp3furmi_rnd:
    RecipeSizing {
        base_size: 6,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexMp3furmi_rnd:
    RecipeSizing {
        base_size: 7,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Mp2fa:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexMp2fa:
    RecipeSizing {
        base_size: 5,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op2fa:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp2fa:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op2fax:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp2fax:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op2fcscc:
    RecipeSizing {
        base_size: 6,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp2fcscc:
    RecipeSizing {
        base_size: 7,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Mp2fcscc:
    RecipeSizing {
        base_size: 7,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexMp2fcscc:
    RecipeSizing {
        base_size: 8,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op2fcmp:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp2fcmp:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Mp2fcmp:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexMp2fcmp:
    RecipeSizing {
        base_size: 5,
        compute_size: base_size,
        branch_range: None,
    },
];
pub static INFO: isa::EncInfo = isa::EncInfo {
    constraints: &RECIPE_CONSTRAINTS,
    sizing: &RECIPE_SIZING,
    names: &RECIPE_NAMES,
};

//clude!(concat!(env!("OUT_DIR"), "/encoding-x86.rs"));

/// Legalize instructions by expansion.
///
/// Use x86-specific instructions if needed.
#[allow(unused_variables, unused_assignments, non_snake_case)]
pub fn x86_expand(
    inst: ir::Inst,
    func: &mut ir::Function,
    cfg: &mut ControlFlowGraph,
    isa: &dyn TargetIsa,
) -> bool {
    use ir::InstBuilder;
    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);
    {
        match pos.func.dfg[inst].opcode() {
            ir::Opcode::Clz => {
                // Unwrap a << clz.i64(x)
                let (x, predicate) =
                    if let ir::InstructionData::Unary { arg, .. } = pos.func.dfg[inst] {
                        let func = &pos.func;
                        let args = [arg];
                        (
                            func.dfg.resolve_aliases(args[0]),
                            func.dfg.value_type(args[0]) == ir::types::I64,
                        )
                    } else {
                        unreachable!("bad instruction format")
                    };
                // Results handled by a << isub(c_sixty_three, index2).
                let results = pos.func.dfg.inst_results(inst);
                let a = &results[0];
                if predicate {
                    let c_minus_one = pos.ins().iconst(ir::types::I64, -1);
                    let c_sixty_three = pos.ins().iconst(ir::types::I64, 63);
                    let (index1, r2flags) = pos.ins().x86_bsr(x);
                    let index2 = pos.ins().selectif(
                        ir::types::I64,
                        ir::condcodes::IntCC::Equal,
                        r2flags,
                        c_minus_one,
                        index1,
                    );
                    pos.func.dfg.replace(inst).isub(c_sixty_three, index2);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
                // Unwrap a << clz.i32(x)
                let (x, predicate) =
                    if let ir::InstructionData::Unary { arg, .. } = pos.func.dfg[inst] {
                        let func = &pos.func;
                        let args = [arg];
                        (
                            func.dfg.resolve_aliases(args[0]),
                            func.dfg.value_type(args[0]) == ir::types::I32,
                        )
                    } else {
                        unreachable!("bad instruction format")
                    };
                // Results handled by a << isub(c_thirty_one, index2).
                let results = pos.func.dfg.inst_results(inst);
                let a = &results[0];
                if predicate {
                    let c_minus_one = pos.ins().iconst(ir::types::I32, -1);
                    let c_thirty_one = pos.ins().iconst(ir::types::I32, 31);
                    let (index1, r2flags) = pos.ins().x86_bsr(x);
                    let index2 = pos.ins().selectif(
                        ir::types::I32,
                        ir::condcodes::IntCC::Equal,
                        r2flags,
                        c_minus_one,
                        index1,
                    );
                    pos.func.dfg.replace(inst).isub(c_thirty_one, index2);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Ctz => {
                // Unwrap a << ctz.i64(x)
                let (x, predicate) =
                    if let ir::InstructionData::Unary { arg, .. } = pos.func.dfg[inst] {
                        let func = &pos.func;
                        let args = [arg];
                        (
                            func.dfg.resolve_aliases(args[0]),
                            func.dfg.value_type(args[0]) == ir::types::I64,
                        )
                    } else {
                        unreachable!("bad instruction format")
                    };
                // Results handled by a << selectif(ir::condcodes::IntCC::Equal, r2flags, c_sixty_four, index1).
                let results = pos.func.dfg.inst_results(inst);
                let a = &results[0];
                if predicate {
                    let c_sixty_four = pos.ins().iconst(ir::types::I64, 64);
                    let (index1, r2flags) = pos.ins().x86_bsf(x);
                    pos.func.dfg.replace(inst).selectif(
                        ir::types::I64,
                        ir::condcodes::IntCC::Equal,
                        r2flags,
                        c_sixty_four,
                        index1,
                    );
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
                // Unwrap a << ctz.i32(x)
                let (x, predicate) =
                    if let ir::InstructionData::Unary { arg, .. } = pos.func.dfg[inst] {
                        let func = &pos.func;
                        let args = [arg];
                        (
                            func.dfg.resolve_aliases(args[0]),
                            func.dfg.value_type(args[0]) == ir::types::I32,
                        )
                    } else {
                        unreachable!("bad instruction format")
                    };
                // Results handled by a << selectif(ir::condcodes::IntCC::Equal, r2flags, c_thirty_two, index1).
                let results = pos.func.dfg.inst_results(inst);
                let a = &results[0];
                if predicate {
                    let c_thirty_two = pos.ins().iconst(ir::types::I32, 32);
                    let (index1, r2flags) = pos.ins().x86_bsf(x);
                    pos.func.dfg.replace(inst).selectif(
                        ir::types::I32,
                        ir::condcodes::IntCC::Equal,
                        r2flags,
                        c_thirty_two,
                        index1,
                    );
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Fcmp => {
                // Unwrap a << fcmp(ir::condcodes::FloatCC::Equal, x, y)
                let (_, x, y, predicate) =
                    if let ir::InstructionData::FloatCompare { cond, ref args, .. } =
                        pos.func.dfg[inst]
                    {
                        let func = &pos.func;
                        (
                            cond,
                            func.dfg.resolve_aliases(args[0]),
                            func.dfg.resolve_aliases(args[1]),
                            predicates::is_equal(cond, ir::condcodes::FloatCC::Equal),
                        )
                    } else {
                        unreachable!("bad instruction format")
                    };
                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a << band(a1, a2).
                let results = pos.func.dfg.inst_results(inst);
                let a = &results[0];
                if predicate {
                    let a1 = pos.ins().fcmp(ir::condcodes::FloatCC::Ordered, x, y);
                    let a2 = pos
                        .ins()
                        .fcmp(ir::condcodes::FloatCC::UnorderedOrEqual, x, y);
                    pos.func.dfg.replace(inst).band(a1, a2);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
                // Unwrap a << fcmp(ir::condcodes::FloatCC::NotEqual, x, y)
                let (_, x, y, predicate) =
                    if let ir::InstructionData::FloatCompare { cond, ref args, .. } =
                        pos.func.dfg[inst]
                    {
                        let func = &pos.func;
                        (
                            cond,
                            func.dfg.resolve_aliases(args[0]),
                            func.dfg.resolve_aliases(args[1]),
                            predicates::is_equal(cond, ir::condcodes::FloatCC::NotEqual),
                        )
                    } else {
                        unreachable!("bad instruction format")
                    };
                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a << bor(a1, a2).
                let results = pos.func.dfg.inst_results(inst);
                let a = &results[0];
                if predicate {
                    let a1 = pos.ins().fcmp(ir::condcodes::FloatCC::Unordered, x, y);
                    let a2 = pos
                        .ins()
                        .fcmp(ir::condcodes::FloatCC::OrderedNotEqual, x, y);
                    pos.func.dfg.replace(inst).bor(a1, a2);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
                // Unwrap a << fcmp(ir::condcodes::FloatCC::LessThan, x, y)
                let (_, x, y, predicate) =
                    if let ir::InstructionData::FloatCompare { cond, ref args, .. } =
                        pos.func.dfg[inst]
                    {
                        let func = &pos.func;
                        (
                            cond,
                            func.dfg.resolve_aliases(args[0]),
                            func.dfg.resolve_aliases(args[1]),
                            predicates::is_equal(cond, ir::condcodes::FloatCC::LessThan),
                        )
                    } else {
                        unreachable!("bad instruction format")
                    };
                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a << fcmp(ir::condcodes::FloatCC::GreaterThan, y, x).
                let results = pos.func.dfg.inst_results(inst);
                let a = &results[0];
                if predicate {
                    pos.func
                        .dfg
                        .replace(inst)
                        .fcmp(ir::condcodes::FloatCC::GreaterThan, y, x);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
                // Unwrap a << fcmp(ir::condcodes::FloatCC::LessThanOrEqual, x, y)
                let (_, x, y, predicate) =
                    if let ir::InstructionData::FloatCompare { cond, ref args, .. } =
                        pos.func.dfg[inst]
                    {
                        let func = &pos.func;
                        (
                            cond,
                            func.dfg.resolve_aliases(args[0]),
                            func.dfg.resolve_aliases(args[1]),
                            predicates::is_equal(cond, ir::condcodes::FloatCC::LessThanOrEqual),
                        )
                    } else {
                        unreachable!("bad instruction format")
                    };
                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a << fcmp(ir::condcodes::FloatCC::GreaterThanOrEqual, y, x).
                let results = pos.func.dfg.inst_results(inst);
                let a = &results[0];
                if predicate {
                    pos.func.dfg.replace(inst).fcmp(
                        ir::condcodes::FloatCC::GreaterThanOrEqual,
                        y,
                        x,
                    );
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
                // Unwrap a << fcmp(ir::condcodes::FloatCC::UnorderedOrGreaterThan, x, y)
                let (_, x, y, predicate) =
                    if let ir::InstructionData::FloatCompare { cond, ref args, .. } =
                        pos.func.dfg[inst]
                    {
                        let func = &pos.func;
                        (
                            cond,
                            func.dfg.resolve_aliases(args[0]),
                            func.dfg.resolve_aliases(args[1]),
                            predicates::is_equal(
                                cond,
                                ir::condcodes::FloatCC::UnorderedOrGreaterThan,
                            ),
                        )
                    } else {
                        unreachable!("bad instruction format")
                    };
                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a << fcmp(ir::condcodes::FloatCC::UnorderedOrLessThan, y, x).
                let results = pos.func.dfg.inst_results(inst);
                let a = &results[0];
                if predicate {
                    pos.func.dfg.replace(inst).fcmp(
                        ir::condcodes::FloatCC::UnorderedOrLessThan,
                        y,
                        x,
                    );
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
                // Unwrap a << fcmp(ir::condcodes::FloatCC::UnorderedOrGreaterThanOrEqual, x, y)
                let (_, x, y, predicate) =
                    if let ir::InstructionData::FloatCompare { cond, ref args, .. } =
                        pos.func.dfg[inst]
                    {
                        let func = &pos.func;
                        (
                            cond,
                            func.dfg.resolve_aliases(args[0]),
                            func.dfg.resolve_aliases(args[1]),
                            predicates::is_equal(
                                cond,
                                ir::condcodes::FloatCC::UnorderedOrGreaterThanOrEqual,
                            ),
                        )
                    } else {
                        unreachable!("bad instruction format")
                    };
                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a << fcmp(ir::condcodes::FloatCC::UnorderedOrLessThanOrEqual, y, x).
                let results = pos.func.dfg.inst_results(inst);
                let a = &results[0];
                if predicate {
                    pos.func.dfg.replace(inst).fcmp(
                        ir::condcodes::FloatCC::UnorderedOrLessThanOrEqual,
                        y,
                        x,
                    );
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Popcnt => {
                // Unwrap qv16 << popcnt.i64(qv1)
                let (qv1, predicate) =
                    if let ir::InstructionData::Unary { arg, .. } = pos.func.dfg[inst] {
                        let func = &pos.func;
                        let args = [arg];
                        (
                            func.dfg.resolve_aliases(args[0]),
                            func.dfg.value_type(args[0]) == ir::types::I64,
                        )
                    } else {
                        unreachable!("bad instruction format")
                    };
                // Results handled by qv16 << ushr_imm(qv15, 56).
                let results = pos.func.dfg.inst_results(inst);
                let qv16 = &results[0];
                if predicate {
                    let qv3 = pos.ins().ushr_imm(qv1, 1);
                    let qc77 = pos.ins().iconst(ir::types::I64, 8608480567731124087);
                    let qv4 = pos.ins().band(qv3, qc77);
                    let qv5 = pos.ins().isub(qv1, qv4);
                    let qv6 = pos.ins().ushr_imm(qv4, 1);
                    let qv7 = pos.ins().band(qv6, qc77);
                    let qv8 = pos.ins().isub(qv5, qv7);
                    let qv9 = pos.ins().ushr_imm(qv7, 1);
                    let qv10 = pos.ins().band(qv9, qc77);
                    let qv11 = pos.ins().isub(qv8, qv10);
                    let qv12 = pos.ins().ushr_imm(qv11, 4);
                    let qv13 = pos.ins().iadd(qv11, qv12);
                    let qc0F = pos.ins().iconst(ir::types::I64, 1085102592571150095);
                    let qv14 = pos.ins().band(qv13, qc0F);
                    let qc01 = pos.ins().iconst(ir::types::I64, 72340172838076673);
                    let qv15 = pos.ins().imul(qv14, qc01);
                    pos.func.dfg.replace(inst).ushr_imm(qv15, 56);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
                // Unwrap lv16 << popcnt.i32(lv1)
                let (lv1, predicate) =
                    if let ir::InstructionData::Unary { arg, .. } = pos.func.dfg[inst] {
                        let func = &pos.func;
                        let args = [arg];
                        (
                            func.dfg.resolve_aliases(args[0]),
                            func.dfg.value_type(args[0]) == ir::types::I32,
                        )
                    } else {
                        unreachable!("bad instruction format")
                    };
                // Results handled by lv16 << ushr_imm(lv15, 24).
                let results = pos.func.dfg.inst_results(inst);
                let lv16 = &results[0];
                if predicate {
                    let lv3 = pos.ins().ushr_imm(lv1, 1);
                    let lc77 = pos.ins().iconst(ir::types::I32, 2004318071);
                    let lv4 = pos.ins().band(lv3, lc77);
                    let lv5 = pos.ins().isub(lv1, lv4);
                    let lv6 = pos.ins().ushr_imm(lv4, 1);
                    let lv7 = pos.ins().band(lv6, lc77);
                    let lv8 = pos.ins().isub(lv5, lv7);
                    let lv9 = pos.ins().ushr_imm(lv7, 1);
                    let lv10 = pos.ins().band(lv9, lc77);
                    let lv11 = pos.ins().isub(lv8, lv10);
                    let lv12 = pos.ins().ushr_imm(lv11, 4);
                    let lv13 = pos.ins().iadd(lv11, lv12);
                    let lc0F = pos.ins().iconst(ir::types::I32, 252645135);
                    let lv14 = pos.ins().band(lv13, lc0F);
                    let lc01 = pos.ins().iconst(ir::types::I32, 16843009);
                    let lv15 = pos.ins().imul(lv14, lc01);
                    pos.func.dfg.replace(inst).ushr_imm(lv15, 24);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Smulhi => {
                // Unwrap res_hi << smulhi(x, y)
                let (x, y, predicate) =
                    if let ir::InstructionData::Binary { ref args, .. } = pos.func.dfg[inst] {
                        let func = &pos.func;
                        (
                            func.dfg.resolve_aliases(args[0]),
                            func.dfg.resolve_aliases(args[1]),
                            true,
                        )
                    } else {
                        unreachable!("bad instruction format")
                    };
                let typeof_x = pos.func.dfg.value_type(x);
                let res_hi;
                {
                    let r = pos.func.dfg.inst_results(inst);
                    res_hi = r[0];
                }
                // typeof_x must belong to TypeSet(lanes={1}, ints={32, 64})
                let predicate = predicate && TYPE_SETS[0].contains(typeof_x);
                if predicate {
                    pos.func.dfg.clear_results(inst);
                    let (res_lo, res_hi) =
                        pos.ins().with_results([None, Some(res_hi)]).x86_smulx(x, y);
                    let removed = pos.remove_inst();
                    debug_assert_eq!(removed, inst);
                    return true;
                }
            }

            ir::Opcode::Umulhi => {
                // Unwrap res_hi << umulhi(x, y)
                let (x, y, predicate) =
                    if let ir::InstructionData::Binary { ref args, .. } = pos.func.dfg[inst] {
                        let func = &pos.func;
                        (
                            func.dfg.resolve_aliases(args[0]),
                            func.dfg.resolve_aliases(args[1]),
                            true,
                        )
                    } else {
                        unreachable!("bad instruction format")
                    };
                let typeof_x = pos.func.dfg.value_type(x);
                let res_hi;
                {
                    let r = pos.func.dfg.inst_results(inst);
                    res_hi = r[0];
                }
                // typeof_x must belong to TypeSet(lanes={1}, ints={32, 64})
                let predicate = predicate && TYPE_SETS[0].contains(typeof_x);
                if predicate {
                    pos.func.dfg.clear_results(inst);
                    let (res_lo, res_hi) =
                        pos.ins().with_results([None, Some(res_hi)]).x86_umulx(x, y);
                    let removed = pos.remove_inst();
                    debug_assert_eq!(removed, inst);
                    return true;
                }
            }

            ir::Opcode::Fmax => {
                expand_minmax(inst, pos.func, cfg, isa);
                return true;
            }

            ir::Opcode::Sdiv => {
                expand_sdivrem(inst, pos.func, cfg, isa);
                return true;
            }

            ir::Opcode::FcvtToSintSat => {
                expand_fcvt_to_sint_sat(inst, pos.func, cfg, isa);
                return true;
            }

            ir::Opcode::FcvtToSint => {
                expand_fcvt_to_sint(inst, pos.func, cfg, isa);
                return true;
            }

            ir::Opcode::Udiv => {
                expand_udivrem(inst, pos.func, cfg, isa);
                return true;
            }

            ir::Opcode::Urem => {
                expand_udivrem(inst, pos.func, cfg, isa);
                return true;
            }

            ir::Opcode::FcvtToUintSat => {
                expand_fcvt_to_uint_sat(inst, pos.func, cfg, isa);
                return true;
            }

            ir::Opcode::FcvtToUint => {
                expand_fcvt_to_uint(inst, pos.func, cfg, isa);
                return true;
            }

            ir::Opcode::FcvtFromUint => {
                expand_fcvt_from_uint(inst, pos.func, cfg, isa);
                return true;
            }

            ir::Opcode::Srem => {
                expand_sdivrem(inst, pos.func, cfg, isa);
                return true;
            }

            ir::Opcode::Fmin => {
                expand_minmax(inst, pos.func, cfg, isa);
                return true;
            }

            _ => {}
        }
    }
    crate::legalizer::expand_flags(inst, pos.func, cfg, isa)
}

// Table of value type sets.
const TYPE_SETS: [ir::instructions::ValueTypeSet; 1] = [ir::instructions::ValueTypeSet {
    // TypeSet(lanes={1}, ints={32, 64})
    lanes: BitSet::<u16>(1),
    ints: BitSet::<u8>(96),
    floats: BitSet::<u8>(0),
    bools: BitSet::<u8>(0),
}];
pub static LEGALIZE_ACTIONS: [isa::Legalize; 4] = [
    crate::legalizer::narrow,
    crate::legalizer::expand_flags,
    crate::legalizer::widen,
    x86_expand,
];

//clude!(concat!(env!("OUT_DIR"), "/legalize-x86.rs"));

pub fn needs_sib_byte(reg: RegUnit) -> bool {
    reg == RU::r12 as RegUnit || reg == RU::rsp as RegUnit
}
pub fn needs_offset(reg: RegUnit) -> bool {
    reg == RU::r13 as RegUnit || reg == RU::rbp as RegUnit
}
pub fn needs_sib_byte_or_offset(reg: RegUnit) -> bool {
    needs_sib_byte(reg) || needs_offset(reg)
}

fn additional_size_if(
    op_index: usize,
    inst: Inst,
    divert: &RegDiversions,
    func: &Function,
    condition_func: fn(RegUnit) -> bool,
) -> u8 {
    let addr_reg = divert.reg(func.dfg.inst_args(inst)[op_index], &func.locations);
    if condition_func(addr_reg) {
        1
    } else {
        0
    }
}

fn size_plus_maybe_offset_for_in_reg_0(
    sizing: &RecipeSizing,
    inst: Inst,
    divert: &RegDiversions,
    func: &Function,
) -> u8 {
    sizing.base_size + additional_size_if(0, inst, divert, func, needs_offset)
}
fn size_plus_maybe_offset_for_in_reg_1(
    sizing: &RecipeSizing,
    inst: Inst,
    divert: &RegDiversions,
    func: &Function,
) -> u8 {
    sizing.base_size + additional_size_if(1, inst, divert, func, needs_offset)
}
fn size_plus_maybe_sib_for_in_reg_0(
    sizing: &RecipeSizing,
    inst: Inst,
    divert: &RegDiversions,
    func: &Function,
) -> u8 {
    sizing.base_size + additional_size_if(0, inst, divert, func, needs_sib_byte)
}
fn size_plus_maybe_sib_for_in_reg_1(
    sizing: &RecipeSizing,
    inst: Inst,
    divert: &RegDiversions,
    func: &Function,
) -> u8 {
    sizing.base_size + additional_size_if(1, inst, divert, func, needs_sib_byte)
}
fn size_plus_maybe_sib_or_offset_for_in_reg_0(
    sizing: &RecipeSizing,
    inst: Inst,
    divert: &RegDiversions,
    func: &Function,
) -> u8 {
    sizing.base_size + additional_size_if(0, inst, divert, func, needs_sib_byte_or_offset)
}
fn size_plus_maybe_sib_or_offset_for_in_reg_1(
    sizing: &RecipeSizing,
    inst: Inst,
    divert: &RegDiversions,
    func: &Function,
) -> u8 {
    sizing.base_size + additional_size_if(1, inst, divert, func, needs_sib_byte_or_offset)
}

/// If the value's definition is a constant immediate, returns its unpacked value, or None
/// otherwise.
fn maybe_iconst_imm(pos: &FuncCursor, value: ir::Value) -> Option<i64> {
    if let ir::ValueDef::Result(inst, _) = &pos.func.dfg.value_def(value) {
        if let ir::InstructionData::UnaryImm {
            opcode: ir::Opcode::Iconst,
            imm,
        } = &pos.func.dfg[*inst]
        {
            let value: i64 = (*imm).into();
            Some(value)
        } else {
            None
        }
    } else {
        None
    }
}

/// Expand the `sdiv` and `srem` instructions using `x86_sdivmodx`.
fn expand_sdivrem(
    inst: ir::Inst,
    func: &mut ir::Function,
    cfg: &mut ControlFlowGraph,
    isa: &dyn TargetIsa,
) {
    let (x, y, is_srem) = match func.dfg[inst] {
        ir::InstructionData::Binary {
            opcode: ir::Opcode::Sdiv,
            args,
        } => (args[0], args[1], false),
        ir::InstructionData::Binary {
            opcode: ir::Opcode::Srem,
            args,
        } => (args[0], args[1], true),
        _ => panic!("Need sdiv/srem: {}", func.dfg.display_inst(inst, None)),
    };

    let old_ebb = func.layout.pp_ebb(inst);
    let result = func.dfg.first_result(inst);
    let ty = func.dfg.value_type(result);

    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);
    pos.func.dfg.clear_results(inst);

    let avoid_div_traps = isa.flags().avoid_div_traps();

    // If we can tolerate native division traps, sdiv doesn't need branching.
    if !avoid_div_traps && !is_srem {
        let xhi = pos.ins().sshr_imm(x, i64::from(ty.lane_bits()) - 1);
        pos.ins().with_result(result).x86_sdivmodx(x, xhi, y);
        pos.remove_inst();
        return;
    }

    // Try to remove checks if the input value is an immediate other than 0 or -1. For these two
    // immediates, we'd ideally replace conditional traps by traps, but this requires more
    // manipulation of the dfg/cfg, which is out of scope here.
    let (could_be_zero, could_be_minus_one) = if let Some(imm) = maybe_iconst_imm(&pos, y) {
        (imm == 0, imm == -1)
    } else {
        (true, true)
    };

    // Put in an explicit division-by-zero trap if the environment requires it.
    if avoid_div_traps && could_be_zero {
        pos.ins().trapz(y, ir::TrapCode::IntegerDivisionByZero);
    }

    if !could_be_minus_one {
        let xhi = pos.ins().sshr_imm(x, i64::from(ty.lane_bits()) - 1);
        let reuse = if is_srem {
            [None, Some(result)]
        } else {
            [Some(result), None]
        };
        pos.ins().with_results(reuse).x86_sdivmodx(x, xhi, y);
        pos.remove_inst();
        return;
    }

    // EBB handling the nominal case.
    let nominal = pos.func.dfg.make_ebb();

    // EBB handling the -1 divisor case.
    let minus_one = pos.func.dfg.make_ebb();

    // Final EBB with one argument representing the final result value.
    let done = pos.func.dfg.make_ebb();

    // Move the `inst` result value onto the `done` EBB.
    pos.func.dfg.attach_ebb_param(done, result);

    // Start by checking for a -1 divisor which needs to be handled specially.
    let is_m1 = pos.ins().ifcmp_imm(y, -1);
    pos.ins().brif(IntCC::Equal, is_m1, minus_one, &[]);
    pos.ins().jump(nominal, &[]);

    // Now it is safe to execute the `x86_sdivmodx` instruction which will still trap on division
    // by zero.
    pos.insert_ebb(nominal);
    let xhi = pos.ins().sshr_imm(x, i64::from(ty.lane_bits()) - 1);
    let (quot, rem) = pos.ins().x86_sdivmodx(x, xhi, y);
    let divres = if is_srem { rem } else { quot };
    pos.ins().jump(done, &[divres]);

    // Now deal with the -1 divisor case.
    pos.insert_ebb(minus_one);
    let m1_result = if is_srem {
        // x % -1 = 0.
        pos.ins().iconst(ty, 0)
    } else {
        // Explicitly check for overflow: Trap when x == INT_MIN.
        debug_assert!(avoid_div_traps, "Native trapping divide handled above");
        let f = pos.ins().ifcmp_imm(x, -1 << (ty.lane_bits() - 1));
        pos.ins()
            .trapif(IntCC::Equal, f, ir::TrapCode::IntegerOverflow);
        // x / -1 = -x.
        pos.ins().irsub_imm(x, 0)
    };

    // Recycle the original instruction as a jump.
    pos.func.dfg.replace(inst).jump(done, &[m1_result]);

    // Finally insert a label for the completion.
    pos.next_inst();
    pos.insert_ebb(done);

    cfg.recompute_ebb(pos.func, old_ebb);
    cfg.recompute_ebb(pos.func, nominal);
    cfg.recompute_ebb(pos.func, minus_one);
    cfg.recompute_ebb(pos.func, done);
}

/// Expand the `udiv` and `urem` instructions using `x86_udivmodx`.
fn expand_udivrem(
    inst: ir::Inst,
    func: &mut ir::Function,
    _cfg: &mut ControlFlowGraph,
    isa: &dyn TargetIsa,
) {
    let (x, y, is_urem) = match func.dfg[inst] {
        ir::InstructionData::Binary {
            opcode: ir::Opcode::Udiv,
            args,
        } => (args[0], args[1], false),
        ir::InstructionData::Binary {
            opcode: ir::Opcode::Urem,
            args,
        } => (args[0], args[1], true),
        _ => panic!("Need udiv/urem: {}", func.dfg.display_inst(inst, None)),
    };
    let avoid_div_traps = isa.flags().avoid_div_traps();
    let result = func.dfg.first_result(inst);
    let ty = func.dfg.value_type(result);

    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);
    pos.func.dfg.clear_results(inst);

    // Put in an explicit division-by-zero trap if the environment requires it.
    if avoid_div_traps {
        let zero_check = if let Some(imm) = maybe_iconst_imm(&pos, y) {
            // Ideally, we'd just replace the conditional trap with a trap when the immediate is
            // zero, but this requires more manipulation of the dfg/cfg, which is out of scope
            // here.
            imm == 0
        } else {
            true
        };
        if zero_check {
            pos.ins().trapz(y, ir::TrapCode::IntegerDivisionByZero);
        }
    }

    // Now it is safe to execute the `x86_udivmodx` instruction.
    let xhi = pos.ins().iconst(ty, 0);
    let reuse = if is_urem {
        [None, Some(result)]
    } else {
        [Some(result), None]
    };
    pos.ins().with_results(reuse).x86_udivmodx(x, xhi, y);
    pos.remove_inst();
}

/// Expand the `fmin` and `fmax` instructions using the x86 `x86_fmin` and `x86_fmax`
/// instructions.
fn expand_minmax(
    inst: ir::Inst,
    func: &mut ir::Function,
    cfg: &mut ControlFlowGraph,
    _isa: &dyn TargetIsa,
) {
    let (x, y, x86_opc, bitwise_opc) = match func.dfg[inst] {
        ir::InstructionData::Binary {
            opcode: ir::Opcode::Fmin,
            args,
        } => (args[0], args[1], ir::Opcode::X86Fmin, ir::Opcode::Bor),
        ir::InstructionData::Binary {
            opcode: ir::Opcode::Fmax,
            args,
        } => (args[0], args[1], ir::Opcode::X86Fmax, ir::Opcode::Band),
        _ => panic!("Expected fmin/fmax: {}", func.dfg.display_inst(inst, None)),
    };
    let old_ebb = func.layout.pp_ebb(inst);

    // We need to handle the following conditions, depending on how x and y compare:
    //
    // 1. LT or GT: The native `x86_opc` min/max instruction does what we need.
    // 2. EQ: We need to use `bitwise_opc` to make sure that
    //    fmin(0.0, -0.0) -> -0.0 and fmax(0.0, -0.0) -> 0.0.
    // 3. UN: We need to produce a quiet NaN that is canonical if the inputs are canonical.

    // EBB handling case 1) where operands are ordered but not equal.
    let one_ebb = func.dfg.make_ebb();

    // EBB handling case 3) where one operand is NaN.
    let uno_ebb = func.dfg.make_ebb();

    // EBB that handles the unordered or equal cases 2) and 3).
    let ueq_ebb = func.dfg.make_ebb();

    // EBB handling case 2) where operands are ordered and equal.
    let eq_ebb = func.dfg.make_ebb();

    // Final EBB with one argument representing the final result value.
    let done = func.dfg.make_ebb();

    // The basic blocks are laid out to minimize branching for the common cases:
    //
    // 1) One branch not taken, one jump.
    // 2) One branch taken.
    // 3) Two branches taken, one jump.

    // Move the `inst` result value onto the `done` EBB.
    let result = func.dfg.first_result(inst);
    let ty = func.dfg.value_type(result);
    func.dfg.clear_results(inst);
    func.dfg.attach_ebb_param(done, result);

    // Test for case 1) ordered and not equal.
    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);
    let cmp_ueq = pos.ins().fcmp(FloatCC::UnorderedOrEqual, x, y);
    pos.ins().brnz(cmp_ueq, ueq_ebb, &[]);
    pos.ins().jump(one_ebb, &[]);

    // Handle the common ordered, not equal (LT|GT) case.
    pos.insert_ebb(one_ebb);
    let one_inst = pos.ins().Binary(x86_opc, ty, x, y).0;
    let one_result = pos.func.dfg.first_result(one_inst);
    pos.ins().jump(done, &[one_result]);

    // Case 3) Unordered.
    // We know that at least one operand is a NaN that needs to be propagated. We simply use an
    // `fadd` instruction which has the same NaN propagation semantics.
    pos.insert_ebb(uno_ebb);
    let uno_result = pos.ins().fadd(x, y);
    pos.ins().jump(done, &[uno_result]);

    // Case 2) or 3).
    pos.insert_ebb(ueq_ebb);
    // Test for case 3) (UN) one value is NaN.
    // TODO: When we get support for flag values, we can reuse the above comparison.
    let cmp_uno = pos.ins().fcmp(FloatCC::Unordered, x, y);
    pos.ins().brnz(cmp_uno, uno_ebb, &[]);
    pos.ins().jump(eq_ebb, &[]);

    // We are now in case 2) where x and y compare EQ.
    // We need a bitwise operation to get the sign right.
    pos.insert_ebb(eq_ebb);
    let bw_inst = pos.ins().Binary(bitwise_opc, ty, x, y).0;
    let bw_result = pos.func.dfg.first_result(bw_inst);
    // This should become a fall-through for this second most common case.
    // Recycle the original instruction as a jump.
    pos.func.dfg.replace(inst).jump(done, &[bw_result]);

    // Finally insert a label for the completion.
    pos.next_inst();
    pos.insert_ebb(done);

    cfg.recompute_ebb(pos.func, old_ebb);
    cfg.recompute_ebb(pos.func, one_ebb);
    cfg.recompute_ebb(pos.func, uno_ebb);
    cfg.recompute_ebb(pos.func, ueq_ebb);
    cfg.recompute_ebb(pos.func, eq_ebb);
    cfg.recompute_ebb(pos.func, done);
}

/// x86 has no unsigned-to-float conversions. We handle the easy case of zero-extending i32 to
/// i64 with a pattern, the rest needs more code.
fn expand_fcvt_from_uint(
    inst: ir::Inst,
    func: &mut ir::Function,
    cfg: &mut ControlFlowGraph,
    _isa: &dyn TargetIsa,
) {
    let x;
    match func.dfg[inst] {
        ir::InstructionData::Unary {
            opcode: ir::Opcode::FcvtFromUint,
            arg,
        } => x = arg,
        _ => panic!("Need fcvt_from_uint: {}", func.dfg.display_inst(inst, None)),
    }
    let xty = func.dfg.value_type(x);
    let result = func.dfg.first_result(inst);
    let ty = func.dfg.value_type(result);
    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);

    // Conversion from unsigned 32-bit is easy on x86-64.
    // TODO: This should be guarded by an ISA check.
    if xty == ir::types::I32 {
        let wide = pos.ins().uextend(ir::types::I64, x);
        pos.func.dfg.replace(inst).fcvt_from_sint(ty, wide);
        return;
    }

    let old_ebb = pos.func.layout.pp_ebb(inst);

    // EBB handling the case where x >= 0.
    let poszero_ebb = pos.func.dfg.make_ebb();

    // EBB handling the case where x < 0.
    let neg_ebb = pos.func.dfg.make_ebb();

    // Final EBB with one argument representing the final result value.
    let done = pos.func.dfg.make_ebb();

    // Move the `inst` result value onto the `done` EBB.
    pos.func.dfg.clear_results(inst);
    pos.func.dfg.attach_ebb_param(done, result);

    // If x as a signed int is not negative, we can use the existing `fcvt_from_sint` instruction.
    let is_neg = pos.ins().icmp_imm(IntCC::SignedLessThan, x, 0);
    pos.ins().brnz(is_neg, neg_ebb, &[]);
    pos.ins().jump(poszero_ebb, &[]);

    // Easy case: just use a signed conversion.
    pos.insert_ebb(poszero_ebb);
    let posres = pos.ins().fcvt_from_sint(ty, x);
    pos.ins().jump(done, &[posres]);

    // Now handle the negative case.
    pos.insert_ebb(neg_ebb);

    // Divide x by two to get it in range for the signed conversion, keep the LSB, and scale it
    // back up on the FP side.
    let ihalf = pos.ins().ushr_imm(x, 1);
    let lsb = pos.ins().band_imm(x, 1);
    let ifinal = pos.ins().bor(ihalf, lsb);
    let fhalf = pos.ins().fcvt_from_sint(ty, ifinal);
    let negres = pos.ins().fadd(fhalf, fhalf);

    // Recycle the original instruction as a jump.
    pos.func.dfg.replace(inst).jump(done, &[negres]);

    // Finally insert a label for the completion.
    pos.next_inst();
    pos.insert_ebb(done);

    cfg.recompute_ebb(pos.func, old_ebb);
    cfg.recompute_ebb(pos.func, poszero_ebb);
    cfg.recompute_ebb(pos.func, neg_ebb);
    cfg.recompute_ebb(pos.func, done);
}

fn expand_fcvt_to_sint(
    inst: ir::Inst,
    func: &mut ir::Function,
    cfg: &mut ControlFlowGraph,
    _isa: &dyn TargetIsa,
) {
    use crate::ir::immediates::{Ieee32, Ieee64};

    let x = match func.dfg[inst] {
        ir::InstructionData::Unary {
            opcode: ir::Opcode::FcvtToSint,
            arg,
        } => arg,
        _ => panic!("Need fcvt_to_sint: {}", func.dfg.display_inst(inst, None)),
    };
    let old_ebb = func.layout.pp_ebb(inst);
    let xty = func.dfg.value_type(x);
    let result = func.dfg.first_result(inst);
    let ty = func.dfg.value_type(result);

    // Final EBB after the bad value checks.
    let done = func.dfg.make_ebb();

    // EBB for checking failure cases.
    let maybe_trap_ebb = func.dfg.make_ebb();

    // The `x86_cvtt2si` performs the desired conversion, but it doesn't trap on NaN or overflow.
    // It produces an INT_MIN result instead.
    func.dfg.replace(inst).x86_cvtt2si(ty, x);

    let mut pos = FuncCursor::new(func).after_inst(inst);
    pos.use_srcloc(inst);

    let is_done = pos
        .ins()
        .icmp_imm(IntCC::NotEqual, result, 1 << (ty.lane_bits() - 1));
    pos.ins().brnz(is_done, done, &[]);
    pos.ins().jump(maybe_trap_ebb, &[]);

    // We now have the following possibilities:
    //
    // 1. INT_MIN was actually the correct conversion result.
    // 2. The input was NaN -> trap bad_toint
    // 3. The input was out of range -> trap int_ovf
    //
    pos.insert_ebb(maybe_trap_ebb);

    // Check for NaN.
    let is_nan = pos.ins().fcmp(FloatCC::Unordered, x, x);
    pos.ins()
        .trapnz(is_nan, ir::TrapCode::BadConversionToInteger);

    // Check for case 1: INT_MIN is the correct result.
    // Determine the smallest floating point number that would convert to INT_MIN.
    let mut overflow_cc = FloatCC::LessThan;
    let output_bits = ty.lane_bits();
    let flimit = match xty {
        ir::types::F32 =>
        // An f32 can represent `i16::min_value() - 1` exactly with precision to spare, so
        // there are values less than -2^(N-1) that convert correctly to INT_MIN.
        {
            pos.ins().f32const(if output_bits < 32 {
                overflow_cc = FloatCC::LessThanOrEqual;
                Ieee32::fcvt_to_sint_negative_overflow(output_bits)
            } else {
                Ieee32::pow2(output_bits - 1).neg()
            })
        }
        ir::types::F64 =>
        // An f64 can represent `i32::min_value() - 1` exactly with precision to spare, so
        // there are values less than -2^(N-1) that convert correctly to INT_MIN.
        {
            pos.ins().f64const(if output_bits < 64 {
                overflow_cc = FloatCC::LessThanOrEqual;
                Ieee64::fcvt_to_sint_negative_overflow(output_bits)
            } else {
                Ieee64::pow2(output_bits - 1).neg()
            })
        }
        _ => panic!("Can't convert {}", xty),
    };
    let overflow = pos.ins().fcmp(overflow_cc, x, flimit);
    pos.ins().trapnz(overflow, ir::TrapCode::IntegerOverflow);

    // Finally, we could have a positive value that is too large.
    let fzero = match xty {
        ir::types::F32 => pos.ins().f32const(Ieee32::with_bits(0)),
        ir::types::F64 => pos.ins().f64const(Ieee64::with_bits(0)),
        _ => panic!("Can't convert {}", xty),
    };
    let overflow = pos.ins().fcmp(FloatCC::GreaterThanOrEqual, x, fzero);
    pos.ins().trapnz(overflow, ir::TrapCode::IntegerOverflow);

    pos.ins().jump(done, &[]);
    pos.insert_ebb(done);

    cfg.recompute_ebb(pos.func, old_ebb);
    cfg.recompute_ebb(pos.func, maybe_trap_ebb);
    cfg.recompute_ebb(pos.func, done);
}

fn expand_fcvt_to_sint_sat(
    inst: ir::Inst,
    func: &mut ir::Function,
    cfg: &mut ControlFlowGraph,
    _isa: &dyn TargetIsa,
) {
    use crate::ir::immediates::{Ieee32, Ieee64};

    let x = match func.dfg[inst] {
        ir::InstructionData::Unary {
            opcode: ir::Opcode::FcvtToSintSat,
            arg,
        } => arg,
        _ => panic!(
            "Need fcvt_to_sint_sat: {}",
            func.dfg.display_inst(inst, None)
        ),
    };

    let old_ebb = func.layout.pp_ebb(inst);
    let xty = func.dfg.value_type(x);
    let result = func.dfg.first_result(inst);
    let ty = func.dfg.value_type(result);

    // Final EBB after the bad value checks.
    let done_ebb = func.dfg.make_ebb();
    let intmin_ebb = func.dfg.make_ebb();
    let minsat_ebb = func.dfg.make_ebb();
    let maxsat_ebb = func.dfg.make_ebb();
    func.dfg.clear_results(inst);
    func.dfg.attach_ebb_param(done_ebb, result);

    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);

    // The `x86_cvtt2si` performs the desired conversion, but it doesn't trap on NaN or
    // overflow. It produces an INT_MIN result instead.
    let cvtt2si = pos.ins().x86_cvtt2si(ty, x);

    let is_done = pos
        .ins()
        .icmp_imm(IntCC::NotEqual, cvtt2si, 1 << (ty.lane_bits() - 1));
    pos.ins().brnz(is_done, done_ebb, &[cvtt2si]);
    pos.ins().jump(intmin_ebb, &[]);

    // We now have the following possibilities:
    //
    // 1. INT_MIN was actually the correct conversion result.
    // 2. The input was NaN -> replace the result value with 0.
    // 3. The input was out of range -> saturate the result to the min/max value.
    pos.insert_ebb(intmin_ebb);

    // Check for NaN, which is truncated to 0.
    let zero = pos.ins().iconst(ty, 0);
    let is_nan = pos.ins().fcmp(FloatCC::Unordered, x, x);
    pos.ins().brnz(is_nan, done_ebb, &[zero]);
    pos.ins().jump(minsat_ebb, &[]);

    // Check for case 1: INT_MIN is the correct result.
    // Determine the smallest floating point number that would convert to INT_MIN.
    pos.insert_ebb(minsat_ebb);
    let mut overflow_cc = FloatCC::LessThan;
    let output_bits = ty.lane_bits();
    let flimit = match xty {
        ir::types::F32 =>
        // An f32 can represent `i16::min_value() - 1` exactly with precision to spare, so
        // there are values less than -2^(N-1) that convert correctly to INT_MIN.
        {
            pos.ins().f32const(if output_bits < 32 {
                overflow_cc = FloatCC::LessThanOrEqual;
                Ieee32::fcvt_to_sint_negative_overflow(output_bits)
            } else {
                Ieee32::pow2(output_bits - 1).neg()
            })
        }
        ir::types::F64 =>
        // An f64 can represent `i32::min_value() - 1` exactly with precision to spare, so
        // there are values less than -2^(N-1) that convert correctly to INT_MIN.
        {
            pos.ins().f64const(if output_bits < 64 {
                overflow_cc = FloatCC::LessThanOrEqual;
                Ieee64::fcvt_to_sint_negative_overflow(output_bits)
            } else {
                Ieee64::pow2(output_bits - 1).neg()
            })
        }
        _ => panic!("Can't convert {}", xty),
    };

    let overflow = pos.ins().fcmp(overflow_cc, x, flimit);
    let min_imm = match ty {
        ir::types::I32 => i32::min_value() as i64,
        ir::types::I64 => i64::min_value(),
        _ => panic!("Don't know the min value for {}", ty),
    };
    let min_value = pos.ins().iconst(ty, min_imm);
    pos.ins().brnz(overflow, done_ebb, &[min_value]);
    pos.ins().jump(maxsat_ebb, &[]);

    // Finally, we could have a positive value that is too large.
    pos.insert_ebb(maxsat_ebb);
    let fzero = match xty {
        ir::types::F32 => pos.ins().f32const(Ieee32::with_bits(0)),
        ir::types::F64 => pos.ins().f64const(Ieee64::with_bits(0)),
        _ => panic!("Can't convert {}", xty),
    };

    let max_imm = match ty {
        ir::types::I32 => i32::max_value() as i64,
        ir::types::I64 => i64::max_value(),
        _ => panic!("Don't know the max value for {}", ty),
    };
    let max_value = pos.ins().iconst(ty, max_imm);

    let overflow = pos.ins().fcmp(FloatCC::GreaterThanOrEqual, x, fzero);
    pos.ins().brnz(overflow, done_ebb, &[max_value]);

    // Recycle the original instruction.
    pos.func.dfg.replace(inst).jump(done_ebb, &[cvtt2si]);

    // Finally insert a label for the completion.
    pos.next_inst();
    pos.insert_ebb(done_ebb);

    cfg.recompute_ebb(pos.func, old_ebb);
    cfg.recompute_ebb(pos.func, intmin_ebb);
    cfg.recompute_ebb(pos.func, minsat_ebb);
    cfg.recompute_ebb(pos.func, maxsat_ebb);
    cfg.recompute_ebb(pos.func, done_ebb);
}

fn expand_fcvt_to_uint(
    inst: ir::Inst,
    func: &mut ir::Function,
    cfg: &mut ControlFlowGraph,
    _isa: &dyn TargetIsa,
) {
    use crate::ir::immediates::{Ieee32, Ieee64};

    let x = match func.dfg[inst] {
        ir::InstructionData::Unary {
            opcode: ir::Opcode::FcvtToUint,
            arg,
        } => arg,
        _ => panic!("Need fcvt_to_uint: {}", func.dfg.display_inst(inst, None)),
    };

    let old_ebb = func.layout.pp_ebb(inst);
    let xty = func.dfg.value_type(x);
    let result = func.dfg.first_result(inst);
    let ty = func.dfg.value_type(result);

    // EBB handle numbers < 2^(N-1).
    let below_uint_max_ebb = func.dfg.make_ebb();

    // EBB handle numbers < 0.
    let below_zero_ebb = func.dfg.make_ebb();

    // EBB handling numbers >= 2^(N-1).
    let large = func.dfg.make_ebb();

    // Final EBB after the bad value checks.
    let done = func.dfg.make_ebb();

    // Move the `inst` result value onto the `done` EBB.
    func.dfg.clear_results(inst);
    func.dfg.attach_ebb_param(done, result);

    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);

    // Start by materializing the floating point constant 2^(N-1) where N is the number of bits in
    // the destination integer type.
    let pow2nm1 = match xty {
        ir::types::F32 => pos.ins().f32const(Ieee32::pow2(ty.lane_bits() - 1)),
        ir::types::F64 => pos.ins().f64const(Ieee64::pow2(ty.lane_bits() - 1)),
        _ => panic!("Can't convert {}", xty),
    };
    let is_large = pos.ins().ffcmp(x, pow2nm1);
    pos.ins()
        .brff(FloatCC::GreaterThanOrEqual, is_large, large, &[]);
    pos.ins().jump(below_uint_max_ebb, &[]);

    // We need to generate a specific trap code when `x` is NaN, so reuse the flags from the
    // previous comparison.
    pos.insert_ebb(below_uint_max_ebb);
    pos.ins().trapff(
        FloatCC::Unordered,
        is_large,
        ir::TrapCode::BadConversionToInteger,
    );

    // Now we know that x < 2^(N-1) and not NaN.
    let sres = pos.ins().x86_cvtt2si(ty, x);
    let is_neg = pos.ins().ifcmp_imm(sres, 0);
    pos.ins()
        .brif(IntCC::SignedGreaterThanOrEqual, is_neg, done, &[sres]);
    pos.ins().jump(below_zero_ebb, &[]);

    pos.insert_ebb(below_zero_ebb);
    pos.ins().trap(ir::TrapCode::IntegerOverflow);

    // Handle the case where x >= 2^(N-1) and not NaN.
    pos.insert_ebb(large);
    let adjx = pos.ins().fsub(x, pow2nm1);
    let lres = pos.ins().x86_cvtt2si(ty, adjx);
    let is_neg = pos.ins().ifcmp_imm(lres, 0);
    pos.ins()
        .trapif(IntCC::SignedLessThan, is_neg, ir::TrapCode::IntegerOverflow);
    let lfinal = pos.ins().iadd_imm(lres, 1 << (ty.lane_bits() - 1));

    // Recycle the original instruction as a jump.
    pos.func.dfg.replace(inst).jump(done, &[lfinal]);

    // Finally insert a label for the completion.
    pos.next_inst();
    pos.insert_ebb(done);

    cfg.recompute_ebb(pos.func, old_ebb);
    cfg.recompute_ebb(pos.func, below_uint_max_ebb);
    cfg.recompute_ebb(pos.func, below_zero_ebb);
    cfg.recompute_ebb(pos.func, large);
    cfg.recompute_ebb(pos.func, done);
}

fn expand_fcvt_to_uint_sat(
    inst: ir::Inst,
    func: &mut ir::Function,
    cfg: &mut ControlFlowGraph,
    _isa: &dyn TargetIsa,
) {
    use crate::ir::immediates::{Ieee32, Ieee64};

    let x = match func.dfg[inst] {
        ir::InstructionData::Unary {
            opcode: ir::Opcode::FcvtToUintSat,
            arg,
        } => arg,
        _ => panic!(
            "Need fcvt_to_uint_sat: {}",
            func.dfg.display_inst(inst, None)
        ),
    };

    let old_ebb = func.layout.pp_ebb(inst);
    let xty = func.dfg.value_type(x);
    let result = func.dfg.first_result(inst);
    let ty = func.dfg.value_type(result);

    // EBB handle numbers < 2^(N-1).
    let below_pow2nm1_or_nan_ebb = func.dfg.make_ebb();
    let below_pow2nm1_ebb = func.dfg.make_ebb();

    // EBB handling numbers >= 2^(N-1).
    let large = func.dfg.make_ebb();

    // EBB handling numbers < 2^N.
    let uint_large_ebb = func.dfg.make_ebb();

    // Final EBB after the bad value checks.
    let done = func.dfg.make_ebb();

    // Move the `inst` result value onto the `done` EBB.
    func.dfg.clear_results(inst);
    func.dfg.attach_ebb_param(done, result);

    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);

    // Start by materializing the floating point constant 2^(N-1) where N is the number of bits in
    // the destination integer type.
    let pow2nm1 = match xty {
        ir::types::F32 => pos.ins().f32const(Ieee32::pow2(ty.lane_bits() - 1)),
        ir::types::F64 => pos.ins().f64const(Ieee64::pow2(ty.lane_bits() - 1)),
        _ => panic!("Can't convert {}", xty),
    };
    let zero = pos.ins().iconst(ty, 0);
    let is_large = pos.ins().ffcmp(x, pow2nm1);
    pos.ins()
        .brff(FloatCC::GreaterThanOrEqual, is_large, large, &[]);
    pos.ins().jump(below_pow2nm1_or_nan_ebb, &[]);

    // We need to generate zero when `x` is NaN, so reuse the flags from the previous comparison.
    pos.insert_ebb(below_pow2nm1_or_nan_ebb);
    pos.ins().brff(FloatCC::Unordered, is_large, done, &[zero]);
    pos.ins().jump(below_pow2nm1_ebb, &[]);

    // Now we know that x < 2^(N-1) and not NaN. If the result of the cvtt2si is positive, we're
    // done; otherwise saturate to the minimum unsigned value, that is 0.
    pos.insert_ebb(below_pow2nm1_ebb);
    let sres = pos.ins().x86_cvtt2si(ty, x);
    let is_neg = pos.ins().ifcmp_imm(sres, 0);
    pos.ins()
        .brif(IntCC::SignedGreaterThanOrEqual, is_neg, done, &[sres]);
    pos.ins().jump(done, &[zero]);

    // Handle the case where x >= 2^(N-1) and not NaN.
    pos.insert_ebb(large);
    let adjx = pos.ins().fsub(x, pow2nm1);
    let lres = pos.ins().x86_cvtt2si(ty, adjx);
    let max_value = pos.ins().iconst(
        ty,
        match ty {
            ir::types::I32 => u32::max_value() as i64,
            ir::types::I64 => u64::max_value() as i64,
            _ => panic!("Can't convert {}", ty),
        },
    );
    let is_neg = pos.ins().ifcmp_imm(lres, 0);
    pos.ins()
        .brif(IntCC::SignedLessThan, is_neg, done, &[max_value]);
    pos.ins().jump(uint_large_ebb, &[]);

    pos.insert_ebb(uint_large_ebb);
    let lfinal = pos.ins().iadd_imm(lres, 1 << (ty.lane_bits() - 1));

    // Recycle the original instruction as a jump.
    pos.func.dfg.replace(inst).jump(done, &[lfinal]);

    // Finally insert a label for the completion.
    pos.next_inst();
    pos.insert_ebb(done);

    cfg.recompute_ebb(pos.func, old_ebb);
    cfg.recompute_ebb(pos.func, below_pow2nm1_or_nan_ebb);
    cfg.recompute_ebb(pos.func, below_pow2nm1_ebb);
    cfg.recompute_ebb(pos.func, large);
    cfg.recompute_ebb(pos.func, uint_large_ebb);
    cfg.recompute_ebb(pos.func, done);
}
