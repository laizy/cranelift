//! Emitting binary RISC-V machine code.

use crate::binemit::{bad_encoding, CodeSink, Reloc};
use crate::ir::{Function, Inst, InstructionData};
use crate::isa::{RegUnit, StackBaseMask, StackRef, TargetIsa};
use crate::predicates::is_signed_int;
use crate::regalloc::RegDiversions;
use core::u32;

 
/// Emit binary machine code for `inst` for the riscv ISA.
#[allow(unused_variables, unreachable_code)]
pub fn emit_inst<CS: CodeSink + ?Sized>(
    func: &Function,
    inst: Inst,
    divert: &mut RegDiversions,
    sink: &mut CS,
    isa: &dyn TargetIsa,
) {
    let encoding = func.encodings[inst];
    let bits = encoding.bits();
    let inst_data = &func.dfg[inst];
    match encoding.recipe() {
        // Recipe R
        0 => {
            if let InstructionData::Binary {
                opcode,
                ref args,
                ..
            } = *inst_data {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_r(bits, in_reg0, in_reg1, out_reg0, sink);
                return;
            }
        }
        // Recipe Rshamt
        1 => {
            if let InstructionData::BinaryImm {
                opcode,
                imm,
                arg,
                ..
            } = *inst_data {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_rshamt(bits, in_reg0, imm.into(), out_reg0, sink);
                return;
            }
        }
        // Recipe Ricmp
        2 => {
            if let InstructionData::IntCompare {
                opcode,
                cond,
                ref args,
                ..
            } = *inst_data {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_r(bits, in_reg0, in_reg1, out_reg0, sink);
                return;
            }
        }
        // Recipe Ii
        3 => {
            if let InstructionData::BinaryImm {
                opcode,
                imm,
                arg,
                ..
            } = *inst_data {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_i(bits, in_reg0, imm.into(), out_reg0, sink);
                return;
            }
        }
        // Recipe Iz
        4 => {
            if let InstructionData::UnaryImm {
                opcode,
                imm,
                ..
            } = *inst_data {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_i(bits, 0, imm.into(), out_reg0, sink);
                return;
            }
        }
        // Recipe Iicmp
        5 => {
            if let InstructionData::IntCompareImm {
                opcode,
                cond,
                imm,
                arg,
                ..
            } = *inst_data {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_i(bits, in_reg0, imm.into(), out_reg0, sink);
                return;
            }
        }
        // Recipe Iret
        6 => {
            if let InstructionData::MultiAry {
                opcode,
                ref args,
                ..
            } = *inst_data {
                let args = args.as_slice(&func.dfg.value_lists);
                // Return instructions are always a jalr to %x1.
                // The return address is provided as a special-purpose link argument.
                put_i(
                    bits,
                    1, // rs1 = %x1
                    0, // no offset.
                    0, // rd = %x0: no address written.
                    sink,
                );
                return;
            }
        }
        // Recipe Icall
        7 => {
            if let InstructionData::CallIndirect {
                opcode,
                sig_ref,
                ref args,
                ..
            } = *inst_data {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                // call_indirect instructions are jalr with rd=%x1.
                put_i(
                    bits,
                    in_reg0,
                    0, // no offset.
                    1, // rd = %x1: link register.
                    sink,
                );
                return;
            }
        }
        // Recipe Icopy
        8 => {
            if let InstructionData::Unary {
                opcode,
                arg,
                ..
            } = *inst_data {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_i(bits, in_reg0, 0, out_reg0, sink);
                return;
            }
        }
        // Recipe Irmov
        9 => {
            if let InstructionData::RegMove {
                opcode,
                src,
                dst,
                arg,
                ..
            } = *inst_data {
                divert.apply(inst_data);
                put_i(bits, src, 0, dst, sink);
                return;
            }
        }
        // Recipe copytossa
        10 => {
            if let InstructionData::CopyToSsa {
                opcode,
                src,
                ..
            } = *inst_data {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_i(bits, src, 0, out_reg0, sink);
                return;
            }
        }
        // Recipe U
        11 => {
            if let InstructionData::UnaryImm {
                opcode,
                imm,
                ..
            } = *inst_data {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_u(bits, imm.into(), out_reg0, sink);
                return;
            }
        }
        // Recipe UJ
        12 => {
            if let InstructionData::Jump {
                opcode,
                destination,
                ref args,
                ..
            } = *inst_data {
                let args = args.as_slice(&func.dfg.value_lists);
                let dest = i64::from(func.offsets[destination]);
                let disp = dest - i64::from(sink.offset());
                put_uj(bits, disp, 0, sink);
                return;
            }
        }
        // Recipe UJcall
        13 => {
            if let InstructionData::Call {
                opcode,
                func_ref,
                ref args,
                ..
            } = *inst_data {
                let args = args.as_slice(&func.dfg.value_lists);
                sink.reloc_external(Reloc::RiscvCall,
                                    &func.dfg.ext_funcs[func_ref].name,
                                    0);
                // rd=%x1 is the standard link register.
                put_uj(bits, 0, 1, sink);
                return;
            }
        }
        // Recipe SB
        14 => {
            if let InstructionData::BranchIcmp {
                opcode,
                cond,
                destination,
                ref args,
                ..
            } = *inst_data {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let dest = i64::from(func.offsets[destination]);
                let disp = dest - i64::from(sink.offset());
                put_sb(bits, disp, in_reg0, in_reg1, sink);
                return;
            }
        }
        // Recipe SBzero
        15 => {
            if let InstructionData::Branch {
                opcode,
                destination,
                ref args,
                ..
            } = *inst_data {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let dest = i64::from(func.offsets[destination]);
                let disp = dest - i64::from(sink.offset());
                put_sb(bits, disp, in_reg0, 0, sink);
                return;
            }
        }
        // Recipe GPsp
        16 => {
            if let InstructionData::Unary {
                opcode,
                arg,
                ..
            } = *inst_data {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_stk0 = StackRef::masked(
                    divert.stack(results[0], &func.locations),
                    StackBaseMask(1),
                    &func.stack_slots,
                ).unwrap();
                unimplemented!();
                return;
            }
        }
        // Recipe GPfi
        17 => {
            if let InstructionData::Unary {
                opcode,
                arg,
                ..
            } = *inst_data {
                let args = [arg];
                let in_stk0 = StackRef::masked(
                    divert.stack(args[0], &func.locations),
                    StackBaseMask(1),
                    &func.stack_slots,
                ).unwrap();
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                unimplemented!();
                return;
            }
        }
        // Recipe stacknull
        18 => {
            if let InstructionData::Unary {
                opcode,
                arg,
                ..
            } = *inst_data {
                let args = [arg];
                let in_stk0 = StackRef::masked(
                    divert.stack(args[0], &func.locations),
                    StackBaseMask(1),
                    &func.stack_slots,
                ).unwrap();
                let results = [func.dfg.first_result(inst)];
                let out_stk0 = StackRef::masked(
                    divert.stack(results[0], &func.locations),
                    StackBaseMask(1),
                    &func.stack_slots,
                ).unwrap();
                return;
            }
        }
        // Recipe fillnull
        19 => {
            if let InstructionData::Unary {
                opcode,
                arg,
                ..
            } = *inst_data {
                let args = [arg];
                let in_stk0 = StackRef::masked(
                    divert.stack(args[0], &func.locations),
                    StackBaseMask(1),
                    &func.stack_slots,
                ).unwrap();
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                return;
            }
        }
        _ => {},
    }
    if encoding.is_legal() {
        bad_encoding(func, inst);
    }
}

 //clude!(concat!(env!("OUT_DIR"), "/binemit-riscv.rs"));

/// R-type instructions.
///
///   31     24  19  14     11 6
///   funct7 rs2 rs1 funct3 rd opcode
///       25  20  15     12  7      0
///
/// Encoding bits: `opcode[6:2] | (funct3 << 5) | (funct7 << 8)`.
fn put_r<CS: CodeSink + ?Sized>(bits: u16, rs1: RegUnit, rs2: RegUnit, rd: RegUnit, sink: &mut CS) {
    let bits = u32::from(bits);
    let opcode5 = bits & 0x1f;
    let funct3 = (bits >> 5) & 0x7;
    let funct7 = (bits >> 8) & 0x7f;
    let rs1 = u32::from(rs1) & 0x1f;
    let rs2 = u32::from(rs2) & 0x1f;
    let rd = u32::from(rd) & 0x1f;

    // 0-6: opcode
    let mut i = 0x3;
    i |= opcode5 << 2;
    i |= rd << 7;
    i |= funct3 << 12;
    i |= rs1 << 15;
    i |= rs2 << 20;
    i |= funct7 << 25;

    sink.put4(i);
}

/// R-type instructions with a shift amount instead of rs2.
///
///   31     25    19  14     11 6
///   funct7 shamt rs1 funct3 rd opcode
///       25    20  15     12  7      0
///
/// Both funct7 and shamt contribute to bit 25. In RV64, shamt uses it for shifts > 31.
///
/// Encoding bits: `opcode[6:2] | (funct3 << 5) | (funct7 << 8)`.
fn put_rshamt<CS: CodeSink + ?Sized>(
    bits: u16,
    rs1: RegUnit,
    shamt: i64,
    rd: RegUnit,
    sink: &mut CS,
) {
    let bits = u32::from(bits);
    let opcode5 = bits & 0x1f;
    let funct3 = (bits >> 5) & 0x7;
    let funct7 = (bits >> 8) & 0x7f;
    let rs1 = u32::from(rs1) & 0x1f;
    let shamt = shamt as u32 & 0x3f;
    let rd = u32::from(rd) & 0x1f;

    // 0-6: opcode
    let mut i = 0x3;
    i |= opcode5 << 2;
    i |= rd << 7;
    i |= funct3 << 12;
    i |= rs1 << 15;
    i |= shamt << 20;
    i |= funct7 << 25;

    sink.put4(i);
}

/// I-type instructions.
///
///   31  19  14     11 6
///   imm rs1 funct3 rd opcode
///    20  15     12  7      0
///
/// Encoding bits: `opcode[6:2] | (funct3 << 5)`
fn put_i<CS: CodeSink + ?Sized>(bits: u16, rs1: RegUnit, imm: i64, rd: RegUnit, sink: &mut CS) {
    let bits = u32::from(bits);
    let opcode5 = bits & 0x1f;
    let funct3 = (bits >> 5) & 0x7;
    let rs1 = u32::from(rs1) & 0x1f;
    let rd = u32::from(rd) & 0x1f;

    // 0-6: opcode
    let mut i = 0x3;
    i |= opcode5 << 2;
    i |= rd << 7;
    i |= funct3 << 12;
    i |= rs1 << 15;
    i |= (imm << 20) as u32;

    sink.put4(i);
}

/// U-type instructions.
///
///   31  11 6
///   imm rd opcode
///    12  7      0
///
/// Encoding bits: `opcode[6:2] | (funct3 << 5)`
fn put_u<CS: CodeSink + ?Sized>(bits: u16, imm: i64, rd: RegUnit, sink: &mut CS) {
    let bits = u32::from(bits);
    let opcode5 = bits & 0x1f;
    let rd = u32::from(rd) & 0x1f;

    // 0-6: opcode
    let mut i = 0x3;
    i |= opcode5 << 2;
    i |= rd << 7;
    i |= imm as u32 & 0xfffff000;

    sink.put4(i);
}

/// SB-type branch instructions.
///
///   31  24  19  14     11  6
///   imm rs2 rs1 funct3 imm opcode
///    25  20  15     12   7      0
///
/// Encoding bits: `opcode[6:2] | (funct3 << 5)`
fn put_sb<CS: CodeSink + ?Sized>(bits: u16, imm: i64, rs1: RegUnit, rs2: RegUnit, sink: &mut CS) {
    let bits = u32::from(bits);
    let opcode5 = bits & 0x1f;
    let funct3 = (bits >> 5) & 0x7;
    let rs1 = u32::from(rs1) & 0x1f;
    let rs2 = u32::from(rs2) & 0x1f;

    debug_assert!(is_signed_int(imm, 13, 1), "SB out of range {:#x}", imm);
    let imm = imm as u32;

    // 0-6: opcode
    let mut i = 0x3;
    i |= opcode5 << 2;
    i |= funct3 << 12;
    i |= rs1 << 15;
    i |= rs2 << 20;

    // The displacement is completely hashed up.
    i |= ((imm >> 11) & 0x1) << 7;
    i |= ((imm >> 1) & 0xf) << 8;
    i |= ((imm >> 5) & 0x3f) << 25;
    i |= ((imm >> 12) & 0x1) << 31;

    sink.put4(i);
}

/// UJ-type jump instructions.
///
///   31  11 6
///   imm rd opcode
///    12  7      0
///
/// Encoding bits: `opcode[6:2]`
fn put_uj<CS: CodeSink + ?Sized>(bits: u16, imm: i64, rd: RegUnit, sink: &mut CS) {
    let bits = u32::from(bits);
    let opcode5 = bits & 0x1f;
    let rd = u32::from(rd) & 0x1f;

    debug_assert!(is_signed_int(imm, 21, 1), "UJ out of range {:#x}", imm);
    let imm = imm as u32;

    // 0-6: opcode
    let mut i = 0x3;
    i |= opcode5 << 2;
    i |= rd << 7;

    // The displacement is completely hashed up.
    i |= imm & 0xff000;
    i |= ((imm >> 11) & 0x1) << 20;
    i |= ((imm >> 1) & 0x3ff) << 21;
    i |= ((imm >> 20) & 0x1) << 31;

    sink.put4(i);
}
