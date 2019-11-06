//! Encoding tables for x86 ISAs.

use super::registers::*;
use crate::bitset::BitSet;
use crate::cursor::{Cursor, FuncCursor};
use crate::flowgraph::ControlFlowGraph;
use crate::ir::condcodes::{FloatCC, IntCC};
use crate::ir::types::*;
use crate::ir::{self, Function, Inst, InstBuilder};
use crate::isa::constraints::*;
use crate::isa::enc_tables::*;
use crate::isa::encoding::base_size;
use crate::isa::encoding::RecipeSizing;
use crate::isa::RegUnit;
use crate::isa::{self, TargetIsa};
use crate::predicates;
use crate::regalloc::RegDiversions;

 
// x86 recipe predicates.
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
fn recipe_predicate_rexop1u_id(_: crate::settings::PredicateView, inst: &ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::UnaryImm { imm, .. } = *inst {
        return predicates::is_signed_int(imm, 32, 0);
    }
    unreachable!();
}
fn recipe_predicate_op1ldwithindex(_: crate::settings::PredicateView, inst: &ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::LoadComplex { offset, .. } = *inst {
        return predicates::is_equal(offset, 0);
    }
    unreachable!();
}
fn recipe_predicate_op1ldwithindexdisp8(_: crate::settings::PredicateView, inst: &ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::LoadComplex { offset, .. } = *inst {
        return predicates::is_signed_int(offset, 8, 0);
    }
    unreachable!();
}
fn recipe_predicate_op1ldwithindexdisp32(_: crate::settings::PredicateView, inst: &ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::LoadComplex { offset, .. } = *inst {
        return predicates::is_signed_int(offset, 32, 0);
    }
    unreachable!();
}
fn recipe_predicate_op1stwithindex(_: crate::settings::PredicateView, inst: &ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::StoreComplex { offset, .. } = *inst {
        return predicates::is_equal(offset, 0);
    }
    unreachable!();
}
fn recipe_predicate_op1stwithindexdisp8(_: crate::settings::PredicateView, inst: &ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::StoreComplex { offset, .. } = *inst {
        return predicates::is_signed_int(offset, 8, 0);
    }
    unreachable!();
}
fn recipe_predicate_op1stwithindexdisp32(_: crate::settings::PredicateView, inst: &ir::InstructionData) -> bool {
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
fn recipe_predicate_op1stdisp8(_: crate::settings::PredicateView, inst: &ir::InstructionData) -> bool {
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
fn recipe_predicate_op1lddisp8(_: crate::settings::PredicateView, inst: &ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::Load { offset, .. } = *inst {
        return predicates::is_signed_int(offset, 8, 0);
    }
    unreachable!();
}
fn recipe_predicate_op1lddisp32(_: crate::settings::PredicateView, inst: &ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::Load { offset, .. } = *inst {
        return predicates::is_signed_int(offset, 32, 0);
    }
    unreachable!();
}
fn recipe_predicate_op1adjustsp_ib(_: crate::settings::PredicateView, inst: &ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::UnaryImm { imm, .. } = *inst {
        return predicates::is_signed_int(imm, 8, 0);
    }
    unreachable!();
}
fn recipe_predicate_op1brfb(_: crate::settings::PredicateView, inst: &ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::BranchFloat { cond, .. } = *inst {
        return predicates::is_equal(cond, ir::condcodes::FloatCC::Ordered) || predicates::is_equal(cond, ir::condcodes::FloatCC::Unordered) || predicates::is_equal(cond, ir::condcodes::FloatCC::OrderedNotEqual) || predicates::is_equal(cond, ir::condcodes::FloatCC::UnorderedOrEqual) || predicates::is_equal(cond, ir::condcodes::FloatCC::GreaterThan) || predicates::is_equal(cond, ir::condcodes::FloatCC::GreaterThanOrEqual) || predicates::is_equal(cond, ir::condcodes::FloatCC::UnorderedOrLessThan) || predicates::is_equal(cond, ir::condcodes::FloatCC::UnorderedOrLessThanOrEqual);
    }
    unreachable!();
}
fn recipe_predicate_rexop1jt_entry(_: crate::settings::PredicateView, inst: &ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::BranchTableEntry { imm, .. } = *inst {
        return predicates::is_equal(imm, 1) || predicates::is_equal(imm, 2) || predicates::is_equal(imm, 4) || predicates::is_equal(imm, 8);
    }
    unreachable!();
}
fn recipe_predicate_trapff(_: crate::settings::PredicateView, inst: &ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::FloatCondTrap { cond, .. } = *inst {
        return predicates::is_equal(cond, ir::condcodes::FloatCC::Ordered) || predicates::is_equal(cond, ir::condcodes::FloatCC::Unordered) || predicates::is_equal(cond, ir::condcodes::FloatCC::OrderedNotEqual) || predicates::is_equal(cond, ir::condcodes::FloatCC::UnorderedOrEqual) || predicates::is_equal(cond, ir::condcodes::FloatCC::GreaterThan) || predicates::is_equal(cond, ir::condcodes::FloatCC::GreaterThanOrEqual) || predicates::is_equal(cond, ir::condcodes::FloatCC::UnorderedOrLessThan) || predicates::is_equal(cond, ir::condcodes::FloatCC::UnorderedOrLessThanOrEqual);
    }
    unreachable!();
}
fn recipe_predicate_op1icscc_ib(_: crate::settings::PredicateView, inst: &ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::IntCompareImm { imm, .. } = *inst {
        return predicates::is_signed_int(imm, 8, 0);
    }
    unreachable!();
}
fn recipe_predicate_op1icscc_id(_: crate::settings::PredicateView, inst: &ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::IntCompareImm { imm, .. } = *inst {
        return predicates::is_signed_int(imm, 32, 0);
    }
    unreachable!();
}
fn recipe_predicate_op2f32imm_z(_: crate::settings::PredicateView, inst: &ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::UnaryIeee32 { imm, .. } = *inst {
        return predicates::is_zero_32_bit_float(imm);
    }
    unreachable!();
}
fn recipe_predicate_mp2f64imm_z(_: crate::settings::PredicateView, inst: &ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::UnaryIeee64 { imm, .. } = *inst {
        return predicates::is_zero_64_bit_float(imm);
    }
    unreachable!();
}
fn recipe_predicate_mp3furmi_rnd(isap: crate::settings::PredicateView, _: &ir::InstructionData) -> bool {
    isap.test(16)
}
fn recipe_predicate_op2fcscc(_: crate::settings::PredicateView, inst: &ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::FloatCompare { cond, .. } = *inst {
        return predicates::is_equal(cond, ir::condcodes::FloatCC::Ordered) || predicates::is_equal(cond, ir::condcodes::FloatCC::Unordered) || predicates::is_equal(cond, ir::condcodes::FloatCC::OrderedNotEqual) || predicates::is_equal(cond, ir::condcodes::FloatCC::UnorderedOrEqual) || predicates::is_equal(cond, ir::condcodes::FloatCC::GreaterThan) || predicates::is_equal(cond, ir::condcodes::FloatCC::GreaterThanOrEqual) || predicates::is_equal(cond, ir::condcodes::FloatCC::UnorderedOrLessThan) || predicates::is_equal(cond, ir::condcodes::FloatCC::UnorderedOrLessThanOrEqual);
    }
    unreachable!();
}
fn recipe_predicate_mp2r_ib_unsigned_fpr(_: crate::settings::PredicateView, inst: &ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::ExtractLane { lane, .. } = *inst {
        return predicates::is_unsigned_int(lane, 8, 0);
    }
    unreachable!();
}
fn recipe_predicate_mp3r_ib_unsigned_r(_: crate::settings::PredicateView, inst: &ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::InsertLane { lane, .. } = *inst {
        return predicates::is_unsigned_int(lane, 8, 0);
    }
    unreachable!();
}

/// x86 recipe predicate table.
///
/// One entry per recipe, set to Some only when the recipe is guarded by a predicate.
pub static RECIPE_PREDICATES: [RecipePredicate; 289] = [
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
    None,
    Some(recipe_predicate_mp2r_ib_unsigned_fpr),
    None,
    Some(recipe_predicate_mp3r_ib_unsigned_r),
    Some(recipe_predicate_mp3r_ib_unsigned_r),
    Some(recipe_predicate_mp3r_ib_unsigned_r),
    Some(recipe_predicate_mp3r_ib_unsigned_r),
    Some(recipe_predicate_mp2r_ib_unsigned_fpr),
    Some(recipe_predicate_mp2r_ib_unsigned_fpr),
    None,
    None,
    Some(recipe_predicate_op1st),
    Some(recipe_predicate_op1stdisp8),
    None,
    Some(recipe_predicate_op1ld),
    Some(recipe_predicate_op1lddisp8),
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
];

// x86 instruction predicates.
fn inst_predicate_0(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::UnaryImm { imm, .. } = *inst {
        let _ = func;
        return predicates::is_unsigned_int(imm, 32, 0);
    }
    unreachable!();
}
fn inst_predicate_1(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::UnaryImm { imm, .. } = *inst {
        let _ = func;
        return predicates::is_zero_int(imm);
    }
    unreachable!();
}
fn inst_predicate_2(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::LoadComplex { ref args, .. } = *inst {
        let _ = func;
        return predicates::has_length_of(args, 2, func);
    }
    unreachable!();
}
fn inst_predicate_3(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::StoreComplex { ref args, .. } = *inst {
        let _ = func;
        return predicates::has_length_of(args, 3, func);
    }
    unreachable!();
}
fn inst_predicate_4(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::FuncAddr { func_ref, .. } = *inst {
        let _ = func;
        return predicates::is_colocated_func(func_ref, func);
    }
    unreachable!();
}
fn inst_predicate_5(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::UnaryGlobalValue { global_value, .. } = *inst {
        let _ = func;
        return predicates::is_colocated_data(global_value, func);
    }
    unreachable!();
}
fn inst_predicate_6(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::Call { func_ref, .. } = *inst {
        let _ = func;
        return predicates::is_colocated_func(func_ref, func);
    }
    unreachable!();
}
fn inst_predicate_7(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    let args = inst.arguments(&func.dfg.value_lists);
    func.dfg.value_type(args[0]) == ir::types::B1
}
fn inst_predicate_8(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    let args = inst.arguments(&func.dfg.value_lists);
    func.dfg.value_type(args[0]) == ir::types::B8
}
fn inst_predicate_9(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    let args = inst.arguments(&func.dfg.value_lists);
    func.dfg.value_type(args[0]) == ir::types::I16
}
fn inst_predicate_10(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    let args = inst.arguments(&func.dfg.value_lists);
    func.dfg.value_type(args[0]) == ir::types::I32
}
fn inst_predicate_11(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    let args = inst.arguments(&func.dfg.value_lists);
    func.dfg.value_type(args[0]) == ir::types::I64
}
fn inst_predicate_12(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    let args = inst.arguments(&func.dfg.value_lists);
    func.dfg.value_type(args[0]) == ir::types::I8
}
fn inst_predicate_13(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::UnaryIeee32 { imm, .. } = *inst {
        let _ = func;
        return predicates::is_zero_32_bit_float(imm);
    }
    unreachable!();
}
fn inst_predicate_14(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::UnaryIeee64 { imm, .. } = *inst {
        let _ = func;
        return predicates::is_zero_64_bit_float(imm);
    }
    unreachable!();
}
fn inst_predicate_15(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    let args = inst.arguments(&func.dfg.value_lists);
    func.dfg.value_type(args[0]) == ir::types::F32
}
fn inst_predicate_16(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    let args = inst.arguments(&func.dfg.value_lists);
    func.dfg.value_type(args[0]) == ir::types::F64
}
fn inst_predicate_17(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    let args = inst.arguments(&func.dfg.value_lists);
    func.dfg.value_type(args[0]) == ir::types::B8X16
}
fn inst_predicate_18(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    let args = inst.arguments(&func.dfg.value_lists);
    func.dfg.value_type(args[0]) == ir::types::B16X8
}
fn inst_predicate_19(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    let args = inst.arguments(&func.dfg.value_lists);
    func.dfg.value_type(args[0]) == ir::types::B32X4
}
fn inst_predicate_20(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    let args = inst.arguments(&func.dfg.value_lists);
    func.dfg.value_type(args[0]) == ir::types::B64X2
}
fn inst_predicate_21(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    let args = inst.arguments(&func.dfg.value_lists);
    func.dfg.value_type(args[0]) == ir::types::I8X16
}
fn inst_predicate_22(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    let args = inst.arguments(&func.dfg.value_lists);
    func.dfg.value_type(args[0]) == ir::types::I16X8
}
fn inst_predicate_23(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    let args = inst.arguments(&func.dfg.value_lists);
    func.dfg.value_type(args[0]) == ir::types::I32X4
}
fn inst_predicate_24(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    let args = inst.arguments(&func.dfg.value_lists);
    func.dfg.value_type(args[0]) == ir::types::I64X2
}
fn inst_predicate_25(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    let args = inst.arguments(&func.dfg.value_lists);
    func.dfg.value_type(args[0]) == ir::types::F32X4
}
fn inst_predicate_26(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    let args = inst.arguments(&func.dfg.value_lists);
    func.dfg.value_type(args[0]) == ir::types::F64X2
}
fn inst_predicate_27(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::UnaryConst { constant_handle, .. } = *inst {
        let _ = func;
        return predicates::is_all_zeroes(func.dfg.constants.get(constant_handle));
    }
    unreachable!();
}
fn inst_predicate_28(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::UnaryConst { constant_handle, .. } = *inst {
        let _ = func;
        return predicates::is_all_ones(func.dfg.constants.get(constant_handle));
    }
    unreachable!();
}
fn inst_predicate_29(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::IntCompare { cond, .. } = *inst {
        let _ = func;
        return predicates::is_equal(cond, IntCC::Equal);
    }
    unreachable!();
}
fn inst_predicate_30(func: &crate::ir::Function, inst: &crate::ir::InstructionData) -> bool {
    if let crate::ir::InstructionData::IntCompare { cond, .. } = *inst {
        let _ = func;
        return predicates::is_equal(cond, IntCC::SignedGreaterThan);
    }
    unreachable!();
}

/// x86 instruction predicate table.
///
/// One entry per instruction predicate, so the encoding bytecode can embed indexes into this
/// table.
pub static INST_PREDICATES: [InstPredicate; 31] = [
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
    inst_predicate_15,
    inst_predicate_16,
    inst_predicate_17,
    inst_predicate_18,
    inst_predicate_19,
    inst_predicate_20,
    inst_predicate_21,
    inst_predicate_22,
    inst_predicate_23,
    inst_predicate_24,
    inst_predicate_25,
    inst_predicate_26,
    inst_predicate_27,
    inst_predicate_28,
    inst_predicate_29,
    inst_predicate_30,
];

/// x86 encoding lists.
///
/// This contains the entire encodings bytecode for every single instruction; the encodings
/// interpreter knows where to start from thanks to the initial lookup in the level 1 and level 2
/// table entries below.
pub static ENCLISTS: [u16; 2068] = [
    // 000000: adjust_sp_down.i64 (I64)
    // --> [RexOp1adjustsp#8029] and stop
    0x00eb, 0x8029,
    // end of adjust_sp_down.i64 (I64)
    // 000002: band.i64 (I64)
    // --> [RexOp1rr#8021] and stop
    // 000002: band.b64 (I64)
    // --> [RexOp1rr#8021] and stop
    0x0007, 0x8021,
    // end of band.b64 (I64)
    // end of band.i64 (I64)
    // 000004: band_imm.i64 (I64)
    // --> [RexOp1r_ib#c083]
    0x002e, 0xc083,
    // --> [RexOp1r_id#c081] and stop
    0x0033, 0xc081,
    // end of band_imm.i64 (I64)
    // 000008: bint.i64 (I64)
    // skip 4 unless inst_predicate_7
    // 000008: bint.i32 (I64)
    // skip 4 unless inst_predicate_7
    // 000008: bint.i8 (I64)
    // skip 4 unless inst_predicate_7
    // 000008: bint.i16 (I64)
    // skip 4 unless inst_predicate_7
    0x5007,
    // --> [RexOp2urm_noflags#4b6]
    // --> [RexOp2urm_noflags#4b6]
    // --> [RexOp2urm_noflags#4b6]
    // --> [RexOp2urm_noflags#4b6]
    0x01bc, 0x04b6,
    // --> [Op2urm_noflags_abcd#4b6]
    // --> [Op2urm_noflags_abcd#4b6]
    // --> [Op2urm_noflags_abcd#4b6]
    // --> [Op2urm_noflags_abcd#4b6]
    0x01be, 0x04b6,
    // stop unless inst_predicate_8
    // stop unless inst_predicate_8
    // stop unless inst_predicate_8
    // stop unless inst_predicate_8
    0x1008,
    // --> [RexOp2urm_noflags#4b6]
    // --> [RexOp2urm_noflags#4b6]
    // --> [RexOp2urm_noflags#4b6]
    // --> [RexOp2urm_noflags#4b6]
    0x01bc, 0x04b6,
    // --> [Op2urm_noflags_abcd#4b6] and stop
    // --> [Op2urm_noflags_abcd#4b6] and stop
    // --> [Op2urm_noflags_abcd#4b6] and stop
    // --> [Op2urm_noflags_abcd#4b6] and stop
    0x01bf, 0x04b6,
    // end of bint.i16 (I64)
    // end of bint.i8 (I64)
    // end of bint.i32 (I64)
    // end of bint.i64 (I64)
    // 000012: bitcast.i64 (I64)
    // stop unless inst_predicate_16
    0x1010,
    // --> [RexMp2rfumr#857e] and stop
    0x01d5, 0x857e,
    // end of bitcast.i64 (I64)
    // 000015: bnot.i64 (I64)
    // --> [RexOp1ur#a0f7] and stop
    // 000015: bnot.b64 (I64)
    // --> [RexOp1ur#a0f7] and stop
    0x0017, 0xa0f7,
    // end of bnot.b64 (I64)
    // end of bnot.i64 (I64)
    // 000017: bor.i64 (I64)
    // --> [RexOp1rr#8009] and stop
    // 000017: bor.b64 (I64)
    // --> [RexOp1rr#8009] and stop
    0x0007, 0x8009,
    // end of bor.b64 (I64)
    // end of bor.i64 (I64)
    // 000019: bor_imm.i64 (I64)
    // --> [RexOp1r_ib#9083]
    0x002e, 0x9083,
    // --> [RexOp1r_id#9081] and stop
    0x0033, 0x9081,
    // end of bor_imm.i64 (I64)
    // 00001d: brnz.i64 (I64)
    // --> [RexOp1tjccb#8075]
    0x016c, 0x8075,
    // --> [RexOp1tjccd#8085] and stop
    0x0171, 0x8085,
    // end of brnz.i64 (I64)
    // 000021: brz.i64 (I64)
    // --> [RexOp1tjccb#8074]
    0x016c, 0x8074,
    // --> [RexOp1tjccd#8084] and stop
    0x0171, 0x8084,
    // end of brz.i64 (I64)
    // 000025: bxor.i64 (I64)
    // --> [RexOp1rr#8031] and stop
    // 000025: bxor.b64 (I64)
    // --> [RexOp1rr#8031] and stop
    0x0007, 0x8031,
    // end of bxor.b64 (I64)
    // end of bxor.i64 (I64)
    // 000027: bxor_imm.i64 (I64)
    // --> [RexOp1r_ib#e083]
    0x002e, 0xe083,
    // --> [RexOp1r_id#e081] and stop
    0x0033, 0xe081,
    // end of bxor_imm.i64 (I64)
    // 00002b: call_indirect.i64 (I64)
    // --> [RexOp1call_r#20ff]
    0x0152, 0x20ff,
    // --> [Op1call_r#20ff] and stop
    // 00002d: call_indirect.i32 (I32)
    // --> [Op1call_r#20ff] and stop
    0x0151, 0x20ff,
    // end of call_indirect.i32 (I32)
    // end of call_indirect.i64 (I64)
    // 00002f: clz.i64 (I64)
    // stop unless PredicateView(14)
    0x102d,
    // --> [RexMp2urm#86bd] and stop
    0x004b, 0x86bd,
    // end of clz.i64 (I64)
    // 000032: copy.i64 (I64)
    // --> [RexOp1umr#8089] and stop
    // 000032: copy.r64 (I64)
    // --> [RexOp1umr#8089] and stop
    0x0027, 0x8089,
    // end of copy.r64 (I64)
    // end of copy.i64 (I64)
    // 000034: copy_nop.i64 (I64)
    // --> [stacknull#00] and stop
    // 000034: copy_nop.i32 (I64)
    // --> [stacknull#00] and stop
    // 000034: copy_nop.i8 (I64)
    // --> [stacknull#00] and stop
    // 000034: copy_nop.i16 (I64)
    // --> [stacknull#00] and stop
    // 000034: copy_nop.f64 (I64)
    // --> [stacknull#00] and stop
    // 000034: copy_nop.f32 (I64)
    // --> [stacknull#00] and stop
    // 000034: copy_nop.b8x16 (I64)
    // --> [stacknull#00] and stop
    // 000034: copy_nop.b16x8 (I64)
    // --> [stacknull#00] and stop
    // 000034: copy_nop.b32x4 (I64)
    // --> [stacknull#00] and stop
    // 000034: copy_nop.b64x2 (I64)
    // --> [stacknull#00] and stop
    // 000034: copy_nop.i8x16 (I64)
    // --> [stacknull#00] and stop
    // 000034: copy_nop.i16x8 (I64)
    // --> [stacknull#00] and stop
    // 000034: copy_nop.i32x4 (I64)
    // --> [stacknull#00] and stop
    // 000034: copy_nop.i64x2 (I64)
    // --> [stacknull#00] and stop
    // 000034: copy_nop.f32x4 (I64)
    // --> [stacknull#00] and stop
    // 000034: copy_nop.f64x2 (I64)
    // --> [stacknull#00] and stop
    // 000034: copy_nop.i32 (I32)
    // --> [stacknull#00] and stop
    // 000034: copy_nop.i8 (I32)
    // --> [stacknull#00] and stop
    // 000034: copy_nop.i16 (I32)
    // --> [stacknull#00] and stop
    // 000034: copy_nop.i64 (I32)
    // --> [stacknull#00] and stop
    // 000034: copy_nop.f64 (I32)
    // --> [stacknull#00] and stop
    // 000034: copy_nop.f32 (I32)
    // --> [stacknull#00] and stop
    // 000034: copy_nop.b8x16 (I32)
    // --> [stacknull#00] and stop
    // 000034: copy_nop.b16x8 (I32)
    // --> [stacknull#00] and stop
    // 000034: copy_nop.b32x4 (I32)
    // --> [stacknull#00] and stop
    // 000034: copy_nop.b64x2 (I32)
    // --> [stacknull#00] and stop
    // 000034: copy_nop.i8x16 (I32)
    // --> [stacknull#00] and stop
    // 000034: copy_nop.i16x8 (I32)
    // --> [stacknull#00] and stop
    // 000034: copy_nop.i32x4 (I32)
    // --> [stacknull#00] and stop
    // 000034: copy_nop.i64x2 (I32)
    // --> [stacknull#00] and stop
    // 000034: copy_nop.f32x4 (I32)
    // --> [stacknull#00] and stop
    // 000034: copy_nop.f64x2 (I32)
    // --> [stacknull#00] and stop
    0x00e7, 0x0000,
    // end of copy_nop.f64x2 (I32)
    // end of copy_nop.f32x4 (I32)
    // end of copy_nop.i64x2 (I32)
    // end of copy_nop.i32x4 (I32)
    // end of copy_nop.i16x8 (I32)
    // end of copy_nop.i8x16 (I32)
    // end of copy_nop.b64x2 (I32)
    // end of copy_nop.b32x4 (I32)
    // end of copy_nop.b16x8 (I32)
    // end of copy_nop.b8x16 (I32)
    // end of copy_nop.f32 (I32)
    // end of copy_nop.f64 (I32)
    // end of copy_nop.i64 (I32)
    // end of copy_nop.i16 (I32)
    // end of copy_nop.i8 (I32)
    // end of copy_nop.i32 (I32)
    // end of copy_nop.f64x2 (I64)
    // end of copy_nop.f32x4 (I64)
    // end of copy_nop.i64x2 (I64)
    // end of copy_nop.i32x4 (I64)
    // end of copy_nop.i16x8 (I64)
    // end of copy_nop.i8x16 (I64)
    // end of copy_nop.b64x2 (I64)
    // end of copy_nop.b32x4 (I64)
    // end of copy_nop.b16x8 (I64)
    // end of copy_nop.b8x16 (I64)
    // end of copy_nop.f32 (I64)
    // end of copy_nop.f64 (I64)
    // end of copy_nop.i16 (I64)
    // end of copy_nop.i8 (I64)
    // end of copy_nop.i32 (I64)
    // end of copy_nop.i64 (I64)
    // 000036: copy_to_ssa.i64 (I64)
    // --> [RexOp1umr_reg_to_ssa#8089] and stop
    // 000036: copy_to_ssa.r64 (I64)
    // --> [RexOp1umr_reg_to_ssa#8089] and stop
    0x00e1, 0x8089,
    // end of copy_to_ssa.r64 (I64)
    // end of copy_to_ssa.i64 (I64)
    // 000038: ctz.i64 (I64)
    // stop unless PredicateView(13)
    0x102c,
    // --> [RexMp2urm#86bc] and stop
    0x004b, 0x86bc,
    // end of ctz.i64 (I64)
    // 00003b: fill.i64 (I64)
    // --> [RexOp1fillSib32#808b] and stop
    // 00003b: fill.r64 (I64)
    // --> [RexOp1fillSib32#808b] and stop
    0x00c9, 0x808b,
    // end of fill.r64 (I64)
    // end of fill.i64 (I64)
    // 00003d: fill_nop.i64 (I64)
    // --> [fillnull#00] and stop
    // 00003d: fill_nop.i32 (I64)
    // --> [fillnull#00] and stop
    // 00003d: fill_nop.b1 (I64)
    // --> [fillnull#00] and stop
    // 00003d: fill_nop.i8 (I64)
    // --> [fillnull#00] and stop
    // 00003d: fill_nop.i16 (I64)
    // --> [fillnull#00] and stop
    // 00003d: fill_nop.i32 (I32)
    // --> [fillnull#00] and stop
    // 00003d: fill_nop.b1 (I32)
    // --> [fillnull#00] and stop
    // 00003d: fill_nop.i8 (I32)
    // --> [fillnull#00] and stop
    // 00003d: fill_nop.i16 (I32)
    // --> [fillnull#00] and stop
    // 00003d: fill_nop.i64 (I32)
    // --> [fillnull#00] and stop
    0x00cf, 0x0000,
    // end of fill_nop.i64 (I32)
    // end of fill_nop.i16 (I32)
    // end of fill_nop.i8 (I32)
    // end of fill_nop.b1 (I32)
    // end of fill_nop.i32 (I32)
    // end of fill_nop.i16 (I64)
    // end of fill_nop.i8 (I64)
    // end of fill_nop.b1 (I64)
    // end of fill_nop.i32 (I64)
    // end of fill_nop.i64 (I64)
    // 00003f: func_addr.i64 (I64)
    // skip 2 unless PredicateView(11)
    0x302a,
    // --> [RexOp1fnaddr8#80b8]
    0x0136, 0x80b8,
    // skip 2 unless PredicateView(9)
    0x3028,
    // --> [RexOp1allones_fnaddr8#80b8]
    0x013a, 0x80b8,
    // skip 2 unless inst_predicate_4
    0x3004,
    // --> [RexOp1pcrel_fnaddr8#808d]
    0x013c, 0x808d,
    // stop unless PredicateView(10)
    0x1029,
    // --> [RexOp1got_fnaddr8#808b] and stop
    0x013f, 0x808b,
    // end of func_addr.i64 (I64)
    // 00004b: get_pinned_reg.i64 (I64)
    // --> [get_pinned_reg#00] and stop
    0x0001, 0x0000,
    // end of get_pinned_reg.i64 (I64)
    // 00004d: iadd.i64 (I64)
    // --> [RexOp1rr#8001] and stop
    0x0007, 0x8001,
    // end of iadd.i64 (I64)
    // 00004f: iadd_ifcarry.i64 (I64)
    // --> [RexOp1rio#8011] and stop
    0x0013, 0x8011,
    // end of iadd_ifcarry.i64 (I64)
    // 000051: iadd_ifcin.i64 (I64)
    // --> [RexOp1rin#8011] and stop
    0x000f, 0x8011,
    // end of iadd_ifcin.i64 (I64)
    // 000053: iadd_ifcout.i64 (I64)
    // --> [RexOp1rout#8001] and stop
    0x000b, 0x8001,
    // end of iadd_ifcout.i64 (I64)
    // 000055: iadd_imm.i64 (I64)
    // --> [RexOp1r_ib#8083]
    0x002e, 0x8083,
    // --> [RexOp1r_id#8081] and stop
    0x0033, 0x8081,
    // end of iadd_imm.i64 (I64)
    // 000059: icmp.i64 (I64)
    // --> [RexOp1icscc#8039] and stop
    0x0193, 0x8039,
    // end of icmp.i64 (I64)
    // 00005b: icmp_imm.i64 (I64)
    // --> [RexOp1icscc_ib#f083]
    0x0196, 0xf083,
    // --> [RexOp1icscc_id#f081] and stop
    0x019b, 0xf081,
    // end of icmp_imm.i64 (I64)
    // 00005f: iconst.i64 (I64)
    // skip 4 unless inst_predicate_0
    0x5000,
    // --> [RexOp1pu_id#b8]
    0x0036, 0x00b8,
    // --> [Op1pu_id#b8]
    0x0034, 0x00b8,
    // --> [RexOp1u_id#80c7]
    0x0038, 0x80c7,
    // --> [RexOp1pu_iq#80b8]
    0x003a, 0x80b8,
    // stop unless inst_predicate_1
    // 000068: iconst.i16 (I64)
    // stop unless inst_predicate_1
    0x1001,
    // --> [RexOp1u_id_z#31]
    // --> [RexOp1u_id_z#31]
    0x0042, 0x0031,
    // --> [Op1u_id_z#31] and stop
    // --> [Op1u_id_z#31] and stop
    0x0041, 0x0031,
    // end of iconst.i16 (I64)
    // end of iconst.i64 (I64)
    // 00006d: ifcmp.i64 (I64)
    // --> [RexOp1rcmp#8039] and stop
    0x019f, 0x8039,
    // end of ifcmp.i64 (I64)
    // 00006f: ifcmp_imm.i64 (I64)
    // --> [RexOp1rcmp_ib#f083]
    0x01a2, 0xf083,
    // --> [RexOp1rcmp_id#f081] and stop
    0x01a7, 0xf081,
    // end of ifcmp_imm.i64 (I64)
    // 000073: ifcmp_sp.i64 (I64)
    // --> [RexOp1rcmp_sp#8039] and stop
    0x01ab, 0x8039,
    // end of ifcmp_sp.i64 (I64)
    // 000075: imul.i64 (I64)
    // --> [RexOp2rrx#84af] and stop
    0x001b, 0x84af,
    // end of imul.i64 (I64)
    // 000077: indirect_jump_table_br.i64 (I64)
    // --> [RexOp1indirect_jmp#40ff]
    0x0184, 0x40ff,
    // --> [Op1indirect_jmp#40ff] and stop
    // 000079: indirect_jump_table_br.i32 (I32)
    // --> [Op1indirect_jmp#40ff] and stop
    0x0187, 0x40ff,
    // end of indirect_jump_table_br.i32 (I32)
    // end of indirect_jump_table_br.i64 (I64)
    // 00007b: ishl.i64 (I64)
    // --> [RexOp1rc#c0d3] and stop
    0x0047, 0xc0d3,
    // end of ishl.i64 (I64)
    // 00007d: ishl_imm.i64 (I64)
    // --> [RexOp1r_ib#c0c1] and stop
    0x002f, 0xc0c1,
    // end of ishl_imm.i64 (I64)
    // 00007f: istore16.i64 (I64)
    // --> [RexMp1st#189]
    // 00007f: istore16.i32 (I64)
    // --> [RexMp1st#189]
    0x008e, 0x0189,
    // --> [Mp1st#189]
    // --> [Mp1st#189]
    0x008c, 0x0189,
    // --> [RexMp1stDisp8#189]
    // --> [RexMp1stDisp8#189]
    0x0096, 0x0189,
    // --> [Mp1stDisp8#189]
    // --> [Mp1stDisp8#189]
    0x0094, 0x0189,
    // --> [RexMp1stDisp32#189]
    // --> [RexMp1stDisp32#189]
    0x009e, 0x0189,
    // --> [Mp1stDisp32#189] and stop
    // --> [Mp1stDisp32#189] and stop
    0x009d, 0x0189,
    // end of istore16.i32 (I64)
    // end of istore16.i64 (I64)
    // 00008b: istore16_complex.i64 (I64)
    // stop unless inst_predicate_3
    // 00008b: istore16_complex.i32 (I64)
    // stop unless inst_predicate_3
    0x1003,
    // --> [RexMp1stWithIndex#189]
    // --> [RexMp1stWithIndex#189]
    0x006a, 0x0189,
    // --> [Mp1stWithIndex#189]
    // --> [Mp1stWithIndex#189]
    0x0068, 0x0189,
    // --> [RexMp1stWithIndexDisp8#189]
    // --> [RexMp1stWithIndexDisp8#189]
    0x0072, 0x0189,
    // --> [Mp1stWithIndexDisp8#189]
    // --> [Mp1stWithIndexDisp8#189]
    0x0070, 0x0189,
    // --> [RexMp1stWithIndexDisp32#189]
    // --> [RexMp1stWithIndexDisp32#189]
    0x007a, 0x0189,
    // --> [Mp1stWithIndexDisp32#189] and stop
    // --> [Mp1stWithIndexDisp32#189] and stop
    0x0079, 0x0189,
    // end of istore16_complex.i32 (I64)
    // end of istore16_complex.i64 (I64)
    // 000098: istore32.i64 (I64)
    // --> [RexOp1st#89]
    // 000098: store.i32 (I64)
    // --> [RexOp1st#89]
    0x008a, 0x0089,
    // --> [Op1st#89]
    // --> [Op1st#89]
    0x0088, 0x0089,
    // --> [RexOp1stDisp8#89]
    // --> [RexOp1stDisp8#89]
    0x0092, 0x0089,
    // --> [Op1stDisp8#89]
    // --> [Op1stDisp8#89]
    0x0090, 0x0089,
    // --> [RexOp1stDisp32#89]
    // --> [RexOp1stDisp32#89]
    0x009a, 0x0089,
    // --> [Op1stDisp32#89] and stop
    // --> [Op1stDisp32#89] and stop
    0x0099, 0x0089,
    // end of store.i32 (I64)
    // end of istore32.i64 (I64)
    // 0000a4: istore8.i64 (I64)
    // --> [RexOp1st#88]
    // 0000a4: istore8.i32 (I64)
    // --> [RexOp1st#88]
    0x008a, 0x0088,
    // --> [Op1st_abcd#88]
    // --> [Op1st_abcd#88]
    0x00a0, 0x0088,
    // --> [RexOp1stDisp8#88]
    // --> [RexOp1stDisp8#88]
    0x0092, 0x0088,
    // --> [Op1stDisp8_abcd#88]
    // --> [Op1stDisp8_abcd#88]
    0x00a2, 0x0088,
    // --> [RexOp1stDisp32#88]
    // --> [RexOp1stDisp32#88]
    0x009a, 0x0088,
    // --> [Op1stDisp32_abcd#88] and stop
    // --> [Op1stDisp32_abcd#88] and stop
    0x00a5, 0x0088,
    // end of istore8.i32 (I64)
    // end of istore8.i64 (I64)
    // 0000b0: istore8_complex.i64 (I64)
    // stop unless inst_predicate_3
    // 0000b0: istore8_complex.i32 (I64)
    // stop unless inst_predicate_3
    0x1003,
    // --> [RexOp1stWithIndex_abcd#88]
    // --> [RexOp1stWithIndex_abcd#88]
    0x007e, 0x0088,
    // --> [Op1stWithIndex_abcd#88]
    // --> [Op1stWithIndex_abcd#88]
    0x007c, 0x0088,
    // --> [RexOp1stWithIndexDisp8_abcd#88]
    // --> [RexOp1stWithIndexDisp8_abcd#88]
    0x0082, 0x0088,
    // --> [Op1stWithIndexDisp8_abcd#88]
    // --> [Op1stWithIndexDisp8_abcd#88]
    0x0080, 0x0088,
    // --> [RexOp1stWithIndexDisp32_abcd#88]
    // --> [RexOp1stWithIndexDisp32_abcd#88]
    0x0086, 0x0088,
    // --> [Op1stWithIndexDisp32_abcd#88] and stop
    // --> [Op1stWithIndexDisp32_abcd#88] and stop
    0x0085, 0x0088,
    // end of istore8_complex.i32 (I64)
    // end of istore8_complex.i64 (I64)
    // 0000bd: isub.i64 (I64)
    // --> [RexOp1rr#8029] and stop
    0x0007, 0x8029,
    // end of isub.i64 (I64)
    // 0000bf: isub_ifbin.i64 (I64)
    // --> [RexOp1rin#8019] and stop
    0x000f, 0x8019,
    // end of isub_ifbin.i64 (I64)
    // 0000c1: isub_ifborrow.i64 (I64)
    // --> [RexOp1rio#8019] and stop
    0x0013, 0x8019,
    // end of isub_ifborrow.i64 (I64)
    // 0000c3: isub_ifbout.i64 (I64)
    // --> [RexOp1rout#8029] and stop
    0x000b, 0x8029,
    // end of isub_ifbout.i64 (I64)
    // 0000c5: jump_table_base.i64 (I64)
    // --> [RexOp1jt_base#808d] and stop
    0x0181, 0x808d,
    // end of jump_table_base.i64 (I64)
    // 0000c7: jump_table_entry.i64 (I64)
    // --> [RexOp1jt_entry#8063] and stop
    0x017d, 0x8063,
    // end of jump_table_entry.i64 (I64)
    // 0000c9: load.i64 (I64)
    // --> [RexOp1ld#808b]
    0x00b0, 0x808b,
    // --> [RexOp1ldDisp8#808b]
    0x00b8, 0x808b,
    // --> [RexOp1ldDisp32#808b] and stop
    0x00c1, 0x808b,
    // end of load.i64 (I64)
    // 0000cf: load_complex.i64 (I64)
    // stop unless inst_predicate_2
    0x1002,
    // --> [RexOp1ldWithIndex#808b]
    0x004e, 0x808b,
    // --> [RexOp1ldWithIndexDisp8#808b]
    0x0056, 0x808b,
    // --> [RexOp1ldWithIndexDisp32#808b] and stop
    0x005f, 0x808b,
    // end of load_complex.i64 (I64)
    // 0000d6: popcnt.i64 (I64)
    // stop unless PredicateView(15)
    0x102e,
    // --> [RexMp2urm#86b8] and stop
    0x004b, 0x86b8,
    // end of popcnt.i64 (I64)
    // 0000d9: regfill.i64 (I64)
    // --> [RexOp1regfill32#808b] and stop
    // 0000d9: regfill.r64 (I64)
    // --> [RexOp1regfill32#808b] and stop
    0x00cd, 0x808b,
    // end of regfill.r64 (I64)
    // end of regfill.i64 (I64)
    // 0000db: regmove.i64 (I64)
    // --> [RexOp1rmov#8089] and stop
    // 0000db: regmove.r64 (I64)
    // --> [RexOp1rmov#8089] and stop
    0x002b, 0x8089,
    // end of regmove.r64 (I64)
    // end of regmove.i64 (I64)
    // 0000dd: regspill.i64 (I64)
    // --> [RexOp1regspill32#8089] and stop
    // 0000dd: regspill.r64 (I64)
    // --> [RexOp1regspill32#8089] and stop
    0x00ad, 0x8089,
    // end of regspill.r64 (I64)
    // end of regspill.i64 (I64)
    // 0000df: rotl.i64 (I64)
    // --> [RexOp1rc#80d3] and stop
    0x0047, 0x80d3,
    // end of rotl.i64 (I64)
    // 0000e1: rotl_imm.i64 (I64)
    // --> [RexOp1r_ib#80c1] and stop
    0x002f, 0x80c1,
    // end of rotl_imm.i64 (I64)
    // 0000e3: rotr.i64 (I64)
    // --> [RexOp1rc#90d3] and stop
    0x0047, 0x90d3,
    // end of rotr.i64 (I64)
    // 0000e5: rotr_imm.i64 (I64)
    // --> [RexOp1r_ib#90c1] and stop
    0x002f, 0x90c1,
    // end of rotr_imm.i64 (I64)
    // 0000e7: selectif.i64 (I64)
    // --> [RexOp2cmov#8440] and stop
    0x01b7, 0x8440,
    // end of selectif.i64 (I64)
    // 0000e9: set_pinned_reg.i64 (I64)
    // --> [RexOp1set_pinned_reg#8089]
    0x0002, 0x8089,
    // --> [RexOp1set_pinned_reg#8089] and stop
    0x0003, 0x8089,
    // end of set_pinned_reg.i64 (I64)
    // 0000ed: sextend.i64 (I64)
    // skip 2 unless inst_predicate_12
    0x300c,
    // --> [RexOp2urm_noflags#84be]
    0x01bc, 0x84be,
    // skip 2 unless inst_predicate_9
    0x3009,
    // --> [RexOp2urm_noflags#84bf]
    0x01bc, 0x84bf,
    // stop unless inst_predicate_10
    0x100a,
    // --> [RexOp1urm_noflags#8063] and stop
    0x01c5, 0x8063,
    // end of sextend.i64 (I64)
    // 0000f6: sload16.i64 (I64)
    // --> [RexOp2ld#84bf]
    0x00b4, 0x84bf,
    // --> [RexOp2ldDisp8#84bf]
    0x00bc, 0x84bf,
    // --> [RexOp2ldDisp32#84bf] and stop
    0x00c5, 0x84bf,
    // end of sload16.i64 (I64)
    // 0000fc: sload16_complex.i64 (I64)
    // stop unless inst_predicate_2
    0x1002,
    // --> [RexOp2ldWithIndex#84bf]
    0x0052, 0x84bf,
    // --> [RexOp2ldWithIndexDisp8#84bf]
    0x005a, 0x84bf,
    // --> [RexOp2ldWithIndexDisp32#84bf] and stop
    0x0063, 0x84bf,
    // end of sload16_complex.i64 (I64)
    // 000103: sload32.i64 (I64)
    // --> [RexOp1ld#8063]
    0x00b0, 0x8063,
    // --> [RexOp1ldDisp8#8063]
    0x00b8, 0x8063,
    // --> [RexOp1ldDisp32#8063] and stop
    0x00c1, 0x8063,
    // end of sload32.i64 (I64)
    // 000109: sload8.i64 (I64)
    // --> [RexOp2ld#84be]
    0x00b4, 0x84be,
    // --> [RexOp2ldDisp8#84be]
    0x00bc, 0x84be,
    // --> [RexOp2ldDisp32#84be] and stop
    0x00c5, 0x84be,
    // end of sload8.i64 (I64)
    // 00010f: sload8_complex.i64 (I64)
    // stop unless inst_predicate_2
    0x1002,
    // --> [RexOp2ldWithIndex#84be]
    0x0052, 0x84be,
    // --> [RexOp2ldWithIndexDisp8#84be]
    0x005a, 0x84be,
    // --> [RexOp2ldWithIndexDisp32#84be] and stop
    0x0063, 0x84be,
    // end of sload8_complex.i64 (I64)
    // 000116: spill.i64 (I64)
    // --> [RexOp1spillSib32#8089] and stop
    // 000116: spill.r64 (I64)
    // --> [RexOp1spillSib32#8089] and stop
    0x00a9, 0x8089,
    // end of spill.r64 (I64)
    // end of spill.i64 (I64)
    // 000118: sshr.i64 (I64)
    // --> [RexOp1rc#f0d3] and stop
    0x0047, 0xf0d3,
    // end of sshr.i64 (I64)
    // 00011a: sshr_imm.i64 (I64)
    // --> [RexOp1r_ib#f0c1] and stop
    0x002f, 0xf0c1,
    // end of sshr_imm.i64 (I64)
    // 00011c: stack_addr.i64 (I64)
    // --> [RexOp1spaddr8_id#808d] and stop
    0x014b, 0x808d,
    // end of stack_addr.i64 (I64)
    // 00011e: store.i64 (I64)
    // --> [RexOp1st#8089]
    0x008a, 0x8089,
    // --> [RexOp1stDisp8#8089]
    0x0092, 0x8089,
    // --> [RexOp1stDisp32#8089] and stop
    0x009b, 0x8089,
    // end of store.i64 (I64)
    // 000124: store_complex.i64 (I64)
    // stop unless inst_predicate_3
    0x1003,
    // --> [RexOp1stWithIndex#8089]
    0x0066, 0x8089,
    // --> [RexOp1stWithIndexDisp8#8089]
    0x006e, 0x8089,
    // --> [RexOp1stWithIndexDisp32#8089] and stop
    0x0077, 0x8089,
    // end of store_complex.i64 (I64)
    // 00012b: symbol_value.i64 (I64)
    // skip 2 unless PredicateView(12)
    0x302b,
    // --> [RexOp1gvaddr8#80b8]
    0x0142, 0x80b8,
    // skip 3 unless PredicateView(10)
    0x4029,
    // skip 2 unless inst_predicate_5
    0x3005,
    // --> [RexOp1pcrel_gvaddr8#808d]
    0x0144, 0x808d,
    // stop unless PredicateView(10)
    0x1029,
    // --> [RexOp1got_gvaddr8#808b] and stop
    0x0147, 0x808b,
    // end of symbol_value.i64 (I64)
    // 000135: uextend.i64 (I64)
    // skip 4 unless inst_predicate_12
    0x500c,
    // --> [RexOp2urm_noflags#4b6]
    0x01bc, 0x04b6,
    // --> [Op2urm_noflags_abcd#4b6]
    0x01be, 0x04b6,
    // skip 4 unless inst_predicate_9
    0x5009,
    // --> [RexOp2urm_noflags#4b7]
    0x01bc, 0x04b7,
    // --> [Op2urm_noflags#4b7]
    0x01c2, 0x04b7,
    // stop unless inst_predicate_10
    0x100a,
    // --> [RexOp1umr#89]
    // 000140: copy.i32 (I64)
    // --> [RexOp1umr#89]
    // 000140: copy.b1 (I64)
    // --> [RexOp1umr#89]
    // 000140: copy.i8 (I64)
    // --> [RexOp1umr#89]
    // 000140: copy.i16 (I64)
    // --> [RexOp1umr#89]
    0x0026, 0x0089,
    // --> [Op1umr#89] and stop
    // --> [Op1umr#89] and stop
    // --> [Op1umr#89] and stop
    // --> [Op1umr#89] and stop
    // --> [Op1umr#89] and stop
    // 000142: copy.i32 (I32)
    // --> [Op1umr#89] and stop
    // 000142: copy.b1 (I32)
    // --> [Op1umr#89] and stop
    // 000142: copy.r32 (I32)
    // --> [Op1umr#89] and stop
    // 000142: copy.i8 (I32)
    // --> [Op1umr#89] and stop
    // 000142: copy.i16 (I32)
    // --> [Op1umr#89] and stop
    0x0025, 0x0089,
    // end of copy.i16 (I32)
    // end of copy.i8 (I32)
    // end of copy.r32 (I32)
    // end of copy.b1 (I32)
    // end of copy.i32 (I32)
    // end of copy.i16 (I64)
    // end of copy.i8 (I64)
    // end of copy.b1 (I64)
    // end of copy.i32 (I64)
    // end of uextend.i64 (I64)
    // 000144: uload16.i64 (I64)
    // --> [RexOp2ld#84b7]
    0x00b4, 0x84b7,
    // --> [RexOp2ldDisp8#84b7]
    0x00bc, 0x84b7,
    // --> [RexOp2ldDisp32#84b7] and stop
    0x00c5, 0x84b7,
    // end of uload16.i64 (I64)
    // 00014a: uload16_complex.i64 (I64)
    // stop unless inst_predicate_2
    0x1002,
    // --> [RexOp2ldWithIndex#84b7]
    0x0052, 0x84b7,
    // --> [RexOp2ldWithIndexDisp8#84b7]
    0x005a, 0x84b7,
    // --> [RexOp2ldWithIndexDisp32#84b7] and stop
    0x0063, 0x84b7,
    // end of uload16_complex.i64 (I64)
    // 000151: uload32.i64 (I64)
    // --> [RexOp1ld#8b]
    // 000151: load.i32 (I64)
    // --> [RexOp1ld#8b]
    0x00b0, 0x008b,
    // --> [Op1ld#8b]
    // --> [Op1ld#8b]
    0x00ae, 0x008b,
    // --> [RexOp1ldDisp8#8b]
    // --> [RexOp1ldDisp8#8b]
    0x00b8, 0x008b,
    // --> [Op1ldDisp8#8b]
    // --> [Op1ldDisp8#8b]
    0x00b6, 0x008b,
    // --> [RexOp1ldDisp32#8b]
    // --> [RexOp1ldDisp32#8b]
    0x00c0, 0x008b,
    // --> [Op1ldDisp32#8b] and stop
    // --> [Op1ldDisp32#8b] and stop
    0x00bf, 0x008b,
    // end of load.i32 (I64)
    // end of uload32.i64 (I64)
    // 00015d: uload8.i64 (I64)
    // --> [RexOp2ld#84b6]
    0x00b4, 0x84b6,
    // --> [RexOp2ldDisp8#84b6]
    0x00bc, 0x84b6,
    // --> [RexOp2ldDisp32#84b6] and stop
    0x00c5, 0x84b6,
    // end of uload8.i64 (I64)
    // 000163: uload8_complex.i64 (I64)
    // stop unless inst_predicate_2
    0x1002,
    // --> [RexOp2ldWithIndex#84b6]
    0x0052, 0x84b6,
    // --> [RexOp2ldWithIndexDisp8#84b6]
    0x005a, 0x84b6,
    // --> [RexOp2ldWithIndexDisp32#84b6] and stop
    0x0063, 0x84b6,
    // end of uload8_complex.i64 (I64)
    // 00016a: ushr.i64 (I64)
    // --> [RexOp1rc#d0d3] and stop
    0x0047, 0xd0d3,
    // end of ushr.i64 (I64)
    // 00016c: ushr_imm.i64 (I64)
    // --> [RexOp1r_ib#d0c1] and stop
    0x002f, 0xd0c1,
    // end of ushr_imm.i64 (I64)
    // 00016e: x86_bsf.i64 (I64)
    // --> [RexOp2bsf_and_bsr#84bc] and stop
    0x01bb, 0x84bc,
    // end of x86_bsf.i64 (I64)
    // 000170: x86_bsr.i64 (I64)
    // --> [RexOp2bsf_and_bsr#84bd] and stop
    0x01bb, 0x84bd,
    // end of x86_bsr.i64 (I64)
    // 000172: x86_cvtt2si.i64 (I64)
    // skip 2 unless inst_predicate_15
    0x300f,
    // --> [RexMp2rfurm#862c]
    0x01e4, 0x862c,
    // stop unless inst_predicate_16
    0x1010,
    // --> [RexMp2rfurm#872c] and stop
    0x01e5, 0x872c,
    // end of x86_cvtt2si.i64 (I64)
    // 000178: x86_pop.i64 (I64)
    // --> [RexOp1popq#58]
    0x00d8, 0x0058,
    // --> [Op1popq#58] and stop
    // 00017a: x86_pop.i32 (I32)
    // --> [Op1popq#58] and stop
    0x00d7, 0x0058,
    // end of x86_pop.i32 (I32)
    // end of x86_pop.i64 (I64)
    // 00017c: x86_push.i64 (I64)
    // --> [RexOp1pushq#50]
    0x00d4, 0x0050,
    // --> [Op1pushq#50] and stop
    // 00017e: x86_push.i32 (I32)
    // --> [Op1pushq#50] and stop
    0x00d3, 0x0050,
    // end of x86_push.i32 (I32)
    // end of x86_push.i64 (I64)
    // 000180: x86_sdivmodx.i64 (I64)
    // --> [RexOp1div#f0f7] and stop
    0x001f, 0xf0f7,
    // end of x86_sdivmodx.i64 (I64)
    // 000182: x86_smulx.i64 (I64)
    // --> [RexOp1mulx#d0f7] and stop
    0x0023, 0xd0f7,
    // end of x86_smulx.i64 (I64)
    // 000184: x86_udivmodx.i64 (I64)
    // --> [RexOp1div#e0f7] and stop
    0x001f, 0xe0f7,
    // end of x86_udivmodx.i64 (I64)
    // 000186: x86_umulx.i64 (I64)
    // --> [RexOp1mulx#c0f7] and stop
    0x0023, 0xc0f7,
    // end of x86_umulx.i64 (I64)
    // 000188: band.i32 (I64)
    // --> [RexOp1rr#21]
    // 000188: band.b32 (I64)
    // --> [RexOp1rr#21]
    // 000188: band.b1 (I64)
    // --> [RexOp1rr#21]
    0x0006, 0x0021,
    // --> [Op1rr#21] and stop
    // --> [Op1rr#21] and stop
    // --> [Op1rr#21] and stop
    // 00018a: band.i32 (I32)
    // --> [Op1rr#21] and stop
    // 00018a: band.b32 (I32)
    // --> [Op1rr#21] and stop
    // 00018a: band.b1 (I32)
    // --> [Op1rr#21] and stop
    0x0005, 0x0021,
    // end of band.b1 (I32)
    // end of band.b32 (I32)
    // end of band.i32 (I32)
    // end of band.b1 (I64)
    // end of band.b32 (I64)
    // end of band.i32 (I64)
    // 00018c: band_imm.i32 (I64)
    // --> [RexOp1r_ib#4083]
    0x002e, 0x4083,
    // --> [Op1r_ib#4083]
    0x002c, 0x4083,
    // --> [RexOp1r_id#4081]
    0x0032, 0x4081,
    // --> [Op1r_id#4081] and stop
    0x0031, 0x4081,
    // end of band_imm.i32 (I64)
    // 000194: bitcast.i32 (I64)
    // stop unless inst_predicate_15
    0x100f,
    // --> [RexMp2rfumr#57e]
    0x01d4, 0x057e,
    // --> [Mp2rfumr#57e] and stop
    0x01d3, 0x057e,
    // end of bitcast.i32 (I64)
    // 000199: bnot.i32 (I64)
    // --> [RexOp1ur#20f7]
    // 000199: bnot.b32 (I64)
    // --> [RexOp1ur#20f7]
    0x0016, 0x20f7,
    // --> [Op1ur#20f7] and stop
    // --> [Op1ur#20f7] and stop
    // 00019b: bnot.i32 (I32)
    // --> [Op1ur#20f7] and stop
    // 00019b: bnot.b32 (I32)
    // --> [Op1ur#20f7] and stop
    0x0015, 0x20f7,
    // end of bnot.b32 (I32)
    // end of bnot.i32 (I32)
    // end of bnot.b32 (I64)
    // end of bnot.i32 (I64)
    // 00019d: bor.i32 (I64)
    // --> [RexOp1rr#09]
    // 00019d: bor.b32 (I64)
    // --> [RexOp1rr#09]
    // 00019d: bor.b1 (I64)
    // --> [RexOp1rr#09]
    0x0006, 0x0009,
    // --> [Op1rr#09] and stop
    // --> [Op1rr#09] and stop
    // --> [Op1rr#09] and stop
    // 00019f: bor.i32 (I32)
    // --> [Op1rr#09] and stop
    // 00019f: bor.b32 (I32)
    // --> [Op1rr#09] and stop
    // 00019f: bor.b1 (I32)
    // --> [Op1rr#09] and stop
    0x0005, 0x0009,
    // end of bor.b1 (I32)
    // end of bor.b32 (I32)
    // end of bor.i32 (I32)
    // end of bor.b1 (I64)
    // end of bor.b32 (I64)
    // end of bor.i32 (I64)
    // 0001a1: bor_imm.i32 (I64)
    // --> [RexOp1r_ib#1083]
    0x002e, 0x1083,
    // --> [Op1r_ib#1083]
    0x002c, 0x1083,
    // --> [RexOp1r_id#1081]
    0x0032, 0x1081,
    // --> [Op1r_id#1081] and stop
    0x0031, 0x1081,
    // end of bor_imm.i32 (I64)
    // 0001a9: brnz.i32 (I64)
    // --> [RexOp1tjccb#75]
    0x016c, 0x0075,
    // --> [Op1tjccb#75]
    0x016a, 0x0075,
    // --> [RexOp1tjccd#85]
    0x0170, 0x0085,
    // --> [Op1tjccd#85] and stop
    0x016f, 0x0085,
    // end of brnz.i32 (I64)
    // 0001b1: brz.i32 (I64)
    // --> [RexOp1tjccb#74]
    0x016c, 0x0074,
    // --> [Op1tjccb#74]
    0x016a, 0x0074,
    // --> [RexOp1tjccd#84]
    0x0170, 0x0084,
    // --> [Op1tjccd#84] and stop
    0x016f, 0x0084,
    // end of brz.i32 (I64)
    // 0001b9: bxor.i32 (I64)
    // --> [RexOp1rr#31]
    // 0001b9: bxor.b32 (I64)
    // --> [RexOp1rr#31]
    // 0001b9: bxor.b1 (I64)
    // --> [RexOp1rr#31]
    0x0006, 0x0031,
    // --> [Op1rr#31] and stop
    // --> [Op1rr#31] and stop
    // --> [Op1rr#31] and stop
    // 0001bb: bxor.i32 (I32)
    // --> [Op1rr#31] and stop
    // 0001bb: bxor.b32 (I32)
    // --> [Op1rr#31] and stop
    // 0001bb: bxor.b1 (I32)
    // --> [Op1rr#31] and stop
    0x0005, 0x0031,
    // end of bxor.b1 (I32)
    // end of bxor.b32 (I32)
    // end of bxor.i32 (I32)
    // end of bxor.b1 (I64)
    // end of bxor.b32 (I64)
    // end of bxor.i32 (I64)
    // 0001bd: bxor_imm.i32 (I64)
    // --> [RexOp1r_ib#6083]
    0x002e, 0x6083,
    // --> [Op1r_ib#6083]
    0x002c, 0x6083,
    // --> [RexOp1r_id#6081]
    0x0032, 0x6081,
    // --> [Op1r_id#6081] and stop
    0x0031, 0x6081,
    // end of bxor_imm.i32 (I64)
    // 0001c5: clz.i32 (I64)
    // stop unless PredicateView(14)
    0x102d,
    // --> [RexMp2urm#6bd]
    0x004a, 0x06bd,
    // --> [Mp2urm#6bd] and stop
    0x0049, 0x06bd,
    // end of clz.i32 (I64)
    // 0001ca: copy_to_ssa.i32 (I64)
    // --> [RexOp1umr_reg_to_ssa#89] and stop
    // 0001ca: copy_to_ssa.b1 (I64)
    // --> [RexOp1umr_reg_to_ssa#89] and stop
    // 0001ca: copy_to_ssa.i8 (I64)
    // --> [RexOp1umr_reg_to_ssa#89] and stop
    // 0001ca: copy_to_ssa.i16 (I64)
    // --> [RexOp1umr_reg_to_ssa#89] and stop
    0x00e1, 0x0089,
    // end of copy_to_ssa.i16 (I64)
    // end of copy_to_ssa.i8 (I64)
    // end of copy_to_ssa.b1 (I64)
    // end of copy_to_ssa.i32 (I64)
    // 0001cc: ctz.i32 (I64)
    // stop unless PredicateView(13)
    0x102c,
    // --> [RexMp2urm#6bc]
    0x004a, 0x06bc,
    // --> [Mp2urm#6bc] and stop
    0x0049, 0x06bc,
    // end of ctz.i32 (I64)
    // 0001d1: fill.i32 (I64)
    // --> [RexOp1fillSib32#8b]
    // 0001d1: fill.b1 (I64)
    // --> [RexOp1fillSib32#8b]
    // 0001d1: fill.i8 (I64)
    // --> [RexOp1fillSib32#8b]
    // 0001d1: fill.i16 (I64)
    // --> [RexOp1fillSib32#8b]
    0x00c8, 0x008b,
    // --> [Op1fillSib32#8b] and stop
    // --> [Op1fillSib32#8b] and stop
    // --> [Op1fillSib32#8b] and stop
    // --> [Op1fillSib32#8b] and stop
    // 0001d3: fill.i32 (I32)
    // --> [Op1fillSib32#8b] and stop
    // 0001d3: fill.b1 (I32)
    // --> [Op1fillSib32#8b] and stop
    // 0001d3: fill.r32 (I32)
    // --> [Op1fillSib32#8b] and stop
    // 0001d3: fill.i8 (I32)
    // --> [Op1fillSib32#8b] and stop
    // 0001d3: fill.i16 (I32)
    // --> [Op1fillSib32#8b] and stop
    0x00c7, 0x008b,
    // end of fill.i16 (I32)
    // end of fill.i8 (I32)
    // end of fill.r32 (I32)
    // end of fill.b1 (I32)
    // end of fill.i32 (I32)
    // end of fill.i16 (I64)
    // end of fill.i8 (I64)
    // end of fill.b1 (I64)
    // end of fill.i32 (I64)
    // 0001d5: iadd.i32 (I64)
    // --> [RexOp1rr#01]
    0x0006, 0x0001,
    // --> [Op1rr#01] and stop
    // 0001d7: iadd.i32 (I32)
    // --> [Op1rr#01] and stop
    0x0005, 0x0001,
    // end of iadd.i32 (I32)
    // end of iadd.i32 (I64)
    // 0001d9: iadd_ifcarry.i32 (I64)
    // --> [RexOp1rio#11]
    0x0012,
    // 0001da: iadd_ifcarry.i32 (I32)
    // --> [Op1rio#11] and stop
    0x0011,
    // --> [Op1rio#11] and stop
    0x0011,
    // end of iadd_ifcarry.i32 (I32)
    0x0011,
    // end of iadd_ifcarry.i32 (I64)
    // 0001dd: iadd_ifcin.i32 (I64)
    // --> [RexOp1rin#11]
    0x000e, 0x0011,
    // --> [Op1rin#11] and stop
    // 0001df: iadd_ifcin.i32 (I32)
    // --> [Op1rin#11] and stop
    0x000d, 0x0011,
    // end of iadd_ifcin.i32 (I32)
    // end of iadd_ifcin.i32 (I64)
    // 0001e1: iadd_ifcout.i32 (I64)
    // --> [RexOp1rout#01]
    0x000a, 0x0001,
    // --> [Op1rout#01] and stop
    // 0001e3: iadd_ifcout.i32 (I32)
    // --> [Op1rout#01] and stop
    0x0009, 0x0001,
    // end of iadd_ifcout.i32 (I32)
    // end of iadd_ifcout.i32 (I64)
    // 0001e5: iadd_imm.i32 (I64)
    // --> [RexOp1r_ib#83]
    0x002e, 0x0083,
    // --> [Op1r_ib#83]
    0x002c, 0x0083,
    // --> [RexOp1r_id#81]
    0x0032, 0x0081,
    // --> [Op1r_id#81] and stop
    0x0031, 0x0081,
    // end of iadd_imm.i32 (I64)
    // 0001ed: icmp.i32 (I64)
    // --> [RexOp1icscc#39]
    0x0192, 0x0039,
    // --> [Op1icscc#39] and stop
    // 0001ef: icmp.i32 (I32)
    // --> [Op1icscc#39] and stop
    0x0191, 0x0039,
    // end of icmp.i32 (I32)
    // end of icmp.i32 (I64)
    // 0001f1: icmp_imm.i32 (I64)
    // --> [RexOp1icscc_ib#7083]
    0x0196, 0x7083,
    // --> [Op1icscc_ib#7083]
    0x0194, 0x7083,
    // --> [RexOp1icscc_id#7081]
    0x019a, 0x7081,
    // --> [Op1icscc_id#7081] and stop
    0x0199, 0x7081,
    // end of icmp_imm.i32 (I64)
    // 0001f9: iconst.i32 (I64)
    // --> [RexOp1pu_id#b8]
    0x0036, 0x00b8,
    // --> [Op1pu_id#b8]
    0x0034, 0x00b8,
    // stop unless inst_predicate_1
    0x1001,
    // --> [RexOp1u_id_z#31]
    0x0042, 0x0031,
    // --> [Op1u_id_z#31] and stop
    0x0041, 0x0031,
    // end of iconst.i32 (I64)
    // 000202: ifcmp.i32 (I64)
    // --> [RexOp1rcmp#39]
    0x019e, 0x0039,
    // --> [Op1rcmp#39] and stop
    // 000204: ifcmp.i32 (I32)
    // --> [Op1rcmp#39] and stop
    0x019d, 0x0039,
    // end of ifcmp.i32 (I32)
    // end of ifcmp.i32 (I64)
    // 000206: ifcmp_imm.i32 (I64)
    // --> [RexOp1rcmp_ib#7083]
    0x01a2, 0x7083,
    // --> [Op1rcmp_ib#7083]
    0x01a0, 0x7083,
    // --> [RexOp1rcmp_id#7081]
    0x01a6, 0x7081,
    // --> [Op1rcmp_id#7081] and stop
    0x01a5, 0x7081,
    // end of ifcmp_imm.i32 (I64)
    // 00020e: imul.i32 (I64)
    // --> [RexOp2rrx#4af]
    0x001a, 0x04af,
    // --> [Op2rrx#4af] and stop
    // 000210: imul.i32 (I32)
    // --> [Op2rrx#4af] and stop
    0x0019, 0x04af,
    // end of imul.i32 (I32)
    // end of imul.i32 (I64)
    // 000212: ireduce.i32 (I64)
    // stop unless inst_predicate_11
    0x100b,
    // --> [null#00] and stop
    0x01c1, 0x0000,
    // end of ireduce.i32 (I64)
    // 000215: ishl.i32 (I64)
    // --> [RexOp1rc#40d3]
    0x0046, 0x40d3,
    // --> [Op1rc#40d3] and stop
    // 000217: ishl.i32 (I32)
    // --> [Op1rc#40d3] and stop
    0x0045, 0x40d3,
    // end of ishl.i32 (I32)
    // end of ishl.i32 (I64)
    // 000219: ishl_imm.i32 (I64)
    // --> [RexOp1r_ib#40c1]
    0x002e, 0x40c1,
    // --> [Op1r_ib#40c1] and stop
    // 00021b: ishl_imm.i32 (I32)
    // --> [Op1r_ib#40c1] and stop
    0x002d, 0x40c1,
    // end of ishl_imm.i32 (I32)
    // end of ishl_imm.i32 (I64)
    // 00021d: isub.i32 (I64)
    // --> [RexOp1rr#29]
    0x0006, 0x0029,
    // --> [Op1rr#29] and stop
    // 00021f: isub.i32 (I32)
    // --> [Op1rr#29] and stop
    0x0005, 0x0029,
    // end of isub.i32 (I32)
    // end of isub.i32 (I64)
    // 000221: isub_ifbin.i32 (I64)
    // --> [RexOp1rin#19]
    0x000e, 0x0019,
    // --> [Op1rin#19] and stop
    // 000223: isub_ifbin.i32 (I32)
    // --> [Op1rin#19] and stop
    0x000d, 0x0019,
    // end of isub_ifbin.i32 (I32)
    // end of isub_ifbin.i32 (I64)
    // 000225: isub_ifborrow.i32 (I64)
    // --> [RexOp1rio#19]
    0x0012, 0x0019,
    // --> [Op1rio#19] and stop
    // 000227: isub_ifborrow.i32 (I32)
    // --> [Op1rio#19] and stop
    0x0011, 0x0019,
    // end of isub_ifborrow.i32 (I32)
    // end of isub_ifborrow.i32 (I64)
    // 000229: isub_ifbout.i32 (I64)
    // --> [RexOp1rout#29]
    0x000a, 0x0029,
    // --> [Op1rout#29] and stop
    // 00022b: isub_ifbout.i32 (I32)
    // --> [Op1rout#29] and stop
    0x0009, 0x0029,
    // end of isub_ifbout.i32 (I32)
    // end of isub_ifbout.i32 (I64)
    // 00022d: load_complex.i32 (I64)
    // stop unless inst_predicate_2
    // 00022d: uload32_complex (I64)
    // stop unless inst_predicate_2
    0x1002,
    // --> [RexOp1ldWithIndex#8b]
    // --> [RexOp1ldWithIndex#8b]
    0x004e, 0x008b,
    // --> [Op1ldWithIndex#8b]
    // --> [Op1ldWithIndex#8b]
    0x004c, 0x008b,
    // --> [RexOp1ldWithIndexDisp8#8b]
    // --> [RexOp1ldWithIndexDisp8#8b]
    0x0056, 0x008b,
    // --> [Op1ldWithIndexDisp8#8b]
    // --> [Op1ldWithIndexDisp8#8b]
    0x0054, 0x008b,
    // --> [RexOp1ldWithIndexDisp32#8b]
    // --> [RexOp1ldWithIndexDisp32#8b]
    0x005e, 0x008b,
    // --> [Op1ldWithIndexDisp32#8b] and stop
    // --> [Op1ldWithIndexDisp32#8b] and stop
    0x005d, 0x008b,
    // end of uload32_complex (I64)
    // end of load_complex.i32 (I64)
    // 00023a: popcnt.i32 (I64)
    // stop unless PredicateView(15)
    0x102e,
    // --> [RexMp2urm#6b8]
    0x004a, 0x06b8,
    // --> [Mp2urm#6b8] and stop
    0x0049, 0x06b8,
    // end of popcnt.i32 (I64)
    // 00023f: regfill.i32 (I64)
    // --> [RexOp1regfill32#8b]
    // 00023f: regfill.b1 (I64)
    // --> [RexOp1regfill32#8b]
    // 00023f: regfill.i8 (I64)
    // --> [RexOp1regfill32#8b]
    // 00023f: regfill.i16 (I64)
    // --> [RexOp1regfill32#8b]
    0x00cc, 0x008b,
    // --> [Op1regfill32#8b] and stop
    // --> [Op1regfill32#8b] and stop
    // --> [Op1regfill32#8b] and stop
    // --> [Op1regfill32#8b] and stop
    // 000241: regfill.i32 (I32)
    // --> [Op1regfill32#8b] and stop
    // 000241: regfill.b1 (I32)
    // --> [Op1regfill32#8b] and stop
    // 000241: regfill.r32 (I32)
    // --> [Op1regfill32#8b] and stop
    // 000241: regfill.i8 (I32)
    // --> [Op1regfill32#8b] and stop
    // 000241: regfill.i16 (I32)
    // --> [Op1regfill32#8b] and stop
    0x00cb, 0x008b,
    // end of regfill.i16 (I32)
    // end of regfill.i8 (I32)
    // end of regfill.r32 (I32)
    // end of regfill.b1 (I32)
    // end of regfill.i32 (I32)
    // end of regfill.i16 (I64)
    // end of regfill.i8 (I64)
    // end of regfill.b1 (I64)
    // end of regfill.i32 (I64)
    // 000243: regmove.i32 (I64)
    // --> [RexOp1rmov#89] and stop
    // 000243: regmove.b32 (I64)
    // --> [RexOp1rmov#89] and stop
    // 000243: regmove.i16 (I64)
    // --> [RexOp1rmov#89] and stop
    // 000243: regmove.b8 (I64)
    // --> [RexOp1rmov#89] and stop
    // 000243: regmove.b16 (I64)
    // --> [RexOp1rmov#89] and stop
    // 000243: regmove.r32 (I64)
    // --> [RexOp1rmov#89] and stop
    0x002b, 0x0089,
    // end of regmove.r32 (I64)
    // end of regmove.b16 (I64)
    // end of regmove.b8 (I64)
    // end of regmove.i16 (I64)
    // end of regmove.b32 (I64)
    // end of regmove.i32 (I64)
    // 000245: regspill.i32 (I64)
    // --> [RexOp1regspill32#89]
    // 000245: regspill.b1 (I64)
    // --> [RexOp1regspill32#89]
    // 000245: regspill.i8 (I64)
    // --> [RexOp1regspill32#89]
    // 000245: regspill.i16 (I64)
    // --> [RexOp1regspill32#89]
    0x00ac, 0x0089,
    // --> [Op1regspill32#89] and stop
    // --> [Op1regspill32#89] and stop
    // --> [Op1regspill32#89] and stop
    // --> [Op1regspill32#89] and stop
    // 000247: regspill.i32 (I32)
    // --> [Op1regspill32#89] and stop
    // 000247: regspill.b1 (I32)
    // --> [Op1regspill32#89] and stop
    // 000247: regspill.r32 (I32)
    // --> [Op1regspill32#89] and stop
    // 000247: regspill.i8 (I32)
    // --> [Op1regspill32#89] and stop
    // 000247: regspill.i16 (I32)
    // --> [Op1regspill32#89] and stop
    0x00ab, 0x0089,
    // end of regspill.i16 (I32)
    // end of regspill.i8 (I32)
    // end of regspill.r32 (I32)
    // end of regspill.b1 (I32)
    // end of regspill.i32 (I32)
    // end of regspill.i16 (I64)
    // end of regspill.i8 (I64)
    // end of regspill.b1 (I64)
    // end of regspill.i32 (I64)
    // 000249: rotl.i32 (I64)
    // --> [RexOp1rc#d3]
    0x0046, 0x00d3,
    // --> [Op1rc#d3] and stop
    // 00024b: rotl.i32 (I32)
    // --> [Op1rc#d3] and stop
    0x0045, 0x00d3,
    // end of rotl.i32 (I32)
    // end of rotl.i32 (I64)
    // 00024d: rotl_imm.i32 (I64)
    // --> [RexOp1r_ib#c1]
    0x002e, 0x00c1,
    // --> [Op1r_ib#c1] and stop
    // 00024f: rotl_imm.i32 (I32)
    // --> [Op1r_ib#c1] and stop
    0x002d, 0x00c1,
    // end of rotl_imm.i32 (I32)
    // end of rotl_imm.i32 (I64)
    // 000251: rotr.i32 (I64)
    // --> [RexOp1rc#10d3]
    0x0046, 0x10d3,
    // --> [Op1rc#10d3] and stop
    // 000253: rotr.i32 (I32)
    // --> [Op1rc#10d3] and stop
    0x0045, 0x10d3,
    // end of rotr.i32 (I32)
    // end of rotr.i32 (I64)
    // 000255: rotr_imm.i32 (I64)
    // --> [RexOp1r_ib#10c1]
    0x002e, 0x10c1,
    // --> [Op1r_ib#10c1] and stop
    // 000257: rotr_imm.i32 (I32)
    // --> [Op1r_ib#10c1] and stop
    0x002d, 0x10c1,
    // end of rotr_imm.i32 (I32)
    // end of rotr_imm.i32 (I64)
    // 000259: selectif.i32 (I64)
    // --> [RexOp2cmov#440]
    0x01b6, 0x0440,
    // --> [Op2cmov#440] and stop
    // 00025b: selectif.i32 (I32)
    // --> [Op2cmov#440] and stop
    0x01b5, 0x0440,
    // end of selectif.i32 (I32)
    // end of selectif.i32 (I64)
    // 00025d: sextend.i32 (I64)
    // skip 4 unless inst_predicate_12
    0x500c,
    // --> [RexOp2urm_noflags#4be]
    0x01bc, 0x04be,
    // --> [Op2urm_noflags_abcd#4be]
    0x01be, 0x04be,
    // stop unless inst_predicate_9
    0x1009,
    // --> [RexOp2urm_noflags#4bf]
    0x01bc, 0x04bf,
    // --> [Op2urm_noflags#4bf] and stop
    0x01c3, 0x04bf,
    // end of sextend.i32 (I64)
    // 000267: sload16.i32 (I64)
    // --> [RexOp2ld#4bf]
    0x00b4, 0x04bf,
    // --> [Op2ld#4bf]
    0x00b2, 0x04bf,
    // --> [RexOp2ldDisp8#4bf]
    0x00bc, 0x04bf,
    // --> [Op2ldDisp8#4bf]
    0x00ba, 0x04bf,
    // --> [RexOp2ldDisp32#4bf]
    0x00c4, 0x04bf,
    // --> [Op2ldDisp32#4bf] and stop
    0x00c3, 0x04bf,
    // end of sload16.i32 (I64)
    // 000273: sload16_complex.i32 (I64)
    // stop unless inst_predicate_2
    0x1002,
    // --> [RexOp2ldWithIndex#4bf]
    0x0052, 0x04bf,
    // --> [Op2ldWithIndex#4bf]
    0x0050, 0x04bf,
    // --> [RexOp2ldWithIndexDisp8#4bf]
    0x005a, 0x04bf,
    // --> [Op2ldWithIndexDisp8#4bf]
    0x0058, 0x04bf,
    // --> [RexOp2ldWithIndexDisp32#4bf]
    0x0062, 0x04bf,
    // --> [Op2ldWithIndexDisp32#4bf] and stop
    0x0061, 0x04bf,
    // end of sload16_complex.i32 (I64)
    // 000280: sload8.i32 (I64)
    // --> [RexOp2ld#4be]
    0x00b4, 0x04be,
    // --> [Op2ld#4be]
    0x00b2, 0x04be,
    // --> [RexOp2ldDisp8#4be]
    0x00bc, 0x04be,
    // --> [Op2ldDisp8#4be]
    0x00ba, 0x04be,
    // --> [RexOp2ldDisp32#4be]
    0x00c4, 0x04be,
    // --> [Op2ldDisp32#4be] and stop
    0x00c3, 0x04be,
    // end of sload8.i32 (I64)
    // 00028c: sload8_complex.i32 (I64)
    // stop unless inst_predicate_2
    0x1002,
    // --> [RexOp2ldWithIndex#4be]
    0x0052, 0x04be,
    // --> [Op2ldWithIndex#4be]
    0x0050, 0x04be,
    // --> [RexOp2ldWithIndexDisp8#4be]
    0x005a, 0x04be,
    // --> [Op2ldWithIndexDisp8#4be]
    0x0058, 0x04be,
    // --> [RexOp2ldWithIndexDisp32#4be]
    0x0062, 0x04be,
    // --> [Op2ldWithIndexDisp32#4be] and stop
    0x0061, 0x04be,
    // end of sload8_complex.i32 (I64)
    // 000299: spill.i32 (I64)
    // --> [RexOp1spillSib32#89]
    // 000299: spill.b1 (I64)
    // --> [RexOp1spillSib32#89]
    // 000299: spill.i8 (I64)
    // --> [RexOp1spillSib32#89]
    // 000299: spill.i16 (I64)
    // --> [RexOp1spillSib32#89]
    0x00a8, 0x0089,
    // --> [Op1spillSib32#89] and stop
    // --> [Op1spillSib32#89] and stop
    // --> [Op1spillSib32#89] and stop
    // --> [Op1spillSib32#89] and stop
    // 00029b: spill.i32 (I32)
    // --> [Op1spillSib32#89] and stop
    // 00029b: spill.b1 (I32)
    // --> [Op1spillSib32#89] and stop
    // 00029b: spill.r32 (I32)
    // --> [Op1spillSib32#89] and stop
    // 00029b: spill.i8 (I32)
    // --> [Op1spillSib32#89] and stop
    // 00029b: spill.i16 (I32)
    // --> [Op1spillSib32#89] and stop
    0x00a7, 0x0089,
    // end of spill.i16 (I32)
    // end of spill.i8 (I32)
    // end of spill.r32 (I32)
    // end of spill.b1 (I32)
    // end of spill.i32 (I32)
    // end of spill.i16 (I64)
    // end of spill.i8 (I64)
    // end of spill.b1 (I64)
    // end of spill.i32 (I64)
    // 00029d: sshr.i32 (I64)
    // --> [RexOp1rc#70d3]
    0x0046, 0x70d3,
    // --> [Op1rc#70d3] and stop
    // 00029f: sshr.i32 (I32)
    // --> [Op1rc#70d3] and stop
    0x0045, 0x70d3,
    // end of sshr.i32 (I32)
    // end of sshr.i32 (I64)
    // 0002a1: sshr_imm.i32 (I64)
    // --> [RexOp1r_ib#70c1]
    0x002e, 0x70c1,
    // --> [Op1r_ib#70c1] and stop
    // 0002a3: sshr_imm.i32 (I32)
    // --> [Op1r_ib#70c1] and stop
    0x002d, 0x70c1,
    // end of sshr_imm.i32 (I32)
    // end of sshr_imm.i32 (I64)
    // 0002a5: store_complex.i32 (I64)
    // stop unless inst_predicate_3
    // 0002a5: istore32_complex (I64)
    // stop unless inst_predicate_3
    0x1003,
    // --> [RexOp1stWithIndex#89]
    // --> [RexOp1stWithIndex#89]
    0x0066, 0x0089,
    // --> [Op1stWithIndex#89]
    // --> [Op1stWithIndex#89]
    0x0064, 0x0089,
    // --> [RexOp1stWithIndexDisp8#89]
    // --> [RexOp1stWithIndexDisp8#89]
    0x006e, 0x0089,
    // --> [Op1stWithIndexDisp8#89]
    // --> [Op1stWithIndexDisp8#89]
    0x006c, 0x0089,
    // --> [RexOp1stWithIndexDisp32#89]
    // --> [RexOp1stWithIndexDisp32#89]
    0x0076, 0x0089,
    // --> [Op1stWithIndexDisp32#89] and stop
    // --> [Op1stWithIndexDisp32#89] and stop
    0x0075, 0x0089,
    // end of istore32_complex (I64)
    // end of store_complex.i32 (I64)
    // 0002b2: uextend.i32 (I64)
    // skip 4 unless inst_predicate_12
    0x500c,
    // --> [RexOp2urm_noflags#4b6]
    0x01bc, 0x04b6,
    // --> [Op2urm_noflags_abcd#4b6]
    0x01be, 0x04b6,
    // stop unless inst_predicate_9
    0x1009,
    // --> [RexOp2urm_noflags#4b7]
    0x01bc, 0x04b7,
    // --> [Op2urm_noflags#4b7] and stop
    0x01c3, 0x04b7,
    // end of uextend.i32 (I64)
    // 0002bc: uload16.i32 (I64)
    // --> [RexOp2ld#4b7]
    0x00b4, 0x04b7,
    // --> [Op2ld#4b7]
    0x00b2, 0x04b7,
    // --> [RexOp2ldDisp8#4b7]
    0x00bc, 0x04b7,
    // --> [Op2ldDisp8#4b7]
    0x00ba, 0x04b7,
    // --> [RexOp2ldDisp32#4b7]
    0x00c4, 0x04b7,
    // --> [Op2ldDisp32#4b7] and stop
    0x00c3, 0x04b7,
    // end of uload16.i32 (I64)
    // 0002c8: uload16_complex.i32 (I64)
    // stop unless inst_predicate_2
    0x1002,
    // --> [RexOp2ldWithIndex#4b7]
    0x0052, 0x04b7,
    // --> [Op2ldWithIndex#4b7]
    0x0050, 0x04b7,
    // --> [RexOp2ldWithIndexDisp8#4b7]
    0x005a, 0x04b7,
    // --> [Op2ldWithIndexDisp8#4b7]
    0x0058, 0x04b7,
    // --> [RexOp2ldWithIndexDisp32#4b7]
    0x0062, 0x04b7,
    // --> [Op2ldWithIndexDisp32#4b7] and stop
    0x0061, 0x04b7,
    // end of uload16_complex.i32 (I64)
    // 0002d5: uload8.i32 (I64)
    // --> [RexOp2ld#4b6]
    0x00b4, 0x04b6,
    // --> [Op2ld#4b6]
    0x00b2, 0x04b6,
    // --> [RexOp2ldDisp8#4b6]
    0x00bc, 0x04b6,
    // --> [Op2ldDisp8#4b6]
    0x00ba, 0x04b6,
    // --> [RexOp2ldDisp32#4b6]
    0x00c4, 0x04b6,
    // --> [Op2ldDisp32#4b6] and stop
    0x00c3, 0x04b6,
    // end of uload8.i32 (I64)
    // 0002e1: uload8_complex.i32 (I64)
    // stop unless inst_predicate_2
    0x1002,
    // --> [RexOp2ldWithIndex#4b6]
    0x0052, 0x04b6,
    // --> [Op2ldWithIndex#4b6]
    0x0050, 0x04b6,
    // --> [RexOp2ldWithIndexDisp8#4b6]
    0x005a, 0x04b6,
    // --> [Op2ldWithIndexDisp8#4b6]
    0x0058, 0x04b6,
    // --> [RexOp2ldWithIndexDisp32#4b6]
    0x0062, 0x04b6,
    // --> [Op2ldWithIndexDisp32#4b6] and stop
    0x0061, 0x04b6,
    // end of uload8_complex.i32 (I64)
    // 0002ee: ushr.i32 (I64)
    // --> [RexOp1rc#50d3]
    0x0046, 0x50d3,
    // --> [Op1rc#50d3] and stop
    // 0002f0: ushr.i32 (I32)
    // --> [Op1rc#50d3] and stop
    0x0045, 0x50d3,
    // end of ushr.i32 (I32)
    // end of ushr.i32 (I64)
    // 0002f2: ushr_imm.i32 (I64)
    // --> [RexOp1r_ib#50c1]
    0x002e, 0x50c1,
    // --> [Op1r_ib#50c1] and stop
    // 0002f4: ushr_imm.i32 (I32)
    // --> [Op1r_ib#50c1] and stop
    0x002d, 0x50c1,
    // end of ushr_imm.i32 (I32)
    // end of ushr_imm.i32 (I64)
    // 0002f6: x86_bsf.i32 (I64)
    // --> [RexOp2bsf_and_bsr#4bc]
    0x01ba, 0x04bc,
    // --> [Op2bsf_and_bsr#4bc] and stop
    // 0002f8: x86_bsf.i32 (I32)
    // --> [Op2bsf_and_bsr#4bc] and stop
    0x01b9, 0x04bc,
    // end of x86_bsf.i32 (I32)
    // end of x86_bsf.i32 (I64)
    // 0002fa: x86_bsr.i32 (I64)
    // --> [RexOp2bsf_and_bsr#4bd]
    0x01ba, 0x04bd,
    // --> [Op2bsf_and_bsr#4bd] and stop
    // 0002fc: x86_bsr.i32 (I32)
    // --> [Op2bsf_and_bsr#4bd] and stop
    0x01b9, 0x04bd,
    // end of x86_bsr.i32 (I32)
    // end of x86_bsr.i32 (I64)
    // 0002fe: x86_cvtt2si.i32 (I64)
    // skip 4 unless inst_predicate_15
    0x500f,
    // --> [RexMp2rfurm#62c]
    0x01e4, 0x062c,
    // --> [Mp2rfurm#62c]
    0x01e2, 0x062c,
    // stop unless inst_predicate_16
    0x1010,
    // --> [RexMp2rfurm#72c]
    0x01e4, 0x072c,
    // --> [Mp2rfurm#72c] and stop
    0x01e3, 0x072c,
    // end of x86_cvtt2si.i32 (I64)
    // 000308: x86_sdivmodx.i32 (I64)
    // --> [RexOp1div#70f7]
    0x001e, 0x70f7,
    // --> [Op1div#70f7] and stop
    // 00030a: x86_sdivmodx.i32 (I32)
    // --> [Op1div#70f7] and stop
    0x001d, 0x70f7,
    // end of x86_sdivmodx.i32 (I32)
    // end of x86_sdivmodx.i32 (I64)
    // 00030c: x86_smulx.i32 (I64)
    // --> [RexOp1mulx#50f7]
    0x0022, 0x50f7,
    // --> [Op1mulx#50f7] and stop
    // 00030e: x86_smulx.i32 (I32)
    // --> [Op1mulx#50f7] and stop
    0x0021, 0x50f7,
    // end of x86_smulx.i32 (I32)
    // end of x86_smulx.i32 (I64)
    // 000310: x86_udivmodx.i32 (I64)
    // --> [RexOp1div#60f7]
    0x001e, 0x60f7,
    // --> [Op1div#60f7] and stop
    // 000312: x86_udivmodx.i32 (I32)
    // --> [Op1div#60f7] and stop
    0x001d, 0x60f7,
    // end of x86_udivmodx.i32 (I32)
    // end of x86_udivmodx.i32 (I64)
    // 000314: x86_umulx.i32 (I64)
    // --> [RexOp1mulx#40f7]
    0x0022, 0x40f7,
    // --> [Op1mulx#40f7] and stop
    // 000316: x86_umulx.i32 (I32)
    // --> [Op1mulx#40f7] and stop
    0x0021, 0x40f7,
    // end of x86_umulx.i32 (I32)
    // end of x86_umulx.i32 (I64)
    // 000318: bconst.b32 (I64)
    // --> [RexOp1pu_id_bool#b8]
    // 000318: bconst.b1 (I64)
    // --> [RexOp1pu_id_bool#b8]
    // 000318: bconst.b8 (I64)
    // --> [RexOp1pu_id_bool#b8]
    // 000318: bconst.b16 (I64)
    // --> [RexOp1pu_id_bool#b8]
    0x003e, 0x00b8,
    // --> [Op1pu_id_bool#b8] and stop
    // --> [Op1pu_id_bool#b8] and stop
    // --> [Op1pu_id_bool#b8] and stop
    // --> [Op1pu_id_bool#b8] and stop
    // 00031a: bconst.b32 (I32)
    // --> [Op1pu_id_bool#b8] and stop
    // 00031a: bconst.b1 (I32)
    // --> [Op1pu_id_bool#b8] and stop
    // 00031a: bconst.b8 (I32)
    // --> [Op1pu_id_bool#b8] and stop
    // 00031a: bconst.b16 (I32)
    // --> [Op1pu_id_bool#b8] and stop
    0x003d, 0x00b8,
    // end of bconst.b16 (I32)
    // end of bconst.b8 (I32)
    // end of bconst.b1 (I32)
    // end of bconst.b32 (I32)
    // end of bconst.b16 (I64)
    // end of bconst.b8 (I64)
    // end of bconst.b1 (I64)
    // end of bconst.b32 (I64)
    // 00031c: bconst.b64 (I64)
    // --> [RexOp1pu_id_bool#b8] and stop
    0x003f, 0x00b8,
    // end of bconst.b64 (I64)
    // 00031e: brnz.b1 (I64)
    // --> [RexOp1t8jccb#75]
    0x0176, 0x0075,
    // --> [Op1t8jccb_abcd#75]
    0x0174, 0x0075,
    // --> [RexOp1t8jccd#85]
    0x017a, 0x0085,
    // --> [Op1t8jccd_abcd#85] and stop
    0x0179, 0x0085,
    // end of brnz.b1 (I64)
    // 000326: brz.b1 (I64)
    // --> [RexOp1t8jccb#74]
    0x0176, 0x0074,
    // --> [Op1t8jccb_abcd#74]
    0x0174, 0x0074,
    // --> [RexOp1t8jccd#84]
    0x017a, 0x0084,
    // --> [Op1t8jccd_abcd#84] and stop
    0x0179, 0x0084,
    // end of brz.b1 (I64)
    // 00032e: regmove.b1 (I64)
    // --> [RexOp1rmov#89]
    0x002a, 0x0089,
    // --> [Op1rmov#89] and stop
    // 000330: regmove.i32 (I32)
    // --> [Op1rmov#89] and stop
    // 000330: regmove.b32 (I32)
    // --> [Op1rmov#89] and stop
    // 000330: regmove.b1 (I32)
    // --> [Op1rmov#89] and stop
    // 000330: regmove.r32 (I32)
    // --> [Op1rmov#89] and stop
    // 000330: regmove.i16 (I32)
    // --> [Op1rmov#89] and stop
    // 000330: regmove.b8 (I32)
    // --> [Op1rmov#89] and stop
    // 000330: regmove.b16 (I32)
    // --> [Op1rmov#89] and stop
    0x0029, 0x0089,
    // end of regmove.b16 (I32)
    // end of regmove.b8 (I32)
    // end of regmove.i16 (I32)
    // end of regmove.r32 (I32)
    // end of regmove.b1 (I32)
    // end of regmove.b32 (I32)
    // end of regmove.i32 (I32)
    // end of regmove.b1 (I64)
    // 000332: is_null.r64 (I64)
    // --> [RexOp1is_zero#8085] and stop
    0x023f, 0x8085,
    // end of is_null.r64 (I64)
    // 000334: null.r64 (I64)
    // --> [RexOp1pu_id_ref#b8]
    0x023a, 0x00b8,
    // --> [Op1pu_id_ref#b8] and stop
    // 000336: null.r32 (I32)
    // --> [Op1pu_id_ref#b8] and stop
    0x0239, 0x00b8,
    // end of null.r32 (I32)
    // end of null.r64 (I64)
    // 000338: iconst.i8 (I64)
    // stop unless inst_predicate_1
    0x1001,
    // --> [RexOp1u_id_z#30]
    0x0042, 0x0030,
    // --> [Op1u_id_z#30] and stop
    0x0041, 0x0030,
    // end of iconst.i8 (I64)
    // 00033d: ireduce.i8 (I64)
    // skip 2 unless inst_predicate_9
    0x3009,
    // --> [null#00]
    0x01c0, 0x0000,
    // skip 2 unless inst_predicate_10
    // 000340: ireduce.i16 (I64)
    // skip 2 unless inst_predicate_10
    0x300a,
    // --> [null#00]
    // --> [null#00]
    0x01c0, 0x0000,
    // stop unless inst_predicate_11
    // stop unless inst_predicate_11
    0x100b,
    // --> [null#00] and stop
    // --> [null#00] and stop
    0x01c1, 0x0000,
    // end of ireduce.i16 (I64)
    // end of ireduce.i8 (I64)
    // 000346: regmove.i8 (I64)
    // --> [RexOp1rmov#89]
    0x002a, 0x0089,
    // --> [RexOp1rmov#89]
    0x002a, 0x0089,
    // --> [Op1rmov#89] and stop
    0x0029, 0x0089,
    // end of regmove.i8 (I64)
    // 00034c: adjust_sp_down_imm (I64)
    // --> [RexOp1adjustsp_ib#d083]
    0x00f0, 0xd083,
    // --> [RexOp1adjustsp_id#d081] and stop
    0x00f3, 0xd081,
    // end of adjust_sp_down_imm (I64)
    // 000350: adjust_sp_up_imm (I64)
    // --> [RexOp1adjustsp_ib#8083]
    0x00f0, 0x8083,
    // --> [RexOp1adjustsp_id#8081] and stop
    0x00f3, 0x8081,
    // end of adjust_sp_up_imm (I64)
    // 000354: brff (I64)
    // --> [RexOp1brfb#70]
    0x0164, 0x0070,
    // --> [Op1brfb#70]
    0x0162, 0x0070,
    // --> [RexOp2brfd#480]
    0x0168, 0x0480,
    // --> [Op2brfd#480] and stop
    0x0167, 0x0480,
    // end of brff (I64)
    // 00035c: brif (I64)
    // --> [RexOp1brib#70]
    0x015c, 0x0070,
    // --> [Op1brib#70]
    0x015a, 0x0070,
    // --> [RexOp2brid#480]
    0x0160, 0x0480,
    // --> [Op2brid#480] and stop
    0x015f, 0x0480,
    // end of brif (I64)
    // 000364: call (I64)
    // skip 2 unless inst_predicate_6
    0x3006,
    // --> [Op1call_id#e8]
    0x014c, 0x00e8,
    // stop unless PredicateView(10)
    0x1029,
    // --> [Op1call_plt_id#e8] and stop
    0x014f, 0x00e8,
    // end of call (I64)
    // 00036a: copy_special (I64)
    // --> [RexOp1copysp#8089] and stop
    0x00db, 0x8089,
    // end of copy_special (I64)
    // 00036c: debugtrap (I64)
    // --> [debugtrap#00] and stop
    // 00036c: debugtrap (I32)
    // --> [debugtrap#00] and stop
    0x018b, 0x0000,
    // end of debugtrap (I32)
    // end of debugtrap (I64)
    // 00036e: f32const (I64)
    // stop unless inst_predicate_13
    0x100d,
    // --> [RexOp2f32imm_z#457]
    0x01ca, 0x0457,
    // --> [Op2f32imm_z#457] and stop
    0x01c7, 0x0457,
    // end of f32const (I64)
    // 000373: f64const (I64)
    // stop unless inst_predicate_14
    0x100e,
    // --> [RexMp2f64imm_z#557]
    0x01cc, 0x0557,
    // --> [Mp2f64imm_z#557] and stop
    0x01c9, 0x0557,
    // end of f64const (I64)
    // 000378: jump (I64)
    // --> [Op1jmpb#eb]
    // 000378: jump (I32)
    // --> [Op1jmpb#eb]
    0x0156, 0x00eb,
    // --> [Op1jmpd#e9] and stop
    // --> [Op1jmpd#e9] and stop
    0x0159, 0x00e9,
    // end of jump (I32)
    // end of jump (I64)
    // 00037c: resumable_trap (I64)
    // --> [Op2trap#40b] and stop
    // 00037c: trap (I64)
    // --> [Op2trap#40b] and stop
    // 00037c: resumable_trap (I32)
    // --> [Op2trap#40b] and stop
    // 00037c: trap (I32)
    // --> [Op2trap#40b] and stop
    0x0189, 0x040b,
    // end of trap (I32)
    // end of resumable_trap (I32)
    // end of trap (I64)
    // end of resumable_trap (I64)
    // 00037e: return (I64)
    // --> [Op1ret#c3] and stop
    // 00037e: return (I32)
    // --> [Op1ret#c3] and stop
    0x0155, 0x00c3,
    // end of return (I32)
    // end of return (I64)
    // 000380: safepoint (I64)
    // --> [safepoint#00] and stop
    // 000380: safepoint (I32)
    // --> [safepoint#00] and stop
    0x0241, 0x0000,
    // end of safepoint (I32)
    // end of safepoint (I64)
    // 000382: sload32_complex (I64)
    // stop unless inst_predicate_2
    0x1002,
    // --> [RexOp1ldWithIndex#8063]
    0x004e, 0x8063,
    // --> [RexOp1ldWithIndexDisp8#8063]
    0x0056, 0x8063,
    // --> [RexOp1ldWithIndexDisp32#8063] and stop
    0x005f, 0x8063,
    // end of sload32_complex (I64)
    // 000389: trapff (I64)
    // --> [trapff#00] and stop
    // 000389: trapff (I32)
    // --> [trapff#00] and stop
    0x018f, 0x0000,
    // end of trapff (I32)
    // end of trapff (I64)
    // 00038b: trapif (I64)
    // --> [trapif#00] and stop
    // 00038b: trapif (I32)
    // --> [trapif#00] and stop
    0x018d, 0x0000,
    // end of trapif (I32)
    // end of trapif (I64)
    // 00038d: trueff (I64)
    // --> [RexOp2setf#490]
    0x01b2, 0x0490,
    // --> [Op2setf_abcd#490] and stop
    // 00038f: trueff (I32)
    // --> [Op2setf_abcd#490] and stop
    0x01b1, 0x0490,
    // end of trueff (I32)
    // end of trueff (I64)
    // 000391: trueif (I64)
    // --> [RexOp2seti#490]
    0x01ae, 0x0490,
    // --> [Op2seti_abcd#490] and stop
    // 000393: trueif (I32)
    // --> [Op2seti_abcd#490] and stop
    0x01ad, 0x0490,
    // end of trueif (I32)
    // end of trueif (I64)
    // 000395: band.f64 (I64)
    // --> [RexOp2fa#454]
    // 000395: band.f32 (I64)
    // --> [RexOp2fa#454]
    0x01f0, 0x0454,
    // --> [Op2fa#454] and stop
    // --> [Op2fa#454] and stop
    // 000397: band.f64 (I32)
    // --> [Op2fa#454] and stop
    // 000397: band.f32 (I32)
    // --> [Op2fa#454] and stop
    0x01ef, 0x0454,
    // end of band.f32 (I32)
    // end of band.f64 (I32)
    // end of band.f32 (I64)
    // end of band.f64 (I64)
    // 000399: band_not.f64 (I64)
    // --> [RexOp2fax#455]
    // 000399: band_not.f32 (I64)
    // --> [RexOp2fax#455]
    0x01f4, 0x0455,
    // --> [Op2fax#455] and stop
    // --> [Op2fax#455] and stop
    // 00039b: band_not.f64 (I32)
    // --> [Op2fax#455] and stop
    // 00039b: band_not.f32 (I32)
    // --> [Op2fax#455] and stop
    0x01f3, 0x0455,
    // end of band_not.f32 (I32)
    // end of band_not.f64 (I32)
    // end of band_not.f32 (I64)
    // end of band_not.f64 (I64)
    // 00039d: bitcast.f64 (I64)
    // stop unless inst_predicate_11
    0x100b,
    // --> [RexMp2frurm#856e] and stop
    0x01d1, 0x856e,
    // end of bitcast.f64 (I64)
    // 0003a0: bor.f64 (I64)
    // --> [RexOp2fa#456]
    // 0003a0: bor.f32 (I64)
    // --> [RexOp2fa#456]
    0x01f0, 0x0456,
    // --> [Op2fa#456] and stop
    // --> [Op2fa#456] and stop
    // 0003a2: bor.f64 (I32)
    // --> [Op2fa#456] and stop
    // 0003a2: bor.f32 (I32)
    // --> [Op2fa#456] and stop
    0x01ef, 0x0456,
    // end of bor.f32 (I32)
    // end of bor.f64 (I32)
    // end of bor.f32 (I64)
    // end of bor.f64 (I64)
    // 0003a4: bxor.f64 (I64)
    // --> [RexOp2fa#457]
    // 0003a4: bxor.f32 (I64)
    // --> [RexOp2fa#457]
    0x01f0, 0x0457,
    // --> [Op2fa#457] and stop
    // --> [Op2fa#457] and stop
    // 0003a6: bxor.f64 (I32)
    // --> [Op2fa#457] and stop
    // 0003a6: bxor.f32 (I32)
    // --> [Op2fa#457] and stop
    0x01ef, 0x0457,
    // end of bxor.f32 (I32)
    // end of bxor.f64 (I32)
    // end of bxor.f32 (I64)
    // end of bxor.f64 (I64)
    // 0003a8: ceil.f64 (I64)
    // stop unless PredicateView(16)
    // 0003a8: floor.f64 (I64)
    // stop unless PredicateView(16)
    // 0003a8: nearest.f64 (I64)
    // stop unless PredicateView(16)
    // 0003a8: trunc.f64 (I64)
    // stop unless PredicateView(16)
    0x102f,
    // --> [RexMp3furmi_rnd#d0b]
    // --> [RexMp3furmi_rnd#d0b]
    // --> [RexMp3furmi_rnd#d0b]
    // --> [RexMp3furmi_rnd#d0b]
    0x01e8, 0x0d0b,
    // --> [Mp3furmi_rnd#d0b] and stop
    // --> [Mp3furmi_rnd#d0b] and stop
    // --> [Mp3furmi_rnd#d0b] and stop
    // --> [Mp3furmi_rnd#d0b] and stop
    0x01e7, 0x0d0b,
    // end of trunc.f64 (I64)
    // end of nearest.f64 (I64)
    // end of floor.f64 (I64)
    // end of ceil.f64 (I64)
    // 0003ad: copy.f64 (I64)
    // --> [RexOp2furm#428]
    // 0003ad: copy.f32 (I64)
    // --> [RexOp2furm#428]
    0x01d8, 0x0428,
    // --> [Op2furm#428] and stop
    // --> [Op2furm#428] and stop
    // 0003af: copy.b8x16 (I64)
    // --> [Op2furm#428] and stop
    // 0003af: copy.b16x8 (I64)
    // --> [Op2furm#428] and stop
    // 0003af: copy.b32x4 (I64)
    // --> [Op2furm#428] and stop
    // 0003af: copy.b64x2 (I64)
    // --> [Op2furm#428] and stop
    // 0003af: copy.i8x16 (I64)
    // --> [Op2furm#428] and stop
    // 0003af: copy.i16x8 (I64)
    // --> [Op2furm#428] and stop
    // 0003af: copy.i32x4 (I64)
    // --> [Op2furm#428] and stop
    // 0003af: copy.i64x2 (I64)
    // --> [Op2furm#428] and stop
    // 0003af: copy.f32x4 (I64)
    // --> [Op2furm#428] and stop
    // 0003af: copy.f64x2 (I64)
    // --> [Op2furm#428] and stop
    // 0003af: copy.f64 (I32)
    // --> [Op2furm#428] and stop
    // 0003af: copy.f32 (I32)
    // --> [Op2furm#428] and stop
    // 0003af: copy.b8x16 (I32)
    // --> [Op2furm#428] and stop
    // 0003af: copy.b16x8 (I32)
    // --> [Op2furm#428] and stop
    // 0003af: copy.b32x4 (I32)
    // --> [Op2furm#428] and stop
    // 0003af: copy.b64x2 (I32)
    // --> [Op2furm#428] and stop
    // 0003af: copy.i8x16 (I32)
    // --> [Op2furm#428] and stop
    // 0003af: copy.i16x8 (I32)
    // --> [Op2furm#428] and stop
    // 0003af: copy.i32x4 (I32)
    // --> [Op2furm#428] and stop
    // 0003af: copy.i64x2 (I32)
    // --> [Op2furm#428] and stop
    // 0003af: copy.f32x4 (I32)
    // --> [Op2furm#428] and stop
    // 0003af: copy.f64x2 (I32)
    // --> [Op2furm#428] and stop
    0x01d7, 0x0428,
    // end of copy.f64x2 (I32)
    // end of copy.f32x4 (I32)
    // end of copy.i64x2 (I32)
    // end of copy.i32x4 (I32)
    // end of copy.i16x8 (I32)
    // end of copy.i8x16 (I32)
    // end of copy.b64x2 (I32)
    // end of copy.b32x4 (I32)
    // end of copy.b16x8 (I32)
    // end of copy.b8x16 (I32)
    // end of copy.f32 (I32)
    // end of copy.f64 (I32)
    // end of copy.f64x2 (I64)
    // end of copy.f32x4 (I64)
    // end of copy.i64x2 (I64)
    // end of copy.i32x4 (I64)
    // end of copy.i16x8 (I64)
    // end of copy.i8x16 (I64)
    // end of copy.b64x2 (I64)
    // end of copy.b32x4 (I64)
    // end of copy.b16x8 (I64)
    // end of copy.b8x16 (I64)
    // end of copy.f32 (I64)
    // end of copy.f64 (I64)
    // 0003b1: copy_to_ssa.f64 (I64)
    // --> [RexMp2furm_reg_to_ssa#710] and stop
    0x00e5, 0x0710,
    // end of copy_to_ssa.f64 (I64)
    // 0003b3: fadd.f64 (I64)
    // --> [RexMp2fa#758]
    0x01ec, 0x0758,
    // --> [Mp2fa#758] and stop
    // 0003b5: fadd.f64 (I32)
    // --> [Mp2fa#758] and stop
    0x01eb, 0x0758,
    // end of fadd.f64 (I32)
    // end of fadd.f64 (I64)
    // 0003b7: fcmp.f64 (I64)
    // --> [RexMp2fcscc#52e]
    0x01fc, 0x052e,
    // --> [Mp2fcscc#52e] and stop
    // 0003b9: fcmp.f64 (I32)
    // --> [Mp2fcscc#52e] and stop
    0x01fb, 0x052e,
    // end of fcmp.f64 (I32)
    // end of fcmp.f64 (I64)
    // 0003bb: fcvt_from_sint.f64 (I64)
    // skip 4 unless inst_predicate_10
    0x500a,
    // --> [RexMp2frurm#72a]
    0x01d0, 0x072a,
    // --> [Mp2frurm#72a]
    0x01ce, 0x072a,
    // stop unless inst_predicate_11
    0x100b,
    // --> [RexMp2frurm#872a] and stop
    0x01d1, 0x872a,
    // end of fcvt_from_sint.f64 (I64)
    // 0003c3: fdiv.f64 (I64)
    // --> [RexMp2fa#75e]
    0x01ec, 0x075e,
    // --> [Mp2fa#75e] and stop
    // 0003c5: fdiv.f64 (I32)
    // --> [Mp2fa#75e] and stop
    0x01eb, 0x075e,
    // end of fdiv.f64 (I32)
    // end of fdiv.f64 (I64)
    // 0003c7: ffcmp.f64 (I64)
    // --> [RexMp2fcmp#52e]
    0x0204, 0x052e,
    // --> [Mp2fcmp#52e] and stop
    // 0003c9: ffcmp.f64 (I32)
    // --> [Mp2fcmp#52e] and stop
    0x0203, 0x052e,
    // end of ffcmp.f64 (I32)
    // end of ffcmp.f64 (I64)
    // 0003cb: fill.f64 (I64)
    // --> [RexMp2ffillSib32#710]
    0x0126, 0x0710,
    // --> [Mp2ffillSib32#710] and stop
    // 0003cd: fill.f64 (I32)
    // --> [Mp2ffillSib32#710] and stop
    0x0125, 0x0710,
    // end of fill.f64 (I32)
    // end of fill.f64 (I64)
    // 0003cf: fill_nop.f64 (I64)
    // --> [ffillnull#00] and stop
    // 0003cf: fill_nop.f32 (I64)
    // --> [ffillnull#00] and stop
    // 0003cf: fill_nop.b8x16 (I64)
    // --> [ffillnull#00] and stop
    // 0003cf: fill_nop.b16x8 (I64)
    // --> [ffillnull#00] and stop
    // 0003cf: fill_nop.b32x4 (I64)
    // --> [ffillnull#00] and stop
    // 0003cf: fill_nop.b64x2 (I64)
    // --> [ffillnull#00] and stop
    // 0003cf: fill_nop.i8x16 (I64)
    // --> [ffillnull#00] and stop
    // 0003cf: fill_nop.i16x8 (I64)
    // --> [ffillnull#00] and stop
    // 0003cf: fill_nop.i32x4 (I64)
    // --> [ffillnull#00] and stop
    // 0003cf: fill_nop.i64x2 (I64)
    // --> [ffillnull#00] and stop
    // 0003cf: fill_nop.f32x4 (I64)
    // --> [ffillnull#00] and stop
    // 0003cf: fill_nop.f64x2 (I64)
    // --> [ffillnull#00] and stop
    // 0003cf: fill_nop.f64 (I32)
    // --> [ffillnull#00] and stop
    // 0003cf: fill_nop.f32 (I32)
    // --> [ffillnull#00] and stop
    // 0003cf: fill_nop.b8x16 (I32)
    // --> [ffillnull#00] and stop
    // 0003cf: fill_nop.b16x8 (I32)
    // --> [ffillnull#00] and stop
    // 0003cf: fill_nop.b32x4 (I32)
    // --> [ffillnull#00] and stop
    // 0003cf: fill_nop.b64x2 (I32)
    // --> [ffillnull#00] and stop
    // 0003cf: fill_nop.i8x16 (I32)
    // --> [ffillnull#00] and stop
    // 0003cf: fill_nop.i16x8 (I32)
    // --> [ffillnull#00] and stop
    // 0003cf: fill_nop.i32x4 (I32)
    // --> [ffillnull#00] and stop
    // 0003cf: fill_nop.i64x2 (I32)
    // --> [ffillnull#00] and stop
    // 0003cf: fill_nop.f32x4 (I32)
    // --> [ffillnull#00] and stop
    // 0003cf: fill_nop.f64x2 (I32)
    // --> [ffillnull#00] and stop
    0x00d1, 0x0000,
    // end of fill_nop.f64x2 (I32)
    // end of fill_nop.f32x4 (I32)
    // end of fill_nop.i64x2 (I32)
    // end of fill_nop.i32x4 (I32)
    // end of fill_nop.i16x8 (I32)
    // end of fill_nop.i8x16 (I32)
    // end of fill_nop.b64x2 (I32)
    // end of fill_nop.b32x4 (I32)
    // end of fill_nop.b16x8 (I32)
    // end of fill_nop.b8x16 (I32)
    // end of fill_nop.f32 (I32)
    // end of fill_nop.f64 (I32)
    // end of fill_nop.f64x2 (I64)
    // end of fill_nop.f32x4 (I64)
    // end of fill_nop.i64x2 (I64)
    // end of fill_nop.i32x4 (I64)
    // end of fill_nop.i16x8 (I64)
    // end of fill_nop.i8x16 (I64)
    // end of fill_nop.b64x2 (I64)
    // end of fill_nop.b32x4 (I64)
    // end of fill_nop.b16x8 (I64)
    // end of fill_nop.b8x16 (I64)
    // end of fill_nop.f32 (I64)
    // end of fill_nop.f64 (I64)
    // 0003d1: fmul.f64 (I64)
    // --> [RexMp2fa#759]
    0x01ec, 0x0759,
    // --> [Mp2fa#759] and stop
    // 0003d3: fmul.f64 (I32)
    // --> [Mp2fa#759] and stop
    0x01eb, 0x0759,
    // end of fmul.f64 (I32)
    // end of fmul.f64 (I64)
    // 0003d5: fpromote.f64 (I64)
    // stop unless inst_predicate_15
    0x100f,
    // --> [RexMp2furm#65a]
    0x01e0, 0x065a,
    // --> [Mp2furm#65a] and stop
    0x01df, 0x065a,
    // end of fpromote.f64 (I64)
    // 0003da: fsub.f64 (I64)
    // --> [RexMp2fa#75c]
    0x01ec, 0x075c,
    // --> [Mp2fa#75c] and stop
    // 0003dc: fsub.f64 (I32)
    // --> [Mp2fa#75c] and stop
    0x01eb, 0x075c,
    // end of fsub.f64 (I32)
    // end of fsub.f64 (I64)
    // 0003de: load.f64 (I64)
    // --> [RexMp2fld#710]
    0x00f6, 0x0710,
    // --> [Mp2fld#710]
    0x00f4, 0x0710,
    // --> [RexMp2fldDisp8#710]
    0x00fa, 0x0710,
    // --> [Mp2fldDisp8#710]
    0x00f8, 0x0710,
    // --> [RexMp2fldDisp32#710]
    0x00fe, 0x0710,
    // --> [Mp2fldDisp32#710] and stop
    0x00fd, 0x0710,
    // end of load.f64 (I64)
    // 0003ea: load_complex.f64 (I64)
    // --> [RexMp2fldWithIndex#710]
    0x0102, 0x0710,
    // --> [Mp2fldWithIndex#710]
    0x0100, 0x0710,
    // --> [RexMp2fldWithIndexDisp8#710]
    0x0106, 0x0710,
    // --> [Mp2fldWithIndexDisp8#710]
    0x0104, 0x0710,
    // --> [RexMp2fldWithIndexDisp32#710]
    0x010a, 0x0710,
    // --> [Mp2fldWithIndexDisp32#710] and stop
    0x0109, 0x0710,
    // end of load_complex.f64 (I64)
    // 0003f6: raw_bitcast.f64 (I64)
    // skip 2 unless inst_predicate_17
    // 0003f6: raw_bitcast.f32 (I64)
    // skip 2 unless inst_predicate_17
    // 0003f6: raw_bitcast.f64 (I32)
    // skip 2 unless inst_predicate_17
    // 0003f6: raw_bitcast.f32 (I32)
    // skip 2 unless inst_predicate_17
    0x3011,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_18
    // skip 2 unless inst_predicate_18
    // skip 2 unless inst_predicate_18
    // skip 2 unless inst_predicate_18
    0x3012,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_19
    // skip 2 unless inst_predicate_19
    // skip 2 unless inst_predicate_19
    // skip 2 unless inst_predicate_19
    0x3013,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_20
    // skip 2 unless inst_predicate_20
    // skip 2 unless inst_predicate_20
    // skip 2 unless inst_predicate_20
    0x3014,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_21
    // skip 2 unless inst_predicate_21
    // skip 2 unless inst_predicate_21
    // skip 2 unless inst_predicate_21
    0x3015,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_22
    // skip 2 unless inst_predicate_22
    // skip 2 unless inst_predicate_22
    // skip 2 unless inst_predicate_22
    0x3016,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_23
    // skip 2 unless inst_predicate_23
    // skip 2 unless inst_predicate_23
    // skip 2 unless inst_predicate_23
    0x3017,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_24
    // skip 2 unless inst_predicate_24
    // skip 2 unless inst_predicate_24
    // skip 2 unless inst_predicate_24
    0x3018,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_25
    // skip 2 unless inst_predicate_25
    // skip 2 unless inst_predicate_25
    // skip 2 unless inst_predicate_25
    0x3019,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // stop unless inst_predicate_26
    // stop unless inst_predicate_26
    // stop unless inst_predicate_26
    // stop unless inst_predicate_26
    0x101a,
    // --> [null_fpr#00] and stop
    // --> [null_fpr#00] and stop
    // 000412: scalar_to_vector.f32x4 (I64)
    // --> [null_fpr#00] and stop
    // 000412: scalar_to_vector.f64x2 (I64)
    // --> [null_fpr#00] and stop
    // --> [null_fpr#00] and stop
    // --> [null_fpr#00] and stop
    // 000412: scalar_to_vector.f32x4 (I32)
    // --> [null_fpr#00] and stop
    // 000412: scalar_to_vector.f64x2 (I32)
    // --> [null_fpr#00] and stop
    0x020b, 0x0000,
    // end of scalar_to_vector.f64x2 (I32)
    // end of scalar_to_vector.f32x4 (I32)
    // end of raw_bitcast.f32 (I32)
    // end of raw_bitcast.f64 (I32)
    // end of scalar_to_vector.f64x2 (I64)
    // end of scalar_to_vector.f32x4 (I64)
    // end of raw_bitcast.f32 (I64)
    // end of raw_bitcast.f64 (I64)
    // 000414: regfill.f64 (I64)
    // --> [RexMp2fregfill32#710]
    0x012a, 0x0710,
    // --> [Mp2fregfill32#710] and stop
    // 000416: regfill.f64 (I32)
    // --> [Mp2fregfill32#710] and stop
    0x0129, 0x0710,
    // end of regfill.f64 (I32)
    // end of regfill.f64 (I64)
    // 000418: regmove.f64 (I64)
    // --> [RexOp2frmov#428] and stop
    // 000418: regmove.f32 (I64)
    // --> [RexOp2frmov#428] and stop
    0x01dd, 0x0428,
    // end of regmove.f32 (I64)
    // end of regmove.f64 (I64)
    // 00041a: regspill.f64 (I64)
    // --> [RexMp2fregspill32#711]
    0x0132, 0x0711,
    // --> [Mp2fregspill32#711] and stop
    // 00041c: regspill.f64 (I32)
    // --> [Mp2fregspill32#711] and stop
    0x0131, 0x0711,
    // end of regspill.f64 (I32)
    // end of regspill.f64 (I64)
    // 00041e: spill.f64 (I64)
    // --> [RexMp2fspillSib32#711]
    0x012e, 0x0711,
    // --> [Mp2fspillSib32#711] and stop
    // 000420: spill.f64 (I32)
    // --> [Mp2fspillSib32#711] and stop
    0x012d, 0x0711,
    // end of spill.f64 (I32)
    // end of spill.f64 (I64)
    // 000422: sqrt.f64 (I64)
    // --> [RexMp2furm#751]
    0x01e0, 0x0751,
    // --> [Mp2furm#751] and stop
    // 000424: sqrt.f64 (I32)
    // --> [Mp2furm#751] and stop
    0x01df, 0x0751,
    // end of sqrt.f64 (I32)
    // end of sqrt.f64 (I64)
    // 000426: store.f64 (I64)
    // --> [RexMp2fst#711]
    0x010e, 0x0711,
    // --> [Mp2fst#711]
    0x010c, 0x0711,
    // --> [RexMp2fstDisp8#711]
    0x0112, 0x0711,
    // --> [Mp2fstDisp8#711]
    0x0110, 0x0711,
    // --> [RexMp2fstDisp32#711]
    0x0116, 0x0711,
    // --> [Mp2fstDisp32#711] and stop
    0x0115, 0x0711,
    // end of store.f64 (I64)
    // 000432: store_complex.f64 (I64)
    // --> [RexMp2fstWithIndex#711]
    0x011a, 0x0711,
    // --> [Mp2fstWithIndex#711]
    0x0118, 0x0711,
    // --> [RexMp2fstWithIndexDisp8#711]
    0x011e, 0x0711,
    // --> [Mp2fstWithIndexDisp8#711]
    0x011c, 0x0711,
    // --> [RexMp2fstWithIndexDisp32#711]
    0x0122, 0x0711,
    // --> [Mp2fstWithIndexDisp32#711] and stop
    0x0121, 0x0711,
    // end of store_complex.f64 (I64)
    // 00043e: x86_fmax.f64 (I64)
    // --> [RexMp2fa#75f]
    0x01ec, 0x075f,
    // --> [Mp2fa#75f] and stop
    // 000440: x86_fmax.f64 (I32)
    // --> [Mp2fa#75f] and stop
    0x01eb, 0x075f,
    // end of x86_fmax.f64 (I32)
    // end of x86_fmax.f64 (I64)
    // 000442: x86_fmin.f64 (I64)
    // --> [RexMp2fa#75d]
    0x01ec, 0x075d,
    // --> [Mp2fa#75d] and stop
    // 000444: x86_fmin.f64 (I32)
    // --> [Mp2fa#75d] and stop
    0x01eb, 0x075d,
    // end of x86_fmin.f64 (I32)
    // end of x86_fmin.f64 (I64)
    // 000446: bitcast.f32 (I64)
    // stop unless inst_predicate_10
    0x100a,
    // --> [RexMp2frurm#56e]
    // 000447: scalar_to_vector.b8x16 (I64)
    // --> [RexMp2frurm#56e]
    // 000447: scalar_to_vector.b16x8 (I64)
    // --> [RexMp2frurm#56e]
    // 000447: scalar_to_vector.b32x4 (I64)
    // --> [RexMp2frurm#56e]
    // 000447: scalar_to_vector.b64x2 (I64)
    // --> [RexMp2frurm#56e]
    // 000447: scalar_to_vector.i8x16 (I64)
    // --> [RexMp2frurm#56e]
    // 000447: scalar_to_vector.i16x8 (I64)
    // --> [RexMp2frurm#56e]
    // 000447: scalar_to_vector.i32x4 (I64)
    // --> [RexMp2frurm#56e]
    // 000447: scalar_to_vector.i64x2 (I64)
    // --> [RexMp2frurm#56e]
    0x01d0, 0x056e,
    // --> [Mp2frurm#56e] and stop
    // --> [Mp2frurm#56e] and stop
    // --> [Mp2frurm#56e] and stop
    // --> [Mp2frurm#56e] and stop
    // --> [Mp2frurm#56e] and stop
    // --> [Mp2frurm#56e] and stop
    // --> [Mp2frurm#56e] and stop
    // --> [Mp2frurm#56e] and stop
    // --> [Mp2frurm#56e] and stop
    // 000449: scalar_to_vector.b8x16 (I32)
    // --> [Mp2frurm#56e] and stop
    // 000449: scalar_to_vector.b16x8 (I32)
    // --> [Mp2frurm#56e] and stop
    // 000449: scalar_to_vector.b32x4 (I32)
    // --> [Mp2frurm#56e] and stop
    // 000449: scalar_to_vector.i8x16 (I32)
    // --> [Mp2frurm#56e] and stop
    // 000449: scalar_to_vector.i16x8 (I32)
    // --> [Mp2frurm#56e] and stop
    // 000449: scalar_to_vector.i32x4 (I32)
    // --> [Mp2frurm#56e] and stop
    0x01cf, 0x056e,
    // end of scalar_to_vector.i32x4 (I32)
    // end of scalar_to_vector.i16x8 (I32)
    // end of scalar_to_vector.i8x16 (I32)
    // end of scalar_to_vector.b32x4 (I32)
    // end of scalar_to_vector.b16x8 (I32)
    // end of scalar_to_vector.b8x16 (I32)
    // end of scalar_to_vector.i64x2 (I64)
    // end of scalar_to_vector.i32x4 (I64)
    // end of scalar_to_vector.i16x8 (I64)
    // end of scalar_to_vector.i8x16 (I64)
    // end of scalar_to_vector.b64x2 (I64)
    // end of scalar_to_vector.b32x4 (I64)
    // end of scalar_to_vector.b16x8 (I64)
    // end of scalar_to_vector.b8x16 (I64)
    // end of bitcast.f32 (I64)
    // 00044b: ceil.f32 (I64)
    // stop unless PredicateView(16)
    // 00044b: floor.f32 (I64)
    // stop unless PredicateView(16)
    // 00044b: nearest.f32 (I64)
    // stop unless PredicateView(16)
    // 00044b: trunc.f32 (I64)
    // stop unless PredicateView(16)
    0x102f,
    // --> [RexMp3furmi_rnd#d0a]
    // --> [RexMp3furmi_rnd#d0a]
    // --> [RexMp3furmi_rnd#d0a]
    // --> [RexMp3furmi_rnd#d0a]
    0x01e8, 0x0d0a,
    // --> [Mp3furmi_rnd#d0a] and stop
    // --> [Mp3furmi_rnd#d0a] and stop
    // --> [Mp3furmi_rnd#d0a] and stop
    // --> [Mp3furmi_rnd#d0a] and stop
    0x01e7, 0x0d0a,
    // end of trunc.f32 (I64)
    // end of nearest.f32 (I64)
    // end of floor.f32 (I64)
    // end of ceil.f32 (I64)
    // 000450: copy_to_ssa.f32 (I64)
    // --> [RexMp2furm_reg_to_ssa#610] and stop
    0x00e5, 0x0610,
    // end of copy_to_ssa.f32 (I64)
    // 000452: fadd.f32 (I64)
    // --> [RexMp2fa#658]
    0x01ec, 0x0658,
    // --> [Mp2fa#658] and stop
    // 000454: fadd.f32 (I32)
    // --> [Mp2fa#658] and stop
    0x01eb, 0x0658,
    // end of fadd.f32 (I32)
    // end of fadd.f32 (I64)
    // 000456: fcmp.f32 (I64)
    // --> [RexOp2fcscc#42e]
    0x01f8, 0x042e,
    // --> [Op2fcscc#42e] and stop
    // 000458: fcmp.f32 (I32)
    // --> [Op2fcscc#42e] and stop
    0x01f7, 0x042e,
    // end of fcmp.f32 (I32)
    // end of fcmp.f32 (I64)
    // 00045a: fcvt_from_sint.f32 (I64)
    // skip 4 unless inst_predicate_10
    0x500a,
    // --> [RexMp2frurm#62a]
    0x01d0, 0x062a,
    // --> [Mp2frurm#62a]
    0x01ce, 0x062a,
    // stop unless inst_predicate_11
    0x100b,
    // --> [RexMp2frurm#862a] and stop
    0x01d1, 0x862a,
    // end of fcvt_from_sint.f32 (I64)
    // 000462: fdemote.f32 (I64)
    // stop unless inst_predicate_16
    0x1010,
    // --> [RexMp2furm#75a]
    0x01e0, 0x075a,
    // --> [Mp2furm#75a] and stop
    0x01df, 0x075a,
    // end of fdemote.f32 (I64)
    // 000467: fdiv.f32 (I64)
    // --> [RexMp2fa#65e]
    0x01ec, 0x065e,
    // --> [Mp2fa#65e] and stop
    // 000469: fdiv.f32 (I32)
    // --> [Mp2fa#65e] and stop
    0x01eb, 0x065e,
    // end of fdiv.f32 (I32)
    // end of fdiv.f32 (I64)
    // 00046b: ffcmp.f32 (I64)
    // --> [RexOp2fcmp#42e]
    0x0200, 0x042e,
    // --> [Op2fcmp#42e] and stop
    // 00046d: ffcmp.f32 (I32)
    // --> [Op2fcmp#42e] and stop
    0x01ff, 0x042e,
    // end of ffcmp.f32 (I32)
    // end of ffcmp.f32 (I64)
    // 00046f: fill.f32 (I64)
    // --> [RexMp2ffillSib32#610]
    0x0126, 0x0610,
    // --> [Mp2ffillSib32#610] and stop
    // 000471: fill.f32 (I32)
    // --> [Mp2ffillSib32#610] and stop
    0x0125, 0x0610,
    // end of fill.f32 (I32)
    // end of fill.f32 (I64)
    // 000473: fmul.f32 (I64)
    // --> [RexMp2fa#659]
    0x01ec, 0x0659,
    // --> [Mp2fa#659] and stop
    // 000475: fmul.f32 (I32)
    // --> [Mp2fa#659] and stop
    0x01eb, 0x0659,
    // end of fmul.f32 (I32)
    // end of fmul.f32 (I64)
    // 000477: fsub.f32 (I64)
    // --> [RexMp2fa#65c]
    0x01ec, 0x065c,
    // --> [Mp2fa#65c] and stop
    // 000479: fsub.f32 (I32)
    // --> [Mp2fa#65c] and stop
    0x01eb, 0x065c,
    // end of fsub.f32 (I32)
    // end of fsub.f32 (I64)
    // 00047b: load.f32 (I64)
    // --> [RexMp2fld#610]
    0x00f6, 0x0610,
    // --> [Mp2fld#610]
    0x00f4, 0x0610,
    // --> [RexMp2fldDisp8#610]
    0x00fa, 0x0610,
    // --> [Mp2fldDisp8#610]
    0x00f8, 0x0610,
    // --> [RexMp2fldDisp32#610]
    0x00fe, 0x0610,
    // --> [Mp2fldDisp32#610] and stop
    0x00fd, 0x0610,
    // end of load.f32 (I64)
    // 000487: load_complex.f32 (I64)
    // --> [RexMp2fldWithIndex#610]
    0x0102, 0x0610,
    // --> [Mp2fldWithIndex#610]
    0x0100, 0x0610,
    // --> [RexMp2fldWithIndexDisp8#610]
    0x0106, 0x0610,
    // --> [Mp2fldWithIndexDisp8#610]
    0x0104, 0x0610,
    // --> [RexMp2fldWithIndexDisp32#610]
    0x010a, 0x0610,
    // --> [Mp2fldWithIndexDisp32#610] and stop
    0x0109, 0x0610,
    // end of load_complex.f32 (I64)
    // 000493: regfill.f32 (I64)
    // --> [RexMp2fregfill32#610]
    0x012a, 0x0610,
    // --> [Mp2fregfill32#610] and stop
    // 000495: regfill.f32 (I32)
    // --> [Mp2fregfill32#610] and stop
    0x0129, 0x0610,
    // end of regfill.f32 (I32)
    // end of regfill.f32 (I64)
    // 000497: regspill.f32 (I64)
    // --> [RexMp2fregspill32#611]
    0x0132, 0x0611,
    // --> [Mp2fregspill32#611] and stop
    // 000499: regspill.f32 (I32)
    // --> [Mp2fregspill32#611] and stop
    0x0131, 0x0611,
    // end of regspill.f32 (I32)
    // end of regspill.f32 (I64)
    // 00049b: spill.f32 (I64)
    // --> [RexMp2fspillSib32#611]
    0x012e, 0x0611,
    // --> [Mp2fspillSib32#611] and stop
    // 00049d: spill.f32 (I32)
    // --> [Mp2fspillSib32#611] and stop
    0x012d, 0x0611,
    // end of spill.f32 (I32)
    // end of spill.f32 (I64)
    // 00049f: sqrt.f32 (I64)
    // --> [RexMp2furm#651]
    0x01e0, 0x0651,
    // --> [Mp2furm#651] and stop
    // 0004a1: sqrt.f32 (I32)
    // --> [Mp2furm#651] and stop
    0x01df, 0x0651,
    // end of sqrt.f32 (I32)
    // end of sqrt.f32 (I64)
    // 0004a3: store.f32 (I64)
    // --> [RexMp2fst#611]
    0x010e, 0x0611,
    // --> [Mp2fst#611]
    0x010c, 0x0611,
    // --> [RexMp2fstDisp8#611]
    0x0112, 0x0611,
    // --> [Mp2fstDisp8#611]
    0x0110, 0x0611,
    // --> [RexMp2fstDisp32#611]
    0x0116, 0x0611,
    // --> [Mp2fstDisp32#611] and stop
    0x0115, 0x0611,
    // end of store.f32 (I64)
    // 0004af: store_complex.f32 (I64)
    // --> [RexMp2fstWithIndex#611]
    0x011a, 0x0611,
    // --> [Mp2fstWithIndex#611]
    0x0118, 0x0611,
    // --> [RexMp2fstWithIndexDisp8#611]
    0x011e, 0x0611,
    // --> [Mp2fstWithIndexDisp8#611]
    0x011c, 0x0611,
    // --> [RexMp2fstWithIndexDisp32#611]
    0x0122, 0x0611,
    // --> [Mp2fstWithIndexDisp32#611] and stop
    0x0121, 0x0611,
    // end of store_complex.f32 (I64)
    // 0004bb: x86_fmax.f32 (I64)
    // --> [RexMp2fa#65f]
    0x01ec, 0x065f,
    // --> [Mp2fa#65f] and stop
    // 0004bd: x86_fmax.f32 (I32)
    // --> [Mp2fa#65f] and stop
    0x01eb, 0x065f,
    // end of x86_fmax.f32 (I32)
    // end of x86_fmax.f32 (I64)
    // 0004bf: x86_fmin.f32 (I64)
    // --> [RexMp2fa#65d]
    0x01ec, 0x065d,
    // --> [Mp2fa#65d] and stop
    // 0004c1: x86_fmin.f32 (I32)
    // --> [Mp2fa#65d] and stop
    0x01eb, 0x065d,
    // end of x86_fmin.f32 (I32)
    // end of x86_fmin.f32 (I64)
    // 0004c3: band.b8x16 (I64)
    // --> [Mp2fa#5db] and stop
    // 0004c3: band.b16x8 (I64)
    // --> [Mp2fa#5db] and stop
    // 0004c3: band.b32x4 (I64)
    // --> [Mp2fa#5db] and stop
    // 0004c3: band.b64x2 (I64)
    // --> [Mp2fa#5db] and stop
    // 0004c3: band.i8x16 (I64)
    // --> [Mp2fa#5db] and stop
    // 0004c3: band.i16x8 (I64)
    // --> [Mp2fa#5db] and stop
    // 0004c3: band.i32x4 (I64)
    // --> [Mp2fa#5db] and stop
    // 0004c3: band.i64x2 (I64)
    // --> [Mp2fa#5db] and stop
    // 0004c3: band.f32x4 (I64)
    // --> [Mp2fa#5db] and stop
    // 0004c3: band.f64x2 (I64)
    // --> [Mp2fa#5db] and stop
    // 0004c3: band.b8x16 (I32)
    // --> [Mp2fa#5db] and stop
    // 0004c3: band.b16x8 (I32)
    // --> [Mp2fa#5db] and stop
    // 0004c3: band.b32x4 (I32)
    // --> [Mp2fa#5db] and stop
    // 0004c3: band.b64x2 (I32)
    // --> [Mp2fa#5db] and stop
    // 0004c3: band.i8x16 (I32)
    // --> [Mp2fa#5db] and stop
    // 0004c3: band.i16x8 (I32)
    // --> [Mp2fa#5db] and stop
    // 0004c3: band.i32x4 (I32)
    // --> [Mp2fa#5db] and stop
    // 0004c3: band.i64x2 (I32)
    // --> [Mp2fa#5db] and stop
    // 0004c3: band.f32x4 (I32)
    // --> [Mp2fa#5db] and stop
    // 0004c3: band.f64x2 (I32)
    // --> [Mp2fa#5db] and stop
    0x01eb, 0x05db,
    // end of band.f64x2 (I32)
    // end of band.f32x4 (I32)
    // end of band.i64x2 (I32)
    // end of band.i32x4 (I32)
    // end of band.i16x8 (I32)
    // end of band.i8x16 (I32)
    // end of band.b64x2 (I32)
    // end of band.b32x4 (I32)
    // end of band.b16x8 (I32)
    // end of band.b8x16 (I32)
    // end of band.f64x2 (I64)
    // end of band.f32x4 (I64)
    // end of band.i64x2 (I64)
    // end of band.i32x4 (I64)
    // end of band.i16x8 (I64)
    // end of band.i8x16 (I64)
    // end of band.b64x2 (I64)
    // end of band.b32x4 (I64)
    // end of band.b16x8 (I64)
    // end of band.b8x16 (I64)
    // 0004c5: band_not.b8x16 (I64)
    // --> [Mp2fax#5df] and stop
    // 0004c5: band_not.b16x8 (I64)
    // --> [Mp2fax#5df] and stop
    // 0004c5: band_not.b32x4 (I64)
    // --> [Mp2fax#5df] and stop
    // 0004c5: band_not.b64x2 (I64)
    // --> [Mp2fax#5df] and stop
    // 0004c5: band_not.i8x16 (I64)
    // --> [Mp2fax#5df] and stop
    // 0004c5: band_not.i16x8 (I64)
    // --> [Mp2fax#5df] and stop
    // 0004c5: band_not.i32x4 (I64)
    // --> [Mp2fax#5df] and stop
    // 0004c5: band_not.i64x2 (I64)
    // --> [Mp2fax#5df] and stop
    // 0004c5: band_not.f32x4 (I64)
    // --> [Mp2fax#5df] and stop
    // 0004c5: band_not.f64x2 (I64)
    // --> [Mp2fax#5df] and stop
    // 0004c5: band_not.b8x16 (I32)
    // --> [Mp2fax#5df] and stop
    // 0004c5: band_not.b16x8 (I32)
    // --> [Mp2fax#5df] and stop
    // 0004c5: band_not.b32x4 (I32)
    // --> [Mp2fax#5df] and stop
    // 0004c5: band_not.b64x2 (I32)
    // --> [Mp2fax#5df] and stop
    // 0004c5: band_not.i8x16 (I32)
    // --> [Mp2fax#5df] and stop
    // 0004c5: band_not.i16x8 (I32)
    // --> [Mp2fax#5df] and stop
    // 0004c5: band_not.i32x4 (I32)
    // --> [Mp2fax#5df] and stop
    // 0004c5: band_not.i64x2 (I32)
    // --> [Mp2fax#5df] and stop
    // 0004c5: band_not.f32x4 (I32)
    // --> [Mp2fax#5df] and stop
    // 0004c5: band_not.f64x2 (I32)
    // --> [Mp2fax#5df] and stop
    0x0231, 0x05df,
    // end of band_not.f64x2 (I32)
    // end of band_not.f32x4 (I32)
    // end of band_not.i64x2 (I32)
    // end of band_not.i32x4 (I32)
    // end of band_not.i16x8 (I32)
    // end of band_not.i8x16 (I32)
    // end of band_not.b64x2 (I32)
    // end of band_not.b32x4 (I32)
    // end of band_not.b16x8 (I32)
    // end of band_not.b8x16 (I32)
    // end of band_not.f64x2 (I64)
    // end of band_not.f32x4 (I64)
    // end of band_not.i64x2 (I64)
    // end of band_not.i32x4 (I64)
    // end of band_not.i16x8 (I64)
    // end of band_not.i8x16 (I64)
    // end of band_not.b64x2 (I64)
    // end of band_not.b32x4 (I64)
    // end of band_not.b16x8 (I64)
    // end of band_not.b8x16 (I64)
    // 0004c7: bor.b8x16 (I64)
    // --> [Mp2fa#5eb] and stop
    // 0004c7: bor.b16x8 (I64)
    // --> [Mp2fa#5eb] and stop
    // 0004c7: bor.b32x4 (I64)
    // --> [Mp2fa#5eb] and stop
    // 0004c7: bor.b64x2 (I64)
    // --> [Mp2fa#5eb] and stop
    // 0004c7: bor.i8x16 (I64)
    // --> [Mp2fa#5eb] and stop
    // 0004c7: bor.i16x8 (I64)
    // --> [Mp2fa#5eb] and stop
    // 0004c7: bor.i32x4 (I64)
    // --> [Mp2fa#5eb] and stop
    // 0004c7: bor.i64x2 (I64)
    // --> [Mp2fa#5eb] and stop
    // 0004c7: bor.f32x4 (I64)
    // --> [Mp2fa#5eb] and stop
    // 0004c7: bor.f64x2 (I64)
    // --> [Mp2fa#5eb] and stop
    // 0004c7: bor.b8x16 (I32)
    // --> [Mp2fa#5eb] and stop
    // 0004c7: bor.b16x8 (I32)
    // --> [Mp2fa#5eb] and stop
    // 0004c7: bor.b32x4 (I32)
    // --> [Mp2fa#5eb] and stop
    // 0004c7: bor.b64x2 (I32)
    // --> [Mp2fa#5eb] and stop
    // 0004c7: bor.i8x16 (I32)
    // --> [Mp2fa#5eb] and stop
    // 0004c7: bor.i16x8 (I32)
    // --> [Mp2fa#5eb] and stop
    // 0004c7: bor.i32x4 (I32)
    // --> [Mp2fa#5eb] and stop
    // 0004c7: bor.i64x2 (I32)
    // --> [Mp2fa#5eb] and stop
    // 0004c7: bor.f32x4 (I32)
    // --> [Mp2fa#5eb] and stop
    // 0004c7: bor.f64x2 (I32)
    // --> [Mp2fa#5eb] and stop
    0x01eb, 0x05eb,
    // end of bor.f64x2 (I32)
    // end of bor.f32x4 (I32)
    // end of bor.i64x2 (I32)
    // end of bor.i32x4 (I32)
    // end of bor.i16x8 (I32)
    // end of bor.i8x16 (I32)
    // end of bor.b64x2 (I32)
    // end of bor.b32x4 (I32)
    // end of bor.b16x8 (I32)
    // end of bor.b8x16 (I32)
    // end of bor.f64x2 (I64)
    // end of bor.f32x4 (I64)
    // end of bor.i64x2 (I64)
    // end of bor.i32x4 (I64)
    // end of bor.i16x8 (I64)
    // end of bor.i8x16 (I64)
    // end of bor.b64x2 (I64)
    // end of bor.b32x4 (I64)
    // end of bor.b16x8 (I64)
    // end of bor.b8x16 (I64)
    // 0004c9: bxor.b8x16 (I64)
    // --> [Mp2fa#5ef] and stop
    // 0004c9: bxor.b16x8 (I64)
    // --> [Mp2fa#5ef] and stop
    // 0004c9: bxor.b32x4 (I64)
    // --> [Mp2fa#5ef] and stop
    // 0004c9: bxor.b64x2 (I64)
    // --> [Mp2fa#5ef] and stop
    // 0004c9: bxor.i8x16 (I64)
    // --> [Mp2fa#5ef] and stop
    // 0004c9: bxor.i16x8 (I64)
    // --> [Mp2fa#5ef] and stop
    // 0004c9: bxor.i32x4 (I64)
    // --> [Mp2fa#5ef] and stop
    // 0004c9: bxor.i64x2 (I64)
    // --> [Mp2fa#5ef] and stop
    // 0004c9: bxor.f32x4 (I64)
    // --> [Mp2fa#5ef] and stop
    // 0004c9: bxor.f64x2 (I64)
    // --> [Mp2fa#5ef] and stop
    // 0004c9: bxor.b8x16 (I32)
    // --> [Mp2fa#5ef] and stop
    // 0004c9: bxor.b16x8 (I32)
    // --> [Mp2fa#5ef] and stop
    // 0004c9: bxor.b32x4 (I32)
    // --> [Mp2fa#5ef] and stop
    // 0004c9: bxor.b64x2 (I32)
    // --> [Mp2fa#5ef] and stop
    // 0004c9: bxor.i8x16 (I32)
    // --> [Mp2fa#5ef] and stop
    // 0004c9: bxor.i16x8 (I32)
    // --> [Mp2fa#5ef] and stop
    // 0004c9: bxor.i32x4 (I32)
    // --> [Mp2fa#5ef] and stop
    // 0004c9: bxor.i64x2 (I32)
    // --> [Mp2fa#5ef] and stop
    // 0004c9: bxor.f32x4 (I32)
    // --> [Mp2fa#5ef] and stop
    // 0004c9: bxor.f64x2 (I32)
    // --> [Mp2fa#5ef] and stop
    0x01eb, 0x05ef,
    // end of bxor.f64x2 (I32)
    // end of bxor.f32x4 (I32)
    // end of bxor.i64x2 (I32)
    // end of bxor.i32x4 (I32)
    // end of bxor.i16x8 (I32)
    // end of bxor.i8x16 (I32)
    // end of bxor.b64x2 (I32)
    // end of bxor.b32x4 (I32)
    // end of bxor.b16x8 (I32)
    // end of bxor.b8x16 (I32)
    // end of bxor.f64x2 (I64)
    // end of bxor.f32x4 (I64)
    // end of bxor.i64x2 (I64)
    // end of bxor.i32x4 (I64)
    // end of bxor.i16x8 (I64)
    // end of bxor.i8x16 (I64)
    // end of bxor.b64x2 (I64)
    // end of bxor.b32x4 (I64)
    // end of bxor.b16x8 (I64)
    // end of bxor.b8x16 (I64)
    // 0004cb: fill.b8x16 (I64)
    // --> [Op2ffillSib32#410] and stop
    // 0004cb: fill.b16x8 (I64)
    // --> [Op2ffillSib32#410] and stop
    // 0004cb: fill.b32x4 (I64)
    // --> [Op2ffillSib32#410] and stop
    // 0004cb: fill.b64x2 (I64)
    // --> [Op2ffillSib32#410] and stop
    // 0004cb: fill.i8x16 (I64)
    // --> [Op2ffillSib32#410] and stop
    // 0004cb: fill.i16x8 (I64)
    // --> [Op2ffillSib32#410] and stop
    // 0004cb: fill.i32x4 (I64)
    // --> [Op2ffillSib32#410] and stop
    // 0004cb: fill.i64x2 (I64)
    // --> [Op2ffillSib32#410] and stop
    // 0004cb: fill.f32x4 (I64)
    // --> [Op2ffillSib32#410] and stop
    // 0004cb: fill.f64x2 (I64)
    // --> [Op2ffillSib32#410] and stop
    // 0004cb: fill.b8x16 (I32)
    // --> [Op2ffillSib32#410] and stop
    // 0004cb: fill.b16x8 (I32)
    // --> [Op2ffillSib32#410] and stop
    // 0004cb: fill.b32x4 (I32)
    // --> [Op2ffillSib32#410] and stop
    // 0004cb: fill.b64x2 (I32)
    // --> [Op2ffillSib32#410] and stop
    // 0004cb: fill.i8x16 (I32)
    // --> [Op2ffillSib32#410] and stop
    // 0004cb: fill.i16x8 (I32)
    // --> [Op2ffillSib32#410] and stop
    // 0004cb: fill.i32x4 (I32)
    // --> [Op2ffillSib32#410] and stop
    // 0004cb: fill.i64x2 (I32)
    // --> [Op2ffillSib32#410] and stop
    // 0004cb: fill.f32x4 (I32)
    // --> [Op2ffillSib32#410] and stop
    // 0004cb: fill.f64x2 (I32)
    // --> [Op2ffillSib32#410] and stop
    0x022d, 0x0410,
    // end of fill.f64x2 (I32)
    // end of fill.f32x4 (I32)
    // end of fill.i64x2 (I32)
    // end of fill.i32x4 (I32)
    // end of fill.i16x8 (I32)
    // end of fill.i8x16 (I32)
    // end of fill.b64x2 (I32)
    // end of fill.b32x4 (I32)
    // end of fill.b16x8 (I32)
    // end of fill.b8x16 (I32)
    // end of fill.f64x2 (I64)
    // end of fill.f32x4 (I64)
    // end of fill.i64x2 (I64)
    // end of fill.i32x4 (I64)
    // end of fill.i16x8 (I64)
    // end of fill.i8x16 (I64)
    // end of fill.b64x2 (I64)
    // end of fill.b32x4 (I64)
    // end of fill.b16x8 (I64)
    // end of fill.b8x16 (I64)
    // 0004cd: load.b8x16 (I64)
    // --> [Op2fld#410]
    // 0004cd: load.b16x8 (I64)
    // --> [Op2fld#410]
    // 0004cd: load.b32x4 (I64)
    // --> [Op2fld#410]
    // 0004cd: load.b64x2 (I64)
    // --> [Op2fld#410]
    // 0004cd: load.i8x16 (I64)
    // --> [Op2fld#410]
    // 0004cd: load.i16x8 (I64)
    // --> [Op2fld#410]
    // 0004cd: load.i32x4 (I64)
    // --> [Op2fld#410]
    // 0004cd: load.i64x2 (I64)
    // --> [Op2fld#410]
    // 0004cd: load.f32x4 (I64)
    // --> [Op2fld#410]
    // 0004cd: load.f64x2 (I64)
    // --> [Op2fld#410]
    // 0004cd: load.b8x16 (I32)
    // --> [Op2fld#410]
    // 0004cd: load.b16x8 (I32)
    // --> [Op2fld#410]
    // 0004cd: load.b32x4 (I32)
    // --> [Op2fld#410]
    // 0004cd: load.b64x2 (I32)
    // --> [Op2fld#410]
    // 0004cd: load.i8x16 (I32)
    // --> [Op2fld#410]
    // 0004cd: load.i16x8 (I32)
    // --> [Op2fld#410]
    // 0004cd: load.i32x4 (I32)
    // --> [Op2fld#410]
    // 0004cd: load.i64x2 (I32)
    // --> [Op2fld#410]
    // 0004cd: load.f32x4 (I32)
    // --> [Op2fld#410]
    // 0004cd: load.f64x2 (I32)
    // --> [Op2fld#410]
    0x0222, 0x0410,
    // --> [Op2fldDisp8#410]
    // --> [Op2fldDisp8#410]
    // --> [Op2fldDisp8#410]
    // --> [Op2fldDisp8#410]
    // --> [Op2fldDisp8#410]
    // --> [Op2fldDisp8#410]
    // --> [Op2fldDisp8#410]
    // --> [Op2fldDisp8#410]
    // --> [Op2fldDisp8#410]
    // --> [Op2fldDisp8#410]
    // --> [Op2fldDisp8#410]
    // --> [Op2fldDisp8#410]
    // --> [Op2fldDisp8#410]
    // --> [Op2fldDisp8#410]
    // --> [Op2fldDisp8#410]
    // --> [Op2fldDisp8#410]
    // --> [Op2fldDisp8#410]
    // --> [Op2fldDisp8#410]
    // --> [Op2fldDisp8#410]
    // --> [Op2fldDisp8#410]
    0x0224, 0x0410,
    // --> [Op2fldDisp32#410] and stop
    // --> [Op2fldDisp32#410] and stop
    // --> [Op2fldDisp32#410] and stop
    // --> [Op2fldDisp32#410] and stop
    // --> [Op2fldDisp32#410] and stop
    // --> [Op2fldDisp32#410] and stop
    // --> [Op2fldDisp32#410] and stop
    // --> [Op2fldDisp32#410] and stop
    // --> [Op2fldDisp32#410] and stop
    // --> [Op2fldDisp32#410] and stop
    // --> [Op2fldDisp32#410] and stop
    // --> [Op2fldDisp32#410] and stop
    // --> [Op2fldDisp32#410] and stop
    // --> [Op2fldDisp32#410] and stop
    // --> [Op2fldDisp32#410] and stop
    // --> [Op2fldDisp32#410] and stop
    // --> [Op2fldDisp32#410] and stop
    // --> [Op2fldDisp32#410] and stop
    // --> [Op2fldDisp32#410] and stop
    // --> [Op2fldDisp32#410] and stop
    0x0227, 0x0410,
    // end of load.f64x2 (I32)
    // end of load.f32x4 (I32)
    // end of load.i64x2 (I32)
    // end of load.i32x4 (I32)
    // end of load.i16x8 (I32)
    // end of load.i8x16 (I32)
    // end of load.b64x2 (I32)
    // end of load.b32x4 (I32)
    // end of load.b16x8 (I32)
    // end of load.b8x16 (I32)
    // end of load.f64x2 (I64)
    // end of load.f32x4 (I64)
    // end of load.i64x2 (I64)
    // end of load.i32x4 (I64)
    // end of load.i16x8 (I64)
    // end of load.i8x16 (I64)
    // end of load.b64x2 (I64)
    // end of load.b32x4 (I64)
    // end of load.b16x8 (I64)
    // end of load.b8x16 (I64)
    // 0004d3: raw_bitcast.b8x16 (I64)
    // skip 2 unless inst_predicate_18
    // 0004d3: raw_bitcast.b8x16 (I32)
    // skip 2 unless inst_predicate_18
    0x3012,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_19
    // skip 2 unless inst_predicate_19
    0x3013,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_20
    // skip 2 unless inst_predicate_20
    0x3014,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_21
    // skip 2 unless inst_predicate_21
    0x3015,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_22
    // skip 2 unless inst_predicate_22
    0x3016,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_23
    // skip 2 unless inst_predicate_23
    0x3017,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_24
    // skip 2 unless inst_predicate_24
    0x3018,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_25
    // skip 2 unless inst_predicate_25
    0x3019,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_26
    // skip 2 unless inst_predicate_26
    0x301a,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_15
    // skip 2 unless inst_predicate_15
    0x300f,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // stop unless inst_predicate_16
    // stop unless inst_predicate_16
    0x1010,
    // --> [null_fpr#00] and stop
    // --> [null_fpr#00] and stop
    0x020b, 0x0000,
    // end of raw_bitcast.b8x16 (I32)
    // end of raw_bitcast.b8x16 (I64)
    // 0004f4: regfill.b8x16 (I64)
    // --> [Op2fregfill32#410] and stop
    // 0004f4: regfill.b16x8 (I64)
    // --> [Op2fregfill32#410] and stop
    // 0004f4: regfill.b32x4 (I64)
    // --> [Op2fregfill32#410] and stop
    // 0004f4: regfill.b64x2 (I64)
    // --> [Op2fregfill32#410] and stop
    // 0004f4: regfill.i8x16 (I64)
    // --> [Op2fregfill32#410] and stop
    // 0004f4: regfill.i16x8 (I64)
    // --> [Op2fregfill32#410] and stop
    // 0004f4: regfill.i32x4 (I64)
    // --> [Op2fregfill32#410] and stop
    // 0004f4: regfill.i64x2 (I64)
    // --> [Op2fregfill32#410] and stop
    // 0004f4: regfill.f32x4 (I64)
    // --> [Op2fregfill32#410] and stop
    // 0004f4: regfill.f64x2 (I64)
    // --> [Op2fregfill32#410] and stop
    // 0004f4: regfill.b8x16 (I32)
    // --> [Op2fregfill32#410] and stop
    // 0004f4: regfill.b16x8 (I32)
    // --> [Op2fregfill32#410] and stop
    // 0004f4: regfill.b32x4 (I32)
    // --> [Op2fregfill32#410] and stop
    // 0004f4: regfill.b64x2 (I32)
    // --> [Op2fregfill32#410] and stop
    // 0004f4: regfill.i8x16 (I32)
    // --> [Op2fregfill32#410] and stop
    // 0004f4: regfill.i16x8 (I32)
    // --> [Op2fregfill32#410] and stop
    // 0004f4: regfill.i32x4 (I32)
    // --> [Op2fregfill32#410] and stop
    // 0004f4: regfill.i64x2 (I32)
    // --> [Op2fregfill32#410] and stop
    // 0004f4: regfill.f32x4 (I32)
    // --> [Op2fregfill32#410] and stop
    // 0004f4: regfill.f64x2 (I32)
    // --> [Op2fregfill32#410] and stop
    0x022f, 0x0410,
    // end of regfill.f64x2 (I32)
    // end of regfill.f32x4 (I32)
    // end of regfill.i64x2 (I32)
    // end of regfill.i32x4 (I32)
    // end of regfill.i16x8 (I32)
    // end of regfill.i8x16 (I32)
    // end of regfill.b64x2 (I32)
    // end of regfill.b32x4 (I32)
    // end of regfill.b16x8 (I32)
    // end of regfill.b8x16 (I32)
    // end of regfill.f64x2 (I64)
    // end of regfill.f32x4 (I64)
    // end of regfill.i64x2 (I64)
    // end of regfill.i32x4 (I64)
    // end of regfill.i16x8 (I64)
    // end of regfill.i8x16 (I64)
    // end of regfill.b64x2 (I64)
    // end of regfill.b32x4 (I64)
    // end of regfill.b16x8 (I64)
    // end of regfill.b8x16 (I64)
    // 0004f6: regmove.b8x16 (I64)
    // --> [Op2frmov#428] and stop
    // 0004f6: regmove.b16x8 (I64)
    // --> [Op2frmov#428] and stop
    // 0004f6: regmove.b32x4 (I64)
    // --> [Op2frmov#428] and stop
    // 0004f6: regmove.b64x2 (I64)
    // --> [Op2frmov#428] and stop
    // 0004f6: regmove.i8x16 (I64)
    // --> [Op2frmov#428] and stop
    // 0004f6: regmove.i16x8 (I64)
    // --> [Op2frmov#428] and stop
    // 0004f6: regmove.i32x4 (I64)
    // --> [Op2frmov#428] and stop
    // 0004f6: regmove.i64x2 (I64)
    // --> [Op2frmov#428] and stop
    // 0004f6: regmove.f32x4 (I64)
    // --> [Op2frmov#428] and stop
    // 0004f6: regmove.f64x2 (I64)
    // --> [Op2frmov#428] and stop
    // 0004f6: regmove.f64 (I32)
    // --> [Op2frmov#428] and stop
    // 0004f6: regmove.f32 (I32)
    // --> [Op2frmov#428] and stop
    // 0004f6: regmove.b8x16 (I32)
    // --> [Op2frmov#428] and stop
    // 0004f6: regmove.b16x8 (I32)
    // --> [Op2frmov#428] and stop
    // 0004f6: regmove.b32x4 (I32)
    // --> [Op2frmov#428] and stop
    // 0004f6: regmove.b64x2 (I32)
    // --> [Op2frmov#428] and stop
    // 0004f6: regmove.i8x16 (I32)
    // --> [Op2frmov#428] and stop
    // 0004f6: regmove.i16x8 (I32)
    // --> [Op2frmov#428] and stop
    // 0004f6: regmove.i32x4 (I32)
    // --> [Op2frmov#428] and stop
    // 0004f6: regmove.i64x2 (I32)
    // --> [Op2frmov#428] and stop
    // 0004f6: regmove.f32x4 (I32)
    // --> [Op2frmov#428] and stop
    // 0004f6: regmove.f64x2 (I32)
    // --> [Op2frmov#428] and stop
    0x01db, 0x0428,
    // end of regmove.f64x2 (I32)
    // end of regmove.f32x4 (I32)
    // end of regmove.i64x2 (I32)
    // end of regmove.i32x4 (I32)
    // end of regmove.i16x8 (I32)
    // end of regmove.i8x16 (I32)
    // end of regmove.b64x2 (I32)
    // end of regmove.b32x4 (I32)
    // end of regmove.b16x8 (I32)
    // end of regmove.b8x16 (I32)
    // end of regmove.f32 (I32)
    // end of regmove.f64 (I32)
    // end of regmove.f64x2 (I64)
    // end of regmove.f32x4 (I64)
    // end of regmove.i64x2 (I64)
    // end of regmove.i32x4 (I64)
    // end of regmove.i16x8 (I64)
    // end of regmove.i8x16 (I64)
    // end of regmove.b64x2 (I64)
    // end of regmove.b32x4 (I64)
    // end of regmove.b16x8 (I64)
    // end of regmove.b8x16 (I64)
    // 0004f8: regspill.b8x16 (I64)
    // --> [Op2fregspill32#411] and stop
    // 0004f8: regspill.b16x8 (I64)
    // --> [Op2fregspill32#411] and stop
    // 0004f8: regspill.b32x4 (I64)
    // --> [Op2fregspill32#411] and stop
    // 0004f8: regspill.b64x2 (I64)
    // --> [Op2fregspill32#411] and stop
    // 0004f8: regspill.i8x16 (I64)
    // --> [Op2fregspill32#411] and stop
    // 0004f8: regspill.i16x8 (I64)
    // --> [Op2fregspill32#411] and stop
    // 0004f8: regspill.i32x4 (I64)
    // --> [Op2fregspill32#411] and stop
    // 0004f8: regspill.i64x2 (I64)
    // --> [Op2fregspill32#411] and stop
    // 0004f8: regspill.f32x4 (I64)
    // --> [Op2fregspill32#411] and stop
    // 0004f8: regspill.f64x2 (I64)
    // --> [Op2fregspill32#411] and stop
    // 0004f8: regspill.b8x16 (I32)
    // --> [Op2fregspill32#411] and stop
    // 0004f8: regspill.b16x8 (I32)
    // --> [Op2fregspill32#411] and stop
    // 0004f8: regspill.b32x4 (I32)
    // --> [Op2fregspill32#411] and stop
    // 0004f8: regspill.b64x2 (I32)
    // --> [Op2fregspill32#411] and stop
    // 0004f8: regspill.i8x16 (I32)
    // --> [Op2fregspill32#411] and stop
    // 0004f8: regspill.i16x8 (I32)
    // --> [Op2fregspill32#411] and stop
    // 0004f8: regspill.i32x4 (I32)
    // --> [Op2fregspill32#411] and stop
    // 0004f8: regspill.i64x2 (I32)
    // --> [Op2fregspill32#411] and stop
    // 0004f8: regspill.f32x4 (I32)
    // --> [Op2fregspill32#411] and stop
    // 0004f8: regspill.f64x2 (I32)
    // --> [Op2fregspill32#411] and stop
    0x022b, 0x0411,
    // end of regspill.f64x2 (I32)
    // end of regspill.f32x4 (I32)
    // end of regspill.i64x2 (I32)
    // end of regspill.i32x4 (I32)
    // end of regspill.i16x8 (I32)
    // end of regspill.i8x16 (I32)
    // end of regspill.b64x2 (I32)
    // end of regspill.b32x4 (I32)
    // end of regspill.b16x8 (I32)
    // end of regspill.b8x16 (I32)
    // end of regspill.f64x2 (I64)
    // end of regspill.f32x4 (I64)
    // end of regspill.i64x2 (I64)
    // end of regspill.i32x4 (I64)
    // end of regspill.i16x8 (I64)
    // end of regspill.i8x16 (I64)
    // end of regspill.b64x2 (I64)
    // end of regspill.b32x4 (I64)
    // end of regspill.b16x8 (I64)
    // end of regspill.b8x16 (I64)
    // 0004fa: spill.b8x16 (I64)
    // --> [Op2fspillSib32#411] and stop
    // 0004fa: spill.b16x8 (I64)
    // --> [Op2fspillSib32#411] and stop
    // 0004fa: spill.b32x4 (I64)
    // --> [Op2fspillSib32#411] and stop
    // 0004fa: spill.b64x2 (I64)
    // --> [Op2fspillSib32#411] and stop
    // 0004fa: spill.i8x16 (I64)
    // --> [Op2fspillSib32#411] and stop
    // 0004fa: spill.i16x8 (I64)
    // --> [Op2fspillSib32#411] and stop
    // 0004fa: spill.i32x4 (I64)
    // --> [Op2fspillSib32#411] and stop
    // 0004fa: spill.i64x2 (I64)
    // --> [Op2fspillSib32#411] and stop
    // 0004fa: spill.f32x4 (I64)
    // --> [Op2fspillSib32#411] and stop
    // 0004fa: spill.f64x2 (I64)
    // --> [Op2fspillSib32#411] and stop
    // 0004fa: spill.b8x16 (I32)
    // --> [Op2fspillSib32#411] and stop
    // 0004fa: spill.b16x8 (I32)
    // --> [Op2fspillSib32#411] and stop
    // 0004fa: spill.b32x4 (I32)
    // --> [Op2fspillSib32#411] and stop
    // 0004fa: spill.b64x2 (I32)
    // --> [Op2fspillSib32#411] and stop
    // 0004fa: spill.i8x16 (I32)
    // --> [Op2fspillSib32#411] and stop
    // 0004fa: spill.i16x8 (I32)
    // --> [Op2fspillSib32#411] and stop
    // 0004fa: spill.i32x4 (I32)
    // --> [Op2fspillSib32#411] and stop
    // 0004fa: spill.i64x2 (I32)
    // --> [Op2fspillSib32#411] and stop
    // 0004fa: spill.f32x4 (I32)
    // --> [Op2fspillSib32#411] and stop
    // 0004fa: spill.f64x2 (I32)
    // --> [Op2fspillSib32#411] and stop
    0x0229, 0x0411,
    // end of spill.f64x2 (I32)
    // end of spill.f32x4 (I32)
    // end of spill.i64x2 (I32)
    // end of spill.i32x4 (I32)
    // end of spill.i16x8 (I32)
    // end of spill.i8x16 (I32)
    // end of spill.b64x2 (I32)
    // end of spill.b32x4 (I32)
    // end of spill.b16x8 (I32)
    // end of spill.b8x16 (I32)
    // end of spill.f64x2 (I64)
    // end of spill.f32x4 (I64)
    // end of spill.i64x2 (I64)
    // end of spill.i32x4 (I64)
    // end of spill.i16x8 (I64)
    // end of spill.i8x16 (I64)
    // end of spill.b64x2 (I64)
    // end of spill.b32x4 (I64)
    // end of spill.b16x8 (I64)
    // end of spill.b8x16 (I64)
    // 0004fc: store.b8x16 (I64)
    // --> [Op2fst#411]
    // 0004fc: store.b16x8 (I64)
    // --> [Op2fst#411]
    // 0004fc: store.b32x4 (I64)
    // --> [Op2fst#411]
    // 0004fc: store.b64x2 (I64)
    // --> [Op2fst#411]
    // 0004fc: store.i8x16 (I64)
    // --> [Op2fst#411]
    // 0004fc: store.i16x8 (I64)
    // --> [Op2fst#411]
    // 0004fc: store.i32x4 (I64)
    // --> [Op2fst#411]
    // 0004fc: store.i64x2 (I64)
    // --> [Op2fst#411]
    // 0004fc: store.f32x4 (I64)
    // --> [Op2fst#411]
    // 0004fc: store.f64x2 (I64)
    // --> [Op2fst#411]
    // 0004fc: store.b8x16 (I32)
    // --> [Op2fst#411]
    // 0004fc: store.b16x8 (I32)
    // --> [Op2fst#411]
    // 0004fc: store.b32x4 (I32)
    // --> [Op2fst#411]
    // 0004fc: store.b64x2 (I32)
    // --> [Op2fst#411]
    // 0004fc: store.i8x16 (I32)
    // --> [Op2fst#411]
    // 0004fc: store.i16x8 (I32)
    // --> [Op2fst#411]
    // 0004fc: store.i32x4 (I32)
    // --> [Op2fst#411]
    // 0004fc: store.i64x2 (I32)
    // --> [Op2fst#411]
    // 0004fc: store.f32x4 (I32)
    // --> [Op2fst#411]
    // 0004fc: store.f64x2 (I32)
    // --> [Op2fst#411]
    0x021c, 0x0411,
    // --> [Op2fstDisp8#411]
    // --> [Op2fstDisp8#411]
    // --> [Op2fstDisp8#411]
    // --> [Op2fstDisp8#411]
    // --> [Op2fstDisp8#411]
    // --> [Op2fstDisp8#411]
    // --> [Op2fstDisp8#411]
    // --> [Op2fstDisp8#411]
    // --> [Op2fstDisp8#411]
    // --> [Op2fstDisp8#411]
    // --> [Op2fstDisp8#411]
    // --> [Op2fstDisp8#411]
    // --> [Op2fstDisp8#411]
    // --> [Op2fstDisp8#411]
    // --> [Op2fstDisp8#411]
    // --> [Op2fstDisp8#411]
    // --> [Op2fstDisp8#411]
    // --> [Op2fstDisp8#411]
    // --> [Op2fstDisp8#411]
    // --> [Op2fstDisp8#411]
    0x021e, 0x0411,
    // --> [Op2fstDisp32#411] and stop
    // --> [Op2fstDisp32#411] and stop
    // --> [Op2fstDisp32#411] and stop
    // --> [Op2fstDisp32#411] and stop
    // --> [Op2fstDisp32#411] and stop
    // --> [Op2fstDisp32#411] and stop
    // --> [Op2fstDisp32#411] and stop
    // --> [Op2fstDisp32#411] and stop
    // --> [Op2fstDisp32#411] and stop
    // --> [Op2fstDisp32#411] and stop
    // --> [Op2fstDisp32#411] and stop
    // --> [Op2fstDisp32#411] and stop
    // --> [Op2fstDisp32#411] and stop
    // --> [Op2fstDisp32#411] and stop
    // --> [Op2fstDisp32#411] and stop
    // --> [Op2fstDisp32#411] and stop
    // --> [Op2fstDisp32#411] and stop
    // --> [Op2fstDisp32#411] and stop
    // --> [Op2fstDisp32#411] and stop
    // --> [Op2fstDisp32#411] and stop
    0x0221, 0x0411,
    // end of store.f64x2 (I32)
    // end of store.f32x4 (I32)
    // end of store.i64x2 (I32)
    // end of store.i32x4 (I32)
    // end of store.i16x8 (I32)
    // end of store.i8x16 (I32)
    // end of store.b64x2 (I32)
    // end of store.b32x4 (I32)
    // end of store.b16x8 (I32)
    // end of store.b8x16 (I32)
    // end of store.f64x2 (I64)
    // end of store.f32x4 (I64)
    // end of store.i64x2 (I64)
    // end of store.i32x4 (I64)
    // end of store.i16x8 (I64)
    // end of store.i8x16 (I64)
    // end of store.b64x2 (I64)
    // end of store.b32x4 (I64)
    // end of store.b16x8 (I64)
    // end of store.b8x16 (I64)
    // 000502: vconst.b8x16 (I64)
    // skip 2 unless inst_predicate_27
    // 000502: vconst.b16x8 (I64)
    // skip 2 unless inst_predicate_27
    // 000502: vconst.b32x4 (I64)
    // skip 2 unless inst_predicate_27
    // 000502: vconst.b64x2 (I64)
    // skip 2 unless inst_predicate_27
    // 000502: vconst.i8x16 (I64)
    // skip 2 unless inst_predicate_27
    // 000502: vconst.i16x8 (I64)
    // skip 2 unless inst_predicate_27
    // 000502: vconst.i32x4 (I64)
    // skip 2 unless inst_predicate_27
    // 000502: vconst.i64x2 (I64)
    // skip 2 unless inst_predicate_27
    // 000502: vconst.f32x4 (I64)
    // skip 2 unless inst_predicate_27
    // 000502: vconst.f64x2 (I64)
    // skip 2 unless inst_predicate_27
    // 000502: vconst.b8x16 (I32)
    // skip 2 unless inst_predicate_27
    // 000502: vconst.b16x8 (I32)
    // skip 2 unless inst_predicate_27
    // 000502: vconst.b32x4 (I32)
    // skip 2 unless inst_predicate_27
    // 000502: vconst.b64x2 (I32)
    // skip 2 unless inst_predicate_27
    // 000502: vconst.i8x16 (I32)
    // skip 2 unless inst_predicate_27
    // 000502: vconst.i16x8 (I32)
    // skip 2 unless inst_predicate_27
    // 000502: vconst.i32x4 (I32)
    // skip 2 unless inst_predicate_27
    // 000502: vconst.i64x2 (I32)
    // skip 2 unless inst_predicate_27
    // 000502: vconst.f32x4 (I32)
    // skip 2 unless inst_predicate_27
    // 000502: vconst.f64x2 (I32)
    // skip 2 unless inst_predicate_27
    0x301b,
    // --> [Mp2vconst_optimized#5ef]
    // --> [Mp2vconst_optimized#5ef]
    // --> [Mp2vconst_optimized#5ef]
    // --> [Mp2vconst_optimized#5ef]
    // --> [Mp2vconst_optimized#5ef]
    // --> [Mp2vconst_optimized#5ef]
    // --> [Mp2vconst_optimized#5ef]
    // --> [Mp2vconst_optimized#5ef]
    // --> [Mp2vconst_optimized#5ef]
    // --> [Mp2vconst_optimized#5ef]
    // --> [Mp2vconst_optimized#5ef]
    // --> [Mp2vconst_optimized#5ef]
    // --> [Mp2vconst_optimized#5ef]
    // --> [Mp2vconst_optimized#5ef]
    // --> [Mp2vconst_optimized#5ef]
    // --> [Mp2vconst_optimized#5ef]
    // --> [Mp2vconst_optimized#5ef]
    // --> [Mp2vconst_optimized#5ef]
    // --> [Mp2vconst_optimized#5ef]
    // --> [Mp2vconst_optimized#5ef]
    0x0218, 0x05ef,
    // skip 2 unless inst_predicate_28
    // skip 2 unless inst_predicate_28
    // skip 2 unless inst_predicate_28
    // skip 2 unless inst_predicate_28
    // skip 2 unless inst_predicate_28
    // skip 2 unless inst_predicate_28
    // skip 2 unless inst_predicate_28
    // skip 2 unless inst_predicate_28
    // skip 2 unless inst_predicate_28
    // skip 2 unless inst_predicate_28
    // skip 2 unless inst_predicate_28
    // skip 2 unless inst_predicate_28
    // skip 2 unless inst_predicate_28
    // skip 2 unless inst_predicate_28
    // skip 2 unless inst_predicate_28
    // skip 2 unless inst_predicate_28
    // skip 2 unless inst_predicate_28
    // skip 2 unless inst_predicate_28
    // skip 2 unless inst_predicate_28
    // skip 2 unless inst_predicate_28
    0x301c,
    // --> [Mp2vconst_optimized#574]
    // --> [Mp2vconst_optimized#574]
    // --> [Mp2vconst_optimized#574]
    // --> [Mp2vconst_optimized#574]
    // --> [Mp2vconst_optimized#574]
    // --> [Mp2vconst_optimized#574]
    // --> [Mp2vconst_optimized#574]
    // --> [Mp2vconst_optimized#574]
    // --> [Mp2vconst_optimized#574]
    // --> [Mp2vconst_optimized#574]
    // --> [Mp2vconst_optimized#574]
    // --> [Mp2vconst_optimized#574]
    // --> [Mp2vconst_optimized#574]
    // --> [Mp2vconst_optimized#574]
    // --> [Mp2vconst_optimized#574]
    // --> [Mp2vconst_optimized#574]
    // --> [Mp2vconst_optimized#574]
    // --> [Mp2vconst_optimized#574]
    // --> [Mp2vconst_optimized#574]
    // --> [Mp2vconst_optimized#574]
    0x0218, 0x0574,
    // --> [Op2vconst#410] and stop
    // --> [Op2vconst#410] and stop
    // --> [Op2vconst#410] and stop
    // --> [Op2vconst#410] and stop
    // --> [Op2vconst#410] and stop
    // --> [Op2vconst#410] and stop
    // --> [Op2vconst#410] and stop
    // --> [Op2vconst#410] and stop
    // --> [Op2vconst#410] and stop
    // --> [Op2vconst#410] and stop
    // --> [Op2vconst#410] and stop
    // --> [Op2vconst#410] and stop
    // --> [Op2vconst#410] and stop
    // --> [Op2vconst#410] and stop
    // --> [Op2vconst#410] and stop
    // --> [Op2vconst#410] and stop
    // --> [Op2vconst#410] and stop
    // --> [Op2vconst#410] and stop
    // --> [Op2vconst#410] and stop
    // --> [Op2vconst#410] and stop
    0x021b, 0x0410,
    // end of vconst.f64x2 (I32)
    // end of vconst.f32x4 (I32)
    // end of vconst.i64x2 (I32)
    // end of vconst.i32x4 (I32)
    // end of vconst.i16x8 (I32)
    // end of vconst.i8x16 (I32)
    // end of vconst.b64x2 (I32)
    // end of vconst.b32x4 (I32)
    // end of vconst.b16x8 (I32)
    // end of vconst.b8x16 (I32)
    // end of vconst.f64x2 (I64)
    // end of vconst.f32x4 (I64)
    // end of vconst.i64x2 (I64)
    // end of vconst.i32x4 (I64)
    // end of vconst.i16x8 (I64)
    // end of vconst.i8x16 (I64)
    // end of vconst.b64x2 (I64)
    // end of vconst.b32x4 (I64)
    // end of vconst.b16x8 (I64)
    // end of vconst.b8x16 (I64)
    // 00050a: x86_pextr.b8x16 (I64)
    // stop unless PredicateView(17)
    // 00050a: x86_pextr.i8x16 (I64)
    // stop unless PredicateView(17)
    // 00050a: x86_pextr.b8x16 (I32)
    // stop unless PredicateView(17)
    // 00050a: x86_pextr.i8x16 (I32)
    // stop unless PredicateView(17)
    0x1030,
    // --> [Mp3r_ib_unsigned_gpr#d14] and stop
    // --> [Mp3r_ib_unsigned_gpr#d14] and stop
    // --> [Mp3r_ib_unsigned_gpr#d14] and stop
    // --> [Mp3r_ib_unsigned_gpr#d14] and stop
    0x0215, 0x0d14,
    // end of x86_pextr.i8x16 (I32)
    // end of x86_pextr.b8x16 (I32)
    // end of x86_pextr.i8x16 (I64)
    // end of x86_pextr.b8x16 (I64)
    // 00050d: x86_pinsr.b8x16 (I64)
    // stop unless PredicateView(17)
    // 00050d: x86_pinsr.i8x16 (I64)
    // stop unless PredicateView(17)
    // 00050d: x86_pinsr.b8x16 (I32)
    // stop unless PredicateView(17)
    // 00050d: x86_pinsr.i8x16 (I32)
    // stop unless PredicateView(17)
    0x1030,
    // --> [Mp3r_ib_unsigned_r#d20] and stop
    // --> [Mp3r_ib_unsigned_r#d20] and stop
    // --> [Mp3r_ib_unsigned_r#d20] and stop
    // --> [Mp3r_ib_unsigned_r#d20] and stop
    0x020d, 0x0d20,
    // end of x86_pinsr.i8x16 (I32)
    // end of x86_pinsr.b8x16 (I32)
    // end of x86_pinsr.i8x16 (I64)
    // end of x86_pinsr.b8x16 (I64)
    // 000510: x86_pshufb.b8x16 (I64)
    // stop unless PredicateView(21)
    // 000510: x86_pshufb.b16x8 (I64)
    // stop unless PredicateView(21)
    // 000510: x86_pshufb.b32x4 (I64)
    // stop unless PredicateView(21)
    // 000510: x86_pshufb.b64x2 (I64)
    // stop unless PredicateView(21)
    // 000510: x86_pshufb.i8x16 (I64)
    // stop unless PredicateView(21)
    // 000510: x86_pshufb.i16x8 (I64)
    // stop unless PredicateView(21)
    // 000510: x86_pshufb.i32x4 (I64)
    // stop unless PredicateView(21)
    // 000510: x86_pshufb.i64x2 (I64)
    // stop unless PredicateView(21)
    // 000510: x86_pshufb.f32x4 (I64)
    // stop unless PredicateView(21)
    // 000510: x86_pshufb.f64x2 (I64)
    // stop unless PredicateView(21)
    // 000510: x86_pshufb.b8x16 (I32)
    // stop unless PredicateView(21)
    // 000510: x86_pshufb.b16x8 (I32)
    // stop unless PredicateView(21)
    // 000510: x86_pshufb.b32x4 (I32)
    // stop unless PredicateView(21)
    // 000510: x86_pshufb.b64x2 (I32)
    // stop unless PredicateView(21)
    // 000510: x86_pshufb.i8x16 (I32)
    // stop unless PredicateView(21)
    // 000510: x86_pshufb.i16x8 (I32)
    // stop unless PredicateView(21)
    // 000510: x86_pshufb.i32x4 (I32)
    // stop unless PredicateView(21)
    // 000510: x86_pshufb.i64x2 (I32)
    // stop unless PredicateView(21)
    // 000510: x86_pshufb.f32x4 (I32)
    // stop unless PredicateView(21)
    // 000510: x86_pshufb.f64x2 (I32)
    // stop unless PredicateView(21)
    0x1034,
    // --> [Mp3fa#900] and stop
    // --> [Mp3fa#900] and stop
    // --> [Mp3fa#900] and stop
    // --> [Mp3fa#900] and stop
    // --> [Mp3fa#900] and stop
    // --> [Mp3fa#900] and stop
    // --> [Mp3fa#900] and stop
    // --> [Mp3fa#900] and stop
    // --> [Mp3fa#900] and stop
    // --> [Mp3fa#900] and stop
    // --> [Mp3fa#900] and stop
    // --> [Mp3fa#900] and stop
    // --> [Mp3fa#900] and stop
    // --> [Mp3fa#900] and stop
    // --> [Mp3fa#900] and stop
    // --> [Mp3fa#900] and stop
    // --> [Mp3fa#900] and stop
    // --> [Mp3fa#900] and stop
    // --> [Mp3fa#900] and stop
    // --> [Mp3fa#900] and stop
    0x0207, 0x0900,
    // end of x86_pshufb.f64x2 (I32)
    // end of x86_pshufb.f32x4 (I32)
    // end of x86_pshufb.i64x2 (I32)
    // end of x86_pshufb.i32x4 (I32)
    // end of x86_pshufb.i16x8 (I32)
    // end of x86_pshufb.i8x16 (I32)
    // end of x86_pshufb.b64x2 (I32)
    // end of x86_pshufb.b32x4 (I32)
    // end of x86_pshufb.b16x8 (I32)
    // end of x86_pshufb.b8x16 (I32)
    // end of x86_pshufb.f64x2 (I64)
    // end of x86_pshufb.f32x4 (I64)
    // end of x86_pshufb.i64x2 (I64)
    // end of x86_pshufb.i32x4 (I64)
    // end of x86_pshufb.i16x8 (I64)
    // end of x86_pshufb.i8x16 (I64)
    // end of x86_pshufb.b64x2 (I64)
    // end of x86_pshufb.b32x4 (I64)
    // end of x86_pshufb.b16x8 (I64)
    // end of x86_pshufb.b8x16 (I64)
    // 000513: x86_ptest.b8x16 (I64)
    // stop unless PredicateView(17)
    // 000513: x86_ptest.b16x8 (I64)
    // stop unless PredicateView(17)
    // 000513: x86_ptest.b32x4 (I64)
    // stop unless PredicateView(17)
    // 000513: x86_ptest.b64x2 (I64)
    // stop unless PredicateView(17)
    // 000513: x86_ptest.i8x16 (I64)
    // stop unless PredicateView(17)
    // 000513: x86_ptest.i16x8 (I64)
    // stop unless PredicateView(17)
    // 000513: x86_ptest.i32x4 (I64)
    // stop unless PredicateView(17)
    // 000513: x86_ptest.i64x2 (I64)
    // stop unless PredicateView(17)
    // 000513: x86_ptest.f32x4 (I64)
    // stop unless PredicateView(17)
    // 000513: x86_ptest.f64x2 (I64)
    // stop unless PredicateView(17)
    // 000513: x86_ptest.b8x16 (I32)
    // stop unless PredicateView(17)
    // 000513: x86_ptest.b16x8 (I32)
    // stop unless PredicateView(17)
    // 000513: x86_ptest.b32x4 (I32)
    // stop unless PredicateView(17)
    // 000513: x86_ptest.b64x2 (I32)
    // stop unless PredicateView(17)
    // 000513: x86_ptest.i8x16 (I32)
    // stop unless PredicateView(17)
    // 000513: x86_ptest.i16x8 (I32)
    // stop unless PredicateView(17)
    // 000513: x86_ptest.i32x4 (I32)
    // stop unless PredicateView(17)
    // 000513: x86_ptest.i64x2 (I32)
    // stop unless PredicateView(17)
    // 000513: x86_ptest.f32x4 (I32)
    // stop unless PredicateView(17)
    // 000513: x86_ptest.f64x2 (I32)
    // stop unless PredicateView(17)
    0x1030,
    // --> [Mp3fcmp#917] and stop
    // --> [Mp3fcmp#917] and stop
    // --> [Mp3fcmp#917] and stop
    // --> [Mp3fcmp#917] and stop
    // --> [Mp3fcmp#917] and stop
    // --> [Mp3fcmp#917] and stop
    // --> [Mp3fcmp#917] and stop
    // --> [Mp3fcmp#917] and stop
    // --> [Mp3fcmp#917] and stop
    // --> [Mp3fcmp#917] and stop
    // --> [Mp3fcmp#917] and stop
    // --> [Mp3fcmp#917] and stop
    // --> [Mp3fcmp#917] and stop
    // --> [Mp3fcmp#917] and stop
    // --> [Mp3fcmp#917] and stop
    // --> [Mp3fcmp#917] and stop
    // --> [Mp3fcmp#917] and stop
    // --> [Mp3fcmp#917] and stop
    // --> [Mp3fcmp#917] and stop
    // --> [Mp3fcmp#917] and stop
    0x0233, 0x0917,
    // end of x86_ptest.f64x2 (I32)
    // end of x86_ptest.f32x4 (I32)
    // end of x86_ptest.i64x2 (I32)
    // end of x86_ptest.i32x4 (I32)
    // end of x86_ptest.i16x8 (I32)
    // end of x86_ptest.i8x16 (I32)
    // end of x86_ptest.b64x2 (I32)
    // end of x86_ptest.b32x4 (I32)
    // end of x86_ptest.b16x8 (I32)
    // end of x86_ptest.b8x16 (I32)
    // end of x86_ptest.f64x2 (I64)
    // end of x86_ptest.f32x4 (I64)
    // end of x86_ptest.i64x2 (I64)
    // end of x86_ptest.i32x4 (I64)
    // end of x86_ptest.i16x8 (I64)
    // end of x86_ptest.i8x16 (I64)
    // end of x86_ptest.b64x2 (I64)
    // end of x86_ptest.b32x4 (I64)
    // end of x86_ptest.b16x8 (I64)
    // end of x86_ptest.b8x16 (I64)
    // 000516: raw_bitcast.b16x8 (I64)
    // skip 2 unless inst_predicate_17
    // 000516: raw_bitcast.b16x8 (I32)
    // skip 2 unless inst_predicate_17
    0x3011,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_19
    // skip 2 unless inst_predicate_19
    0x3013,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_20
    // skip 2 unless inst_predicate_20
    0x3014,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_21
    // skip 2 unless inst_predicate_21
    0x3015,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_22
    // skip 2 unless inst_predicate_22
    0x3016,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_23
    // skip 2 unless inst_predicate_23
    0x3017,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_24
    // skip 2 unless inst_predicate_24
    0x3018,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_25
    // skip 2 unless inst_predicate_25
    0x3019,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_26
    // skip 2 unless inst_predicate_26
    0x301a,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_15
    // skip 2 unless inst_predicate_15
    0x300f,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // stop unless inst_predicate_16
    // stop unless inst_predicate_16
    0x1010,
    // --> [null_fpr#00] and stop
    // --> [null_fpr#00] and stop
    0x020b, 0x0000,
    // end of raw_bitcast.b16x8 (I32)
    // end of raw_bitcast.b16x8 (I64)
    // 000537: x86_pextr.b16x8 (I64)
    // stop unless PredicateView(17)
    // 000537: x86_pextr.i16x8 (I64)
    // stop unless PredicateView(17)
    // 000537: x86_pextr.b16x8 (I32)
    // stop unless PredicateView(17)
    // 000537: x86_pextr.i16x8 (I32)
    // stop unless PredicateView(17)
    0x1030,
    // --> [Mp3r_ib_unsigned_gpr#d15] and stop
    // --> [Mp3r_ib_unsigned_gpr#d15] and stop
    // --> [Mp3r_ib_unsigned_gpr#d15] and stop
    // --> [Mp3r_ib_unsigned_gpr#d15] and stop
    0x0215, 0x0d15,
    // end of x86_pextr.i16x8 (I32)
    // end of x86_pextr.b16x8 (I32)
    // end of x86_pextr.i16x8 (I64)
    // end of x86_pextr.b16x8 (I64)
    // 00053a: x86_pinsr.b16x8 (I64)
    // --> [Mp2r_ib_unsigned_r#5c4] and stop
    // 00053a: x86_pinsr.i16x8 (I64)
    // --> [Mp2r_ib_unsigned_r#5c4] and stop
    // 00053a: x86_pinsr.b16x8 (I32)
    // --> [Mp2r_ib_unsigned_r#5c4] and stop
    // 00053a: x86_pinsr.i16x8 (I32)
    // --> [Mp2r_ib_unsigned_r#5c4] and stop
    0x020f, 0x05c4,
    // end of x86_pinsr.i16x8 (I32)
    // end of x86_pinsr.b16x8 (I32)
    // end of x86_pinsr.i16x8 (I64)
    // end of x86_pinsr.b16x8 (I64)
    // 00053c: raw_bitcast.b32x4 (I64)
    // skip 2 unless inst_predicate_17
    // 00053c: raw_bitcast.b32x4 (I32)
    // skip 2 unless inst_predicate_17
    0x3011,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_18
    // skip 2 unless inst_predicate_18
    0x3012,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_20
    // skip 2 unless inst_predicate_20
    0x3014,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_21
    // skip 2 unless inst_predicate_21
    0x3015,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_22
    // skip 2 unless inst_predicate_22
    0x3016,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_23
    // skip 2 unless inst_predicate_23
    0x3017,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_24
    // skip 2 unless inst_predicate_24
    0x3018,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_25
    // skip 2 unless inst_predicate_25
    0x3019,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_26
    // skip 2 unless inst_predicate_26
    0x301a,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_15
    // skip 2 unless inst_predicate_15
    0x300f,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // stop unless inst_predicate_16
    // stop unless inst_predicate_16
    0x1010,
    // --> [null_fpr#00] and stop
    // --> [null_fpr#00] and stop
    0x020b, 0x0000,
    // end of raw_bitcast.b32x4 (I32)
    // end of raw_bitcast.b32x4 (I64)
    // 00055d: x86_pextr.b32x4 (I64)
    // stop unless PredicateView(17)
    // 00055d: x86_pextr.i32x4 (I64)
    // stop unless PredicateView(17)
    // 00055d: x86_pextr.f32x4 (I64)
    // stop unless PredicateView(17)
    // 00055d: x86_pextr.b32x4 (I32)
    // stop unless PredicateView(17)
    // 00055d: x86_pextr.i32x4 (I32)
    // stop unless PredicateView(17)
    // 00055d: x86_pextr.f32x4 (I32)
    // stop unless PredicateView(17)
    0x1030,
    // --> [Mp3r_ib_unsigned_gpr#d16] and stop
    // --> [Mp3r_ib_unsigned_gpr#d16] and stop
    // --> [Mp3r_ib_unsigned_gpr#d16] and stop
    // --> [Mp3r_ib_unsigned_gpr#d16] and stop
    // --> [Mp3r_ib_unsigned_gpr#d16] and stop
    // --> [Mp3r_ib_unsigned_gpr#d16] and stop
    0x0215, 0x0d16,
    // end of x86_pextr.f32x4 (I32)
    // end of x86_pextr.i32x4 (I32)
    // end of x86_pextr.b32x4 (I32)
    // end of x86_pextr.f32x4 (I64)
    // end of x86_pextr.i32x4 (I64)
    // end of x86_pextr.b32x4 (I64)
    // 000560: x86_pinsr.b32x4 (I64)
    // stop unless PredicateView(17)
    // 000560: x86_pinsr.i32x4 (I64)
    // stop unless PredicateView(17)
    // 000560: x86_pinsr.f32x4 (I64)
    // stop unless PredicateView(17)
    // 000560: x86_pinsr.b32x4 (I32)
    // stop unless PredicateView(17)
    // 000560: x86_pinsr.i32x4 (I32)
    // stop unless PredicateView(17)
    // 000560: x86_pinsr.f32x4 (I32)
    // stop unless PredicateView(17)
    0x1030,
    // --> [Mp3r_ib_unsigned_r#d22] and stop
    // --> [Mp3r_ib_unsigned_r#d22] and stop
    // --> [Mp3r_ib_unsigned_r#d22] and stop
    // --> [Mp3r_ib_unsigned_r#d22] and stop
    // --> [Mp3r_ib_unsigned_r#d22] and stop
    // --> [Mp3r_ib_unsigned_r#d22] and stop
    0x020d, 0x0d22,
    // end of x86_pinsr.f32x4 (I32)
    // end of x86_pinsr.i32x4 (I32)
    // end of x86_pinsr.b32x4 (I32)
    // end of x86_pinsr.f32x4 (I64)
    // end of x86_pinsr.i32x4 (I64)
    // end of x86_pinsr.b32x4 (I64)
    // 000563: x86_pshufd.b32x4 (I64)
    // --> [Mp2r_ib_unsigned_fpr#570] and stop
    // 000563: x86_pshufd.i32x4 (I64)
    // --> [Mp2r_ib_unsigned_fpr#570] and stop
    // 000563: x86_pshufd.f32x4 (I64)
    // --> [Mp2r_ib_unsigned_fpr#570] and stop
    // 000563: x86_pshufd.b32x4 (I32)
    // --> [Mp2r_ib_unsigned_fpr#570] and stop
    // 000563: x86_pshufd.i32x4 (I32)
    // --> [Mp2r_ib_unsigned_fpr#570] and stop
    // 000563: x86_pshufd.f32x4 (I32)
    // --> [Mp2r_ib_unsigned_fpr#570] and stop
    0x0209, 0x0570,
    // end of x86_pshufd.f32x4 (I32)
    // end of x86_pshufd.i32x4 (I32)
    // end of x86_pshufd.b32x4 (I32)
    // end of x86_pshufd.f32x4 (I64)
    // end of x86_pshufd.i32x4 (I64)
    // end of x86_pshufd.b32x4 (I64)
    // 000565: raw_bitcast.b64x2 (I64)
    // skip 2 unless inst_predicate_17
    // 000565: raw_bitcast.b64x2 (I32)
    // skip 2 unless inst_predicate_17
    0x3011,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_18
    // skip 2 unless inst_predicate_18
    0x3012,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_19
    // skip 2 unless inst_predicate_19
    0x3013,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_21
    // skip 2 unless inst_predicate_21
    0x3015,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_22
    // skip 2 unless inst_predicate_22
    0x3016,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_23
    // skip 2 unless inst_predicate_23
    0x3017,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_24
    // skip 2 unless inst_predicate_24
    0x3018,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_25
    // skip 2 unless inst_predicate_25
    0x3019,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_26
    // skip 2 unless inst_predicate_26
    0x301a,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_15
    // skip 2 unless inst_predicate_15
    0x300f,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // stop unless inst_predicate_16
    // stop unless inst_predicate_16
    0x1010,
    // --> [null_fpr#00] and stop
    // --> [null_fpr#00] and stop
    0x020b, 0x0000,
    // end of raw_bitcast.b64x2 (I32)
    // end of raw_bitcast.b64x2 (I64)
    // 000586: x86_pextr.b64x2 (I64)
    // stop unless PredicateView(17)
    // 000586: x86_pextr.i64x2 (I64)
    // stop unless PredicateView(17)
    // 000586: x86_pextr.f64x2 (I64)
    // stop unless PredicateView(17)
    0x1030,
    // --> [RexMp3r_ib_unsigned_gpr#8d16] and stop
    // --> [RexMp3r_ib_unsigned_gpr#8d16] and stop
    // --> [RexMp3r_ib_unsigned_gpr#8d16] and stop
    0x0217, 0x8d16,
    // end of x86_pextr.f64x2 (I64)
    // end of x86_pextr.i64x2 (I64)
    // end of x86_pextr.b64x2 (I64)
    // 000589: x86_pinsr.b64x2 (I64)
    // stop unless PredicateView(17)
    // 000589: x86_pinsr.i64x2 (I64)
    // stop unless PredicateView(17)
    // 000589: x86_pinsr.f64x2 (I64)
    // stop unless PredicateView(17)
    0x1030,
    // --> [RexMp3r_ib_unsigned_r#8d22] and stop
    // --> [RexMp3r_ib_unsigned_r#8d22] and stop
    // --> [RexMp3r_ib_unsigned_r#8d22] and stop
    0x0211, 0x8d22,
    // end of x86_pinsr.f64x2 (I64)
    // end of x86_pinsr.i64x2 (I64)
    // end of x86_pinsr.b64x2 (I64)
    // 00058c: iadd.i8x16 (I64)
    // --> [Mp2fa#5fc] and stop
    // 00058c: iadd.i8x16 (I32)
    // --> [Mp2fa#5fc] and stop
    0x01eb, 0x05fc,
    // end of iadd.i8x16 (I32)
    // end of iadd.i8x16 (I64)
    // 00058e: icmp.i8x16 (I64)
    // skip 2 unless inst_predicate_29
    // 00058e: icmp.i8x16 (I32)
    // skip 2 unless inst_predicate_29
    0x301d,
    // --> [Mp2icscc_fpr#574]
    // --> [Mp2icscc_fpr#574]
    0x0234, 0x0574,
    // stop unless inst_predicate_30
    // stop unless inst_predicate_30
    0x101e,
    // --> [Mp2icscc_fpr#564] and stop
    // --> [Mp2icscc_fpr#564] and stop
    0x0235, 0x0564,
    // end of icmp.i8x16 (I32)
    // end of icmp.i8x16 (I64)
    // 000594: isub.i8x16 (I64)
    // --> [Mp2fa#5f8] and stop
    // 000594: isub.i8x16 (I32)
    // --> [Mp2fa#5f8] and stop
    0x01eb, 0x05f8,
    // end of isub.i8x16 (I32)
    // end of isub.i8x16 (I64)
    // 000596: raw_bitcast.i8x16 (I64)
    // skip 2 unless inst_predicate_17
    // 000596: raw_bitcast.i8x16 (I32)
    // skip 2 unless inst_predicate_17
    0x3011,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_18
    // skip 2 unless inst_predicate_18
    0x3012,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_19
    // skip 2 unless inst_predicate_19
    0x3013,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_20
    // skip 2 unless inst_predicate_20
    0x3014,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_22
    // skip 2 unless inst_predicate_22
    0x3016,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_23
    // skip 2 unless inst_predicate_23
    0x3017,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_24
    // skip 2 unless inst_predicate_24
    0x3018,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_25
    // skip 2 unless inst_predicate_25
    0x3019,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_26
    // skip 2 unless inst_predicate_26
    0x301a,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_15
    // skip 2 unless inst_predicate_15
    0x300f,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // stop unless inst_predicate_16
    // stop unless inst_predicate_16
    0x1010,
    // --> [null_fpr#00] and stop
    // --> [null_fpr#00] and stop
    0x020b, 0x0000,
    // end of raw_bitcast.i8x16 (I32)
    // end of raw_bitcast.i8x16 (I64)
    // 0005b7: sadd_sat.i8x16 (I64)
    // --> [Mp2fa#5ec] and stop
    // 0005b7: sadd_sat.i8x16 (I32)
    // --> [Mp2fa#5ec] and stop
    0x01eb, 0x05ec,
    // end of sadd_sat.i8x16 (I32)
    // end of sadd_sat.i8x16 (I64)
    // 0005b9: ssub_sat.i8x16 (I64)
    // --> [Mp2fa#5e8] and stop
    // 0005b9: ssub_sat.i8x16 (I32)
    // --> [Mp2fa#5e8] and stop
    0x01eb, 0x05e8,
    // end of ssub_sat.i8x16 (I32)
    // end of ssub_sat.i8x16 (I64)
    // 0005bb: uadd_sat.i8x16 (I64)
    // --> [Mp2fa#5dc] and stop
    // 0005bb: uadd_sat.i8x16 (I32)
    // --> [Mp2fa#5dc] and stop
    0x01eb, 0x05dc,
    // end of uadd_sat.i8x16 (I32)
    // end of uadd_sat.i8x16 (I64)
    // 0005bd: usub_sat.i8x16 (I64)
    // --> [Mp2fa#5d8] and stop
    // 0005bd: usub_sat.i8x16 (I32)
    // --> [Mp2fa#5d8] and stop
    0x01eb, 0x05d8,
    // end of usub_sat.i8x16 (I32)
    // end of usub_sat.i8x16 (I64)
    // 0005bf: x86_pmaxs.i8x16 (I64)
    // stop unless PredicateView(17)
    // 0005bf: x86_pmaxs.i8x16 (I32)
    // stop unless PredicateView(17)
    0x1030,
    // --> [Mp3fa#93c] and stop
    // --> [Mp3fa#93c] and stop
    0x0207, 0x093c,
    // end of x86_pmaxs.i8x16 (I32)
    // end of x86_pmaxs.i8x16 (I64)
    // 0005c2: x86_pmaxu.i8x16 (I64)
    // --> [Mp2fa#5de] and stop
    // 0005c2: x86_pmaxu.i8x16 (I32)
    // --> [Mp2fa#5de] and stop
    0x01eb, 0x05de,
    // end of x86_pmaxu.i8x16 (I32)
    // end of x86_pmaxu.i8x16 (I64)
    // 0005c4: x86_pmins.i8x16 (I64)
    // stop unless PredicateView(17)
    // 0005c4: x86_pmins.i8x16 (I32)
    // stop unless PredicateView(17)
    0x1030,
    // --> [Mp3fa#938] and stop
    // --> [Mp3fa#938] and stop
    0x0207, 0x0938,
    // end of x86_pmins.i8x16 (I32)
    // end of x86_pmins.i8x16 (I64)
    // 0005c7: x86_pminu.i8x16 (I64)
    // --> [Mp2fa#5da] and stop
    // 0005c7: x86_pminu.i8x16 (I32)
    // --> [Mp2fa#5da] and stop
    0x01eb, 0x05da,
    // end of x86_pminu.i8x16 (I32)
    // end of x86_pminu.i8x16 (I64)
    // 0005c9: iadd.i16x8 (I64)
    // --> [Mp2fa#5fd] and stop
    // 0005c9: iadd.i16x8 (I32)
    // --> [Mp2fa#5fd] and stop
    0x01eb, 0x05fd,
    // end of iadd.i16x8 (I32)
    // end of iadd.i16x8 (I64)
    // 0005cb: icmp.i16x8 (I64)
    // skip 2 unless inst_predicate_29
    // 0005cb: icmp.i16x8 (I32)
    // skip 2 unless inst_predicate_29
    0x301d,
    // --> [Mp2icscc_fpr#575]
    // --> [Mp2icscc_fpr#575]
    0x0234, 0x0575,
    // stop unless inst_predicate_30
    // stop unless inst_predicate_30
    0x101e,
    // --> [Mp2icscc_fpr#565] and stop
    // --> [Mp2icscc_fpr#565] and stop
    0x0235, 0x0565,
    // end of icmp.i16x8 (I32)
    // end of icmp.i16x8 (I64)
    // 0005d1: imul.i16x8 (I64)
    // --> [Mp2fa#5d5] and stop
    // 0005d1: imul.i16x8 (I32)
    // --> [Mp2fa#5d5] and stop
    0x01eb, 0x05d5,
    // end of imul.i16x8 (I32)
    // end of imul.i16x8 (I64)
    // 0005d3: isub.i16x8 (I64)
    // --> [Mp2fa#5f9] and stop
    // 0005d3: isub.i16x8 (I32)
    // --> [Mp2fa#5f9] and stop
    0x01eb, 0x05f9,
    // end of isub.i16x8 (I32)
    // end of isub.i16x8 (I64)
    // 0005d5: raw_bitcast.i16x8 (I64)
    // skip 2 unless inst_predicate_17
    // 0005d5: raw_bitcast.i16x8 (I32)
    // skip 2 unless inst_predicate_17
    0x3011,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_18
    // skip 2 unless inst_predicate_18
    0x3012,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_19
    // skip 2 unless inst_predicate_19
    0x3013,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_20
    // skip 2 unless inst_predicate_20
    0x3014,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_21
    // skip 2 unless inst_predicate_21
    0x3015,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_23
    // skip 2 unless inst_predicate_23
    0x3017,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_24
    // skip 2 unless inst_predicate_24
    0x3018,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_25
    // skip 2 unless inst_predicate_25
    0x3019,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_26
    // skip 2 unless inst_predicate_26
    0x301a,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_15
    // skip 2 unless inst_predicate_15
    0x300f,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // stop unless inst_predicate_16
    // stop unless inst_predicate_16
    0x1010,
    // --> [null_fpr#00] and stop
    // --> [null_fpr#00] and stop
    0x020b, 0x0000,
    // end of raw_bitcast.i16x8 (I32)
    // end of raw_bitcast.i16x8 (I64)
    // 0005f6: sadd_sat.i16x8 (I64)
    // --> [Mp2fa#5ed] and stop
    // 0005f6: sadd_sat.i16x8 (I32)
    // --> [Mp2fa#5ed] and stop
    0x01eb, 0x05ed,
    // end of sadd_sat.i16x8 (I32)
    // end of sadd_sat.i16x8 (I64)
    // 0005f8: ssub_sat.i16x8 (I64)
    // --> [Mp2fa#5e9] and stop
    // 0005f8: ssub_sat.i16x8 (I32)
    // --> [Mp2fa#5e9] and stop
    0x01eb, 0x05e9,
    // end of ssub_sat.i16x8 (I32)
    // end of ssub_sat.i16x8 (I64)
    // 0005fa: uadd_sat.i16x8 (I64)
    // --> [Mp2fa#5dd] and stop
    // 0005fa: uadd_sat.i16x8 (I32)
    // --> [Mp2fa#5dd] and stop
    0x01eb, 0x05dd,
    // end of uadd_sat.i16x8 (I32)
    // end of uadd_sat.i16x8 (I64)
    // 0005fc: usub_sat.i16x8 (I64)
    // --> [Mp2fa#5d9] and stop
    // 0005fc: usub_sat.i16x8 (I32)
    // --> [Mp2fa#5d9] and stop
    0x01eb, 0x05d9,
    // end of usub_sat.i16x8 (I32)
    // end of usub_sat.i16x8 (I64)
    // 0005fe: x86_pmaxs.i16x8 (I64)
    // --> [Mp2fa#5ee] and stop
    // 0005fe: x86_pmaxs.i16x8 (I32)
    // --> [Mp2fa#5ee] and stop
    0x01eb, 0x05ee,
    // end of x86_pmaxs.i16x8 (I32)
    // end of x86_pmaxs.i16x8 (I64)
    // 000600: x86_pmaxu.i16x8 (I64)
    // stop unless PredicateView(17)
    // 000600: x86_pmaxu.i16x8 (I32)
    // stop unless PredicateView(17)
    0x1030,
    // --> [Mp3fa#93e] and stop
    // --> [Mp3fa#93e] and stop
    0x0207, 0x093e,
    // end of x86_pmaxu.i16x8 (I32)
    // end of x86_pmaxu.i16x8 (I64)
    // 000603: x86_pmins.i16x8 (I64)
    // --> [Mp2fa#5ea] and stop
    // 000603: x86_pmins.i16x8 (I32)
    // --> [Mp2fa#5ea] and stop
    0x01eb, 0x05ea,
    // end of x86_pmins.i16x8 (I32)
    // end of x86_pmins.i16x8 (I64)
    // 000605: x86_pminu.i16x8 (I64)
    // stop unless PredicateView(17)
    // 000605: x86_pminu.i16x8 (I32)
    // stop unless PredicateView(17)
    0x1030,
    // --> [Mp3fa#93a] and stop
    // --> [Mp3fa#93a] and stop
    0x0207, 0x093a,
    // end of x86_pminu.i16x8 (I32)
    // end of x86_pminu.i16x8 (I64)
    // 000608: x86_psll.i16x8 (I64)
    // --> [Mp2fa#5f1] and stop
    // 000608: x86_psll.i16x8 (I32)
    // --> [Mp2fa#5f1] and stop
    0x01eb, 0x05f1,
    // end of x86_psll.i16x8 (I32)
    // end of x86_psll.i16x8 (I64)
    // 00060a: x86_psra.i16x8 (I64)
    // --> [Mp2fa#5e1] and stop
    // 00060a: x86_psra.i16x8 (I32)
    // --> [Mp2fa#5e1] and stop
    0x01eb, 0x05e1,
    // end of x86_psra.i16x8 (I32)
    // end of x86_psra.i16x8 (I64)
    // 00060c: x86_psrl.i16x8 (I64)
    // --> [Mp2fa#5d1] and stop
    // 00060c: x86_psrl.i16x8 (I32)
    // --> [Mp2fa#5d1] and stop
    0x01eb, 0x05d1,
    // end of x86_psrl.i16x8 (I32)
    // end of x86_psrl.i16x8 (I64)
    // 00060e: iadd.i32x4 (I64)
    // --> [Mp2fa#5fe] and stop
    // 00060e: iadd.i32x4 (I32)
    // --> [Mp2fa#5fe] and stop
    0x01eb, 0x05fe,
    // end of iadd.i32x4 (I32)
    // end of iadd.i32x4 (I64)
    // 000610: icmp.i32x4 (I64)
    // skip 2 unless inst_predicate_29
    // 000610: icmp.i32x4 (I32)
    // skip 2 unless inst_predicate_29
    0x301d,
    // --> [Mp2icscc_fpr#576]
    // --> [Mp2icscc_fpr#576]
    0x0234, 0x0576,
    // stop unless inst_predicate_30
    // stop unless inst_predicate_30
    0x101e,
    // --> [Mp2icscc_fpr#566] and stop
    // --> [Mp2icscc_fpr#566] and stop
    0x0235, 0x0566,
    // end of icmp.i32x4 (I32)
    // end of icmp.i32x4 (I64)
    // 000616: imul.i32x4 (I64)
    // stop unless PredicateView(17)
    // 000616: imul.i32x4 (I32)
    // stop unless PredicateView(17)
    0x1030,
    // --> [Mp3fa#940] and stop
    // --> [Mp3fa#940] and stop
    0x0207, 0x0940,
    // end of imul.i32x4 (I32)
    // end of imul.i32x4 (I64)
    // 000619: isub.i32x4 (I64)
    // --> [Mp2fa#5fa] and stop
    // 000619: isub.i32x4 (I32)
    // --> [Mp2fa#5fa] and stop
    0x01eb, 0x05fa,
    // end of isub.i32x4 (I32)
    // end of isub.i32x4 (I64)
    // 00061b: raw_bitcast.i32x4 (I64)
    // skip 2 unless inst_predicate_17
    // 00061b: raw_bitcast.i32x4 (I32)
    // skip 2 unless inst_predicate_17
    0x3011,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_18
    // skip 2 unless inst_predicate_18
    0x3012,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_19
    // skip 2 unless inst_predicate_19
    0x3013,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_20
    // skip 2 unless inst_predicate_20
    0x3014,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_21
    // skip 2 unless inst_predicate_21
    0x3015,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_22
    // skip 2 unless inst_predicate_22
    0x3016,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_24
    // skip 2 unless inst_predicate_24
    0x3018,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_25
    // skip 2 unless inst_predicate_25
    0x3019,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_26
    // skip 2 unless inst_predicate_26
    0x301a,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_15
    // skip 2 unless inst_predicate_15
    0x300f,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // stop unless inst_predicate_16
    // stop unless inst_predicate_16
    0x1010,
    // --> [null_fpr#00] and stop
    // --> [null_fpr#00] and stop
    0x020b, 0x0000,
    // end of raw_bitcast.i32x4 (I32)
    // end of raw_bitcast.i32x4 (I64)
    // 00063c: x86_pmaxs.i32x4 (I64)
    // stop unless PredicateView(17)
    // 00063c: x86_pmaxs.i32x4 (I32)
    // stop unless PredicateView(17)
    0x1030,
    // --> [Mp3fa#93d] and stop
    // --> [Mp3fa#93d] and stop
    0x0207, 0x093d,
    // end of x86_pmaxs.i32x4 (I32)
    // end of x86_pmaxs.i32x4 (I64)
    // 00063f: x86_pmaxu.i32x4 (I64)
    // stop unless PredicateView(17)
    // 00063f: x86_pmaxu.i32x4 (I32)
    // stop unless PredicateView(17)
    0x1030,
    // --> [Mp3fa#93f] and stop
    // --> [Mp3fa#93f] and stop
    0x0207, 0x093f,
    // end of x86_pmaxu.i32x4 (I32)
    // end of x86_pmaxu.i32x4 (I64)
    // 000642: x86_pmins.i32x4 (I64)
    // stop unless PredicateView(17)
    // 000642: x86_pmins.i32x4 (I32)
    // stop unless PredicateView(17)
    0x1030,
    // --> [Mp3fa#939] and stop
    // --> [Mp3fa#939] and stop
    0x0207, 0x0939,
    // end of x86_pmins.i32x4 (I32)
    // end of x86_pmins.i32x4 (I64)
    // 000645: x86_pminu.i32x4 (I64)
    // stop unless PredicateView(17)
    // 000645: x86_pminu.i32x4 (I32)
    // stop unless PredicateView(17)
    0x1030,
    // --> [Mp3fa#93b] and stop
    // --> [Mp3fa#93b] and stop
    0x0207, 0x093b,
    // end of x86_pminu.i32x4 (I32)
    // end of x86_pminu.i32x4 (I64)
    // 000648: x86_psll.i32x4 (I64)
    // --> [Mp2fa#5f2] and stop
    // 000648: x86_psll.i32x4 (I32)
    // --> [Mp2fa#5f2] and stop
    0x01eb, 0x05f2,
    // end of x86_psll.i32x4 (I32)
    // end of x86_psll.i32x4 (I64)
    // 00064a: x86_psra.i32x4 (I64)
    // --> [Mp2fa#5e2] and stop
    // 00064a: x86_psra.i32x4 (I32)
    // --> [Mp2fa#5e2] and stop
    0x01eb, 0x05e2,
    // end of x86_psra.i32x4 (I32)
    // end of x86_psra.i32x4 (I64)
    // 00064c: x86_psrl.i32x4 (I64)
    // --> [Mp2fa#5d2] and stop
    // 00064c: x86_psrl.i32x4 (I32)
    // --> [Mp2fa#5d2] and stop
    0x01eb, 0x05d2,
    // end of x86_psrl.i32x4 (I32)
    // end of x86_psrl.i32x4 (I64)
    // 00064e: bitcast.i64x2 (I64)
    // skip 4 unless inst_predicate_10
    0x500a,
    // --> [RexMp2frurm#56e]
    0x01d0, 0x056e,
    // --> [Mp2frurm#56e]
    0x01ce, 0x056e,
    // stop unless inst_predicate_11
    0x100b,
    // --> [RexMp2frurm#856e] and stop
    0x01d1, 0x856e,
    // end of bitcast.i64x2 (I64)
    // 000656: iadd.i64x2 (I64)
    // --> [Mp2fa#5d4] and stop
    // 000656: iadd.i64x2 (I32)
    // --> [Mp2fa#5d4] and stop
    0x01eb, 0x05d4,
    // end of iadd.i64x2 (I32)
    // end of iadd.i64x2 (I64)
    // 000658: icmp.i64x2 (I64)
    // skip 3 unless PredicateView(17)
    // 000658: icmp.i64x2 (I32)
    // skip 3 unless PredicateView(17)
    0x4030,
    // skip 2 unless inst_predicate_29
    // skip 2 unless inst_predicate_29
    0x301d,
    // --> [Mp3icscc_fpr#929]
    // --> [Mp3icscc_fpr#929]
    0x0236, 0x0929,
    // stop unless PredicateView(19)
    // stop unless PredicateView(19)
    0x1032,
    // stop unless inst_predicate_30
    // stop unless inst_predicate_30
    0x101e,
    // --> [Mp3icscc_fpr#937] and stop
    // --> [Mp3icscc_fpr#937] and stop
    0x0237, 0x0937,
    // end of icmp.i64x2 (I32)
    // end of icmp.i64x2 (I64)
    // 000660: isub.i64x2 (I64)
    // --> [Mp2fa#5fb] and stop
    // 000660: isub.i64x2 (I32)
    // --> [Mp2fa#5fb] and stop
    0x01eb, 0x05fb,
    // end of isub.i64x2 (I32)
    // end of isub.i64x2 (I64)
    // 000662: raw_bitcast.i64x2 (I64)
    // skip 2 unless inst_predicate_17
    // 000662: raw_bitcast.i64x2 (I32)
    // skip 2 unless inst_predicate_17
    0x3011,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_18
    // skip 2 unless inst_predicate_18
    0x3012,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_19
    // skip 2 unless inst_predicate_19
    0x3013,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_20
    // skip 2 unless inst_predicate_20
    0x3014,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_21
    // skip 2 unless inst_predicate_21
    0x3015,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_22
    // skip 2 unless inst_predicate_22
    0x3016,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_23
    // skip 2 unless inst_predicate_23
    0x3017,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_25
    // skip 2 unless inst_predicate_25
    0x3019,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_26
    // skip 2 unless inst_predicate_26
    0x301a,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_15
    // skip 2 unless inst_predicate_15
    0x300f,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // stop unless inst_predicate_16
    // stop unless inst_predicate_16
    0x1010,
    // --> [null_fpr#00] and stop
    // --> [null_fpr#00] and stop
    0x020b, 0x0000,
    // end of raw_bitcast.i64x2 (I32)
    // end of raw_bitcast.i64x2 (I64)
    // 000683: x86_psll.i64x2 (I64)
    // --> [Mp2fa#5f3] and stop
    // 000683: x86_psll.i64x2 (I32)
    // --> [Mp2fa#5f3] and stop
    0x01eb, 0x05f3,
    // end of x86_psll.i64x2 (I32)
    // end of x86_psll.i64x2 (I64)
    // 000685: x86_psrl.i64x2 (I64)
    // --> [Mp2fa#5d3] and stop
    // 000685: x86_psrl.i64x2 (I32)
    // --> [Mp2fa#5d3] and stop
    0x01eb, 0x05d3,
    // end of x86_psrl.i64x2 (I32)
    // end of x86_psrl.i64x2 (I64)
    // 000687: raw_bitcast.f32x4 (I64)
    // skip 2 unless inst_predicate_17
    // 000687: raw_bitcast.f32x4 (I32)
    // skip 2 unless inst_predicate_17
    0x3011,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_18
    // skip 2 unless inst_predicate_18
    0x3012,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_19
    // skip 2 unless inst_predicate_19
    0x3013,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_20
    // skip 2 unless inst_predicate_20
    0x3014,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_21
    // skip 2 unless inst_predicate_21
    0x3015,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_22
    // skip 2 unless inst_predicate_22
    0x3016,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_23
    // skip 2 unless inst_predicate_23
    0x3017,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_24
    // skip 2 unless inst_predicate_24
    0x3018,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_26
    // skip 2 unless inst_predicate_26
    0x301a,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_15
    // skip 2 unless inst_predicate_15
    0x300f,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // stop unless inst_predicate_16
    // stop unless inst_predicate_16
    0x1010,
    // --> [null_fpr#00] and stop
    // --> [null_fpr#00] and stop
    0x020b, 0x0000,
    // end of raw_bitcast.f32x4 (I32)
    // end of raw_bitcast.f32x4 (I64)
    // 0006a8: x86_insertps.f32x4 (I64)
    // stop unless PredicateView(17)
    // 0006a8: x86_insertps.f32x4 (I32)
    // stop unless PredicateView(17)
    0x1030,
    // --> [Mp3fa_ib#d21] and stop
    // --> [Mp3fa_ib#d21] and stop
    0x0213, 0x0d21,
    // end of x86_insertps.f32x4 (I32)
    // end of x86_insertps.f32x4 (I64)
    // 0006ab: raw_bitcast.f64x2 (I64)
    // skip 2 unless inst_predicate_17
    // 0006ab: raw_bitcast.f64x2 (I32)
    // skip 2 unless inst_predicate_17
    0x3011,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_18
    // skip 2 unless inst_predicate_18
    0x3012,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_19
    // skip 2 unless inst_predicate_19
    0x3013,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_20
    // skip 2 unless inst_predicate_20
    0x3014,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_21
    // skip 2 unless inst_predicate_21
    0x3015,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_22
    // skip 2 unless inst_predicate_22
    0x3016,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_23
    // skip 2 unless inst_predicate_23
    0x3017,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_24
    // skip 2 unless inst_predicate_24
    0x3018,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_25
    // skip 2 unless inst_predicate_25
    0x3019,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // skip 2 unless inst_predicate_15
    // skip 2 unless inst_predicate_15
    0x300f,
    // --> [null_fpr#00]
    // --> [null_fpr#00]
    0x020a, 0x0000,
    // stop unless inst_predicate_16
    // stop unless inst_predicate_16
    0x1010,
    // --> [null_fpr#00] and stop
    // --> [null_fpr#00] and stop
    0x020b, 0x0000,
    // end of raw_bitcast.f64x2 (I32)
    // end of raw_bitcast.f64x2 (I64)
    // 0006cc: x86_movlhps.f64x2 (I64)
    // --> [Op2fa#416] and stop
    // 0006cc: x86_movlhps.f64x2 (I32)
    // --> [Op2fa#416] and stop
    0x01ef, 0x0416,
    // end of x86_movlhps.f64x2 (I32)
    // end of x86_movlhps.f64x2 (I64)
    // 0006ce: x86_movsd.f64x2 (I64)
    // --> [Mp2fa#710] and stop
    // 0006ce: x86_movsd.f64x2 (I32)
    // --> [Mp2fa#710] and stop
    0x01eb, 0x0710,
    // end of x86_movsd.f64x2 (I32)
    // end of x86_movsd.f64x2 (I64)
    // 0006d0: adjust_sp_down.i32 (I32)
    // --> [Op1adjustsp#29] and stop
    0x00e9, 0x0029,
    // end of adjust_sp_down.i32 (I32)
    // 0006d2: band_imm.i32 (I32)
    // --> [Op1r_ib#4083]
    0x002c, 0x4083,
    // --> [Op1r_id#4081] and stop
    0x0031, 0x4081,
    // end of band_imm.i32 (I32)
    // 0006d6: bint.i32 (I32)
    // skip 2 unless inst_predicate_7
    // 0006d6: bint.i8 (I32)
    // skip 2 unless inst_predicate_7
    // 0006d6: bint.i16 (I32)
    // skip 2 unless inst_predicate_7
    0x3007,
    // --> [Op2urm_noflags_abcd#4b6]
    // --> [Op2urm_noflags_abcd#4b6]
    // --> [Op2urm_noflags_abcd#4b6]
    0x01be, 0x04b6,
    // stop unless inst_predicate_8
    // stop unless inst_predicate_8
    // stop unless inst_predicate_8
    0x1008,
    // --> [Op2urm_noflags_abcd#4b6] and stop
    // --> [Op2urm_noflags_abcd#4b6] and stop
    // --> [Op2urm_noflags_abcd#4b6] and stop
    0x01bf, 0x04b6,
    // end of bint.i16 (I32)
    // end of bint.i8 (I32)
    // end of bint.i32 (I32)
    // 0006dc: bitcast.i32 (I32)
    // stop unless inst_predicate_15
    0x100f,
    // --> [Mp2rfumr#57e] and stop
    0x01d3, 0x057e,
    // end of bitcast.i32 (I32)
    // 0006df: bor_imm.i32 (I32)
    // --> [Op1r_ib#1083]
    0x002c, 0x1083,
    // --> [Op1r_id#1081] and stop
    0x0031, 0x1081,
    // end of bor_imm.i32 (I32)
    // 0006e3: brnz.i32 (I32)
    // --> [Op1tjccb#75]
    0x016a, 0x0075,
    // --> [Op1tjccd#85] and stop
    0x016f, 0x0085,
    // end of brnz.i32 (I32)
    // 0006e7: brz.i32 (I32)
    // --> [Op1tjccb#74]
    0x016a, 0x0074,
    // --> [Op1tjccd#84] and stop
    0x016f, 0x0084,
    // end of brz.i32 (I32)
    // 0006eb: bxor_imm.i32 (I32)
    // --> [Op1r_ib#6083]
    0x002c, 0x6083,
    // --> [Op1r_id#6081] and stop
    0x0031, 0x6081,
    // end of bxor_imm.i32 (I32)
    // 0006ef: clz.i32 (I32)
    // stop unless PredicateView(14)
    0x102d,
    // --> [Mp2urm#6bd] and stop
    0x0049, 0x06bd,
    // end of clz.i32 (I32)
    // 0006f2: copy_to_ssa.i32 (I32)
    // --> [Op1umr_reg_to_ssa#89] and stop
    // 0006f2: copy_to_ssa.b1 (I32)
    // --> [Op1umr_reg_to_ssa#89] and stop
    // 0006f2: copy_to_ssa.r32 (I32)
    // --> [Op1umr_reg_to_ssa#89] and stop
    // 0006f2: copy_to_ssa.i8 (I32)
    // --> [Op1umr_reg_to_ssa#89] and stop
    // 0006f2: copy_to_ssa.i16 (I32)
    // --> [Op1umr_reg_to_ssa#89] and stop
    0x00df, 0x0089,
    // end of copy_to_ssa.i16 (I32)
    // end of copy_to_ssa.i8 (I32)
    // end of copy_to_ssa.r32 (I32)
    // end of copy_to_ssa.b1 (I32)
    // end of copy_to_ssa.i32 (I32)
    // 0006f4: ctz.i32 (I32)
    // stop unless PredicateView(13)
    0x102c,
    // --> [Mp2urm#6bc] and stop
    0x0049, 0x06bc,
    // end of ctz.i32 (I32)
    // 0006f7: func_addr.i32 (I32)
    // skip 2 unless PredicateView(11)
    0x302a,
    // --> [Op1fnaddr4#b8]
    0x0134, 0x00b8,
    // stop unless PredicateView(9)
    0x1028,
    // --> [Op1allones_fnaddr4#b8] and stop
    0x0139, 0x00b8,
    // end of func_addr.i32 (I32)
    // 0006fd: iadd_imm.i32 (I32)
    // --> [Op1r_ib#83]
    0x002c, 0x0083,
    // --> [Op1r_id#81] and stop
    0x0031, 0x0081,
    // end of iadd_imm.i32 (I32)
    // 000701: icmp_imm.i32 (I32)
    // --> [Op1icscc_ib#7083]
    0x0194, 0x7083,
    // --> [Op1icscc_id#7081] and stop
    0x0199, 0x7081,
    // end of icmp_imm.i32 (I32)
    // 000705: iconst.i32 (I32)
    // --> [Op1pu_id#b8]
    0x0034, 0x00b8,
    // stop unless inst_predicate_1
    // 000707: iconst.i16 (I32)
    // stop unless inst_predicate_1
    0x1001,
    // --> [Op1u_id_z#31] and stop
    // --> [Op1u_id_z#31] and stop
    0x0041, 0x0031,
    // end of iconst.i16 (I32)
    // end of iconst.i32 (I32)
    // 00070a: ifcmp_imm.i32 (I32)
    // --> [Op1rcmp_ib#7083]
    0x01a0, 0x7083,
    // --> [Op1rcmp_id#7081] and stop
    0x01a5, 0x7081,
    // end of ifcmp_imm.i32 (I32)
    // 00070e: ifcmp_sp.i32 (I32)
    // --> [Op1rcmp_sp#39] and stop
    0x01a9, 0x0039,
    // end of ifcmp_sp.i32 (I32)
    // 000710: istore16.i32 (I32)
    // --> [Mp1st#189]
    0x008c, 0x0189,
    // --> [Mp1stDisp8#189]
    0x0094, 0x0189,
    // --> [Mp1stDisp32#189] and stop
    0x009d, 0x0189,
    // end of istore16.i32 (I32)
    // 000716: istore16_complex.i32 (I32)
    // stop unless inst_predicate_3
    0x1003,
    // --> [Mp1stWithIndex#189]
    0x0068, 0x0189,
    // --> [Mp1stWithIndexDisp8#189]
    0x0070, 0x0189,
    // --> [Mp1stWithIndexDisp32#189] and stop
    0x0079, 0x0189,
    // end of istore16_complex.i32 (I32)
    // 00071d: istore8.i32 (I32)
    // --> [Op1st_abcd#88]
    0x00a0, 0x0088,
    // --> [Op1stDisp8_abcd#88]
    0x00a2, 0x0088,
    // --> [Op1stDisp32_abcd#88] and stop
    0x00a5, 0x0088,
    // end of istore8.i32 (I32)
    // 000723: istore8_complex.i32 (I32)
    // stop unless inst_predicate_3
    0x1003,
    // --> [Op1stWithIndex_abcd#88]
    0x007c, 0x0088,
    // --> [Op1stWithIndexDisp8_abcd#88]
    0x0080, 0x0088,
    // --> [Op1stWithIndexDisp32_abcd#88] and stop
    0x0085, 0x0088,
    // end of istore8_complex.i32 (I32)
    // 00072a: jump_table_base.i32 (I32)
    // --> [Op1jt_base#8d] and stop
    0x0183, 0x008d,
    // end of jump_table_base.i32 (I32)
    // 00072c: jump_table_entry.i32 (I32)
    // --> [Op1jt_entry#8b] and stop
    0x017f, 0x008b,
    // end of jump_table_entry.i32 (I32)
    // 00072e: load.i32 (I32)
    // --> [Op1ld#8b]
    0x00ae, 0x008b,
    // --> [Op1ldDisp8#8b]
    0x00b6, 0x008b,
    // --> [Op1ldDisp32#8b] and stop
    0x00bf, 0x008b,
    // end of load.i32 (I32)
    // 000734: load_complex.i32 (I32)
    // stop unless inst_predicate_2
    0x1002,
    // --> [Op1ldWithIndex#8b]
    0x004c, 0x008b,
    // --> [Op1ldWithIndexDisp8#8b]
    0x0054, 0x008b,
    // --> [Op1ldWithIndexDisp32#8b] and stop
    0x005d, 0x008b,
    // end of load_complex.i32 (I32)
    // 00073b: popcnt.i32 (I32)
    // stop unless PredicateView(15)
    0x102e,
    // --> [Mp2urm#6b8] and stop
    0x0049, 0x06b8,
    // end of popcnt.i32 (I32)
    // 00073e: sextend.i32 (I32)
    // skip 2 unless inst_predicate_12
    0x300c,
    // --> [Op2urm_noflags_abcd#4be]
    0x01be, 0x04be,
    // stop unless inst_predicate_9
    0x1009,
    // --> [Op2urm_noflags#4bf] and stop
    0x01c3, 0x04bf,
    // end of sextend.i32 (I32)
    // 000744: sload16.i32 (I32)
    // --> [Op2ld#4bf]
    0x00b2, 0x04bf,
    // --> [Op2ldDisp8#4bf]
    0x00ba, 0x04bf,
    // --> [Op2ldDisp32#4bf] and stop
    0x00c3, 0x04bf,
    // end of sload16.i32 (I32)
    // 00074a: sload16_complex.i32 (I32)
    // stop unless inst_predicate_2
    0x1002,
    // --> [Op2ldWithIndex#4bf]
    0x0050, 0x04bf,
    // --> [Op2ldWithIndexDisp8#4bf]
    0x0058, 0x04bf,
    // --> [Op2ldWithIndexDisp32#4bf] and stop
    0x0061, 0x04bf,
    // end of sload16_complex.i32 (I32)
    // 000751: sload8.i32 (I32)
    // --> [Op2ld#4be]
    0x00b2, 0x04be,
    // --> [Op2ldDisp8#4be]
    0x00ba, 0x04be,
    // --> [Op2ldDisp32#4be] and stop
    0x00c3, 0x04be,
    // end of sload8.i32 (I32)
    // 000757: sload8_complex.i32 (I32)
    // stop unless inst_predicate_2
    0x1002,
    // --> [Op2ldWithIndex#4be]
    0x0050, 0x04be,
    // --> [Op2ldWithIndexDisp8#4be]
    0x0058, 0x04be,
    // --> [Op2ldWithIndexDisp32#4be] and stop
    0x0061, 0x04be,
    // end of sload8_complex.i32 (I32)
    // 00075e: stack_addr.i32 (I32)
    // --> [Op1spaddr4_id#8d] and stop
    0x0149, 0x008d,
    // end of stack_addr.i32 (I32)
    // 000760: store.i32 (I32)
    // --> [Op1st#89]
    0x0088, 0x0089,
    // --> [Op1stDisp8#89]
    0x0090, 0x0089,
    // --> [Op1stDisp32#89] and stop
    0x0099, 0x0089,
    // end of store.i32 (I32)
    // 000766: store_complex.i32 (I32)
    // stop unless inst_predicate_3
    0x1003,
    // --> [Op1stWithIndex#89]
    0x0064, 0x0089,
    // --> [Op1stWithIndexDisp8#89]
    0x006c, 0x0089,
    // --> [Op1stWithIndexDisp32#89] and stop
    0x0075, 0x0089,
    // end of store_complex.i32 (I32)
    // 00076d: symbol_value.i32 (I32)
    // stop unless PredicateView(12)
    0x102b,
    // --> [Op1gvaddr4#b8] and stop
    0x0141, 0x00b8,
    // end of symbol_value.i32 (I32)
    // 000770: uextend.i32 (I32)
    // skip 2 unless inst_predicate_12
    0x300c,
    // --> [Op2urm_noflags_abcd#4b6]
    0x01be, 0x04b6,
    // stop unless inst_predicate_9
    0x1009,
    // --> [Op2urm_noflags#4b7] and stop
    0x01c3, 0x04b7,
    // end of uextend.i32 (I32)
    // 000776: uload16.i32 (I32)
    // --> [Op2ld#4b7]
    0x00b2, 0x04b7,
    // --> [Op2ldDisp8#4b7]
    0x00ba, 0x04b7,
    // --> [Op2ldDisp32#4b7] and stop
    0x00c3, 0x04b7,
    // end of uload16.i32 (I32)
    // 00077c: uload16_complex.i32 (I32)
    // stop unless inst_predicate_2
    0x1002,
    // --> [Op2ldWithIndex#4b7]
    0x0050, 0x04b7,
    // --> [Op2ldWithIndexDisp8#4b7]
    0x0058, 0x04b7,
    // --> [Op2ldWithIndexDisp32#4b7] and stop
    0x0061, 0x04b7,
    // end of uload16_complex.i32 (I32)
    // 000783: uload8.i32 (I32)
    // --> [Op2ld#4b6]
    0x00b2, 0x04b6,
    // --> [Op2ldDisp8#4b6]
    0x00ba, 0x04b6,
    // --> [Op2ldDisp32#4b6] and stop
    0x00c3, 0x04b6,
    // end of uload8.i32 (I32)
    // 000789: uload8_complex.i32 (I32)
    // stop unless inst_predicate_2
    0x1002,
    // --> [Op2ldWithIndex#4b6]
    0x0050, 0x04b6,
    // --> [Op2ldWithIndexDisp8#4b6]
    0x0058, 0x04b6,
    // --> [Op2ldWithIndexDisp32#4b6] and stop
    0x0061, 0x04b6,
    // end of uload8_complex.i32 (I32)
    // 000790: x86_cvtt2si.i32 (I32)
    // skip 2 unless inst_predicate_15
    0x300f,
    // --> [Mp2rfurm#62c]
    0x01e2, 0x062c,
    // stop unless inst_predicate_16
    0x1010,
    // --> [Mp2rfurm#72c] and stop
    0x01e3, 0x072c,
    // end of x86_cvtt2si.i32 (I32)
    // 000796: brnz.b1 (I32)
    // --> [Op1t8jccd_long#85]
    0x0172, 0x0085,
    // --> [Op1t8jccb_abcd#75]
    0x0174, 0x0075,
    // --> [Op1t8jccd_abcd#85] and stop
    0x0179, 0x0085,
    // end of brnz.b1 (I32)
    // 00079c: brz.b1 (I32)
    // --> [Op1t8jccd_long#84]
    0x0172, 0x0084,
    // --> [Op1t8jccb_abcd#74]
    0x0174, 0x0074,
    // --> [Op1t8jccd_abcd#84] and stop
    0x0179, 0x0084,
    // end of brz.b1 (I32)
    // 0007a2: is_null.r32 (I32)
    // --> [Op1is_zero#85] and stop
    0x023d, 0x0085,
    // end of is_null.r32 (I32)
    // 0007a4: iconst.i8 (I32)
    // stop unless inst_predicate_1
    0x1001,
    // --> [Op1u_id_z#30] and stop
    0x0041, 0x0030,
    // end of iconst.i8 (I32)
    // 0007a7: ireduce.i8 (I32)
    // skip 2 unless inst_predicate_9
    0x3009,
    // --> [null#00]
    0x01c0, 0x0000,
    // stop unless inst_predicate_10
    // 0007aa: ireduce.i16 (I32)
    // stop unless inst_predicate_10
    0x100a,
    // --> [null#00] and stop
    // --> [null#00] and stop
    0x01c1, 0x0000,
    // end of ireduce.i16 (I32)
    // end of ireduce.i8 (I32)
    // 0007ad: regmove.i8 (I32)
    // --> [Op1rmov#89]
    0x0028, 0x0089,
    // --> [Op1rmov#89] and stop
    0x0029, 0x0089,
    // end of regmove.i8 (I32)
    // 0007b1: ceil.f64 (I32)
    // stop unless PredicateView(16)
    // 0007b1: floor.f64 (I32)
    // stop unless PredicateView(16)
    // 0007b1: nearest.f64 (I32)
    // stop unless PredicateView(16)
    // 0007b1: trunc.f64 (I32)
    // stop unless PredicateView(16)
    0x102f,
    // --> [Mp3furmi_rnd#d0b] and stop
    // --> [Mp3furmi_rnd#d0b] and stop
    // --> [Mp3furmi_rnd#d0b] and stop
    // --> [Mp3furmi_rnd#d0b] and stop
    0x01e7, 0x0d0b,
    // end of trunc.f64 (I32)
    // end of nearest.f64 (I32)
    // end of floor.f64 (I32)
    // end of ceil.f64 (I32)
    // 0007b4: copy_to_ssa.f64 (I32)
    // --> [Mp2furm_reg_to_ssa#710] and stop
    0x00e3, 0x0710,
    // end of copy_to_ssa.f64 (I32)
    // 0007b6: fcvt_from_sint.f64 (I32)
    // stop unless inst_predicate_10
    0x100a,
    // --> [Mp2frurm#72a] and stop
    0x01cf, 0x072a,
    // end of fcvt_from_sint.f64 (I32)
    // 0007b9: fpromote.f64 (I32)
    // stop unless inst_predicate_15
    0x100f,
    // --> [Mp2furm#65a] and stop
    0x01df, 0x065a,
    // end of fpromote.f64 (I32)
    // 0007bc: load.f64 (I32)
    // --> [Mp2fld#710]
    0x00f4, 0x0710,
    // --> [Mp2fldDisp8#710]
    0x00f8, 0x0710,
    // --> [Mp2fldDisp32#710] and stop
    0x00fd, 0x0710,
    // end of load.f64 (I32)
    // 0007c2: load_complex.f64 (I32)
    // --> [Mp2fldWithIndex#710]
    0x0100, 0x0710,
    // --> [Mp2fldWithIndexDisp8#710]
    0x0104, 0x0710,
    // --> [Mp2fldWithIndexDisp32#710] and stop
    0x0109, 0x0710,
    // end of load_complex.f64 (I32)
    // 0007c8: store.f64 (I32)
    // --> [Mp2fst#711]
    0x010c, 0x0711,
    // --> [Mp2fstDisp8#711]
    0x0110, 0x0711,
    // --> [Mp2fstDisp32#711] and stop
    0x0115, 0x0711,
    // end of store.f64 (I32)
    // 0007ce: store_complex.f64 (I32)
    // --> [Mp2fstWithIndex#711]
    0x0118, 0x0711,
    // --> [Mp2fstWithIndexDisp8#711]
    0x011c, 0x0711,
    // --> [Mp2fstWithIndexDisp32#711] and stop
    0x0121, 0x0711,
    // end of store_complex.f64 (I32)
    // 0007d4: bitcast.f32 (I32)
    // stop unless inst_predicate_10
    // 0007d4: bitcast.i64x2 (I32)
    // stop unless inst_predicate_10
    0x100a,
    // --> [Mp2frurm#56e] and stop
    // --> [Mp2frurm#56e] and stop
    0x01cf, 0x056e,
    // end of bitcast.i64x2 (I32)
    // end of bitcast.f32 (I32)
    // 0007d7: ceil.f32 (I32)
    // stop unless PredicateView(16)
    // 0007d7: floor.f32 (I32)
    // stop unless PredicateView(16)
    // 0007d7: nearest.f32 (I32)
    // stop unless PredicateView(16)
    // 0007d7: trunc.f32 (I32)
    // stop unless PredicateView(16)
    0x102f,
    // --> [Mp3furmi_rnd#d0a] and stop
    // --> [Mp3furmi_rnd#d0a] and stop
    // --> [Mp3furmi_rnd#d0a] and stop
    // --> [Mp3furmi_rnd#d0a] and stop
    0x01e7, 0x0d0a,
    // end of trunc.f32 (I32)
    // end of nearest.f32 (I32)
    // end of floor.f32 (I32)
    // end of ceil.f32 (I32)
    // 0007da: copy_to_ssa.f32 (I32)
    // --> [Mp2furm_reg_to_ssa#610] and stop
    0x00e3, 0x0610,
    // end of copy_to_ssa.f32 (I32)
    // 0007dc: fcvt_from_sint.f32 (I32)
    // stop unless inst_predicate_10
    0x100a,
    // --> [Mp2frurm#62a] and stop
    0x01cf, 0x062a,
    // end of fcvt_from_sint.f32 (I32)
    // 0007df: fdemote.f32 (I32)
    // stop unless inst_predicate_16
    0x1010,
    // --> [Mp2furm#75a] and stop
    0x01df, 0x075a,
    // end of fdemote.f32 (I32)
    // 0007e2: load.f32 (I32)
    // --> [Mp2fld#610]
    0x00f4, 0x0610,
    // --> [Mp2fldDisp8#610]
    0x00f8, 0x0610,
    // --> [Mp2fldDisp32#610] and stop
    0x00fd, 0x0610,
    // end of load.f32 (I32)
    // 0007e8: load_complex.f32 (I32)
    // --> [Mp2fldWithIndex#610]
    0x0100, 0x0610,
    // --> [Mp2fldWithIndexDisp8#610]
    0x0104, 0x0610,
    // --> [Mp2fldWithIndexDisp32#610] and stop
    0x0109, 0x0610,
    // end of load_complex.f32 (I32)
    // 0007ee: store.f32 (I32)
    // --> [Mp2fst#611]
    0x010c, 0x0611,
    // --> [Mp2fstDisp8#611]
    0x0110, 0x0611,
    // --> [Mp2fstDisp32#611] and stop
    0x0115, 0x0611,
    // end of store.f32 (I32)
    // 0007f4: store_complex.f32 (I32)
    // --> [Mp2fstWithIndex#611]
    0x0118, 0x0611,
    // --> [Mp2fstWithIndexDisp8#611]
    0x011c, 0x0611,
    // --> [Mp2fstWithIndexDisp32#611] and stop
    0x0121, 0x0611,
    // end of store_complex.f32 (I32)
    // 0007fa: adjust_sp_down_imm (I32)
    // --> [Op1adjustsp_ib#5083]
    0x00ec, 0x5083,
    // --> [Op1adjustsp_id#5081] and stop
    0x00ef, 0x5081,
    // end of adjust_sp_down_imm (I32)
    // 0007fe: adjust_sp_up_imm (I32)
    // --> [Op1adjustsp_ib#83]
    0x00ec, 0x0083,
    // --> [Op1adjustsp_id#81] and stop
    0x00ef, 0x0081,
    // end of adjust_sp_up_imm (I32)
    // 000802: brff (I32)
    // --> [Op1brfb#70]
    0x0162, 0x0070,
    // --> [Op2brfd#480] and stop
    0x0167, 0x0480,
    // end of brff (I32)
    // 000806: brif (I32)
    // --> [Op1brib#70]
    0x015a, 0x0070,
    // --> [Op2brid#480] and stop
    0x015f, 0x0480,
    // end of brif (I32)
    // 00080a: call (I32)
    // --> [Op1call_id#e8] and stop
    0x014d, 0x00e8,
    // end of call (I32)
    // 00080c: copy_special (I32)
    // --> [Op1copysp#89] and stop
    0x00dd, 0x0089,
    // end of copy_special (I32)
    // 00080e: f32const (I32)
    // stop unless inst_predicate_13
    0x100d,
    // --> [Op2f32imm_z#457] and stop
    0x01c7, 0x0457,
    // end of f32const (I32)
    // 000811: f64const (I32)
    // stop unless inst_predicate_14
    0x100e,
    // --> [Mp2f64imm_z#557] and stop
    0x01c9, 0x0557,
];

/// x86 level 2 hash tables.
///
/// This hash table, keyed by instruction opcode, contains all the starting offsets for the
/// encodings interpreter, for all the CPU modes. It is jumped to after a lookup on the
/// instruction's controlling type in the level 1 hash table.
pub static LEVEL2: [Level2Entry<u16>; 1774] = [
    // I64
    // 000000: i64, 128 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x000025 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bnot), offset: 0x000015 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Brz), offset: 0x000021 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Brnz), offset: 0x00001d },
    Level2Entry { opcode: Some(crate::ir::Opcode::BandImm), offset: 0x000004 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BorImm), offset: 0x000019 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BxorImm), offset: 0x000027 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Rotl), offset: 0x0000df },
    Level2Entry { opcode: Some(crate::ir::Opcode::JumpTableEntry), offset: 0x0000c7 },
    Level2Entry { opcode: Some(crate::ir::Opcode::JumpTableBase), offset: 0x0000c5 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IndirectJumpTableBr), offset: 0x000077 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Ishl), offset: 0x00007b },
    Level2Entry { opcode: Some(crate::ir::Opcode::RotlImm), offset: 0x0000e1 },
    Level2Entry { opcode: Some(crate::ir::Opcode::RotrImm), offset: 0x0000e5 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IshlImm), offset: 0x00007d },
    Level2Entry { opcode: Some(crate::ir::Opcode::Ushr), offset: 0x00016a },
    Level2Entry { opcode: Some(crate::ir::Opcode::Sshr), offset: 0x000118 },
    Level2Entry { opcode: Some(crate::ir::Opcode::SshrImm), offset: 0x00011a },
    Level2Entry { opcode: Some(crate::ir::Opcode::Clz), offset: 0x00002f },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Ctz), offset: 0x000038 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CallIndirect), offset: 0x00002b },
    Level2Entry { opcode: Some(crate::ir::Opcode::FuncAddr), offset: 0x00003f },
    Level2Entry { opcode: Some(crate::ir::Opcode::Load), offset: 0x0000c9 },
    Level2Entry { opcode: Some(crate::ir::Opcode::LoadComplex), offset: 0x0000cf },
    Level2Entry { opcode: Some(crate::ir::Opcode::Store), offset: 0x00011e },
    Level2Entry { opcode: Some(crate::ir::Opcode::StoreComplex), offset: 0x000124 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Popcnt), offset: 0x0000d6 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Uload8), offset: 0x00015d },
    Level2Entry { opcode: Some(crate::ir::Opcode::Rotr), offset: 0x0000e3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Sload8), offset: 0x000109 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Istore8), offset: 0x0000a4 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Istore8Complex), offset: 0x0000b0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Sload8Complex), offset: 0x00010f },
    Level2Entry { opcode: Some(crate::ir::Opcode::Uload16), offset: 0x000144 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Sload16), offset: 0x0000f6 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Sload16Complex), offset: 0x0000fc },
    Level2Entry { opcode: Some(crate::ir::Opcode::Istore16), offset: 0x00007f },
    Level2Entry { opcode: Some(crate::ir::Opcode::Istore16Complex), offset: 0x00008b },
    Level2Entry { opcode: Some(crate::ir::Opcode::Uload32), offset: 0x000151 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Uload16Complex), offset: 0x00014a },
    Level2Entry { opcode: Some(crate::ir::Opcode::Sload32), offset: 0x000103 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bitcast), offset: 0x000012 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Istore32), offset: 0x000098 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bint), offset: 0x000008 },
    Level2Entry { opcode: Some(crate::ir::Opcode::StackAddr), offset: 0x00011c },
    Level2Entry { opcode: Some(crate::ir::Opcode::SymbolValue), offset: 0x00012b },
    Level2Entry { opcode: Some(crate::ir::Opcode::Uextend), offset: 0x000135 },
    Level2Entry { opcode: Some(crate::ir::Opcode::GetPinnedReg), offset: 0x00004b },
    Level2Entry { opcode: Some(crate::ir::Opcode::SetPinnedReg), offset: 0x0000e9 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Iconst), offset: 0x00005f },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Uload8Complex), offset: 0x000163 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Sextend), offset: 0x0000ed },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::UshrImm), offset: 0x00016c },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Udivmodx), offset: 0x000184 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Selectif), offset: 0x0000e7 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Sdivmodx), offset: 0x000180 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x000032 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x000116 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x00003b },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x00003d },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x0000db },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pop), offset: 0x000178 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyToSsa), offset: 0x000036 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000034 },
    Level2Entry { opcode: Some(crate::ir::Opcode::AdjustSpDown), offset: 0x000000 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Bsr), offset: 0x000170 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Bsf), offset: 0x00016e },
    Level2Entry { opcode: Some(crate::ir::Opcode::IfcmpSp), offset: 0x000073 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x0000dd },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x0000d9 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Push), offset: 0x00017c },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Smulx), offset: 0x000182 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Cvtt2si), offset: 0x000172 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Umulx), offset: 0x000186 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Icmp), offset: 0x000059 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IcmpImm), offset: 0x00005b },
    Level2Entry { opcode: Some(crate::ir::Opcode::Ifcmp), offset: 0x00006d },
    Level2Entry { opcode: Some(crate::ir::Opcode::IfcmpImm), offset: 0x00006f },
    Level2Entry { opcode: Some(crate::ir::Opcode::Iadd), offset: 0x00004d },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Isub), offset: 0x0000bd },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Imul), offset: 0x000075 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IaddImm), offset: 0x000055 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IaddIfcin), offset: 0x000051 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IaddIfcout), offset: 0x000053 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IaddIfcarry), offset: 0x00004f },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IsubIfbin), offset: 0x0000bf },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IsubIfbout), offset: 0x0000c3 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IsubIfborrow), offset: 0x0000c1 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x000002 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x000017 },
    // 000080: i32, 128 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x0001b9 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bnot), offset: 0x000199 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Brz), offset: 0x0001b1 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Brnz), offset: 0x0001a9 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BandImm), offset: 0x00018c },
    Level2Entry { opcode: Some(crate::ir::Opcode::BorImm), offset: 0x0001a1 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BxorImm), offset: 0x0001bd },
    Level2Entry { opcode: Some(crate::ir::Opcode::Rotl), offset: 0x000249 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Rotr), offset: 0x000251 },
    Level2Entry { opcode: Some(crate::ir::Opcode::RotlImm), offset: 0x00024d },
    Level2Entry { opcode: Some(crate::ir::Opcode::RotrImm), offset: 0x000255 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Ishl), offset: 0x000215 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Ushr), offset: 0x0002ee },
    Level2Entry { opcode: Some(crate::ir::Opcode::Sshr), offset: 0x00029d },
    Level2Entry { opcode: Some(crate::ir::Opcode::IshlImm), offset: 0x000219 },
    Level2Entry { opcode: Some(crate::ir::Opcode::UshrImm), offset: 0x0002f2 },
    Level2Entry { opcode: Some(crate::ir::Opcode::SshrImm), offset: 0x0002a1 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Clz), offset: 0x0001c5 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Ctz), offset: 0x0001cc },
    Level2Entry { opcode: Some(crate::ir::Opcode::Popcnt), offset: 0x00023a },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Load), offset: 0x000151 },
    Level2Entry { opcode: Some(crate::ir::Opcode::LoadComplex), offset: 0x00022d },
    Level2Entry { opcode: Some(crate::ir::Opcode::Store), offset: 0x000098 },
    Level2Entry { opcode: Some(crate::ir::Opcode::StoreComplex), offset: 0x0002a5 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Uload8), offset: 0x0002d5 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Uload8Complex), offset: 0x0002e1 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Sload8), offset: 0x000280 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Sload8Complex), offset: 0x00028c },
    Level2Entry { opcode: Some(crate::ir::Opcode::Istore8), offset: 0x0000a4 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Istore8Complex), offset: 0x0000b0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Uload16), offset: 0x0002bc },
    Level2Entry { opcode: Some(crate::ir::Opcode::Uload16Complex), offset: 0x0002c8 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Sload16), offset: 0x000267 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Sload16Complex), offset: 0x000273 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Istore16), offset: 0x00007f },
    Level2Entry { opcode: Some(crate::ir::Opcode::Istore16Complex), offset: 0x00008b },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bitcast), offset: 0x000194 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bint), offset: 0x000008 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Ireduce), offset: 0x000212 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Uextend), offset: 0x0002b2 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Sextend), offset: 0x00025d },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Iconst), offset: 0x0001f9 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Udivmodx), offset: 0x000310 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Selectif), offset: 0x000259 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Sdivmodx), offset: 0x000308 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x000140 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x000299 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x0001d1 },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x00003d },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x000243 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Umulx), offset: 0x000314 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyToSsa), offset: 0x0001ca },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000034 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Bsf), offset: 0x0002f6 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Bsr), offset: 0x0002fa },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Smulx), offset: 0x00030c },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Cvtt2si), offset: 0x0002fe },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x000245 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x00023f },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Icmp), offset: 0x0001ed },
    Level2Entry { opcode: Some(crate::ir::Opcode::IcmpImm), offset: 0x0001f1 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Ifcmp), offset: 0x000202 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IfcmpImm), offset: 0x000206 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Iadd), offset: 0x0001d5 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Isub), offset: 0x00021d },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Imul), offset: 0x00020e },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IaddImm), offset: 0x0001e5 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IaddIfcin), offset: 0x0001dd },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IaddIfcout), offset: 0x0001e1 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IaddIfcarry), offset: 0x0001d9 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IsubIfbin), offset: 0x000221 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IsubIfbout), offset: 0x000229 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IsubIfborrow), offset: 0x000225 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x000188 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x00019d },
    // 000100: b32, 8 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x0001b9 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bnot), offset: 0x000199 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bconst), offset: 0x000318 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x000243 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x000188 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x00019d },
    // 000108: b64, 8 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x000025 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bnot), offset: 0x000015 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bconst), offset: 0x00031c },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x000002 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x000017 },
    // 000110: b1, 32 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x0001b9 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x000140 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Brz), offset: 0x000326 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Brnz), offset: 0x00031e },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x0001d1 },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x00003d },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x00032e },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyToSsa), offset: 0x0001ca },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x000299 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x000245 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x00023f },
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
    Level2Entry { opcode: Some(crate::ir::Opcode::Bconst), offset: 0x000318 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x000188 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x00019d },
    // 000130: r64, 16 entries
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x000032 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x000116 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x00003b },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x0000db },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyToSsa), offset: 0x000036 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IsNull), offset: 0x000332 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Null), offset: 0x000334 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x0000dd },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x0000d9 },
    // 000140: i8, 16 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bint), offset: 0x000008 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x000140 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Ireduce), offset: 0x00033d },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x0001d1 },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x00003d },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x000346 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Iconst), offset: 0x000338 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyToSsa), offset: 0x0001ca },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000034 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x000299 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x000245 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x00023f },
    // 000150: i16, 16 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bint), offset: 0x000008 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x000140 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Ireduce), offset: 0x000340 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x0001d1 },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x00003d },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x000243 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Iconst), offset: 0x000068 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyToSsa), offset: 0x0001ca },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000034 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x000299 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x000245 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x00023f },
    // 000160: b8, 4 entries
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bconst), offset: 0x000318 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x000243 },
    // 000164: b16, 4 entries
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bconst), offset: 0x000318 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x000243 },
    // 000168: r32, 2 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x000243 },
    Level2Entry { opcode: None, offset: 0 },
    // 00016a: typeless, 32 entries
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Jump), offset: 0x000378 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Brif), offset: 0x00035c },
    Level2Entry { opcode: Some(crate::ir::Opcode::Brff), offset: 0x000354 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopySpecial), offset: 0x00036a },
    Level2Entry { opcode: Some(crate::ir::Opcode::Trueif), offset: 0x000391 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Trueff), offset: 0x00038d },
    Level2Entry { opcode: Some(crate::ir::Opcode::AdjustSpUpImm), offset: 0x000350 },
    Level2Entry { opcode: Some(crate::ir::Opcode::AdjustSpDownImm), offset: 0x00034c },
    Level2Entry { opcode: Some(crate::ir::Opcode::Debugtrap), offset: 0x00036c },
    Level2Entry { opcode: Some(crate::ir::Opcode::Istore32Complex), offset: 0x0002a5 },
    Level2Entry { opcode: Some(crate::ir::Opcode::ResumableTrap), offset: 0x00037c },
    Level2Entry { opcode: Some(crate::ir::Opcode::Safepoint), offset: 0x000380 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Sload32Complex), offset: 0x000382 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Trapff), offset: 0x000389 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Return), offset: 0x00037e },
    Level2Entry { opcode: Some(crate::ir::Opcode::Trapif), offset: 0x00038b },
    Level2Entry { opcode: Some(crate::ir::Opcode::Call), offset: 0x000364 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Trap), offset: 0x00037c },
    Level2Entry { opcode: Some(crate::ir::Opcode::F32const), offset: 0x00036e },
    Level2Entry { opcode: Some(crate::ir::Opcode::F64const), offset: 0x000373 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Uload32Complex), offset: 0x00022d },
    Level2Entry { opcode: None, offset: 0 },
    // 00018a: f64, 64 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x0003a4 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BandNot), offset: 0x000399 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x0003ad },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x0003cb },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x0003cf },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x000418 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Fmin), offset: 0x000442 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyToSsa), offset: 0x0003b1 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000034 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Fmax), offset: 0x00043e },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x00041e },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x00041a },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x000414 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fcmp), offset: 0x0003b7 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Ffcmp), offset: 0x0003c7 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fadd), offset: 0x0003b3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fsub), offset: 0x0003da },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fmul), offset: 0x0003d1 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fdiv), offset: 0x0003c3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Sqrt), offset: 0x000422 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Load), offset: 0x0003de },
    Level2Entry { opcode: Some(crate::ir::Opcode::LoadComplex), offset: 0x0003ea },
    Level2Entry { opcode: Some(crate::ir::Opcode::Store), offset: 0x000426 },
    Level2Entry { opcode: Some(crate::ir::Opcode::StoreComplex), offset: 0x000432 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Ceil), offset: 0x0003a8 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Floor), offset: 0x0003a8 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Trunc), offset: 0x0003a8 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Nearest), offset: 0x0003a8 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bitcast), offset: 0x00039d },
    Level2Entry { opcode: Some(crate::ir::Opcode::RawBitcast), offset: 0x0003f6 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fpromote), offset: 0x0003d5 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::FcvtFromSint), offset: 0x0003bb },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x000395 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x0003a0 },
    // 0001ca: f32, 64 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x0003a4 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BandNot), offset: 0x000399 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x0003ad },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x00046f },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x0003cf },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x000418 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Fmin), offset: 0x0004bf },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyToSsa), offset: 0x000450 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000034 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Fmax), offset: 0x0004bb },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x00049b },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x000497 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x000493 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fcmp), offset: 0x000456 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Ffcmp), offset: 0x00046b },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fadd), offset: 0x000452 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fsub), offset: 0x000477 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fmul), offset: 0x000473 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fdiv), offset: 0x000467 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Sqrt), offset: 0x00049f },
    Level2Entry { opcode: Some(crate::ir::Opcode::Load), offset: 0x00047b },
    Level2Entry { opcode: Some(crate::ir::Opcode::LoadComplex), offset: 0x000487 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Store), offset: 0x0004a3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::StoreComplex), offset: 0x0004af },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Ceil), offset: 0x00044b },
    Level2Entry { opcode: Some(crate::ir::Opcode::Floor), offset: 0x00044b },
    Level2Entry { opcode: Some(crate::ir::Opcode::Trunc), offset: 0x00044b },
    Level2Entry { opcode: Some(crate::ir::Opcode::Nearest), offset: 0x00044b },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bitcast), offset: 0x000446 },
    Level2Entry { opcode: Some(crate::ir::Opcode::RawBitcast), offset: 0x0003f6 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fdemote), offset: 0x000462 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::FcvtFromSint), offset: 0x00045a },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x000395 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x0003a0 },
    // 00020a: b8x16, 32 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x0004c9 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BandNot), offset: 0x0004c5 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x0003af },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x0004cb },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x0003cf },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x0004f6 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000034 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pshufb), offset: 0x000510 },
    Level2Entry { opcode: Some(crate::ir::Opcode::RawBitcast), offset: 0x0004d3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::ScalarToVector), offset: 0x000447 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x0004f8 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x0004f4 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pinsr), offset: 0x00050d },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x0004fa },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Ptest), offset: 0x000513 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pextr), offset: 0x00050a },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Load), offset: 0x0004cd },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Store), offset: 0x0004fc },
    Level2Entry { opcode: Some(crate::ir::Opcode::Vconst), offset: 0x000502 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x0004c3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x0004c7 },
    // 00022a: b16x8, 32 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x0004c9 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BandNot), offset: 0x0004c5 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x0003af },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x0004cb },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x0003cf },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x0004f6 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000034 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pshufb), offset: 0x000510 },
    Level2Entry { opcode: Some(crate::ir::Opcode::RawBitcast), offset: 0x000516 },
    Level2Entry { opcode: Some(crate::ir::Opcode::ScalarToVector), offset: 0x000447 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x0004f8 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x0004f4 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pinsr), offset: 0x00053a },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x0004fa },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Ptest), offset: 0x000513 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pextr), offset: 0x000537 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Load), offset: 0x0004cd },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Store), offset: 0x0004fc },
    Level2Entry { opcode: Some(crate::ir::Opcode::Vconst), offset: 0x000502 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x0004c3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x0004c7 },
    // 00024a: b32x4, 32 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x0004c9 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BandNot), offset: 0x0004c5 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x0003af },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x0004cb },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x0003cf },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x0004f6 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000034 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pshufd), offset: 0x000563 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pshufb), offset: 0x000510 },
    Level2Entry { opcode: Some(crate::ir::Opcode::RawBitcast), offset: 0x00053c },
    Level2Entry { opcode: Some(crate::ir::Opcode::ScalarToVector), offset: 0x000447 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x0004f8 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x0004f4 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pinsr), offset: 0x000560 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x0004fa },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Ptest), offset: 0x000513 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pextr), offset: 0x00055d },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Load), offset: 0x0004cd },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Store), offset: 0x0004fc },
    Level2Entry { opcode: Some(crate::ir::Opcode::Vconst), offset: 0x000502 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x0004c3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x0004c7 },
    // 00026a: b64x2, 32 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x0004c9 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BandNot), offset: 0x0004c5 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x0003af },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x0004cb },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x0003cf },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x0004f6 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000034 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pshufb), offset: 0x000510 },
    Level2Entry { opcode: Some(crate::ir::Opcode::RawBitcast), offset: 0x000565 },
    Level2Entry { opcode: Some(crate::ir::Opcode::ScalarToVector), offset: 0x000447 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x0004f8 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x0004f4 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pinsr), offset: 0x000589 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x0004fa },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Ptest), offset: 0x000513 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pextr), offset: 0x000586 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Load), offset: 0x0004cd },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Store), offset: 0x0004fc },
    Level2Entry { opcode: Some(crate::ir::Opcode::Vconst), offset: 0x000502 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x0004c3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x0004c7 },
    // 00028a: i8x16, 64 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x0004c9 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BandNot), offset: 0x0004c5 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x0003af },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x0004cb },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x0003cf },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x0004f6 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000034 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pshufb), offset: 0x000510 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pextr), offset: 0x00050a },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x0004fa },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x0004f8 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x0004f4 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pinsr), offset: 0x00050d },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Ptest), offset: 0x000513 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pmaxs), offset: 0x0005bf },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pmaxu), offset: 0x0005c2 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pmins), offset: 0x0005c4 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Load), offset: 0x0004cd },
    Level2Entry { opcode: Some(crate::ir::Opcode::Icmp), offset: 0x00058e },
    Level2Entry { opcode: Some(crate::ir::Opcode::Store), offset: 0x0004fc },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pminu), offset: 0x0005c7 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Iadd), offset: 0x00058c },
    Level2Entry { opcode: Some(crate::ir::Opcode::UaddSat), offset: 0x0005bb },
    Level2Entry { opcode: Some(crate::ir::Opcode::SaddSat), offset: 0x0005b7 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Isub), offset: 0x000594 },
    Level2Entry { opcode: Some(crate::ir::Opcode::UsubSat), offset: 0x0005bd },
    Level2Entry { opcode: Some(crate::ir::Opcode::SsubSat), offset: 0x0005b9 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::RawBitcast), offset: 0x000596 },
    Level2Entry { opcode: Some(crate::ir::Opcode::ScalarToVector), offset: 0x000447 },
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
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Vconst), offset: 0x000502 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x0004c3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x0004c7 },
    // 0002ca: i16x8, 64 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x0004c9 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BandNot), offset: 0x0004c5 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x0003af },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x0004cb },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x0003cf },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x0004f6 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000034 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pshufb), offset: 0x000510 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pextr), offset: 0x000537 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x0004fa },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x0004f8 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x0004f4 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pinsr), offset: 0x00053a },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Psll), offset: 0x000608 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Psrl), offset: 0x00060c },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Psra), offset: 0x00060a },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Ptest), offset: 0x000513 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pmaxs), offset: 0x0005fe },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pmaxu), offset: 0x000600 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pmins), offset: 0x000603 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Load), offset: 0x0004cd },
    Level2Entry { opcode: Some(crate::ir::Opcode::Icmp), offset: 0x0005cb },
    Level2Entry { opcode: Some(crate::ir::Opcode::Store), offset: 0x0004fc },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pminu), offset: 0x000605 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Iadd), offset: 0x0005c9 },
    Level2Entry { opcode: Some(crate::ir::Opcode::UaddSat), offset: 0x0005fa },
    Level2Entry { opcode: Some(crate::ir::Opcode::SaddSat), offset: 0x0005f6 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Isub), offset: 0x0005d3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::UsubSat), offset: 0x0005fc },
    Level2Entry { opcode: Some(crate::ir::Opcode::SsubSat), offset: 0x0005f8 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Imul), offset: 0x0005d1 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::RawBitcast), offset: 0x0005d5 },
    Level2Entry { opcode: Some(crate::ir::Opcode::ScalarToVector), offset: 0x000447 },
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
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Vconst), offset: 0x000502 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x0004c3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x0004c7 },
    // 00030a: i32x4, 64 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x0004c9 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BandNot), offset: 0x0004c5 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x0003af },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x0004cb },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x0003cf },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x0004f6 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000034 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pshufd), offset: 0x000563 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pshufb), offset: 0x000510 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pextr), offset: 0x00055d },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x0004fa },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x0004f8 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x0004f4 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pinsr), offset: 0x000560 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Psll), offset: 0x000648 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Psrl), offset: 0x00064c },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Psra), offset: 0x00064a },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Ptest), offset: 0x000513 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pmaxs), offset: 0x00063c },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pmaxu), offset: 0x00063f },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pmins), offset: 0x000642 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Load), offset: 0x0004cd },
    Level2Entry { opcode: Some(crate::ir::Opcode::Icmp), offset: 0x000610 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Store), offset: 0x0004fc },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pminu), offset: 0x000645 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Iadd), offset: 0x00060e },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Isub), offset: 0x000619 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Imul), offset: 0x000616 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::RawBitcast), offset: 0x00061b },
    Level2Entry { opcode: Some(crate::ir::Opcode::ScalarToVector), offset: 0x000447 },
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
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Vconst), offset: 0x000502 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x0004c3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x0004c7 },
    // 00034a: i64x2, 64 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x0004c9 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BandNot), offset: 0x0004c5 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x0003af },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x0004cb },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x0003cf },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x0004f6 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000034 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pshufb), offset: 0x000510 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pextr), offset: 0x000586 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x0004fa },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x0004f8 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x0004f4 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pinsr), offset: 0x000589 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Psll), offset: 0x000683 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Psrl), offset: 0x000685 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Ptest), offset: 0x000513 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Load), offset: 0x0004cd },
    Level2Entry { opcode: Some(crate::ir::Opcode::Icmp), offset: 0x000658 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Store), offset: 0x0004fc },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Iadd), offset: 0x000656 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Isub), offset: 0x000660 },
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
    Level2Entry { opcode: Some(crate::ir::Opcode::Bitcast), offset: 0x00064e },
    Level2Entry { opcode: Some(crate::ir::Opcode::RawBitcast), offset: 0x000662 },
    Level2Entry { opcode: Some(crate::ir::Opcode::ScalarToVector), offset: 0x000447 },
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
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Vconst), offset: 0x000502 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x0004c3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x0004c7 },
    // 00038a: f32x4, 32 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x0004c9 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BandNot), offset: 0x0004c5 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x0003af },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x0004cb },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x0003cf },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x0004f6 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000034 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pshufd), offset: 0x000563 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pshufb), offset: 0x000510 },
    Level2Entry { opcode: Some(crate::ir::Opcode::RawBitcast), offset: 0x000687 },
    Level2Entry { opcode: Some(crate::ir::Opcode::ScalarToVector), offset: 0x000412 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x0004f8 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x0004f4 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pinsr), offset: 0x000560 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Insertps), offset: 0x0006a8 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x0004fa },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Ptest), offset: 0x000513 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pextr), offset: 0x00055d },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Load), offset: 0x0004cd },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Store), offset: 0x0004fc },
    Level2Entry { opcode: Some(crate::ir::Opcode::Vconst), offset: 0x000502 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x0004c3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x0004c7 },
    // 0003aa: f64x2, 32 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x0004c9 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BandNot), offset: 0x0004c5 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x0003af },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x0004cb },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x0003cf },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x0004f6 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000034 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pshufb), offset: 0x000510 },
    Level2Entry { opcode: Some(crate::ir::Opcode::RawBitcast), offset: 0x0006ab },
    Level2Entry { opcode: Some(crate::ir::Opcode::ScalarToVector), offset: 0x000412 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x0004f8 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x0004f4 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Movlhps), offset: 0x0006cc },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x0004fa },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pinsr), offset: 0x000589 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Ptest), offset: 0x000513 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Movsd), offset: 0x0006ce },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pextr), offset: 0x000586 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Load), offset: 0x0004cd },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Store), offset: 0x0004fc },
    Level2Entry { opcode: Some(crate::ir::Opcode::Vconst), offset: 0x000502 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x0004c3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x0004c7 },
    // I32
    // 0003ca: i32, 128 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x0001bb },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bnot), offset: 0x00019b },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Brz), offset: 0x0006e7 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Brnz), offset: 0x0006e3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BandImm), offset: 0x0006d2 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BorImm), offset: 0x0006df },
    Level2Entry { opcode: Some(crate::ir::Opcode::BxorImm), offset: 0x0006eb },
    Level2Entry { opcode: Some(crate::ir::Opcode::Rotl), offset: 0x00024b },
    Level2Entry { opcode: Some(crate::ir::Opcode::JumpTableEntry), offset: 0x00072c },
    Level2Entry { opcode: Some(crate::ir::Opcode::JumpTableBase), offset: 0x00072a },
    Level2Entry { opcode: Some(crate::ir::Opcode::IndirectJumpTableBr), offset: 0x000079 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Ishl), offset: 0x000217 },
    Level2Entry { opcode: Some(crate::ir::Opcode::RotlImm), offset: 0x00024f },
    Level2Entry { opcode: Some(crate::ir::Opcode::RotrImm), offset: 0x000257 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IshlImm), offset: 0x00021b },
    Level2Entry { opcode: Some(crate::ir::Opcode::Ushr), offset: 0x0002f0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Sshr), offset: 0x00029f },
    Level2Entry { opcode: Some(crate::ir::Opcode::SshrImm), offset: 0x0002a3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Clz), offset: 0x0006ef },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Ctz), offset: 0x0006f4 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CallIndirect), offset: 0x00002d },
    Level2Entry { opcode: Some(crate::ir::Opcode::FuncAddr), offset: 0x0006f7 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Load), offset: 0x00072e },
    Level2Entry { opcode: Some(crate::ir::Opcode::LoadComplex), offset: 0x000734 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Store), offset: 0x000760 },
    Level2Entry { opcode: Some(crate::ir::Opcode::StoreComplex), offset: 0x000766 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Popcnt), offset: 0x00073b },
    Level2Entry { opcode: Some(crate::ir::Opcode::Uload8), offset: 0x000783 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Rotr), offset: 0x000253 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Sload8), offset: 0x000751 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Istore8), offset: 0x00071d },
    Level2Entry { opcode: Some(crate::ir::Opcode::Istore8Complex), offset: 0x000723 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Sload8Complex), offset: 0x000757 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Uload16), offset: 0x000776 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Sload16), offset: 0x000744 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Sload16Complex), offset: 0x00074a },
    Level2Entry { opcode: Some(crate::ir::Opcode::Istore16), offset: 0x000710 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Istore16Complex), offset: 0x000716 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Uload16Complex), offset: 0x00077c },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bitcast), offset: 0x0006dc },
    Level2Entry { opcode: Some(crate::ir::Opcode::Uload8Complex), offset: 0x000789 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bint), offset: 0x0006d6 },
    Level2Entry { opcode: Some(crate::ir::Opcode::StackAddr), offset: 0x00075e },
    Level2Entry { opcode: Some(crate::ir::Opcode::SymbolValue), offset: 0x00076d },
    Level2Entry { opcode: Some(crate::ir::Opcode::Uextend), offset: 0x000770 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Sextend), offset: 0x00073e },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Iconst), offset: 0x000705 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::UshrImm), offset: 0x0002f4 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Udivmodx), offset: 0x000312 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Selectif), offset: 0x00025b },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Sdivmodx), offset: 0x00030a },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x000142 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x00029b },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x0001d3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x00003d },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x000330 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pop), offset: 0x00017a },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyToSsa), offset: 0x0006f2 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000034 },
    Level2Entry { opcode: Some(crate::ir::Opcode::AdjustSpDown), offset: 0x0006d0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Bsr), offset: 0x0002fc },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Bsf), offset: 0x0002f8 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IfcmpSp), offset: 0x00070e },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x000247 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x000241 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Push), offset: 0x00017e },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Smulx), offset: 0x00030e },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Cvtt2si), offset: 0x000790 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Umulx), offset: 0x000316 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Icmp), offset: 0x0001ef },
    Level2Entry { opcode: Some(crate::ir::Opcode::IcmpImm), offset: 0x000701 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Ifcmp), offset: 0x000204 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IfcmpImm), offset: 0x00070a },
    Level2Entry { opcode: Some(crate::ir::Opcode::Iadd), offset: 0x0001d7 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Isub), offset: 0x00021f },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Imul), offset: 0x000210 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IaddImm), offset: 0x0006fd },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IaddIfcin), offset: 0x0001df },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IaddIfcout), offset: 0x0001e3 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IaddIfcarry), offset: 0x0001da },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IsubIfbin), offset: 0x000223 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IsubIfbout), offset: 0x00022b },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IsubIfborrow), offset: 0x000227 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x00018a },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x00019f },
    // 00044a: b32, 8 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x0001bb },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bnot), offset: 0x00019b },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bconst), offset: 0x00031a },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x000330 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x00018a },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x00019f },
    // 000452: b1, 32 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x0001bb },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x000142 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Brz), offset: 0x00079c },
    Level2Entry { opcode: Some(crate::ir::Opcode::Brnz), offset: 0x000796 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x0001d3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x00003d },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x000330 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyToSsa), offset: 0x0006f2 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x00029b },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x000247 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x000241 },
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
    Level2Entry { opcode: Some(crate::ir::Opcode::Bconst), offset: 0x00031a },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x00018a },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x00019f },
    // 000472: r32, 16 entries
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x000142 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x00029b },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x0001d3 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x000330 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyToSsa), offset: 0x0006f2 },
    Level2Entry { opcode: Some(crate::ir::Opcode::IsNull), offset: 0x0007a2 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Null), offset: 0x000336 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x000247 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x000241 },
    // 000482: i8, 16 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bint), offset: 0x0006d6 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x000142 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Ireduce), offset: 0x0007a7 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x0001d3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x00003d },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x0007ad },
    Level2Entry { opcode: Some(crate::ir::Opcode::Iconst), offset: 0x0007a4 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyToSsa), offset: 0x0006f2 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000034 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x00029b },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x000247 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x000241 },
    // 000492: i16, 16 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bint), offset: 0x0006d6 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x000142 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Ireduce), offset: 0x0007aa },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x0001d3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x00003d },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x000330 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Iconst), offset: 0x000707 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyToSsa), offset: 0x0006f2 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000034 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x00029b },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x000247 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x000241 },
    // 0004a2: b8, 4 entries
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bconst), offset: 0x00031a },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x000330 },
    // 0004a6: b16, 4 entries
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bconst), offset: 0x00031a },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x000330 },
    // 0004aa: i64, 4 entries
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000034 },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x00003d },
    Level2Entry { opcode: None, offset: 0 },
    // 0004ae: f64, 64 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x0003a6 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BandNot), offset: 0x00039b },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x0003af },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x0003cd },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x0003cf },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x0004f6 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Fmin), offset: 0x000444 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyToSsa), offset: 0x0007b4 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000034 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Fmax), offset: 0x000440 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x000420 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x00041c },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x000416 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fcmp), offset: 0x0003b9 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Ffcmp), offset: 0x0003c9 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fadd), offset: 0x0003b5 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fsub), offset: 0x0003dc },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fmul), offset: 0x0003d3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fdiv), offset: 0x0003c5 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Sqrt), offset: 0x000424 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Load), offset: 0x0007bc },
    Level2Entry { opcode: Some(crate::ir::Opcode::LoadComplex), offset: 0x0007c2 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Store), offset: 0x0007c8 },
    Level2Entry { opcode: Some(crate::ir::Opcode::StoreComplex), offset: 0x0007ce },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Ceil), offset: 0x0007b1 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Floor), offset: 0x0007b1 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Trunc), offset: 0x0007b1 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Nearest), offset: 0x0007b1 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::RawBitcast), offset: 0x0003f6 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fpromote), offset: 0x0007b9 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::FcvtFromSint), offset: 0x0007b6 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x000397 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x0003a2 },
    // 0004ee: f32, 64 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x0003a6 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BandNot), offset: 0x00039b },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x0003af },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x000471 },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x0003cf },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x0004f6 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Fmin), offset: 0x0004c1 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyToSsa), offset: 0x0007da },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000034 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Fmax), offset: 0x0004bd },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x00049d },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x000499 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x000495 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fcmp), offset: 0x000458 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Ffcmp), offset: 0x00046d },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fadd), offset: 0x000454 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fsub), offset: 0x000479 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fmul), offset: 0x000475 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fdiv), offset: 0x000469 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Sqrt), offset: 0x0004a1 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Load), offset: 0x0007e2 },
    Level2Entry { opcode: Some(crate::ir::Opcode::LoadComplex), offset: 0x0007e8 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Store), offset: 0x0007ee },
    Level2Entry { opcode: Some(crate::ir::Opcode::StoreComplex), offset: 0x0007f4 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Ceil), offset: 0x0007d7 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Floor), offset: 0x0007d7 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Trunc), offset: 0x0007d7 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Nearest), offset: 0x0007d7 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bitcast), offset: 0x0007d4 },
    Level2Entry { opcode: Some(crate::ir::Opcode::RawBitcast), offset: 0x0003f6 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fdemote), offset: 0x0007df },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::FcvtFromSint), offset: 0x0007dc },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x000397 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x0003a2 },
    // 00052e: typeless, 32 entries
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Jump), offset: 0x000378 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Brif), offset: 0x000806 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Brff), offset: 0x000802 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopySpecial), offset: 0x00080c },
    Level2Entry { opcode: Some(crate::ir::Opcode::Trueif), offset: 0x000393 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Trueff), offset: 0x00038f },
    Level2Entry { opcode: Some(crate::ir::Opcode::AdjustSpUpImm), offset: 0x0007fe },
    Level2Entry { opcode: Some(crate::ir::Opcode::AdjustSpDownImm), offset: 0x0007fa },
    Level2Entry { opcode: Some(crate::ir::Opcode::Debugtrap), offset: 0x00036c },
    Level2Entry { opcode: Some(crate::ir::Opcode::Trap), offset: 0x00037c },
    Level2Entry { opcode: Some(crate::ir::Opcode::ResumableTrap), offset: 0x00037c },
    Level2Entry { opcode: Some(crate::ir::Opcode::Safepoint), offset: 0x000380 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Trapif), offset: 0x00038b },
    Level2Entry { opcode: Some(crate::ir::Opcode::Trapff), offset: 0x000389 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Return), offset: 0x00037e },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Call), offset: 0x00080a },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::F32const), offset: 0x00080e },
    Level2Entry { opcode: Some(crate::ir::Opcode::F64const), offset: 0x000811 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    // 00054e: b8x16, 32 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x0004c9 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BandNot), offset: 0x0004c5 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x0003af },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x0004cb },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x0003cf },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x0004f6 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000034 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pshufb), offset: 0x000510 },
    Level2Entry { opcode: Some(crate::ir::Opcode::RawBitcast), offset: 0x0004d3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::ScalarToVector), offset: 0x000449 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x0004f8 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x0004f4 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pinsr), offset: 0x00050d },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x0004fa },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Ptest), offset: 0x000513 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pextr), offset: 0x00050a },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Load), offset: 0x0004cd },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Store), offset: 0x0004fc },
    Level2Entry { opcode: Some(crate::ir::Opcode::Vconst), offset: 0x000502 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x0004c3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x0004c7 },
    // 00056e: b16x8, 32 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x0004c9 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BandNot), offset: 0x0004c5 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x0003af },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x0004cb },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x0003cf },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x0004f6 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000034 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pshufb), offset: 0x000510 },
    Level2Entry { opcode: Some(crate::ir::Opcode::RawBitcast), offset: 0x000516 },
    Level2Entry { opcode: Some(crate::ir::Opcode::ScalarToVector), offset: 0x000449 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x0004f8 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x0004f4 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pinsr), offset: 0x00053a },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x0004fa },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Ptest), offset: 0x000513 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pextr), offset: 0x000537 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Load), offset: 0x0004cd },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Store), offset: 0x0004fc },
    Level2Entry { opcode: Some(crate::ir::Opcode::Vconst), offset: 0x000502 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x0004c3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x0004c7 },
    // 00058e: b32x4, 32 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x0004c9 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BandNot), offset: 0x0004c5 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x0003af },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x0004cb },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x0003cf },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x0004f6 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000034 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pshufd), offset: 0x000563 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pshufb), offset: 0x000510 },
    Level2Entry { opcode: Some(crate::ir::Opcode::RawBitcast), offset: 0x00053c },
    Level2Entry { opcode: Some(crate::ir::Opcode::ScalarToVector), offset: 0x000449 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x0004f8 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x0004f4 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pinsr), offset: 0x000560 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x0004fa },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Ptest), offset: 0x000513 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pextr), offset: 0x00055d },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Load), offset: 0x0004cd },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Store), offset: 0x0004fc },
    Level2Entry { opcode: Some(crate::ir::Opcode::Vconst), offset: 0x000502 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x0004c3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x0004c7 },
    // 0005ae: b64x2, 32 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x0004c9 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BandNot), offset: 0x0004c5 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x0003af },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x0004cb },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x0003cf },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x0004f6 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000034 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pshufb), offset: 0x000510 },
    Level2Entry { opcode: Some(crate::ir::Opcode::RawBitcast), offset: 0x000565 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x0004fa },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x0004f8 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x0004f4 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Ptest), offset: 0x000513 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Load), offset: 0x0004cd },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Store), offset: 0x0004fc },
    Level2Entry { opcode: Some(crate::ir::Opcode::Vconst), offset: 0x000502 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x0004c3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x0004c7 },
    // 0005ce: i8x16, 64 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x0004c9 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BandNot), offset: 0x0004c5 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x0003af },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x0004cb },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x0003cf },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x0004f6 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000034 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pshufb), offset: 0x000510 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pextr), offset: 0x00050a },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x0004fa },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x0004f8 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x0004f4 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pinsr), offset: 0x00050d },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Ptest), offset: 0x000513 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pmaxs), offset: 0x0005bf },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pmaxu), offset: 0x0005c2 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pmins), offset: 0x0005c4 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Load), offset: 0x0004cd },
    Level2Entry { opcode: Some(crate::ir::Opcode::Icmp), offset: 0x00058e },
    Level2Entry { opcode: Some(crate::ir::Opcode::Store), offset: 0x0004fc },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pminu), offset: 0x0005c7 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Iadd), offset: 0x00058c },
    Level2Entry { opcode: Some(crate::ir::Opcode::UaddSat), offset: 0x0005bb },
    Level2Entry { opcode: Some(crate::ir::Opcode::SaddSat), offset: 0x0005b7 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Isub), offset: 0x000594 },
    Level2Entry { opcode: Some(crate::ir::Opcode::UsubSat), offset: 0x0005bd },
    Level2Entry { opcode: Some(crate::ir::Opcode::SsubSat), offset: 0x0005b9 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::RawBitcast), offset: 0x000596 },
    Level2Entry { opcode: Some(crate::ir::Opcode::ScalarToVector), offset: 0x000449 },
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
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Vconst), offset: 0x000502 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x0004c3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x0004c7 },
    // 00060e: i16x8, 64 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x0004c9 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BandNot), offset: 0x0004c5 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x0003af },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x0004cb },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x0003cf },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x0004f6 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000034 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pshufb), offset: 0x000510 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pextr), offset: 0x000537 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x0004fa },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x0004f8 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x0004f4 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pinsr), offset: 0x00053a },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Psll), offset: 0x000608 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Psrl), offset: 0x00060c },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Psra), offset: 0x00060a },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Ptest), offset: 0x000513 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pmaxs), offset: 0x0005fe },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pmaxu), offset: 0x000600 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pmins), offset: 0x000603 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Load), offset: 0x0004cd },
    Level2Entry { opcode: Some(crate::ir::Opcode::Icmp), offset: 0x0005cb },
    Level2Entry { opcode: Some(crate::ir::Opcode::Store), offset: 0x0004fc },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pminu), offset: 0x000605 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Iadd), offset: 0x0005c9 },
    Level2Entry { opcode: Some(crate::ir::Opcode::UaddSat), offset: 0x0005fa },
    Level2Entry { opcode: Some(crate::ir::Opcode::SaddSat), offset: 0x0005f6 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Isub), offset: 0x0005d3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::UsubSat), offset: 0x0005fc },
    Level2Entry { opcode: Some(crate::ir::Opcode::SsubSat), offset: 0x0005f8 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Imul), offset: 0x0005d1 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::RawBitcast), offset: 0x0005d5 },
    Level2Entry { opcode: Some(crate::ir::Opcode::ScalarToVector), offset: 0x000449 },
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
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Vconst), offset: 0x000502 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x0004c3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x0004c7 },
    // 00064e: i32x4, 64 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x0004c9 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BandNot), offset: 0x0004c5 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x0003af },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x0004cb },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x0003cf },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x0004f6 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000034 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pshufd), offset: 0x000563 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pshufb), offset: 0x000510 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pextr), offset: 0x00055d },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x0004fa },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x0004f8 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x0004f4 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pinsr), offset: 0x000560 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Psll), offset: 0x000648 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Psrl), offset: 0x00064c },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Psra), offset: 0x00064a },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Ptest), offset: 0x000513 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pmaxs), offset: 0x00063c },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pmaxu), offset: 0x00063f },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pmins), offset: 0x000642 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Load), offset: 0x0004cd },
    Level2Entry { opcode: Some(crate::ir::Opcode::Icmp), offset: 0x000610 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Store), offset: 0x0004fc },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pminu), offset: 0x000645 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Iadd), offset: 0x00060e },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Isub), offset: 0x000619 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Imul), offset: 0x000616 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::RawBitcast), offset: 0x00061b },
    Level2Entry { opcode: Some(crate::ir::Opcode::ScalarToVector), offset: 0x000449 },
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
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Vconst), offset: 0x000502 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x0004c3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x0004c7 },
    // 00068e: i64x2, 32 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x0004c9 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Isub), offset: 0x000660 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BandNot), offset: 0x0004c5 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x0003af },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x0004cb },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x0003cf },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x0004f6 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000034 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bitcast), offset: 0x0007d4 },
    Level2Entry { opcode: Some(crate::ir::Opcode::RawBitcast), offset: 0x000662 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x0004fa },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x0004f8 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x0004f4 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pshufb), offset: 0x000510 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Psll), offset: 0x000683 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Psrl), offset: 0x000685 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Ptest), offset: 0x000513 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Load), offset: 0x0004cd },
    Level2Entry { opcode: Some(crate::ir::Opcode::Icmp), offset: 0x000658 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Store), offset: 0x0004fc },
    Level2Entry { opcode: Some(crate::ir::Opcode::Vconst), offset: 0x000502 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Iadd), offset: 0x000656 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x0004c3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x0004c7 },
    // 0006ae: f32x4, 32 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x0004c9 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BandNot), offset: 0x0004c5 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x0003af },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x0004cb },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x0003cf },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x0004f6 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000034 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pshufd), offset: 0x000563 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pshufb), offset: 0x000510 },
    Level2Entry { opcode: Some(crate::ir::Opcode::RawBitcast), offset: 0x000687 },
    Level2Entry { opcode: Some(crate::ir::Opcode::ScalarToVector), offset: 0x000412 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x0004f8 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x0004f4 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pinsr), offset: 0x000560 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Insertps), offset: 0x0006a8 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x0004fa },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Ptest), offset: 0x000513 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pextr), offset: 0x00055d },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Load), offset: 0x0004cd },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Store), offset: 0x0004fc },
    Level2Entry { opcode: Some(crate::ir::Opcode::Vconst), offset: 0x000502 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x0004c3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x0004c7 },
    // 0006ce: f64x2, 32 entries
    Level2Entry { opcode: Some(crate::ir::Opcode::Bxor), offset: 0x0004c9 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::BandNot), offset: 0x0004c5 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Copy), offset: 0x0003af },
    Level2Entry { opcode: Some(crate::ir::Opcode::Fill), offset: 0x0004cb },
    Level2Entry { opcode: Some(crate::ir::Opcode::FillNop), offset: 0x0003cf },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regmove), offset: 0x0004f6 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::CopyNop), offset: 0x000034 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Pshufb), offset: 0x000510 },
    Level2Entry { opcode: Some(crate::ir::Opcode::RawBitcast), offset: 0x0006ab },
    Level2Entry { opcode: Some(crate::ir::Opcode::ScalarToVector), offset: 0x000412 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regspill), offset: 0x0004f8 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Regfill), offset: 0x0004f4 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Movlhps), offset: 0x0006cc },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Spill), offset: 0x0004fa },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Ptest), offset: 0x000513 },
    Level2Entry { opcode: Some(crate::ir::Opcode::X86Movsd), offset: 0x0006ce },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Load), offset: 0x0004cd },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Store), offset: 0x0004fc },
    Level2Entry { opcode: Some(crate::ir::Opcode::Vconst), offset: 0x000502 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: None, offset: 0 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Band), offset: 0x0004c3 },
    Level2Entry { opcode: Some(crate::ir::Opcode::Bor), offset: 0x0004c7 },
];

/// x86 level 1 hash table for the CPU mode I64.
///
/// This hash table, keyed by instruction controlling type, contains all the level 2
/// hash-tables offsets for the given CPU mode, as well as a legalization identifier indicating
/// which legalization scheme to apply when the instruction doesn't have any valid encoding for
/// this CPU mode.
pub static LEVEL1_I64: [Level1Entry<u16>; 32] = [
    Level1Entry { ty: ir::types::INVALID, log2len: 5, offset: 0x00016a, legalize: 0 }, // expand_flags
    Level1Entry { ty: ir::types::F32X4, log2len: 5, offset: 0x00038a, legalize: 3 }, // x86_narrow
    Level1Entry { ty: ir::types::B16X8, log2len: 5, offset: 0x00022a, legalize: 3 }, // x86_narrow
    Level1Entry { ty: ir::types::INVALID, log2len: !0, offset: 0, legalize: 3 },
    Level1Entry { ty: ir::types::B64X2, log2len: 5, offset: 0x00026a, legalize: 3 }, // x86_narrow
    Level1Entry { ty: ir::types::I8X16, log2len: 6, offset: 0x00028a, legalize: 3 }, // x86_narrow
    Level1Entry { ty: ir::types::B8X16, log2len: 5, offset: 0x00020a, legalize: 3 }, // x86_narrow
    Level1Entry { ty: ir::types::I16X8, log2len: 6, offset: 0x0002ca, legalize: 3 }, // x86_narrow
    Level1Entry { ty: ir::types::INVALID, log2len: !0, offset: 0, legalize: 3 },
    Level1Entry { ty: ir::types::I64X2, log2len: 6, offset: 0x00034a, legalize: 3 }, // x86_narrow
    Level1Entry { ty: ir::types::INVALID, log2len: !0, offset: 0, legalize: 3 },
    Level1Entry { ty: ir::types::INVALID, log2len: !0, offset: 0, legalize: 3 },
    Level1Entry { ty: ir::types::F64X2, log2len: 5, offset: 0x0003aa, legalize: 3 }, // x86_narrow
    Level1Entry { ty: ir::types::I32X4, log2len: 6, offset: 0x00030a, legalize: 3 }, // x86_narrow
    Level1Entry { ty: ir::types::INVALID, log2len: !0, offset: 0, legalize: 3 },
    Level1Entry { ty: ir::types::INVALID, log2len: !0, offset: 0, legalize: 3 },
    Level1Entry { ty: ir::types::B1, log2len: 5, offset: 0x000110, legalize: 0 }, // expand_flags
    Level1Entry { ty: ir::types::B8, log2len: 2, offset: 0x000160, legalize: 3 }, // x86_narrow
    Level1Entry { ty: ir::types::B16, log2len: 2, offset: 0x000164, legalize: 3 }, // x86_narrow
    Level1Entry { ty: ir::types::B32, log2len: 3, offset: 0x000100, legalize: 3 }, // x86_narrow
    Level1Entry { ty: ir::types::B64, log2len: 3, offset: 0x000108, legalize: 3 }, // x86_narrow
    Level1Entry { ty: ir::types::INVALID, log2len: !0, offset: 0, legalize: 3 },
    Level1Entry { ty: ir::types::I8, log2len: 4, offset: 0x000140, legalize: 1 }, // widen
    Level1Entry { ty: ir::types::I16, log2len: 4, offset: 0x000150, legalize: 1 }, // widen
    Level1Entry { ty: ir::types::I32, log2len: 7, offset: 0x000080, legalize: 2 }, // x86_expand
    Level1Entry { ty: ir::types::I64, log2len: 7, offset: 0x000000, legalize: 2 }, // x86_expand
    Level1Entry { ty: ir::types::INVALID, log2len: !0, offset: 0, legalize: 3 },
    Level1Entry { ty: ir::types::F32, log2len: 6, offset: 0x0001ca, legalize: 2 }, // x86_expand
    Level1Entry { ty: ir::types::F64, log2len: 6, offset: 0x00018a, legalize: 2 }, // x86_expand
    Level1Entry { ty: ir::types::B32X4, log2len: 5, offset: 0x00024a, legalize: 3 }, // x86_narrow
    Level1Entry { ty: ir::types::R32, log2len: 1, offset: 0x000168, legalize: 3 }, // x86_narrow
    Level1Entry { ty: ir::types::R64, log2len: 4, offset: 0x000130, legalize: 3 }, // x86_narrow
];

/// x86 level 1 hash table for the CPU mode I32.
///
/// This hash table, keyed by instruction controlling type, contains all the level 2
/// hash-tables offsets for the given CPU mode, as well as a legalization identifier indicating
/// which legalization scheme to apply when the instruction doesn't have any valid encoding for
/// this CPU mode.
pub static LEVEL1_I32: [Level1Entry<u16>; 32] = [
    Level1Entry { ty: ir::types::INVALID, log2len: 5, offset: 0x00052e, legalize: 0 }, // expand_flags
    Level1Entry { ty: ir::types::F32X4, log2len: 5, offset: 0x0006ae, legalize: 4 }, // narrow_flags
    Level1Entry { ty: ir::types::B16X8, log2len: 5, offset: 0x00056e, legalize: 4 }, // narrow_flags
    Level1Entry { ty: ir::types::INVALID, log2len: !0, offset: 0, legalize: 4 },
    Level1Entry { ty: ir::types::B64X2, log2len: 5, offset: 0x0005ae, legalize: 4 }, // narrow_flags
    Level1Entry { ty: ir::types::I8X16, log2len: 6, offset: 0x0005ce, legalize: 4 }, // narrow_flags
    Level1Entry { ty: ir::types::INVALID, log2len: !0, offset: 0, legalize: 4 },
    Level1Entry { ty: ir::types::I16X8, log2len: 6, offset: 0x00060e, legalize: 4 }, // narrow_flags
    Level1Entry { ty: ir::types::INVALID, log2len: !0, offset: 0, legalize: 4 },
    Level1Entry { ty: ir::types::I64X2, log2len: 5, offset: 0x00068e, legalize: 4 }, // narrow_flags
    Level1Entry { ty: ir::types::INVALID, log2len: !0, offset: 0, legalize: 4 },
    Level1Entry { ty: ir::types::INVALID, log2len: !0, offset: 0, legalize: 4 },
    Level1Entry { ty: ir::types::F64X2, log2len: 5, offset: 0x0006ce, legalize: 4 }, // narrow_flags
    Level1Entry { ty: ir::types::I32X4, log2len: 6, offset: 0x00064e, legalize: 4 }, // narrow_flags
    Level1Entry { ty: ir::types::INVALID, log2len: !0, offset: 0, legalize: 4 },
    Level1Entry { ty: ir::types::INVALID, log2len: !0, offset: 0, legalize: 4 },
    Level1Entry { ty: ir::types::B1, log2len: 5, offset: 0x000452, legalize: 0 }, // expand_flags
    Level1Entry { ty: ir::types::B8, log2len: 2, offset: 0x0004a2, legalize: 4 }, // narrow_flags
    Level1Entry { ty: ir::types::B16, log2len: 2, offset: 0x0004a6, legalize: 4 }, // narrow_flags
    Level1Entry { ty: ir::types::B32, log2len: 3, offset: 0x00044a, legalize: 4 }, // narrow_flags
    Level1Entry { ty: ir::types::B8X16, log2len: 5, offset: 0x00054e, legalize: 4 }, // narrow_flags
    Level1Entry { ty: ir::types::INVALID, log2len: !0, offset: 0, legalize: 4 },
    Level1Entry { ty: ir::types::I8, log2len: 4, offset: 0x000482, legalize: 1 }, // widen
    Level1Entry { ty: ir::types::I16, log2len: 4, offset: 0x000492, legalize: 1 }, // widen
    Level1Entry { ty: ir::types::I32, log2len: 7, offset: 0x0003ca, legalize: 2 }, // x86_expand
    Level1Entry { ty: ir::types::I64, log2len: 2, offset: 0x0004aa, legalize: 4 }, // narrow_flags
    Level1Entry { ty: ir::types::INVALID, log2len: !0, offset: 0, legalize: 4 },
    Level1Entry { ty: ir::types::F32, log2len: 6, offset: 0x0004ee, legalize: 2 }, // x86_expand
    Level1Entry { ty: ir::types::F64, log2len: 6, offset: 0x0004ae, legalize: 2 }, // x86_expand
    Level1Entry { ty: ir::types::B32X4, log2len: 5, offset: 0x00058e, legalize: 4 }, // narrow_flags
    Level1Entry { ty: ir::types::R32, log2len: 4, offset: 0x000472, legalize: 4 }, // narrow_flags
    Level1Entry { ty: ir::types::INVALID, log2len: !0, offset: 0, legalize: 4 },
];

/// x86 recipe names, using the same recipe index spaces as the one specified by the
/// corresponding binemit file.
static RECIPE_NAMES: [&str; 289] = [
    "get_pinned_reg",
    "RexOp1set_pinned_reg",
    "Op1rr",
    "RexOp1rr",
    "Op1rout",
    "RexOp1rout",
    "Op1rin",
    "RexOp1rin",
    "Op1rio",
    "RexOp1rio",
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
    "Op1u_id_z",
    "RexOp1u_id_z",
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
    "fillnull",
    "ffillnull",
    "Op1pushq",
    "RexOp1pushq",
    "Op1popq",
    "RexOp1popq",
    "RexOp1copysp",
    "Op1copysp",
    "Op1umr_reg_to_ssa",
    "RexOp1umr_reg_to_ssa",
    "Mp2furm_reg_to_ssa",
    "RexMp2furm_reg_to_ssa",
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
    "RexOp2urm_noflags",
    "Op2urm_noflags_abcd",
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
    "Mp3fa",
    "Mp2r_ib_unsigned_fpr",
    "null_fpr",
    "Mp3r_ib_unsigned_r",
    "Mp2r_ib_unsigned_r",
    "RexMp3r_ib_unsigned_r",
    "Mp3fa_ib",
    "Mp3r_ib_unsigned_gpr",
    "RexMp3r_ib_unsigned_gpr",
    "Mp2vconst_optimized",
    "Op2vconst",
    "Op2fst",
    "Op2fstDisp8",
    "Op2fstDisp32",
    "Op2fld",
    "Op2fldDisp8",
    "Op2fldDisp32",
    "Op2fspillSib32",
    "Op2fregspill32",
    "Op2ffillSib32",
    "Op2fregfill32",
    "Mp2fax",
    "Mp3fcmp",
    "Mp2icscc_fpr",
    "Mp3icscc_fpr",
    "Op1pu_id_ref",
    "RexOp1pu_id_ref",
    "Op1is_zero",
    "RexOp1is_zero",
    "safepoint",
];

/// x86 recipe constraints list, using the same recipe index spaces as the one
/// specified by the corresponding binemit file. These constraints are used by register
/// allocation to select the right location to use for input and output values.
static RECIPE_CONSTRAINTS: [RecipeConstraints; 289] = [
    // Constraints for recipe get_pinned_reg:
    RecipeConstraints {
        ins: &[],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::FixedReg(15),
                regclass: &GPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: true,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1set_pinned_reg:
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
        clobbers_flags: false,
    },
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR8_DATA,
            },
        ],
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1rout:
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::FixedReg(32),
                regclass: &FLAG_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: true,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1rout:
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::FixedReg(32),
                regclass: &FLAG_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: true,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1rin:
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
            OperandConstraint {
                kind: ConstraintKind::FixedReg(32),
                regclass: &FLAG_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR8_DATA,
            },
        ],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1rin:
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
            OperandConstraint {
                kind: ConstraintKind::FixedReg(32),
                regclass: &FLAG_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR_DATA,
            },
        ],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1rio:
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
            OperandConstraint {
                kind: ConstraintKind::FixedTied(32),
                regclass: &FLAG_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::FixedTied(32),
                regclass: &FLAG_DATA,
            },
        ],
        fixed_ins: true,
        fixed_outs: true,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1rio:
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
            OperandConstraint {
                kind: ConstraintKind::FixedTied(32),
                regclass: &FLAG_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::FixedTied(32),
                regclass: &FLAG_DATA,
            },
        ],
        fixed_ins: true,
        fixed_outs: true,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1ur:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1ur:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR_DATA,
            },
        ],
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR8_DATA,
            },
        ],
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR_DATA,
            },
        ],
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
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1umr:
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
        clobbers_flags: false,
    },
    // Constraints for recipe Op1rmov:
    RecipeConstraints {
        ins: &[
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
    // Constraints for recipe RexOp1rmov:
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
        clobbers_flags: false,
    },
    // Constraints for recipe Op1r_ib:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1r_ib:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1r_id:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1r_id:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1pu_id:
    RecipeConstraints {
        ins: &[],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1pu_id:
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
    // Constraints for recipe RexOp1u_id:
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
    // Constraints for recipe RexOp1pu_iq:
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
    // Constraints for recipe Op1pu_id_bool:
    RecipeConstraints {
        ins: &[],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1pu_id_bool:
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
    // Constraints for recipe Op1u_id_z:
    RecipeConstraints {
        ins: &[],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1u_id_z:
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR8_DATA,
            },
        ],
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR_DATA,
            },
        ],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe Mp2urm:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexMp2urm:
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
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
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
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
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1spillSib32:
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
        clobbers_flags: false,
    },
    // Constraints for recipe Op1regspill32:
    RecipeConstraints {
        ins: &[
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
    // Constraints for recipe RexOp1regspill32:
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
        clobbers_flags: false,
    },
    // Constraints for recipe Op1ld:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1ld:
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
        clobbers_flags: false,
    },
    // Constraints for recipe Op2ld:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp2ld:
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
        clobbers_flags: false,
    },
    // Constraints for recipe Op1ldDisp8:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1ldDisp8:
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
        clobbers_flags: false,
    },
    // Constraints for recipe Op2ldDisp8:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp2ldDisp8:
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
        clobbers_flags: false,
    },
    // Constraints for recipe Op1ldDisp32:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1ldDisp32:
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
        clobbers_flags: false,
    },
    // Constraints for recipe Op2ldDisp32:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp2ldDisp32:
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
        clobbers_flags: false,
    },
    // Constraints for recipe Op1fillSib32:
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
                regclass: &GPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1fillSib32:
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
    // Constraints for recipe Op1regfill32:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Stack,
                regclass: &GPR_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1regfill32:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Stack,
                regclass: &GPR_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
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
    // Constraints for recipe ffillnull:
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
                regclass: &FPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op1pushq:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1pushq:
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
    // Constraints for recipe Op1popq:
    RecipeConstraints {
        ins: &[],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1popq:
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
    // Constraints for recipe Op1umr_reg_to_ssa:
    RecipeConstraints {
        ins: &[],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1umr_reg_to_ssa:
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
        clobbers_flags: false,
    },
    // Constraints for recipe Mp2furm_reg_to_ssa:
    RecipeConstraints {
        ins: &[],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp2furm_reg_to_ssa:
    RecipeConstraints {
        ins: &[],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
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
    // Constraints for recipe Op1adjustsp:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1adjustsp:
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
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp2fld:
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
                regclass: &FPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Mp2fldDisp8:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp2fldDisp8:
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
                regclass: &FPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Mp2fldDisp32:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp2fldDisp32:
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
                regclass: &FPR_DATA,
            },
        ],
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
            },
        ],
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
            },
        ],
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
            },
        ],
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
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Stack,
                regclass: &FPR_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp2ffillSib32:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Stack,
                regclass: &FPR_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Mp2fregfill32:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Stack,
                regclass: &FPR_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp2fregfill32:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Stack,
                regclass: &FPR_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Mp2fspillSib32:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Stack,
                regclass: &FPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp2fspillSib32:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Stack,
                regclass: &FPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Mp2fregspill32:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp2fregspill32:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op1fnaddr4:
    RecipeConstraints {
        ins: &[],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1fnaddr8:
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
    // Constraints for recipe Op1allones_fnaddr4:
    RecipeConstraints {
        ins: &[],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1allones_fnaddr8:
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
    // Constraints for recipe RexOp1pcrel_fnaddr8:
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
    // Constraints for recipe RexOp1got_fnaddr8:
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
    // Constraints for recipe Op1gvaddr4:
    RecipeConstraints {
        ins: &[],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1gvaddr8:
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
    // Constraints for recipe RexOp1pcrel_gvaddr8:
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
    // Constraints for recipe RexOp1got_gvaddr8:
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
    // Constraints for recipe Op1spaddr4_id:
    RecipeConstraints {
        ins: &[],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1spaddr8_id:
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
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1call_r:
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
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::FixedReg(32),
                regclass: &FLAG_DATA,
            },
        ],
        outs: &[],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1brib:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::FixedReg(32),
                regclass: &FLAG_DATA,
            },
        ],
        outs: &[],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op2brid:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::FixedReg(32),
                regclass: &FLAG_DATA,
            },
        ],
        outs: &[],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp2brid:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::FixedReg(32),
                regclass: &FLAG_DATA,
            },
        ],
        outs: &[],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op1brfb:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::FixedReg(32),
                regclass: &FLAG_DATA,
            },
        ],
        outs: &[],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1brfb:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::FixedReg(32),
                regclass: &FLAG_DATA,
            },
        ],
        outs: &[],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op2brfd:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::FixedReg(32),
                regclass: &FLAG_DATA,
            },
        ],
        outs: &[],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp2brfd:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::FixedReg(32),
                regclass: &FLAG_DATA,
            },
        ],
        outs: &[],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op1tjccb:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1tjccb:
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
    // Constraints for recipe Op1tjccd:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1tjccd:
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
    // Constraints for recipe Op1t8jccd_long:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1t8jccb_abcd:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &ABCD_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1t8jccb:
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
    // Constraints for recipe Op1t8jccd_abcd:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &ABCD_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1t8jccd:
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1jt_base:
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
        clobbers_flags: false,
    },
    // Constraints for recipe Op1jt_base:
    RecipeConstraints {
        ins: &[],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1indirect_jmp:
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
        clobbers_flags: false,
    },
    // Constraints for recipe Op1indirect_jmp:
    RecipeConstraints {
        ins: &[
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
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::FixedReg(32),
                regclass: &FLAG_DATA,
            },
        ],
        outs: &[],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe trapff:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::FixedReg(32),
                regclass: &FLAG_DATA,
            },
        ],
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &ABCD_DATA,
            },
        ],
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &ABCD_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1icscc_ib:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &ABCD_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1icscc_ib:
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
                regclass: &ABCD_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1icscc_id:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &ABCD_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1icscc_id:
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
                regclass: &ABCD_DATA,
            },
        ],
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
        outs: &[
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
        outs: &[
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
    // Constraints for recipe Op1rcmp_ib:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[
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
    // Constraints for recipe RexOp1rcmp_ib:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        outs: &[
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
    // Constraints for recipe Op1rcmp_id:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[
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
    // Constraints for recipe RexOp1rcmp_id:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        outs: &[
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
    // Constraints for recipe Op1rcmp_sp:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[
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
    // Constraints for recipe RexOp1rcmp_sp:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        outs: &[
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
    // Constraints for recipe Op2seti_abcd:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::FixedReg(32),
                regclass: &FLAG_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &ABCD_DATA,
            },
        ],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp2seti:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::FixedReg(32),
                regclass: &FLAG_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op2setf_abcd:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::FixedReg(32),
                regclass: &FLAG_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &ABCD_DATA,
            },
        ],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp2setf:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::FixedReg(32),
                regclass: &FLAG_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(2),
                regclass: &GPR8_DATA,
            },
        ],
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(2),
                regclass: &GPR_DATA,
            },
        ],
        fixed_ins: true,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: false,
    },
    // Constraints for recipe Op2bsf_and_bsr:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
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
    // Constraints for recipe RexOp2urm_noflags:
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
        clobbers_flags: false,
    },
    // Constraints for recipe Op2urm_noflags_abcd:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &ABCD_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe null:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &GPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe Op2urm_noflags:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp1urm_noflags:
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
        clobbers_flags: false,
    },
    // Constraints for recipe Op2f32imm_z:
    RecipeConstraints {
        ins: &[],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Mp2f64imm_z:
    RecipeConstraints {
        ins: &[],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp2f32imm_z:
    RecipeConstraints {
        ins: &[],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexMp2f64imm_z:
    RecipeConstraints {
        ins: &[],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe Mp2frurm:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp2frurm:
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
                regclass: &FPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Mp2rfumr:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp2rfumr:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
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
    // Constraints for recipe Op2furm:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp2furm:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op2frmov:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexOp2frmov:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Mp2furm:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp2furm:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Mp2rfurm:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe RexMp2rfurm:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
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
    // Constraints for recipe Mp3furmi_rnd:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexMp3furmi_rnd:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
            },
        ],
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &FPR8_DATA,
            },
        ],
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &FPR_DATA,
            },
        ],
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &FPR8_DATA,
            },
        ],
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &FPR_DATA,
            },
        ],
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(1),
                regclass: &FPR8_DATA,
            },
        ],
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(1),
                regclass: &FPR_DATA,
            },
        ],
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &ABCD_DATA,
            },
        ],
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &ABCD_DATA,
            },
        ],
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &ABCD_DATA,
            },
        ],
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &ABCD_DATA,
            },
        ],
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
        outs: &[
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
        outs: &[
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
        outs: &[
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
        outs: &[
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
    // Constraints for recipe Mp3fa:
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &FPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe Mp2r_ib_unsigned_fpr:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe null_fpr:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &FPR_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &FPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe Mp3r_ib_unsigned_r:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &FPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &FPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe Mp2r_ib_unsigned_r:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &FPR8_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &FPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe RexMp3r_ib_unsigned_r:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &FPR_DATA,
            },
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &FPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe Mp3fa_ib:
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &FPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe Mp3r_ib_unsigned_gpr:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexMp3r_ib_unsigned_gpr:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR_DATA,
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
    // Constraints for recipe Mp2vconst_optimized:
    RecipeConstraints {
        ins: &[],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op2vconst:
    RecipeConstraints {
        ins: &[],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op2fst:
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
    // Constraints for recipe Op2fstDisp8:
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
    // Constraints for recipe Op2fstDisp32:
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
    // Constraints for recipe Op2fld:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op2fldDisp8:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op2fldDisp32:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op2fspillSib32:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Stack,
                regclass: &FPR_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op2fregspill32:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op2ffillSib32:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Stack,
                regclass: &FPR_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &FPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Op2fregfill32:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Stack,
                regclass: &FPR_DATA,
            },
        ],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: false,
    },
    // Constraints for recipe Mp2fax:
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(1),
                regclass: &FPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe Mp3fcmp:
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
        outs: &[
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
    // Constraints for recipe Mp2icscc_fpr:
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &FPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe Mp3icscc_fpr:
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
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Tied(0),
                regclass: &FPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: true,
        clobbers_flags: true,
    },
    // Constraints for recipe Op1pu_id_ref:
    RecipeConstraints {
        ins: &[],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1pu_id_ref:
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
    // Constraints for recipe Op1is_zero:
    RecipeConstraints {
        ins: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &GPR8_DATA,
            },
        ],
        outs: &[
            OperandConstraint {
                kind: ConstraintKind::Reg,
                regclass: &ABCD_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe RexOp1is_zero:
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
                regclass: &ABCD_DATA,
            },
        ],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
    // Constraints for recipe safepoint:
    RecipeConstraints {
        ins: &[],
        outs: &[],
        fixed_ins: false,
        fixed_outs: false,
        tied_ops: false,
        clobbers_flags: true,
    },
];

/// x86 recipe sizing descriptors, using the same recipe index spaces as the one
/// specified by the corresponding binemit file. These are used to compute the final size of an
/// instruction, as well as to compute the range of branches.
static RECIPE_SIZING: [RecipeSizing; 289] = [
    // Code size information for recipe get_pinned_reg:
    RecipeSizing {
        base_size: 0,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1set_pinned_reg:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
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
    // Code size information for recipe Op1rout:
    RecipeSizing {
        base_size: 2,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1rout:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1rin:
    RecipeSizing {
        base_size: 2,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1rin:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1rio:
    RecipeSizing {
        base_size: 2,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1rio:
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
    // Code size information for recipe Op1u_id_z:
    RecipeSizing {
        base_size: 2,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1u_id_z:
    RecipeSizing {
        base_size: 3,
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
    // Code size information for recipe fillnull:
    RecipeSizing {
        base_size: 0,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe ffillnull:
    RecipeSizing {
        base_size: 0,
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
    // Code size information for recipe Op1umr_reg_to_ssa:
    RecipeSizing {
        base_size: 2,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1umr_reg_to_ssa:
    RecipeSizing {
        base_size: 3,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Mp2furm_reg_to_ssa:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexMp2furm_reg_to_ssa:
    RecipeSizing {
        base_size: 5,
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
        branch_range: Some(BranchRange { origin: 5, bits: 32 }),
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
        branch_range: Some(BranchRange { origin: 6, bits: 32 }),
    },
    // Code size information for recipe RexOp2brid:
    RecipeSizing {
        base_size: 7,
        compute_size: base_size,
        branch_range: Some(BranchRange { origin: 7, bits: 32 }),
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
        branch_range: Some(BranchRange { origin: 6, bits: 32 }),
    },
    // Code size information for recipe RexOp2brfd:
    RecipeSizing {
        base_size: 7,
        compute_size: base_size,
        branch_range: Some(BranchRange { origin: 7, bits: 32 }),
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
        branch_range: Some(BranchRange { origin: 8, bits: 32 }),
    },
    // Code size information for recipe RexOp1tjccd:
    RecipeSizing {
        base_size: 9,
        compute_size: base_size,
        branch_range: Some(BranchRange { origin: 9, bits: 32 }),
    },
    // Code size information for recipe Op1t8jccd_long:
    RecipeSizing {
        base_size: 12,
        compute_size: base_size,
        branch_range: Some(BranchRange { origin: 12, bits: 32 }),
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
        branch_range: Some(BranchRange { origin: 8, bits: 32 }),
    },
    // Code size information for recipe RexOp1t8jccd:
    RecipeSizing {
        base_size: 9,
        compute_size: base_size,
        branch_range: Some(BranchRange { origin: 9, bits: 32 }),
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
    // Code size information for recipe RexOp2urm_noflags:
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
    // Code size information for recipe Mp3fa:
    RecipeSizing {
        base_size: 5,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Mp2r_ib_unsigned_fpr:
    RecipeSizing {
        base_size: 5,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe null_fpr:
    RecipeSizing {
        base_size: 0,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Mp3r_ib_unsigned_r:
    RecipeSizing {
        base_size: 6,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Mp2r_ib_unsigned_r:
    RecipeSizing {
        base_size: 5,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexMp3r_ib_unsigned_r:
    RecipeSizing {
        base_size: 7,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Mp3fa_ib:
    RecipeSizing {
        base_size: 6,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Mp3r_ib_unsigned_gpr:
    RecipeSizing {
        base_size: 6,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexMp3r_ib_unsigned_gpr:
    RecipeSizing {
        base_size: 7,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Mp2vconst_optimized:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op2vconst:
    RecipeSizing {
        base_size: 7,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op2fst:
    RecipeSizing {
        base_size: 3,
        compute_size: size_plus_maybe_sib_or_offset_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe Op2fstDisp8:
    RecipeSizing {
        base_size: 4,
        compute_size: size_plus_maybe_sib_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe Op2fstDisp32:
    RecipeSizing {
        base_size: 7,
        compute_size: size_plus_maybe_sib_for_in_reg_1,
        branch_range: None,
    },
    // Code size information for recipe Op2fld:
    RecipeSizing {
        base_size: 3,
        compute_size: size_plus_maybe_sib_or_offset_for_in_reg_0,
        branch_range: None,
    },
    // Code size information for recipe Op2fldDisp8:
    RecipeSizing {
        base_size: 4,
        compute_size: size_plus_maybe_sib_for_in_reg_0,
        branch_range: None,
    },
    // Code size information for recipe Op2fldDisp32:
    RecipeSizing {
        base_size: 7,
        compute_size: size_plus_maybe_sib_for_in_reg_0,
        branch_range: None,
    },
    // Code size information for recipe Op2fspillSib32:
    RecipeSizing {
        base_size: 8,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op2fregspill32:
    RecipeSizing {
        base_size: 8,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op2ffillSib32:
    RecipeSizing {
        base_size: 8,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op2fregfill32:
    RecipeSizing {
        base_size: 8,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Mp2fax:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Mp3fcmp:
    RecipeSizing {
        base_size: 5,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Mp2icscc_fpr:
    RecipeSizing {
        base_size: 4,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Mp3icscc_fpr:
    RecipeSizing {
        base_size: 5,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1pu_id_ref:
    RecipeSizing {
        base_size: 5,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1pu_id_ref:
    RecipeSizing {
        base_size: 6,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe Op1is_zero:
    RecipeSizing {
        base_size: 5,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe RexOp1is_zero:
    RecipeSizing {
        base_size: 6,
        compute_size: base_size,
        branch_range: None,
    },
    // Code size information for recipe safepoint:
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

 //clude!(concat!(env!("OUT_DIR"), "/encoding-x86.rs"));
 
/// Legalize instructions by expansion.
///
/// Use x86-specific instructions if needed.
#[allow(unused_variables,unused_assignments,non_snake_case)]
pub fn x86_expand(
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
            ir::Opcode::Clz => {
                // Unwrap fields from instruction format a := clz.i64(x)
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

                // Results handled by a := isub(c_sixty_three, index2).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I64 {
                    let c_minus_one = pos.ins().iconst(ir::types::I64, -1);
                    let c_sixty_three = pos.ins().iconst(ir::types::I64, 63);
                    let (index1, r2flags) = pos.ins().x86_bsr(x);
                    let index2 = pos.ins().selectif(ir::types::I64, ir::condcodes::IntCC::Equal, r2flags, c_minus_one, index1);
                    let a = pos.func.dfg.replace(inst).isub(c_sixty_three, index2);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I32 {
                    let c_minus_one = pos.ins().iconst(ir::types::I32, -1);
                    let c_thirty_one = pos.ins().iconst(ir::types::I32, 31);
                    let (index1, r2flags) = pos.ins().x86_bsr(x);
                    let index2 = pos.ins().selectif(ir::types::I32, ir::condcodes::IntCC::Equal, r2flags, c_minus_one, index1);
                    let a = pos.func.dfg.replace(inst).isub(c_thirty_one, index2);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Ctz => {
                // Unwrap fields from instruction format a := ctz.i64(x)
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

                // Results handled by a := selectif(ir::condcodes::IntCC::Equal, r2flags, c_sixty_four, index1).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I64 {
                    let c_sixty_four = pos.ins().iconst(ir::types::I64, 64);
                    let (index1, r2flags) = pos.ins().x86_bsf(x);
                    let a = pos.func.dfg.replace(inst).selectif(ir::types::I64, ir::condcodes::IntCC::Equal, r2flags, c_sixty_four, index1);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I32 {
                    let c_thirty_two = pos.ins().iconst(ir::types::I32, 32);
                    let (index1, r2flags) = pos.ins().x86_bsf(x);
                    let a = pos.func.dfg.replace(inst).selectif(ir::types::I32, ir::condcodes::IntCC::Equal, r2flags, c_thirty_two, index1);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Fcmp => {
                // Unwrap fields from instruction format a := fcmp(ir::condcodes::FloatCC::Equal, x, y)
                let (cond, x, y, args) = if let ir::InstructionData::FloatCompare {
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

                let typeof_x = pos.func.dfg.value_type(x);
                // Results handled by a := band(a1, a2).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if predicates::is_equal(cond, ir::condcodes::FloatCC::Equal) {
                    let a1 = pos.ins().fcmp(ir::condcodes::FloatCC::Ordered, x, y);
                    let a2 = pos.ins().fcmp(ir::condcodes::FloatCC::UnorderedOrEqual, x, y);
                    let a = pos.func.dfg.replace(inst).band(a1, a2);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::FloatCC::NotEqual) {
                    let a1 = pos.ins().fcmp(ir::condcodes::FloatCC::Unordered, x, y);
                    let a2 = pos.ins().fcmp(ir::condcodes::FloatCC::OrderedNotEqual, x, y);
                    let a = pos.func.dfg.replace(inst).bor(a1, a2);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::FloatCC::LessThan) {
                    let a = pos.func.dfg.replace(inst).fcmp(ir::condcodes::FloatCC::GreaterThan, y, x);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::FloatCC::LessThanOrEqual) {
                    let a = pos.func.dfg.replace(inst).fcmp(ir::condcodes::FloatCC::GreaterThanOrEqual, y, x);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::FloatCC::UnorderedOrGreaterThan) {
                    let a = pos.func.dfg.replace(inst).fcmp(ir::condcodes::FloatCC::UnorderedOrLessThan, y, x);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::FloatCC::UnorderedOrGreaterThanOrEqual) {
                    let a = pos.func.dfg.replace(inst).fcmp(ir::condcodes::FloatCC::UnorderedOrLessThanOrEqual, y, x);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Popcnt => {
                // Unwrap fields from instruction format r := popcnt.i64(x)
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

                // Results handled by r := ushr_imm(qv15, 56).
                let r = pos.func.dfg.inst_results(inst);
                let r = &r[0];
                let typeof_r = pos.func.dfg.value_type(*r);

                if pos.func.dfg.value_type(args[0]) == ir::types::I64 {
                    let qv3 = pos.ins().ushr_imm(x, 1);
                    let qc77 = pos.ins().iconst(ir::types::I64, 8608480567731124087);
                    let qv4 = pos.ins().band(qv3, qc77);
                    let qv5 = pos.ins().isub(x, qv4);
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
                    let r = pos.func.dfg.replace(inst).ushr_imm(qv15, 56);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I32 {
                    let lv3 = pos.ins().ushr_imm(x, 1);
                    let lc77 = pos.ins().iconst(ir::types::I32, 2004318071);
                    let lv4 = pos.ins().band(lv3, lc77);
                    let lv5 = pos.ins().isub(x, lv4);
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
                    let r = pos.func.dfg.replace(inst).ushr_imm(lv15, 24);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Smulhi => {
                // Unwrap fields from instruction format res_hi := smulhi(x, y)
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
                let res_hi;
                {
                    let r = pos.func.dfg.inst_results(inst);
                    res_hi = r[0];
                }

                let predicate = true;
                // typeof_x must belong to TypeSet(lanes={1}, ints={32, 64})
                let predicate = predicate && TYPE_SETS[0].contains(typeof_x);
                if predicate {
                    pos.func.dfg.clear_results(inst);
                    let (res_lo, res_hi) = pos.ins().with_results([None, Some(res_hi)]).x86_smulx(x, y);
                    let removed = pos.remove_inst();
                    debug_assert_eq!(removed, inst);
                    return true;
                }
            }

            ir::Opcode::Umulhi => {
                // Unwrap fields from instruction format res_hi := umulhi(x, y)
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
                let res_hi;
                {
                    let r = pos.func.dfg.inst_results(inst);
                    res_hi = r[0];
                }

                let predicate = true;
                // typeof_x must belong to TypeSet(lanes={1}, ints={32, 64})
                let predicate = predicate && TYPE_SETS[0].contains(typeof_x);
                if predicate {
                    pos.func.dfg.clear_results(inst);
                    let (res_lo, res_hi) = pos.ins().with_results([None, Some(res_hi)]).x86_umulx(x, y);
                    let removed = pos.remove_inst();
                    debug_assert_eq!(removed, inst);
                    return true;
                }
            }

            ir::Opcode::FcvtFromUint => {
                expand_fcvt_from_uint(inst, func, cfg, isa);
                return true;
            }

            ir::Opcode::FcvtToSint => {
                expand_fcvt_to_sint(inst, func, cfg, isa);
                return true;
            }

            ir::Opcode::FcvtToSintSat => {
                expand_fcvt_to_sint_sat(inst, func, cfg, isa);
                return true;
            }

            ir::Opcode::FcvtToUint => {
                expand_fcvt_to_uint(inst, func, cfg, isa);
                return true;
            }

            ir::Opcode::FcvtToUintSat => {
                expand_fcvt_to_uint_sat(inst, func, cfg, isa);
                return true;
            }

            ir::Opcode::Fmax => {
                expand_minmax(inst, func, cfg, isa);
                return true;
            }

            ir::Opcode::Fmin => {
                expand_minmax(inst, func, cfg, isa);
                return true;
            }

            ir::Opcode::Sdiv => {
                expand_sdivrem(inst, func, cfg, isa);
                return true;
            }

            ir::Opcode::Srem => {
                expand_sdivrem(inst, func, cfg, isa);
                return true;
            }

            ir::Opcode::Udiv => {
                expand_udivrem(inst, func, cfg, isa);
                return true;
            }

            ir::Opcode::Urem => {
                expand_udivrem(inst, func, cfg, isa);
                return true;
            }

            _ => {},
        }
    }
    crate::legalizer::expand_flags(inst, func, cfg, isa)
}

/// Legalize instructions by narrowing.
///
/// Use x86-specific instructions if needed.
#[allow(unused_variables,unused_assignments,non_snake_case)]
pub fn x86_narrow(
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
            ir::Opcode::Bitselect => {
                // Unwrap fields from instruction format d := bitselect.b8x16(c, x, y)
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

                // Results handled by d := bor(a, b).
                let r = pos.func.dfg.inst_results(inst);
                let d = &r[0];
                let typeof_d = pos.func.dfg.value_type(*d);

                if pos.func.dfg.value_type(args[0]) == ir::types::B8X16 {
                    let a = pos.ins().band(x, c);
                    let b = pos.ins().band_not(y, c);
                    let d = pos.func.dfg.replace(inst).bor(a, b);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::B16X8 {
                    let a = pos.ins().band(x, c);
                    let b = pos.ins().band_not(y, c);
                    let d = pos.func.dfg.replace(inst).bor(a, b);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::B32X4 {
                    let a = pos.ins().band(x, c);
                    let b = pos.ins().band_not(y, c);
                    let d = pos.func.dfg.replace(inst).bor(a, b);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::B64X2 {
                    let a = pos.ins().band(x, c);
                    let b = pos.ins().band_not(y, c);
                    let d = pos.func.dfg.replace(inst).bor(a, b);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I8X16 {
                    let a = pos.ins().band(x, c);
                    let b = pos.ins().band_not(y, c);
                    let d = pos.func.dfg.replace(inst).bor(a, b);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16X8 {
                    let a = pos.ins().band(x, c);
                    let b = pos.ins().band_not(y, c);
                    let d = pos.func.dfg.replace(inst).bor(a, b);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I32X4 {
                    let a = pos.ins().band(x, c);
                    let b = pos.ins().band_not(y, c);
                    let d = pos.func.dfg.replace(inst).bor(a, b);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I64X2 {
                    let a = pos.ins().band(x, c);
                    let b = pos.ins().band_not(y, c);
                    let d = pos.func.dfg.replace(inst).bor(a, b);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::F32X4 {
                    let a = pos.ins().band(x, c);
                    let b = pos.ins().band_not(y, c);
                    let d = pos.func.dfg.replace(inst).bor(a, b);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::F64X2 {
                    let a = pos.ins().band(x, c);
                    let b = pos.ins().band_not(y, c);
                    let d = pos.func.dfg.replace(inst).bor(a, b);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Bnot => {
                // Unwrap fields from instruction format y := bnot.b8x16(x)
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

                // Results handled by y := bxor(a, x).
                let r = pos.func.dfg.inst_results(inst);
                let y = &r[0];
                let typeof_y = pos.func.dfg.value_type(*y);

                if pos.func.dfg.value_type(args[0]) == ir::types::B8X16 {
                    let const0 = pos.func.dfg.constants.insert(vec![255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255].into());
                    let a = pos.ins().vconst(ir::types::B8X16, const0);
                    let y = pos.func.dfg.replace(inst).bxor(a, x);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::B16X8 {
                    let const0 = pos.func.dfg.constants.insert(vec![255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255].into());
                    let a = pos.ins().vconst(ir::types::B16X8, const0);
                    let y = pos.func.dfg.replace(inst).bxor(a, x);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::B32X4 {
                    let const0 = pos.func.dfg.constants.insert(vec![255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255].into());
                    let a = pos.ins().vconst(ir::types::B32X4, const0);
                    let y = pos.func.dfg.replace(inst).bxor(a, x);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::B64X2 {
                    let const0 = pos.func.dfg.constants.insert(vec![255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255].into());
                    let a = pos.ins().vconst(ir::types::B64X2, const0);
                    let y = pos.func.dfg.replace(inst).bxor(a, x);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I8X16 {
                    let const0 = pos.func.dfg.constants.insert(vec![255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255].into());
                    let a = pos.ins().vconst(ir::types::I8X16, const0);
                    let y = pos.func.dfg.replace(inst).bxor(a, x);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16X8 {
                    let const0 = pos.func.dfg.constants.insert(vec![255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255].into());
                    let a = pos.ins().vconst(ir::types::I16X8, const0);
                    let y = pos.func.dfg.replace(inst).bxor(a, x);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I32X4 {
                    let const0 = pos.func.dfg.constants.insert(vec![255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255].into());
                    let a = pos.ins().vconst(ir::types::I32X4, const0);
                    let y = pos.func.dfg.replace(inst).bxor(a, x);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I64X2 {
                    let const0 = pos.func.dfg.constants.insert(vec![255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255].into());
                    let a = pos.ins().vconst(ir::types::I64X2, const0);
                    let y = pos.func.dfg.replace(inst).bxor(a, x);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::F32X4 {
                    let const0 = pos.func.dfg.constants.insert(vec![255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255].into());
                    let a = pos.ins().vconst(ir::types::F32X4, const0);
                    let y = pos.func.dfg.replace(inst).bxor(a, x);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::F64X2 {
                    let const0 = pos.func.dfg.constants.insert(vec![255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255].into());
                    let a = pos.ins().vconst(ir::types::F64X2, const0);
                    let y = pos.func.dfg.replace(inst).bxor(a, x);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Icmp => {
                // Unwrap fields from instruction format c := icmp.i8x16(ir::condcodes::IntCC::NotEqual, a, b)
                let (cond, a, b, args) = if let ir::InstructionData::IntCompare {
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

                // Results handled by c := bnot(x).
                let r = pos.func.dfg.inst_results(inst);
                let c = &r[0];
                let typeof_c = pos.func.dfg.value_type(*c);

                if predicates::is_equal(cond, ir::condcodes::IntCC::NotEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I8X16 {
                    let x = pos.ins().icmp(ir::condcodes::IntCC::Equal, a, b);
                    let c = pos.func.dfg.replace(inst).bnot(x);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::NotEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I16X8 {
                    let x = pos.ins().icmp(ir::condcodes::IntCC::Equal, a, b);
                    let c = pos.func.dfg.replace(inst).bnot(x);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::NotEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I32X4 {
                    let x = pos.ins().icmp(ir::condcodes::IntCC::Equal, a, b);
                    let c = pos.func.dfg.replace(inst).bnot(x);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::NotEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I64X2 {
                    let x = pos.ins().icmp(ir::condcodes::IntCC::Equal, a, b);
                    let c = pos.func.dfg.replace(inst).bnot(x);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedGreaterThan) && pos.func.dfg.value_type(args[0]) == ir::types::I8X16 {
                    let x = pos.ins().x86_pmaxu(a, b);
                    let c = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::Equal, a, x);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedGreaterThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I8X16 {
                    let x = pos.ins().x86_pmins(a, b);
                    let c = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::Equal, x, b);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedGreaterThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I8X16 {
                    let x = pos.ins().x86_pminu(a, b);
                    let c = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::Equal, x, b);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedLessThan) && pos.func.dfg.value_type(args[0]) == ir::types::I8X16 {
                    let c = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::SignedGreaterThan, b, a);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedLessThan) && pos.func.dfg.value_type(args[0]) == ir::types::I8X16 {
                    let c = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::UnsignedGreaterThan, b, a);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedLessThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I8X16 {
                    let c = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::SignedGreaterThanOrEqual, b, a);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedLessThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I8X16 {
                    let c = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::UnsignedGreaterThanOrEqual, b, a);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedGreaterThan) && pos.func.dfg.value_type(args[0]) == ir::types::I16X8 {
                    let x = pos.ins().x86_pmaxu(a, b);
                    let c = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::Equal, a, x);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedGreaterThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I16X8 {
                    let x = pos.ins().x86_pmins(a, b);
                    let c = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::Equal, x, b);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedGreaterThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I16X8 {
                    let x = pos.ins().x86_pminu(a, b);
                    let c = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::Equal, x, b);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedLessThan) && pos.func.dfg.value_type(args[0]) == ir::types::I16X8 {
                    let c = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::SignedGreaterThan, b, a);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedLessThan) && pos.func.dfg.value_type(args[0]) == ir::types::I16X8 {
                    let c = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::UnsignedGreaterThan, b, a);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedLessThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I16X8 {
                    let c = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::SignedGreaterThanOrEqual, b, a);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedLessThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I16X8 {
                    let c = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::UnsignedGreaterThanOrEqual, b, a);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedGreaterThan) && pos.func.dfg.value_type(args[0]) == ir::types::I32X4 {
                    let x = pos.ins().x86_pmaxu(a, b);
                    let c = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::Equal, a, x);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedGreaterThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I32X4 {
                    let x = pos.ins().x86_pmins(a, b);
                    let c = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::Equal, x, b);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedGreaterThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I32X4 {
                    let x = pos.ins().x86_pminu(a, b);
                    let c = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::Equal, x, b);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedLessThan) && pos.func.dfg.value_type(args[0]) == ir::types::I32X4 {
                    let c = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::SignedGreaterThan, b, a);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedLessThan) && pos.func.dfg.value_type(args[0]) == ir::types::I32X4 {
                    let c = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::UnsignedGreaterThan, b, a);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::SignedLessThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I32X4 {
                    let c = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::SignedGreaterThanOrEqual, b, a);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if predicates::is_equal(cond, ir::condcodes::IntCC::UnsignedLessThanOrEqual) && pos.func.dfg.value_type(args[0]) == ir::types::I32X4 {
                    let c = pos.func.dfg.replace(inst).icmp(ir::condcodes::IntCC::UnsignedGreaterThanOrEqual, b, a);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Ishl => {
                // Unwrap fields from instruction format a := ishl.i16x8(x, y)
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

                let typeof_y = pos.func.dfg.value_type(y);
                // Results handled by a := x86_psll(x, b).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I16X8 {
                    let b = pos.ins().bitcast(ir::types::I64X2, y);
                    let a = pos.func.dfg.replace(inst).x86_psll(x, b);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I32X4 {
                    let b = pos.ins().bitcast(ir::types::I64X2, y);
                    let a = pos.func.dfg.replace(inst).x86_psll(x, b);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I64X2 {
                    let b = pos.ins().bitcast(ir::types::I64X2, y);
                    let a = pos.func.dfg.replace(inst).x86_psll(x, b);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Splat => {
                // Unwrap fields from instruction format y := splat.b8x16(x)
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

                // Results handled by y := x86_pshufb(a, c).
                let r = pos.func.dfg.inst_results(inst);
                let y = &r[0];
                let typeof_y = pos.func.dfg.value_type(*y);

                if pos.func.dfg.ctrl_typevar(inst) == ir::types::B8X16 {
                    let a = pos.ins().scalar_to_vector(ir::types::B8X16, x);
                    let b = pos.ins().f64const(0);
                    let c = pos.ins().raw_bitcast(ir::types::B8X16, b);
                    let y = pos.func.dfg.replace(inst).x86_pshufb(a, c);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.ctrl_typevar(inst) == ir::types::I8X16 {
                    let a = pos.ins().scalar_to_vector(ir::types::I8X16, x);
                    let b = pos.ins().f64const(0);
                    let c = pos.ins().raw_bitcast(ir::types::I8X16, b);
                    let y = pos.func.dfg.replace(inst).x86_pshufb(a, c);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.ctrl_typevar(inst) == ir::types::B16X8 {
                    let a = pos.ins().scalar_to_vector(ir::types::B16X8, x);
                    let b = pos.ins().insertlane(a, 1, x);
                    let c = pos.ins().raw_bitcast(ir::types::I32X4, b);
                    let d = pos.ins().x86_pshufd(c, 0);
                    let y = pos.func.dfg.replace(inst).raw_bitcast(ir::types::B16X8, d);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.ctrl_typevar(inst) == ir::types::I16X8 {
                    let a = pos.ins().scalar_to_vector(ir::types::I16X8, x);
                    let b = pos.ins().insertlane(a, 1, x);
                    let c = pos.ins().raw_bitcast(ir::types::I32X4, b);
                    let d = pos.ins().x86_pshufd(c, 0);
                    let y = pos.func.dfg.replace(inst).raw_bitcast(ir::types::I16X8, d);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.ctrl_typevar(inst) == ir::types::B32X4 {
                    let a = pos.ins().scalar_to_vector(ir::types::B32X4, x);
                    let y = pos.func.dfg.replace(inst).x86_pshufd(a, 0);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.ctrl_typevar(inst) == ir::types::I32X4 {
                    let a = pos.ins().scalar_to_vector(ir::types::I32X4, x);
                    let y = pos.func.dfg.replace(inst).x86_pshufd(a, 0);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.ctrl_typevar(inst) == ir::types::F32X4 {
                    let a = pos.ins().scalar_to_vector(ir::types::F32X4, x);
                    let y = pos.func.dfg.replace(inst).x86_pshufd(a, 0);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.ctrl_typevar(inst) == ir::types::B64X2 {
                    let a = pos.ins().scalar_to_vector(ir::types::B64X2, x);
                    let y = pos.func.dfg.replace(inst).insertlane(a, 1, x);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.ctrl_typevar(inst) == ir::types::I64X2 {
                    let a = pos.ins().scalar_to_vector(ir::types::I64X2, x);
                    let y = pos.func.dfg.replace(inst).insertlane(a, 1, x);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.ctrl_typevar(inst) == ir::types::F64X2 {
                    let a = pos.ins().scalar_to_vector(ir::types::F64X2, x);
                    let y = pos.func.dfg.replace(inst).insertlane(a, 1, x);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Sshr => {
                // Unwrap fields from instruction format a := sshr.i16x8(x, y)
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

                let typeof_y = pos.func.dfg.value_type(y);
                // Results handled by a := x86_psra(x, b).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I16X8 {
                    let b = pos.ins().bitcast(ir::types::I64X2, y);
                    let a = pos.func.dfg.replace(inst).x86_psra(x, b);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I32X4 {
                    let b = pos.ins().bitcast(ir::types::I64X2, y);
                    let a = pos.func.dfg.replace(inst).x86_psra(x, b);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I64X2 {
                    let b = pos.ins().bitcast(ir::types::I64X2, y);
                    let a = pos.func.dfg.replace(inst).x86_psra(x, b);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Ushr => {
                // Unwrap fields from instruction format a := ushr.i16x8(x, y)
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

                let typeof_y = pos.func.dfg.value_type(y);
                // Results handled by a := x86_psrl(x, b).
                let r = pos.func.dfg.inst_results(inst);
                let a = &r[0];
                let typeof_a = pos.func.dfg.value_type(*a);

                if pos.func.dfg.value_type(args[0]) == ir::types::I16X8 {
                    let b = pos.ins().bitcast(ir::types::I64X2, y);
                    let a = pos.func.dfg.replace(inst).x86_psrl(x, b);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I32X4 {
                    let b = pos.ins().bitcast(ir::types::I64X2, y);
                    let a = pos.func.dfg.replace(inst).x86_psrl(x, b);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I64X2 {
                    let b = pos.ins().bitcast(ir::types::I64X2, y);
                    let a = pos.func.dfg.replace(inst).x86_psrl(x, b);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::VallTrue => {
                // Unwrap fields from instruction format y := vall_true.b8x16(x)
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

                // Results handled by y := trueif(ir::condcodes::IntCC::Equal, d).
                let r = pos.func.dfg.inst_results(inst);
                let y = &r[0];
                let typeof_y = pos.func.dfg.value_type(*y);

                if pos.func.dfg.value_type(args[0]) == ir::types::B8X16 {
                    let const0 = pos.func.dfg.constants.insert(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].into());
                    let a = pos.ins().vconst(ir::types::I8X16, const0);
                    let b = pos.ins().raw_bitcast(ir::types::I8X16, x);
                    let c = pos.ins().icmp(ir::condcodes::IntCC::Equal, b, a);
                    let d = pos.ins().x86_ptest(c, c);
                    let y = pos.func.dfg.replace(inst).trueif(ir::condcodes::IntCC::Equal, d);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::B16X8 {
                    let const0 = pos.func.dfg.constants.insert(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].into());
                    let a = pos.ins().vconst(ir::types::I16X8, const0);
                    let b = pos.ins().raw_bitcast(ir::types::I16X8, x);
                    let c = pos.ins().icmp(ir::condcodes::IntCC::Equal, b, a);
                    let d = pos.ins().x86_ptest(c, c);
                    let y = pos.func.dfg.replace(inst).trueif(ir::condcodes::IntCC::Equal, d);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::B32X4 {
                    let const0 = pos.func.dfg.constants.insert(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].into());
                    let a = pos.ins().vconst(ir::types::I32X4, const0);
                    let b = pos.ins().raw_bitcast(ir::types::I32X4, x);
                    let c = pos.ins().icmp(ir::condcodes::IntCC::Equal, b, a);
                    let d = pos.ins().x86_ptest(c, c);
                    let y = pos.func.dfg.replace(inst).trueif(ir::condcodes::IntCC::Equal, d);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::B64X2 {
                    let const0 = pos.func.dfg.constants.insert(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].into());
                    let a = pos.ins().vconst(ir::types::I64X2, const0);
                    let b = pos.ins().raw_bitcast(ir::types::I64X2, x);
                    let c = pos.ins().icmp(ir::condcodes::IntCC::Equal, b, a);
                    let d = pos.ins().x86_ptest(c, c);
                    let y = pos.func.dfg.replace(inst).trueif(ir::condcodes::IntCC::Equal, d);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I8X16 {
                    let const0 = pos.func.dfg.constants.insert(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].into());
                    let a = pos.ins().vconst(ir::types::I8X16, const0);
                    let c = pos.ins().icmp(ir::condcodes::IntCC::Equal, x, a);
                    let d = pos.ins().x86_ptest(c, c);
                    let y = pos.func.dfg.replace(inst).trueif(ir::condcodes::IntCC::Equal, d);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16X8 {
                    let const0 = pos.func.dfg.constants.insert(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].into());
                    let a = pos.ins().vconst(ir::types::I16X8, const0);
                    let c = pos.ins().icmp(ir::condcodes::IntCC::Equal, x, a);
                    let d = pos.ins().x86_ptest(c, c);
                    let y = pos.func.dfg.replace(inst).trueif(ir::condcodes::IntCC::Equal, d);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I32X4 {
                    let const0 = pos.func.dfg.constants.insert(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].into());
                    let a = pos.ins().vconst(ir::types::I32X4, const0);
                    let c = pos.ins().icmp(ir::condcodes::IntCC::Equal, x, a);
                    let d = pos.ins().x86_ptest(c, c);
                    let y = pos.func.dfg.replace(inst).trueif(ir::condcodes::IntCC::Equal, d);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I64X2 {
                    let const0 = pos.func.dfg.constants.insert(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].into());
                    let a = pos.ins().vconst(ir::types::I64X2, const0);
                    let c = pos.ins().icmp(ir::condcodes::IntCC::Equal, x, a);
                    let d = pos.ins().x86_ptest(c, c);
                    let y = pos.func.dfg.replace(inst).trueif(ir::condcodes::IntCC::Equal, d);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::F32X4 {
                    let const0 = pos.func.dfg.constants.insert(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].into());
                    let a = pos.ins().vconst(ir::types::I32X4, const0);
                    let b = pos.ins().raw_bitcast(ir::types::I32X4, x);
                    let c = pos.ins().icmp(ir::condcodes::IntCC::Equal, b, a);
                    let d = pos.ins().x86_ptest(c, c);
                    let y = pos.func.dfg.replace(inst).trueif(ir::condcodes::IntCC::Equal, d);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::F64X2 {
                    let const0 = pos.func.dfg.constants.insert(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].into());
                    let a = pos.ins().vconst(ir::types::I64X2, const0);
                    let b = pos.ins().raw_bitcast(ir::types::I64X2, x);
                    let c = pos.ins().icmp(ir::condcodes::IntCC::Equal, b, a);
                    let d = pos.ins().x86_ptest(c, c);
                    let y = pos.func.dfg.replace(inst).trueif(ir::condcodes::IntCC::Equal, d);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::VanyTrue => {
                // Unwrap fields from instruction format y := vany_true.b8x16(x)
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

                // Results handled by y := trueif(ir::condcodes::IntCC::NotEqual, a).
                let r = pos.func.dfg.inst_results(inst);
                let y = &r[0];
                let typeof_y = pos.func.dfg.value_type(*y);

                if pos.func.dfg.value_type(args[0]) == ir::types::B8X16 {
                    let a = pos.ins().x86_ptest(x, x);
                    let y = pos.func.dfg.replace(inst).trueif(ir::condcodes::IntCC::NotEqual, a);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::B16X8 {
                    let a = pos.ins().x86_ptest(x, x);
                    let y = pos.func.dfg.replace(inst).trueif(ir::condcodes::IntCC::NotEqual, a);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::B32X4 {
                    let a = pos.ins().x86_ptest(x, x);
                    let y = pos.func.dfg.replace(inst).trueif(ir::condcodes::IntCC::NotEqual, a);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::B64X2 {
                    let a = pos.ins().x86_ptest(x, x);
                    let y = pos.func.dfg.replace(inst).trueif(ir::condcodes::IntCC::NotEqual, a);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I8X16 {
                    let a = pos.ins().x86_ptest(x, x);
                    let y = pos.func.dfg.replace(inst).trueif(ir::condcodes::IntCC::NotEqual, a);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I16X8 {
                    let a = pos.ins().x86_ptest(x, x);
                    let y = pos.func.dfg.replace(inst).trueif(ir::condcodes::IntCC::NotEqual, a);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I32X4 {
                    let a = pos.ins().x86_ptest(x, x);
                    let y = pos.func.dfg.replace(inst).trueif(ir::condcodes::IntCC::NotEqual, a);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::I64X2 {
                    let a = pos.ins().x86_ptest(x, x);
                    let y = pos.func.dfg.replace(inst).trueif(ir::condcodes::IntCC::NotEqual, a);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::F32X4 {
                    let a = pos.ins().x86_ptest(x, x);
                    let y = pos.func.dfg.replace(inst).trueif(ir::condcodes::IntCC::NotEqual, a);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }

                if pos.func.dfg.value_type(args[0]) == ir::types::F64X2 {
                    let a = pos.ins().x86_ptest(x, x);
                    let y = pos.func.dfg.replace(inst).trueif(ir::condcodes::IntCC::NotEqual, a);
                    if pos.current_inst() == Some(inst) {
                        pos.next_inst();
                    }
                    return true;
                }
            }

            ir::Opcode::Extractlane => {
                convert_extractlane(inst, func, cfg, isa);
                return true;
            }

            ir::Opcode::Ineg => {
                convert_ineg(inst, func, cfg, isa);
                return true;
            }

            ir::Opcode::Insertlane => {
                convert_insertlane(inst, func, cfg, isa);
                return true;
            }

            ir::Opcode::Shuffle => {
                convert_shuffle(inst, func, cfg, isa);
                return true;
            }

            _ => {},
        }
    }
    crate::legalizer::narrow_flags(inst, func, cfg, isa)
}

// Table of value type sets.
const TYPE_SETS: [ir::instructions::ValueTypeSet; 1] = [
    ir::instructions::ValueTypeSet {
        // TypeSet(lanes={1}, ints={32, 64})
        lanes: BitSet::<u16>(1),
        ints: BitSet::<u8>(96),
        floats: BitSet::<u8>(0),
        bools: BitSet::<u8>(0),
        refs: BitSet::<u8>(0),
    },
];
pub static LEGALIZE_ACTIONS: [isa::Legalize; 5] = [
    crate::legalizer::expand_flags,
    crate::legalizer::widen,
    x86_expand,
    x86_narrow,
    crate::legalizer::narrow_flags,
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

    // Conversion from an unsigned int smaller than 64bit is easy on x86-64.
    match xty {
        ir::types::I8 | ir::types::I16 | ir::types::I32 => {
            // TODO: This should be guarded by an ISA check.
            let wide = pos.ins().uextend(ir::types::I64, x);
            pos.func.dfg.replace(inst).fcvt_from_sint(ty, wide);
            return;
        }
        ir::types::I64 => {}
        _ => unimplemented!(),
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

/// Convert shuffle instructions.
fn convert_shuffle(
    inst: ir::Inst,
    func: &mut ir::Function,
    _cfg: &mut ControlFlowGraph,
    _isa: &dyn TargetIsa,
) {
    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);

    if let ir::InstructionData::Shuffle { args, mask, .. } = pos.func.dfg[inst] {
        // A mask-building helper: in 128-bit SIMD, 0-15 indicate which lane to read from and a 1
        // in the most significant position zeroes the lane.
        let zero_unknown_lane_index = |b: u8| if b > 15 { 0b10000000 } else { b };

        // We only have to worry about aliasing here because copies will be introduced later (in
        // regalloc).
        let a = pos.func.dfg.resolve_aliases(args[0]);
        let b = pos.func.dfg.resolve_aliases(args[1]);
        let mask = pos
            .func
            .dfg
            .immediates
            .get(mask)
            .expect("The shuffle immediate should have been recorded before this point")
            .clone();
        if a == b {
            // PSHUFB the first argument (since it is the same as the second).
            let constructed_mask = mask
                .iter()
                // If the mask is greater than 15 it still may be referring to a lane in b.
                .map(|&b| if b > 15 { b.wrapping_sub(16) } else { b })
                .map(zero_unknown_lane_index)
                .collect();
            let handle = pos.func.dfg.constants.insert(constructed_mask);
            // Move the built mask into another XMM register.
            let a_type = pos.func.dfg.value_type(a);
            let mask_value = pos.ins().vconst(a_type, handle);
            // Shuffle the single incoming argument.
            pos.func.dfg.replace(inst).x86_pshufb(a, mask_value);
        } else {
            // PSHUFB the first argument, placing zeroes for unused lanes.
            let constructed_mask = mask.iter().cloned().map(zero_unknown_lane_index).collect();
            let handle = pos.func.dfg.constants.insert(constructed_mask);
            // Move the built mask into another XMM register.
            let a_type = pos.func.dfg.value_type(a);
            let mask_value = pos.ins().vconst(a_type, handle);
            // Shuffle the first argument.
            let shuffled_first_arg = pos.ins().x86_pshufb(a, mask_value);

            // PSHUFB the second argument, placing zeroes for unused lanes.
            let constructed_mask = mask
                .iter()
                .map(|b| b.wrapping_sub(16))
                .map(zero_unknown_lane_index)
                .collect();
            let handle = pos.func.dfg.constants.insert(constructed_mask);
            // Move the built mask into another XMM register.
            let b_type = pos.func.dfg.value_type(b);
            let mask_value = pos.ins().vconst(b_type, handle);
            // Shuffle the second argument.
            let shuffled_second_arg = pos.ins().x86_pshufb(b, mask_value);

            // OR the vectors together to form the final shuffled value.
            pos.func
                .dfg
                .replace(inst)
                .bor(shuffled_first_arg, shuffled_second_arg);

            // TODO when AVX512 is enabled we should replace this sequence with a single VPERMB
        };
    }
}

/// Because floats already exist in XMM registers, we can keep them there when executing a CLIF
/// extractlane instruction
fn convert_extractlane(
    inst: ir::Inst,
    func: &mut ir::Function,
    _cfg: &mut ControlFlowGraph,
    _isa: &dyn TargetIsa,
) {
    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);

    if let ir::InstructionData::ExtractLane {
        opcode: ir::Opcode::Extractlane,
        arg,
        lane,
    } = pos.func.dfg[inst]
    {
        // NOTE: the following legalization assumes that the upper bits of the XMM register do
        // not need to be zeroed during extractlane.
        let value_type = pos.func.dfg.value_type(arg);
        if value_type.lane_type().is_float() {
            // Floats are already in XMM registers and can stay there.
            let shuffled = if lane != 0 {
                // Replace the extractlane with a PSHUFD to get the float in the right place.
                match value_type {
                    F32X4 => {
                        // Move the selected lane to the 0 lane.
                        let shuffle_mask: u8 = 0b00_00_00_00 | lane;
                        pos.ins().x86_pshufd(arg, shuffle_mask)
                    }
                    F64X2 => {
                        assert_eq!(lane, 1);
                        // Because we know the lane == 1, we move the upper 64 bits to the lower
                        // 64 bits, leaving the top 64 bits as-is.
                        let shuffle_mask = 0b11_10_11_10;
                        let bitcast = pos.ins().raw_bitcast(F32X4, arg);
                        pos.ins().x86_pshufd(bitcast, shuffle_mask)
                    }
                    _ => unreachable!(),
                }
            } else {
                // Remove the extractlane instruction, leaving the float where it is.
                arg
            };
            // Then we must bitcast to the right type.
            pos.func
                .dfg
                .replace(inst)
                .raw_bitcast(value_type.lane_type(), shuffled);
        } else {
            // For non-floats, lower with the usual PEXTR* instruction.
            pos.func.dfg.replace(inst).x86_pextr(arg, lane);
        }
    }
}

/// Because floats exist in XMM registers, we can keep them there when executing a CLIF
/// insertlane instruction
fn convert_insertlane(
    inst: ir::Inst,
    func: &mut ir::Function,
    _cfg: &mut ControlFlowGraph,
    _isa: &dyn TargetIsa,
) {
    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);

    if let ir::InstructionData::InsertLane {
        opcode: ir::Opcode::Insertlane,
        args: [vector, replacement],
        lane,
    } = pos.func.dfg[inst]
    {
        let value_type = pos.func.dfg.value_type(vector);
        if value_type.lane_type().is_float() {
            // Floats are already in XMM registers and can stay there.
            match value_type {
                F32X4 => {
                    assert!(lane > 0 && lane <= 3);
                    let immediate = 0b00_00_00_00 | lane << 4;
                    // Insert 32-bits from replacement (at index 00, bits 7:8) to vector (lane
                    // shifted into bits 5:6).
                    pos.func
                        .dfg
                        .replace(inst)
                        .x86_insertps(vector, immediate, replacement)
                }
                F64X2 => {
                    let replacement_as_vector = pos.ins().raw_bitcast(F64X2, replacement); // only necessary due to SSA types
                    if lane == 0 {
                        // Move the lowest quadword in replacement to vector without changing
                        // the upper bits.
                        pos.func
                            .dfg
                            .replace(inst)
                            .x86_movsd(vector, replacement_as_vector)
                    } else {
                        assert_eq!(lane, 1);
                        // Move the low 64 bits of replacement vector to the high 64 bits of the
                        // vector.
                        pos.func
                            .dfg
                            .replace(inst)
                            .x86_movlhps(vector, replacement_as_vector)
                    }
                }
                _ => unreachable!(),
            };
        } else {
            // For non-floats, lower with the usual PINSR* instruction.
            pos.func
                .dfg
                .replace(inst)
                .x86_pinsr(vector, lane, replacement);
        }
    }
}

/// For SIMD negation, convert an `ineg` to a `vconst + isub`.
fn convert_ineg(
    inst: ir::Inst,
    func: &mut ir::Function,
    _cfg: &mut ControlFlowGraph,
    _isa: &dyn TargetIsa,
) {
    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);

    if let ir::InstructionData::Unary {
        opcode: ir::Opcode::Ineg,
        arg,
    } = pos.func.dfg[inst]
    {
        let value_type = pos.func.dfg.value_type(arg);
        if value_type.is_vector() && value_type.lane_type().is_int() {
            let zero_immediate = pos.func.dfg.constants.insert(vec![0; 16].into());
            let zero_value = pos.ins().vconst(value_type, zero_immediate); // this should be legalized to a PXOR
            pos.func.dfg.replace(inst).isub(zero_value, arg);
        }
    }
}
