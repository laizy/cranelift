//! Emitting binary x86 machine code.

use super::enc_tables::{needs_offset, needs_sib_byte};
use super::registers::RU;
use crate::binemit::{bad_encoding, CodeSink, Reloc};
use crate::ir::condcodes::{CondCode, FloatCC, IntCC};
use crate::ir::{Ebb, Function, Inst, InstructionData, JumpTable, Opcode, TrapCode};
use crate::isa::{RegUnit, StackBase, StackBaseMask, StackRef};
use crate::regalloc::RegDiversions;

/// Emit binary machine code for `inst` for the x86 ISA.
#[allow(unused_variables, unreachable_code)]
pub fn emit_inst<CS: CodeSink + ?Sized>(
    func: &Function,
    inst: Inst,
    divert: &mut RegDiversions,
    sink: &mut CS,
) {
    let encoding = func.encodings[inst];
    let bits = encoding.bits();
    match func.encodings[inst].recipe() {
        // Recipe Op1rr
        0 => {
            if let InstructionData::Binary {
                opcode, ref args, ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                put_op1(bits, rex2(in_reg0, in_reg1), sink);
                modrm_rr(in_reg0, in_reg1, sink);
                return;
            }
        }
        // Recipe RexOp1rr
        1 => {
            if let InstructionData::Binary {
                opcode, ref args, ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                put_rexop1(bits, rex2(in_reg0, in_reg1), sink);
                modrm_rr(in_reg0, in_reg1, sink);
                return;
            }
        }
        // Recipe Op1ur
        2 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                put_op1(bits, rex1(in_reg0), sink);
                modrm_r_bits(in_reg0, bits, sink);
                return;
            }
        }
        // Recipe RexOp1ur
        3 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                put_rexop1(bits, rex1(in_reg0), sink);
                modrm_r_bits(in_reg0, bits, sink);
                return;
            }
        }
        // Recipe Op2rrx
        4 => {
            if let InstructionData::Binary {
                opcode, ref args, ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                put_op2(bits, rex2(in_reg1, in_reg0), sink);
                modrm_rr(in_reg1, in_reg0, sink);
                return;
            }
        }
        // Recipe RexOp2rrx
        5 => {
            if let InstructionData::Binary {
                opcode, ref args, ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                put_rexop2(bits, rex2(in_reg1, in_reg0), sink);
                modrm_rr(in_reg1, in_reg0, sink);
                return;
            }
        }
        // Recipe Op1div
        6 => {
            if let InstructionData::Ternary {
                opcode, ref args, ..
            } = func.dfg[inst]
            {
                let in_reg2 = divert.reg(args[2], &func.locations);
                sink.trap(TrapCode::IntegerDivisionByZero, func.srclocs[inst]);
                put_op1(bits, rex1(in_reg2), sink);
                modrm_r_bits(in_reg2, bits, sink);
                return;
            }
        }
        // Recipe RexOp1div
        7 => {
            if let InstructionData::Ternary {
                opcode, ref args, ..
            } = func.dfg[inst]
            {
                let in_reg2 = divert.reg(args[2], &func.locations);
                sink.trap(TrapCode::IntegerDivisionByZero, func.srclocs[inst]);
                put_rexop1(bits, rex1(in_reg2), sink);
                modrm_r_bits(in_reg2, bits, sink);
                return;
            }
        }
        // Recipe Op1mulx
        8 => {
            if let InstructionData::Binary {
                opcode, ref args, ..
            } = func.dfg[inst]
            {
                let in_reg1 = divert.reg(args[1], &func.locations);
                put_op1(bits, rex1(in_reg1), sink);
                modrm_r_bits(in_reg1, bits, sink);
                return;
            }
        }
        // Recipe RexOp1mulx
        9 => {
            if let InstructionData::Binary {
                opcode, ref args, ..
            } = func.dfg[inst]
            {
                let in_reg1 = divert.reg(args[1], &func.locations);
                put_rexop1(bits, rex1(in_reg1), sink);
                modrm_r_bits(in_reg1, bits, sink);
                return;
            }
        }
        // Recipe Op1umr
        10 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_op1(bits, rex2(out_reg0, in_reg0), sink);
                modrm_rr(out_reg0, in_reg0, sink);
                return;
            }
        }
        // Recipe RexOp1umr
        11 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_rexop1(bits, rex2(out_reg0, in_reg0), sink);
                modrm_rr(out_reg0, in_reg0, sink);
                return;
            }
        }
        // Recipe Op1rmov
        12 => {
            if let InstructionData::RegMove {
                opcode,
                src,
                dst,
                arg,
                ..
            } = func.dfg[inst]
            {
                divert.regmove(arg, src, dst);
                put_op1(bits, rex2(dst, src), sink);
                modrm_rr(dst, src, sink);
                return;
            }
        }
        // Recipe RexOp1rmov
        13 => {
            if let InstructionData::RegMove {
                opcode,
                src,
                dst,
                arg,
                ..
            } = func.dfg[inst]
            {
                divert.regmove(arg, src, dst);
                put_rexop1(bits, rex2(dst, src), sink);
                modrm_rr(dst, src, sink);
                return;
            }
        }
        // Recipe Op1r_ib
        14 => {
            if let InstructionData::BinaryImm {
                opcode, imm, arg, ..
            } = func.dfg[inst]
            {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                put_op1(bits, rex1(in_reg0), sink);
                modrm_r_bits(in_reg0, bits, sink);
                let imm: i64 = imm.into();
                sink.put1(imm as u8);
                return;
            }
        }
        // Recipe RexOp1r_ib
        15 => {
            if let InstructionData::BinaryImm {
                opcode, imm, arg, ..
            } = func.dfg[inst]
            {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                put_rexop1(bits, rex1(in_reg0), sink);
                modrm_r_bits(in_reg0, bits, sink);
                let imm: i64 = imm.into();
                sink.put1(imm as u8);
                return;
            }
        }
        // Recipe Op1r_id
        16 => {
            if let InstructionData::BinaryImm {
                opcode, imm, arg, ..
            } = func.dfg[inst]
            {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                put_op1(bits, rex1(in_reg0), sink);
                modrm_r_bits(in_reg0, bits, sink);
                let imm: i64 = imm.into();
                sink.put4(imm as u32);
                return;
            }
        }
        // Recipe RexOp1r_id
        17 => {
            if let InstructionData::BinaryImm {
                opcode, imm, arg, ..
            } = func.dfg[inst]
            {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                put_rexop1(bits, rex1(in_reg0), sink);
                modrm_r_bits(in_reg0, bits, sink);
                let imm: i64 = imm.into();
                sink.put4(imm as u32);
                return;
            }
        }
        // Recipe Op1pu_id
        18 => {
            if let InstructionData::UnaryImm { opcode, imm, .. } = func.dfg[inst] {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                // The destination register is encoded in the low bits of the opcode.
                // No ModR/M.
                put_op1(bits | (out_reg0 & 7), rex1(out_reg0), sink);
                let imm: i64 = imm.into();
                sink.put4(imm as u32);
                return;
            }
        }
        // Recipe RexOp1pu_id
        19 => {
            if let InstructionData::UnaryImm { opcode, imm, .. } = func.dfg[inst] {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                // The destination register is encoded in the low bits of the opcode.
                // No ModR/M.
                put_rexop1(bits | (out_reg0 & 7), rex1(out_reg0), sink);
                let imm: i64 = imm.into();
                sink.put4(imm as u32);
                return;
            }
        }
        // Recipe RexOp1u_id
        20 => {
            if let InstructionData::UnaryImm { opcode, imm, .. } = func.dfg[inst] {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_rexop1(bits, rex1(out_reg0), sink);
                modrm_r_bits(out_reg0, bits, sink);
                let imm: i64 = imm.into();
                sink.put4(imm as u32);
                return;
            }
        }
        // Recipe RexOp1pu_iq
        21 => {
            if let InstructionData::UnaryImm { opcode, imm, .. } = func.dfg[inst] {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_rexop1(bits | (out_reg0 & 7), rex1(out_reg0), sink);
                let imm: i64 = imm.into();
                sink.put8(imm as u64);
                return;
            }
        }
        // Recipe Op1pu_id_bool
        22 => {
            if let InstructionData::UnaryBool { opcode, imm, .. } = func.dfg[inst] {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                // The destination register is encoded in the low bits of the opcode.
                // No ModR/M.
                put_op1(bits | (out_reg0 & 7), rex1(out_reg0), sink);
                let imm: u32 = if imm { 1 } else { 0 };
                sink.put4(imm);
                return;
            }
        }
        // Recipe RexOp1pu_id_bool
        23 => {
            if let InstructionData::UnaryBool { opcode, imm, .. } = func.dfg[inst] {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                // The destination register is encoded in the low bits of the opcode.
                // No ModR/M.
                put_rexop1(bits | (out_reg0 & 7), rex1(out_reg0), sink);
                let imm: u32 = if imm { 1 } else { 0 };
                sink.put4(imm);
                return;
            }
        }
        // Recipe Op1rc
        24 => {
            if let InstructionData::Binary {
                opcode, ref args, ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                put_op1(bits, rex1(in_reg0), sink);
                modrm_r_bits(in_reg0, bits, sink);
                return;
            }
        }
        // Recipe RexOp1rc
        25 => {
            if let InstructionData::Binary {
                opcode, ref args, ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                put_rexop1(bits, rex1(in_reg0), sink);
                modrm_r_bits(in_reg0, bits, sink);
                return;
            }
        }
        // Recipe Mp2urm
        26 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_mp2(bits, rex2(in_reg0, out_reg0), sink);
                modrm_rr(in_reg0, out_reg0, sink);
                return;
            }
        }
        // Recipe RexMp2urm
        27 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_rexmp2(bits, rex2(in_reg0, out_reg0), sink);
                modrm_rr(in_reg0, out_reg0, sink);
                return;
            }
        }
        // Recipe Op1ldWithIndex
        28 => {
            if let InstructionData::LoadComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_op1(bits, rex3(in_reg0, out_reg0, in_reg1), sink);
                // The else branch always inserts an SIB byte.
                if needs_offset(in_reg0) {
                    modrm_sib_disp8(out_reg0, sink);
                    sib(0, in_reg1, in_reg0, sink);
                    sink.put1(0);
                } else {
                    modrm_sib(out_reg0, sink);
                    sib(0, in_reg1, in_reg0, sink);
                }
                return;
            }
        }
        // Recipe RexOp1ldWithIndex
        29 => {
            if let InstructionData::LoadComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexop1(bits, rex3(in_reg0, out_reg0, in_reg1), sink);
                // The else branch always inserts an SIB byte.
                if needs_offset(in_reg0) {
                    modrm_sib_disp8(out_reg0, sink);
                    sib(0, in_reg1, in_reg0, sink);
                    sink.put1(0);
                } else {
                    modrm_sib(out_reg0, sink);
                    sib(0, in_reg1, in_reg0, sink);
                }
                return;
            }
        }
        // Recipe Op2ldWithIndex
        30 => {
            if let InstructionData::LoadComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_op2(bits, rex3(in_reg0, out_reg0, in_reg1), sink);
                // The else branch always inserts an SIB byte.
                if needs_offset(in_reg0) {
                    modrm_sib_disp8(out_reg0, sink);
                    sib(0, in_reg1, in_reg0, sink);
                    sink.put1(0);
                } else {
                    modrm_sib(out_reg0, sink);
                    sib(0, in_reg1, in_reg0, sink);
                }
                return;
            }
        }
        // Recipe RexOp2ldWithIndex
        31 => {
            if let InstructionData::LoadComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexop2(bits, rex3(in_reg0, out_reg0, in_reg1), sink);
                // The else branch always inserts an SIB byte.
                if needs_offset(in_reg0) {
                    modrm_sib_disp8(out_reg0, sink);
                    sib(0, in_reg1, in_reg0, sink);
                    sink.put1(0);
                } else {
                    modrm_sib(out_reg0, sink);
                    sib(0, in_reg1, in_reg0, sink);
                }
                return;
            }
        }
        // Recipe Op1ldWithIndexDisp8
        32 => {
            if let InstructionData::LoadComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_op1(bits, rex3(in_reg0, out_reg0, in_reg1), sink);
                modrm_sib_disp8(out_reg0, sink);
                sib(0, in_reg1, in_reg0, sink);
                let offset: i32 = offset.into();
                sink.put1(offset as u8);
                return;
            }
        }
        // Recipe RexOp1ldWithIndexDisp8
        33 => {
            if let InstructionData::LoadComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexop1(bits, rex3(in_reg0, out_reg0, in_reg1), sink);
                modrm_sib_disp8(out_reg0, sink);
                sib(0, in_reg1, in_reg0, sink);
                let offset: i32 = offset.into();
                sink.put1(offset as u8);
                return;
            }
        }
        // Recipe Op2ldWithIndexDisp8
        34 => {
            if let InstructionData::LoadComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_op2(bits, rex3(in_reg0, out_reg0, in_reg1), sink);
                modrm_sib_disp8(out_reg0, sink);
                sib(0, in_reg1, in_reg0, sink);
                let offset: i32 = offset.into();
                sink.put1(offset as u8);
                return;
            }
        }
        // Recipe RexOp2ldWithIndexDisp8
        35 => {
            if let InstructionData::LoadComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexop2(bits, rex3(in_reg0, out_reg0, in_reg1), sink);
                modrm_sib_disp8(out_reg0, sink);
                sib(0, in_reg1, in_reg0, sink);
                let offset: i32 = offset.into();
                sink.put1(offset as u8);
                return;
            }
        }
        // Recipe Op1ldWithIndexDisp32
        36 => {
            if let InstructionData::LoadComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_op1(bits, rex3(in_reg0, out_reg0, in_reg1), sink);
                modrm_sib_disp32(out_reg0, sink);
                sib(0, in_reg1, in_reg0, sink);
                let offset: i32 = offset.into();
                sink.put4(offset as u32);
                return;
            }
        }
        // Recipe RexOp1ldWithIndexDisp32
        37 => {
            if let InstructionData::LoadComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexop1(bits, rex3(in_reg0, out_reg0, in_reg1), sink);
                modrm_sib_disp32(out_reg0, sink);
                sib(0, in_reg1, in_reg0, sink);
                let offset: i32 = offset.into();
                sink.put4(offset as u32);
                return;
            }
        }
        // Recipe Op2ldWithIndexDisp32
        38 => {
            if let InstructionData::LoadComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_op2(bits, rex3(in_reg0, out_reg0, in_reg1), sink);
                modrm_sib_disp32(out_reg0, sink);
                sib(0, in_reg1, in_reg0, sink);
                let offset: i32 = offset.into();
                sink.put4(offset as u32);
                return;
            }
        }
        // Recipe RexOp2ldWithIndexDisp32
        39 => {
            if let InstructionData::LoadComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexop2(bits, rex3(in_reg0, out_reg0, in_reg1), sink);
                modrm_sib_disp32(out_reg0, sink);
                sib(0, in_reg1, in_reg0, sink);
                let offset: i32 = offset.into();
                sink.put4(offset as u32);
                return;
            }
        }
        // Recipe Op1stWithIndex
        40 => {
            if let InstructionData::StoreComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let in_reg2 = divert.reg(args[2], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_op1(bits, rex3(in_reg1, in_reg0, in_reg2), sink);
                // The else branch always inserts an SIB byte.
                if needs_offset(in_reg1) {
                    modrm_sib_disp8(in_reg0, sink);
                    sib(0, in_reg2, in_reg1, sink);
                    sink.put1(0);
                } else {
                    modrm_sib(in_reg0, sink);
                    sib(0, in_reg2, in_reg1, sink);
                }
                return;
            }
        }
        // Recipe RexOp1stWithIndex
        41 => {
            if let InstructionData::StoreComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let in_reg2 = divert.reg(args[2], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexop1(bits, rex3(in_reg1, in_reg0, in_reg2), sink);
                // The else branch always inserts an SIB byte.
                if needs_offset(in_reg1) {
                    modrm_sib_disp8(in_reg0, sink);
                    sib(0, in_reg2, in_reg1, sink);
                    sink.put1(0);
                } else {
                    modrm_sib(in_reg0, sink);
                    sib(0, in_reg2, in_reg1, sink);
                }
                return;
            }
        }
        // Recipe Mp1stWithIndex
        42 => {
            if let InstructionData::StoreComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let in_reg2 = divert.reg(args[2], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_mp1(bits, rex3(in_reg1, in_reg0, in_reg2), sink);
                // The else branch always inserts an SIB byte.
                if needs_offset(in_reg1) {
                    modrm_sib_disp8(in_reg0, sink);
                    sib(0, in_reg2, in_reg1, sink);
                    sink.put1(0);
                } else {
                    modrm_sib(in_reg0, sink);
                    sib(0, in_reg2, in_reg1, sink);
                }
                return;
            }
        }
        // Recipe RexMp1stWithIndex
        43 => {
            if let InstructionData::StoreComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let in_reg2 = divert.reg(args[2], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexmp1(bits, rex3(in_reg1, in_reg0, in_reg2), sink);
                // The else branch always inserts an SIB byte.
                if needs_offset(in_reg1) {
                    modrm_sib_disp8(in_reg0, sink);
                    sib(0, in_reg2, in_reg1, sink);
                    sink.put1(0);
                } else {
                    modrm_sib(in_reg0, sink);
                    sib(0, in_reg2, in_reg1, sink);
                }
                return;
            }
        }
        // Recipe Op1stWithIndexDisp8
        44 => {
            if let InstructionData::StoreComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let in_reg2 = divert.reg(args[2], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_op1(bits, rex3(in_reg1, in_reg0, in_reg2), sink);
                modrm_sib_disp8(in_reg0, sink);
                sib(0, in_reg2, in_reg1, sink);
                let offset: i32 = offset.into();
                sink.put1(offset as u8);
                return;
            }
        }
        // Recipe RexOp1stWithIndexDisp8
        45 => {
            if let InstructionData::StoreComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let in_reg2 = divert.reg(args[2], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexop1(bits, rex3(in_reg1, in_reg0, in_reg2), sink);
                modrm_sib_disp8(in_reg0, sink);
                sib(0, in_reg2, in_reg1, sink);
                let offset: i32 = offset.into();
                sink.put1(offset as u8);
                return;
            }
        }
        // Recipe Mp1stWithIndexDisp8
        46 => {
            if let InstructionData::StoreComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let in_reg2 = divert.reg(args[2], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_mp1(bits, rex3(in_reg1, in_reg0, in_reg2), sink);
                modrm_sib_disp8(in_reg0, sink);
                sib(0, in_reg2, in_reg1, sink);
                let offset: i32 = offset.into();
                sink.put1(offset as u8);
                return;
            }
        }
        // Recipe RexMp1stWithIndexDisp8
        47 => {
            if let InstructionData::StoreComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let in_reg2 = divert.reg(args[2], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexmp1(bits, rex3(in_reg1, in_reg0, in_reg2), sink);
                modrm_sib_disp8(in_reg0, sink);
                sib(0, in_reg2, in_reg1, sink);
                let offset: i32 = offset.into();
                sink.put1(offset as u8);
                return;
            }
        }
        // Recipe Op1stWithIndexDisp32
        48 => {
            if let InstructionData::StoreComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let in_reg2 = divert.reg(args[2], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_op1(bits, rex3(in_reg1, in_reg0, in_reg2), sink);
                modrm_sib_disp32(in_reg0, sink);
                sib(0, in_reg2, in_reg1, sink);
                let offset: i32 = offset.into();
                sink.put4(offset as u32);
                return;
            }
        }
        // Recipe RexOp1stWithIndexDisp32
        49 => {
            if let InstructionData::StoreComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let in_reg2 = divert.reg(args[2], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexop1(bits, rex3(in_reg1, in_reg0, in_reg2), sink);
                modrm_sib_disp32(in_reg0, sink);
                sib(0, in_reg2, in_reg1, sink);
                let offset: i32 = offset.into();
                sink.put4(offset as u32);
                return;
            }
        }
        // Recipe Mp1stWithIndexDisp32
        50 => {
            if let InstructionData::StoreComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let in_reg2 = divert.reg(args[2], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_mp1(bits, rex3(in_reg1, in_reg0, in_reg2), sink);
                modrm_sib_disp32(in_reg0, sink);
                sib(0, in_reg2, in_reg1, sink);
                let offset: i32 = offset.into();
                sink.put4(offset as u32);
                return;
            }
        }
        // Recipe RexMp1stWithIndexDisp32
        51 => {
            if let InstructionData::StoreComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let in_reg2 = divert.reg(args[2], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexmp1(bits, rex3(in_reg1, in_reg0, in_reg2), sink);
                modrm_sib_disp32(in_reg0, sink);
                sib(0, in_reg2, in_reg1, sink);
                let offset: i32 = offset.into();
                sink.put4(offset as u32);
                return;
            }
        }
        // Recipe Op1stWithIndex_abcd
        52 => {
            if let InstructionData::StoreComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let in_reg2 = divert.reg(args[2], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_op1(bits, rex3(in_reg1, in_reg0, in_reg2), sink);
                // The else branch always inserts an SIB byte.
                if needs_offset(in_reg1) {
                    modrm_sib_disp8(in_reg0, sink);
                    sib(0, in_reg2, in_reg1, sink);
                    sink.put1(0);
                } else {
                    modrm_sib(in_reg0, sink);
                    sib(0, in_reg2, in_reg1, sink);
                }
                return;
            }
        }
        // Recipe RexOp1stWithIndex_abcd
        53 => {
            if let InstructionData::StoreComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let in_reg2 = divert.reg(args[2], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexop1(bits, rex3(in_reg1, in_reg0, in_reg2), sink);
                // The else branch always inserts an SIB byte.
                if needs_offset(in_reg1) {
                    modrm_sib_disp8(in_reg0, sink);
                    sib(0, in_reg2, in_reg1, sink);
                    sink.put1(0);
                } else {
                    modrm_sib(in_reg0, sink);
                    sib(0, in_reg2, in_reg1, sink);
                }
                return;
            }
        }
        // Recipe Op1stWithIndexDisp8_abcd
        54 => {
            if let InstructionData::StoreComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let in_reg2 = divert.reg(args[2], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_op1(bits, rex3(in_reg1, in_reg0, in_reg2), sink);
                modrm_sib_disp8(in_reg0, sink);
                sib(0, in_reg2, in_reg1, sink);
                let offset: i32 = offset.into();
                sink.put1(offset as u8);
                return;
            }
        }
        // Recipe RexOp1stWithIndexDisp8_abcd
        55 => {
            if let InstructionData::StoreComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let in_reg2 = divert.reg(args[2], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexop1(bits, rex3(in_reg1, in_reg0, in_reg2), sink);
                modrm_sib_disp8(in_reg0, sink);
                sib(0, in_reg2, in_reg1, sink);
                let offset: i32 = offset.into();
                sink.put1(offset as u8);
                return;
            }
        }
        // Recipe Op1stWithIndexDisp32_abcd
        56 => {
            if let InstructionData::StoreComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let in_reg2 = divert.reg(args[2], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_op1(bits, rex3(in_reg1, in_reg0, in_reg2), sink);
                modrm_sib_disp32(in_reg0, sink);
                sib(0, in_reg2, in_reg1, sink);
                let offset: i32 = offset.into();
                sink.put4(offset as u32);
                return;
            }
        }
        // Recipe RexOp1stWithIndexDisp32_abcd
        57 => {
            if let InstructionData::StoreComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let in_reg2 = divert.reg(args[2], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexop1(bits, rex3(in_reg1, in_reg0, in_reg2), sink);
                modrm_sib_disp32(in_reg0, sink);
                sib(0, in_reg2, in_reg1, sink);
                let offset: i32 = offset.into();
                sink.put4(offset as u32);
                return;
            }
        }
        // Recipe Op1st
        58 => {
            if let InstructionData::Store {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_op1(bits, rex2(in_reg1, in_reg0), sink);
                if needs_sib_byte(in_reg1) {
                    modrm_sib(in_reg0, sink);
                    sib_noindex(in_reg1, sink);
                } else if needs_offset(in_reg1) {
                    modrm_disp8(in_reg1, in_reg0, sink);
                    sink.put1(0);
                } else {
                    modrm_rm(in_reg1, in_reg0, sink);
                }
                return;
            }
        }
        // Recipe RexOp1st
        59 => {
            if let InstructionData::Store {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexop1(bits, rex2(in_reg1, in_reg0), sink);
                if needs_sib_byte(in_reg1) {
                    modrm_sib(in_reg0, sink);
                    sib_noindex(in_reg1, sink);
                } else if needs_offset(in_reg1) {
                    modrm_disp8(in_reg1, in_reg0, sink);
                    sink.put1(0);
                } else {
                    modrm_rm(in_reg1, in_reg0, sink);
                }
                return;
            }
        }
        // Recipe Mp1st
        60 => {
            if let InstructionData::Store {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_mp1(bits, rex2(in_reg1, in_reg0), sink);
                if needs_sib_byte(in_reg1) {
                    modrm_sib(in_reg0, sink);
                    sib_noindex(in_reg1, sink);
                } else if needs_offset(in_reg1) {
                    modrm_disp8(in_reg1, in_reg0, sink);
                    sink.put1(0);
                } else {
                    modrm_rm(in_reg1, in_reg0, sink);
                }
                return;
            }
        }
        // Recipe RexMp1st
        61 => {
            if let InstructionData::Store {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexmp1(bits, rex2(in_reg1, in_reg0), sink);
                if needs_sib_byte(in_reg1) {
                    modrm_sib(in_reg0, sink);
                    sib_noindex(in_reg1, sink);
                } else if needs_offset(in_reg1) {
                    modrm_disp8(in_reg1, in_reg0, sink);
                    sink.put1(0);
                } else {
                    modrm_rm(in_reg1, in_reg0, sink);
                }
                return;
            }
        }
        // Recipe Op1stDisp8
        62 => {
            if let InstructionData::Store {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_op1(bits, rex2(in_reg1, in_reg0), sink);
                if needs_sib_byte(in_reg1) {
                    modrm_sib_disp8(in_reg0, sink);
                    sib_noindex(in_reg1, sink);
                } else {
                    modrm_disp8(in_reg1, in_reg0, sink);
                }
                let offset: i32 = offset.into();
                sink.put1(offset as u8);
                return;
            }
        }
        // Recipe RexOp1stDisp8
        63 => {
            if let InstructionData::Store {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexop1(bits, rex2(in_reg1, in_reg0), sink);
                if needs_sib_byte(in_reg1) {
                    modrm_sib_disp8(in_reg0, sink);
                    sib_noindex(in_reg1, sink);
                } else {
                    modrm_disp8(in_reg1, in_reg0, sink);
                }
                let offset: i32 = offset.into();
                sink.put1(offset as u8);
                return;
            }
        }
        // Recipe Mp1stDisp8
        64 => {
            if let InstructionData::Store {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_mp1(bits, rex2(in_reg1, in_reg0), sink);
                if needs_sib_byte(in_reg1) {
                    modrm_sib_disp8(in_reg0, sink);
                    sib_noindex(in_reg1, sink);
                } else {
                    modrm_disp8(in_reg1, in_reg0, sink);
                }
                let offset: i32 = offset.into();
                sink.put1(offset as u8);
                return;
            }
        }
        // Recipe RexMp1stDisp8
        65 => {
            if let InstructionData::Store {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexmp1(bits, rex2(in_reg1, in_reg0), sink);
                if needs_sib_byte(in_reg1) {
                    modrm_sib_disp8(in_reg0, sink);
                    sib_noindex(in_reg1, sink);
                } else {
                    modrm_disp8(in_reg1, in_reg0, sink);
                }
                let offset: i32 = offset.into();
                sink.put1(offset as u8);
                return;
            }
        }
        // Recipe Op1stDisp32
        66 => {
            if let InstructionData::Store {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_op1(bits, rex2(in_reg1, in_reg0), sink);
                if needs_sib_byte(in_reg1) {
                    modrm_sib_disp32(in_reg0, sink);
                    sib_noindex(in_reg1, sink);
                } else {
                    modrm_disp32(in_reg1, in_reg0, sink);
                }
                let offset: i32 = offset.into();
                sink.put4(offset as u32);
                return;
            }
        }
        // Recipe RexOp1stDisp32
        67 => {
            if let InstructionData::Store {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexop1(bits, rex2(in_reg1, in_reg0), sink);
                if needs_sib_byte(in_reg1) {
                    modrm_sib_disp32(in_reg0, sink);
                    sib_noindex(in_reg1, sink);
                } else {
                    modrm_disp32(in_reg1, in_reg0, sink);
                }
                let offset: i32 = offset.into();
                sink.put4(offset as u32);
                return;
            }
        }
        // Recipe Mp1stDisp32
        68 => {
            if let InstructionData::Store {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_mp1(bits, rex2(in_reg1, in_reg0), sink);
                if needs_sib_byte(in_reg1) {
                    modrm_sib_disp32(in_reg0, sink);
                    sib_noindex(in_reg1, sink);
                } else {
                    modrm_disp32(in_reg1, in_reg0, sink);
                }
                let offset: i32 = offset.into();
                sink.put4(offset as u32);
                return;
            }
        }
        // Recipe RexMp1stDisp32
        69 => {
            if let InstructionData::Store {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexmp1(bits, rex2(in_reg1, in_reg0), sink);
                if needs_sib_byte(in_reg1) {
                    modrm_sib_disp32(in_reg0, sink);
                    sib_noindex(in_reg1, sink);
                } else {
                    modrm_disp32(in_reg1, in_reg0, sink);
                }
                let offset: i32 = offset.into();
                sink.put4(offset as u32);
                return;
            }
        }
        // Recipe Op1st_abcd
        70 => {
            if let InstructionData::Store {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_op1(bits, rex2(in_reg1, in_reg0), sink);
                if needs_sib_byte(in_reg1) {
                    modrm_sib(in_reg0, sink);
                    sib_noindex(in_reg1, sink);
                } else if needs_offset(in_reg1) {
                    modrm_disp8(in_reg1, in_reg0, sink);
                    sink.put1(0);
                } else {
                    modrm_rm(in_reg1, in_reg0, sink);
                }
                return;
            }
        }
        // Recipe Op1stDisp8_abcd
        71 => {
            if let InstructionData::Store {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_op1(bits, rex2(in_reg1, in_reg0), sink);
                if needs_sib_byte(in_reg1) {
                    modrm_sib_disp8(in_reg0, sink);
                    sib_noindex(in_reg1, sink);
                } else {
                    modrm_disp8(in_reg1, in_reg0, sink);
                }
                let offset: i32 = offset.into();
                sink.put1(offset as u8);
                return;
            }
        }
        // Recipe Op1stDisp32_abcd
        72 => {
            if let InstructionData::Store {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_op1(bits, rex2(in_reg1, in_reg0), sink);
                if needs_sib_byte(in_reg1) {
                    modrm_sib_disp32(in_reg0, sink);
                    sib_noindex(in_reg1, sink);
                } else {
                    modrm_disp32(in_reg1, in_reg0, sink);
                }
                let offset: i32 = offset.into();
                sink.put4(offset as u32);
                return;
            }
        }
        // Recipe Op1spillSib32
        73 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_stk0 = StackRef::masked(
                    divert.stack(results[0], &func.locations),
                    StackBaseMask(1),
                    &func.stack_slots,
                )
                .unwrap();
                sink.trap(TrapCode::StackOverflow, func.srclocs[inst]);
                let base = stk_base(out_stk0.base);
                put_op1(bits, rex2(base, in_reg0), sink);
                modrm_sib_disp32(in_reg0, sink);
                sib_noindex(base, sink);
                sink.put4(out_stk0.offset as u32);
                return;
            }
        }
        // Recipe RexOp1spillSib32
        74 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_stk0 = StackRef::masked(
                    divert.stack(results[0], &func.locations),
                    StackBaseMask(1),
                    &func.stack_slots,
                )
                .unwrap();
                sink.trap(TrapCode::StackOverflow, func.srclocs[inst]);
                let base = stk_base(out_stk0.base);
                put_rexop1(bits, rex2(base, in_reg0), sink);
                modrm_sib_disp32(in_reg0, sink);
                sib_noindex(base, sink);
                sink.put4(out_stk0.offset as u32);
                return;
            }
        }
        // Recipe Op1regspill32
        75 => {
            if let InstructionData::RegSpill {
                opcode,
                src,
                dst,
                arg,
                ..
            } = func.dfg[inst]
            {
                divert.regspill(arg, src, dst);
                sink.trap(TrapCode::StackOverflow, func.srclocs[inst]);
                let dst = StackRef::sp(dst, &func.stack_slots);
                let base = stk_base(dst.base);
                put_op1(bits, rex2(base, src), sink);
                modrm_sib_disp32(src, sink);
                sib_noindex(base, sink);
                sink.put4(dst.offset as u32);
                return;
            }
        }
        // Recipe RexOp1regspill32
        76 => {
            if let InstructionData::RegSpill {
                opcode,
                src,
                dst,
                arg,
                ..
            } = func.dfg[inst]
            {
                divert.regspill(arg, src, dst);
                sink.trap(TrapCode::StackOverflow, func.srclocs[inst]);
                let dst = StackRef::sp(dst, &func.stack_slots);
                let base = stk_base(dst.base);
                put_rexop1(bits, rex2(base, src), sink);
                modrm_sib_disp32(src, sink);
                sib_noindex(base, sink);
                sink.put4(dst.offset as u32);
                return;
            }
        }
        // Recipe Op1ld
        77 => {
            if let InstructionData::Load {
                opcode,
                flags,
                offset,
                arg,
                ..
            } = func.dfg[inst]
            {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_op1(bits, rex2(in_reg0, out_reg0), sink);
                if needs_sib_byte(in_reg0) {
                    modrm_sib(out_reg0, sink);
                    sib_noindex(in_reg0, sink);
                } else if needs_offset(in_reg0) {
                    modrm_disp8(in_reg0, out_reg0, sink);
                    sink.put1(0);
                } else {
                    modrm_rm(in_reg0, out_reg0, sink);
                }
                return;
            }
        }
        // Recipe RexOp1ld
        78 => {
            if let InstructionData::Load {
                opcode,
                flags,
                offset,
                arg,
                ..
            } = func.dfg[inst]
            {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexop1(bits, rex2(in_reg0, out_reg0), sink);
                if needs_sib_byte(in_reg0) {
                    modrm_sib(out_reg0, sink);
                    sib_noindex(in_reg0, sink);
                } else if needs_offset(in_reg0) {
                    modrm_disp8(in_reg0, out_reg0, sink);
                    sink.put1(0);
                } else {
                    modrm_rm(in_reg0, out_reg0, sink);
                }
                return;
            }
        }
        // Recipe Op2ld
        79 => {
            if let InstructionData::Load {
                opcode,
                flags,
                offset,
                arg,
                ..
            } = func.dfg[inst]
            {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_op2(bits, rex2(in_reg0, out_reg0), sink);
                if needs_sib_byte(in_reg0) {
                    modrm_sib(out_reg0, sink);
                    sib_noindex(in_reg0, sink);
                } else if needs_offset(in_reg0) {
                    modrm_disp8(in_reg0, out_reg0, sink);
                    sink.put1(0);
                } else {
                    modrm_rm(in_reg0, out_reg0, sink);
                }
                return;
            }
        }
        // Recipe RexOp2ld
        80 => {
            if let InstructionData::Load {
                opcode,
                flags,
                offset,
                arg,
                ..
            } = func.dfg[inst]
            {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexop2(bits, rex2(in_reg0, out_reg0), sink);
                if needs_sib_byte(in_reg0) {
                    modrm_sib(out_reg0, sink);
                    sib_noindex(in_reg0, sink);
                } else if needs_offset(in_reg0) {
                    modrm_disp8(in_reg0, out_reg0, sink);
                    sink.put1(0);
                } else {
                    modrm_rm(in_reg0, out_reg0, sink);
                }
                return;
            }
        }
        // Recipe Op1ldDisp8
        81 => {
            if let InstructionData::Load {
                opcode,
                flags,
                offset,
                arg,
                ..
            } = func.dfg[inst]
            {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_op1(bits, rex2(in_reg0, out_reg0), sink);
                if needs_sib_byte(in_reg0) {
                    modrm_sib_disp8(out_reg0, sink);
                    sib_noindex(in_reg0, sink);
                } else {
                    modrm_disp8(in_reg0, out_reg0, sink);
                }
                let offset: i32 = offset.into();
                sink.put1(offset as u8);
                return;
            }
        }
        // Recipe RexOp1ldDisp8
        82 => {
            if let InstructionData::Load {
                opcode,
                flags,
                offset,
                arg,
                ..
            } = func.dfg[inst]
            {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexop1(bits, rex2(in_reg0, out_reg0), sink);
                if needs_sib_byte(in_reg0) {
                    modrm_sib_disp8(out_reg0, sink);
                    sib_noindex(in_reg0, sink);
                } else {
                    modrm_disp8(in_reg0, out_reg0, sink);
                }
                let offset: i32 = offset.into();
                sink.put1(offset as u8);
                return;
            }
        }
        // Recipe Op2ldDisp8
        83 => {
            if let InstructionData::Load {
                opcode,
                flags,
                offset,
                arg,
                ..
            } = func.dfg[inst]
            {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_op2(bits, rex2(in_reg0, out_reg0), sink);
                if needs_sib_byte(in_reg0) {
                    modrm_sib_disp8(out_reg0, sink);
                    sib_noindex(in_reg0, sink);
                } else {
                    modrm_disp8(in_reg0, out_reg0, sink);
                }
                let offset: i32 = offset.into();
                sink.put1(offset as u8);
                return;
            }
        }
        // Recipe RexOp2ldDisp8
        84 => {
            if let InstructionData::Load {
                opcode,
                flags,
                offset,
                arg,
                ..
            } = func.dfg[inst]
            {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexop2(bits, rex2(in_reg0, out_reg0), sink);
                if needs_sib_byte(in_reg0) {
                    modrm_sib_disp8(out_reg0, sink);
                    sib_noindex(in_reg0, sink);
                } else {
                    modrm_disp8(in_reg0, out_reg0, sink);
                }
                let offset: i32 = offset.into();
                sink.put1(offset as u8);
                return;
            }
        }
        // Recipe Op1ldDisp32
        85 => {
            if let InstructionData::Load {
                opcode,
                flags,
                offset,
                arg,
                ..
            } = func.dfg[inst]
            {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_op1(bits, rex2(in_reg0, out_reg0), sink);
                if needs_sib_byte(in_reg0) {
                    modrm_sib_disp32(out_reg0, sink);
                    sib_noindex(in_reg0, sink);
                } else {
                    modrm_disp32(in_reg0, out_reg0, sink);
                }
                let offset: i32 = offset.into();
                sink.put4(offset as u32);
                return;
            }
        }
        // Recipe RexOp1ldDisp32
        86 => {
            if let InstructionData::Load {
                opcode,
                flags,
                offset,
                arg,
                ..
            } = func.dfg[inst]
            {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexop1(bits, rex2(in_reg0, out_reg0), sink);
                if needs_sib_byte(in_reg0) {
                    modrm_sib_disp32(out_reg0, sink);
                    sib_noindex(in_reg0, sink);
                } else {
                    modrm_disp32(in_reg0, out_reg0, sink);
                }
                let offset: i32 = offset.into();
                sink.put4(offset as u32);
                return;
            }
        }
        // Recipe Op2ldDisp32
        87 => {
            if let InstructionData::Load {
                opcode,
                flags,
                offset,
                arg,
                ..
            } = func.dfg[inst]
            {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_op2(bits, rex2(in_reg0, out_reg0), sink);
                if needs_sib_byte(in_reg0) {
                    modrm_sib_disp32(out_reg0, sink);
                    sib_noindex(in_reg0, sink);
                } else {
                    modrm_disp32(in_reg0, out_reg0, sink);
                }
                let offset: i32 = offset.into();
                sink.put4(offset as u32);
                return;
            }
        }
        // Recipe RexOp2ldDisp32
        88 => {
            if let InstructionData::Load {
                opcode,
                flags,
                offset,
                arg,
                ..
            } = func.dfg[inst]
            {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexop2(bits, rex2(in_reg0, out_reg0), sink);
                if needs_sib_byte(in_reg0) {
                    modrm_sib_disp32(out_reg0, sink);
                    sib_noindex(in_reg0, sink);
                } else {
                    modrm_disp32(in_reg0, out_reg0, sink);
                }
                let offset: i32 = offset.into();
                sink.put4(offset as u32);
                return;
            }
        }
        // Recipe Op1fillSib32
        89 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_stk0 = StackRef::masked(
                    divert.stack(args[0], &func.locations),
                    StackBaseMask(1),
                    &func.stack_slots,
                )
                .unwrap();
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                let base = stk_base(in_stk0.base);
                put_op1(bits, rex2(base, out_reg0), sink);
                modrm_sib_disp32(out_reg0, sink);
                sib_noindex(base, sink);
                sink.put4(in_stk0.offset as u32);
                return;
            }
        }
        // Recipe RexOp1fillSib32
        90 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_stk0 = StackRef::masked(
                    divert.stack(args[0], &func.locations),
                    StackBaseMask(1),
                    &func.stack_slots,
                )
                .unwrap();
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                let base = stk_base(in_stk0.base);
                put_rexop1(bits, rex2(base, out_reg0), sink);
                modrm_sib_disp32(out_reg0, sink);
                sib_noindex(base, sink);
                sink.put4(in_stk0.offset as u32);
                return;
            }
        }
        // Recipe Op1regfill32
        91 => {
            if let InstructionData::RegFill {
                opcode,
                src,
                dst,
                arg,
                ..
            } = func.dfg[inst]
            {
                divert.regfill(arg, src, dst);
                let src = StackRef::sp(src, &func.stack_slots);
                let base = stk_base(src.base);
                put_op1(bits, rex2(base, dst), sink);
                modrm_sib_disp32(dst, sink);
                sib_noindex(base, sink);
                sink.put4(src.offset as u32);
                return;
            }
        }
        // Recipe RexOp1regfill32
        92 => {
            if let InstructionData::RegFill {
                opcode,
                src,
                dst,
                arg,
                ..
            } = func.dfg[inst]
            {
                divert.regfill(arg, src, dst);
                let src = StackRef::sp(src, &func.stack_slots);
                let base = stk_base(src.base);
                put_rexop1(bits, rex2(base, dst), sink);
                modrm_sib_disp32(dst, sink);
                sib_noindex(base, sink);
                sink.put4(src.offset as u32);
                return;
            }
        }
        // Recipe Op1pushq
        93 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                sink.trap(TrapCode::StackOverflow, func.srclocs[inst]);
                put_op1(bits | (in_reg0 & 7), rex1(in_reg0), sink);
                return;
            }
        }
        // Recipe RexOp1pushq
        94 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                sink.trap(TrapCode::StackOverflow, func.srclocs[inst]);
                put_rexop1(bits | (in_reg0 & 7), rex1(in_reg0), sink);
                return;
            }
        }
        // Recipe Op1popq
        95 => {
            if let InstructionData::NullAry { opcode, .. } = func.dfg[inst] {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_op1(bits | (out_reg0 & 7), rex1(out_reg0), sink);
                return;
            }
        }
        // Recipe RexOp1popq
        96 => {
            if let InstructionData::NullAry { opcode, .. } = func.dfg[inst] {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_rexop1(bits | (out_reg0 & 7), rex1(out_reg0), sink);
                return;
            }
        }
        // Recipe RexOp1copysp
        97 => {
            if let InstructionData::CopySpecial {
                opcode, src, dst, ..
            } = func.dfg[inst]
            {
                put_rexop1(bits, rex2(dst, src), sink);
                modrm_rr(dst, src, sink);
                return;
            }
        }
        // Recipe Op1copysp
        98 => {
            if let InstructionData::CopySpecial {
                opcode, src, dst, ..
            } = func.dfg[inst]
            {
                put_op1(bits, rex2(dst, src), sink);
                modrm_rr(dst, src, sink);
                return;
            }
        }
        // Recipe stacknull
        99 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_stk0 = StackRef::masked(
                    divert.stack(args[0], &func.locations),
                    StackBaseMask(1),
                    &func.stack_slots,
                )
                .unwrap();
                let results = [func.dfg.first_result(inst)];
                let out_stk0 = StackRef::masked(
                    divert.stack(results[0], &func.locations),
                    StackBaseMask(1),
                    &func.stack_slots,
                )
                .unwrap();
                return;
            }
        }
        // Recipe Op1adjustsp
        100 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                put_op1(bits, rex2(RU::rsp.into(), in_reg0), sink);
                modrm_rr(RU::rsp.into(), in_reg0, sink);
                return;
            }
        }
        // Recipe RexOp1adjustsp
        101 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                put_rexop1(bits, rex2(RU::rsp.into(), in_reg0), sink);
                modrm_rr(RU::rsp.into(), in_reg0, sink);
                return;
            }
        }
        // Recipe Op1adjustsp_ib
        102 => {
            if let InstructionData::UnaryImm { opcode, imm, .. } = func.dfg[inst] {
                put_op1(bits, rex1(RU::rsp.into()), sink);
                modrm_r_bits(RU::rsp.into(), bits, sink);
                let imm: i64 = imm.into();
                sink.put1(imm as u8);
                return;
            }
        }
        // Recipe Op1adjustsp_id
        103 => {
            if let InstructionData::UnaryImm { opcode, imm, .. } = func.dfg[inst] {
                put_op1(bits, rex1(RU::rsp.into()), sink);
                modrm_r_bits(RU::rsp.into(), bits, sink);
                let imm: i64 = imm.into();
                sink.put4(imm as u32);
                return;
            }
        }
        // Recipe RexOp1adjustsp_ib
        104 => {
            if let InstructionData::UnaryImm { opcode, imm, .. } = func.dfg[inst] {
                put_rexop1(bits, rex1(RU::rsp.into()), sink);
                modrm_r_bits(RU::rsp.into(), bits, sink);
                let imm: i64 = imm.into();
                sink.put1(imm as u8);
                return;
            }
        }
        // Recipe RexOp1adjustsp_id
        105 => {
            if let InstructionData::UnaryImm { opcode, imm, .. } = func.dfg[inst] {
                put_rexop1(bits, rex1(RU::rsp.into()), sink);
                modrm_r_bits(RU::rsp.into(), bits, sink);
                let imm: i64 = imm.into();
                sink.put4(imm as u32);
                return;
            }
        }
        // Recipe Mp2fld
        106 => {
            if let InstructionData::Load {
                opcode,
                flags,
                offset,
                arg,
                ..
            } = func.dfg[inst]
            {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_mp2(bits, rex2(in_reg0, out_reg0), sink);
                if needs_sib_byte(in_reg0) {
                    modrm_sib(out_reg0, sink);
                    sib_noindex(in_reg0, sink);
                } else if needs_offset(in_reg0) {
                    modrm_disp8(in_reg0, out_reg0, sink);
                    sink.put1(0);
                } else {
                    modrm_rm(in_reg0, out_reg0, sink);
                }
                return;
            }
        }
        // Recipe RexMp2fld
        107 => {
            if let InstructionData::Load {
                opcode,
                flags,
                offset,
                arg,
                ..
            } = func.dfg[inst]
            {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexmp2(bits, rex2(in_reg0, out_reg0), sink);
                if needs_sib_byte(in_reg0) {
                    modrm_sib(out_reg0, sink);
                    sib_noindex(in_reg0, sink);
                } else if needs_offset(in_reg0) {
                    modrm_disp8(in_reg0, out_reg0, sink);
                    sink.put1(0);
                } else {
                    modrm_rm(in_reg0, out_reg0, sink);
                }
                return;
            }
        }
        // Recipe Mp2fldDisp8
        108 => {
            if let InstructionData::Load {
                opcode,
                flags,
                offset,
                arg,
                ..
            } = func.dfg[inst]
            {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_mp2(bits, rex2(in_reg0, out_reg0), sink);
                if needs_sib_byte(in_reg0) {
                    modrm_sib_disp8(out_reg0, sink);
                    sib_noindex(in_reg0, sink);
                } else {
                    modrm_disp8(in_reg0, out_reg0, sink);
                }
                let offset: i32 = offset.into();
                sink.put1(offset as u8);
                return;
            }
        }
        // Recipe RexMp2fldDisp8
        109 => {
            if let InstructionData::Load {
                opcode,
                flags,
                offset,
                arg,
                ..
            } = func.dfg[inst]
            {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexmp2(bits, rex2(in_reg0, out_reg0), sink);
                if needs_sib_byte(in_reg0) {
                    modrm_sib_disp8(out_reg0, sink);
                    sib_noindex(in_reg0, sink);
                } else {
                    modrm_disp8(in_reg0, out_reg0, sink);
                }
                let offset: i32 = offset.into();
                sink.put1(offset as u8);
                return;
            }
        }
        // Recipe Mp2fldDisp32
        110 => {
            if let InstructionData::Load {
                opcode,
                flags,
                offset,
                arg,
                ..
            } = func.dfg[inst]
            {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_mp2(bits, rex2(in_reg0, out_reg0), sink);
                if needs_sib_byte(in_reg0) {
                    modrm_sib_disp32(out_reg0, sink);
                    sib_noindex(in_reg0, sink);
                } else {
                    modrm_disp32(in_reg0, out_reg0, sink);
                }
                let offset: i32 = offset.into();
                sink.put4(offset as u32);
                return;
            }
        }
        // Recipe RexMp2fldDisp32
        111 => {
            if let InstructionData::Load {
                opcode,
                flags,
                offset,
                arg,
                ..
            } = func.dfg[inst]
            {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexmp2(bits, rex2(in_reg0, out_reg0), sink);
                if needs_sib_byte(in_reg0) {
                    modrm_sib_disp32(out_reg0, sink);
                    sib_noindex(in_reg0, sink);
                } else {
                    modrm_disp32(in_reg0, out_reg0, sink);
                }
                let offset: i32 = offset.into();
                sink.put4(offset as u32);
                return;
            }
        }
        // Recipe Mp2fldWithIndex
        112 => {
            if let InstructionData::LoadComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_mp2(bits, rex3(in_reg0, out_reg0, in_reg1), sink);
                // The else branch always inserts an SIB byte.
                if needs_offset(in_reg0) {
                    modrm_sib_disp8(out_reg0, sink);
                    sib(0, in_reg1, in_reg0, sink);
                    sink.put1(0);
                } else {
                    modrm_sib(out_reg0, sink);
                    sib(0, in_reg1, in_reg0, sink);
                }
                return;
            }
        }
        // Recipe RexMp2fldWithIndex
        113 => {
            if let InstructionData::LoadComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexmp2(bits, rex3(in_reg0, out_reg0, in_reg1), sink);
                // The else branch always inserts an SIB byte.
                if needs_offset(in_reg0) {
                    modrm_sib_disp8(out_reg0, sink);
                    sib(0, in_reg1, in_reg0, sink);
                    sink.put1(0);
                } else {
                    modrm_sib(out_reg0, sink);
                    sib(0, in_reg1, in_reg0, sink);
                }
                return;
            }
        }
        // Recipe Mp2fldWithIndexDisp8
        114 => {
            if let InstructionData::LoadComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_mp2(bits, rex3(in_reg0, out_reg0, in_reg1), sink);
                modrm_sib_disp8(out_reg0, sink);
                sib(0, in_reg1, in_reg0, sink);
                let offset: i32 = offset.into();
                sink.put1(offset as u8);
                return;
            }
        }
        // Recipe RexMp2fldWithIndexDisp8
        115 => {
            if let InstructionData::LoadComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexmp2(bits, rex3(in_reg0, out_reg0, in_reg1), sink);
                modrm_sib_disp8(out_reg0, sink);
                sib(0, in_reg1, in_reg0, sink);
                let offset: i32 = offset.into();
                sink.put1(offset as u8);
                return;
            }
        }
        // Recipe Mp2fldWithIndexDisp32
        116 => {
            if let InstructionData::LoadComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_mp2(bits, rex3(in_reg0, out_reg0, in_reg1), sink);
                modrm_sib_disp32(out_reg0, sink);
                sib(0, in_reg1, in_reg0, sink);
                let offset: i32 = offset.into();
                sink.put4(offset as u32);
                return;
            }
        }
        // Recipe RexMp2fldWithIndexDisp32
        117 => {
            if let InstructionData::LoadComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexmp2(bits, rex3(in_reg0, out_reg0, in_reg1), sink);
                modrm_sib_disp32(out_reg0, sink);
                sib(0, in_reg1, in_reg0, sink);
                let offset: i32 = offset.into();
                sink.put4(offset as u32);
                return;
            }
        }
        // Recipe Mp2fst
        118 => {
            if let InstructionData::Store {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_mp2(bits, rex2(in_reg1, in_reg0), sink);
                if needs_sib_byte(in_reg1) {
                    modrm_sib(in_reg0, sink);
                    sib_noindex(in_reg1, sink);
                } else if needs_offset(in_reg1) {
                    modrm_disp8(in_reg1, in_reg0, sink);
                    sink.put1(0);
                } else {
                    modrm_rm(in_reg1, in_reg0, sink);
                }
                return;
            }
        }
        // Recipe RexMp2fst
        119 => {
            if let InstructionData::Store {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexmp2(bits, rex2(in_reg1, in_reg0), sink);
                if needs_sib_byte(in_reg1) {
                    modrm_sib(in_reg0, sink);
                    sib_noindex(in_reg1, sink);
                } else if needs_offset(in_reg1) {
                    modrm_disp8(in_reg1, in_reg0, sink);
                    sink.put1(0);
                } else {
                    modrm_rm(in_reg1, in_reg0, sink);
                }
                return;
            }
        }
        // Recipe Mp2fstDisp8
        120 => {
            if let InstructionData::Store {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_mp2(bits, rex2(in_reg1, in_reg0), sink);
                if needs_sib_byte(in_reg1) {
                    modrm_sib_disp8(in_reg0, sink);
                    sib_noindex(in_reg1, sink);
                } else {
                    modrm_disp8(in_reg1, in_reg0, sink);
                }
                let offset: i32 = offset.into();
                sink.put1(offset as u8);
                return;
            }
        }
        // Recipe RexMp2fstDisp8
        121 => {
            if let InstructionData::Store {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexmp2(bits, rex2(in_reg1, in_reg0), sink);
                if needs_sib_byte(in_reg1) {
                    modrm_sib_disp8(in_reg0, sink);
                    sib_noindex(in_reg1, sink);
                } else {
                    modrm_disp8(in_reg1, in_reg0, sink);
                }
                let offset: i32 = offset.into();
                sink.put1(offset as u8);
                return;
            }
        }
        // Recipe Mp2fstDisp32
        122 => {
            if let InstructionData::Store {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_mp2(bits, rex2(in_reg1, in_reg0), sink);
                if needs_sib_byte(in_reg1) {
                    modrm_sib_disp32(in_reg0, sink);
                    sib_noindex(in_reg1, sink);
                } else {
                    modrm_disp32(in_reg1, in_reg0, sink);
                }
                let offset: i32 = offset.into();
                sink.put4(offset as u32);
                return;
            }
        }
        // Recipe RexMp2fstDisp32
        123 => {
            if let InstructionData::Store {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexmp2(bits, rex2(in_reg1, in_reg0), sink);
                if needs_sib_byte(in_reg1) {
                    modrm_sib_disp32(in_reg0, sink);
                    sib_noindex(in_reg1, sink);
                } else {
                    modrm_disp32(in_reg1, in_reg0, sink);
                }
                let offset: i32 = offset.into();
                sink.put4(offset as u32);
                return;
            }
        }
        // Recipe Mp2fstWithIndex
        124 => {
            if let InstructionData::StoreComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let in_reg2 = divert.reg(args[2], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_mp2(bits, rex3(in_reg1, in_reg0, in_reg2), sink);
                // The else branch always inserts an SIB byte.
                if needs_offset(in_reg1) {
                    modrm_sib_disp8(in_reg0, sink);
                    sib(0, in_reg2, in_reg1, sink);
                    sink.put1(0);
                } else {
                    modrm_sib(in_reg0, sink);
                    sib(0, in_reg2, in_reg1, sink);
                }
                return;
            }
        }
        // Recipe RexMp2fstWithIndex
        125 => {
            if let InstructionData::StoreComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let in_reg2 = divert.reg(args[2], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexmp2(bits, rex3(in_reg1, in_reg0, in_reg2), sink);
                // The else branch always inserts an SIB byte.
                if needs_offset(in_reg1) {
                    modrm_sib_disp8(in_reg0, sink);
                    sib(0, in_reg2, in_reg1, sink);
                    sink.put1(0);
                } else {
                    modrm_sib(in_reg0, sink);
                    sib(0, in_reg2, in_reg1, sink);
                }
                return;
            }
        }
        // Recipe Mp2fstWithIndexDisp8
        126 => {
            if let InstructionData::StoreComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let in_reg2 = divert.reg(args[2], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_mp2(bits, rex3(in_reg1, in_reg0, in_reg2), sink);
                modrm_sib_disp8(in_reg0, sink);
                sib(0, in_reg2, in_reg1, sink);
                let offset: i32 = offset.into();
                sink.put1(offset as u8);
                return;
            }
        }
        // Recipe RexMp2fstWithIndexDisp8
        127 => {
            if let InstructionData::StoreComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let in_reg2 = divert.reg(args[2], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexmp2(bits, rex3(in_reg1, in_reg0, in_reg2), sink);
                modrm_sib_disp8(in_reg0, sink);
                sib(0, in_reg2, in_reg1, sink);
                let offset: i32 = offset.into();
                sink.put1(offset as u8);
                return;
            }
        }
        // Recipe Mp2fstWithIndexDisp32
        128 => {
            if let InstructionData::StoreComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let in_reg2 = divert.reg(args[2], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_mp2(bits, rex3(in_reg1, in_reg0, in_reg2), sink);
                modrm_sib_disp32(in_reg0, sink);
                sib(0, in_reg2, in_reg1, sink);
                let offset: i32 = offset.into();
                sink.put4(offset as u32);
                return;
            }
        }
        // Recipe RexMp2fstWithIndexDisp32
        129 => {
            if let InstructionData::StoreComplex {
                opcode,
                flags,
                offset,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let in_reg2 = divert.reg(args[2], &func.locations);
                if !flags.notrap() {
                    sink.trap(TrapCode::HeapOutOfBounds, func.srclocs[inst]);
                }
                put_rexmp2(bits, rex3(in_reg1, in_reg0, in_reg2), sink);
                modrm_sib_disp32(in_reg0, sink);
                sib(0, in_reg2, in_reg1, sink);
                let offset: i32 = offset.into();
                sink.put4(offset as u32);
                return;
            }
        }
        // Recipe Mp2ffillSib32
        130 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_stk0 = StackRef::masked(
                    divert.stack(args[0], &func.locations),
                    StackBaseMask(1),
                    &func.stack_slots,
                )
                .unwrap();
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                let base = stk_base(in_stk0.base);
                put_mp2(bits, rex2(base, out_reg0), sink);
                modrm_sib_disp32(out_reg0, sink);
                sib_noindex(base, sink);
                sink.put4(in_stk0.offset as u32);
                return;
            }
        }
        // Recipe RexMp2ffillSib32
        131 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_stk0 = StackRef::masked(
                    divert.stack(args[0], &func.locations),
                    StackBaseMask(1),
                    &func.stack_slots,
                )
                .unwrap();
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                let base = stk_base(in_stk0.base);
                put_rexmp2(bits, rex2(base, out_reg0), sink);
                modrm_sib_disp32(out_reg0, sink);
                sib_noindex(base, sink);
                sink.put4(in_stk0.offset as u32);
                return;
            }
        }
        // Recipe Mp2fregfill32
        132 => {
            if let InstructionData::RegFill {
                opcode,
                src,
                dst,
                arg,
                ..
            } = func.dfg[inst]
            {
                divert.regfill(arg, src, dst);
                let src = StackRef::sp(src, &func.stack_slots);
                let base = stk_base(src.base);
                put_mp2(bits, rex2(base, dst), sink);
                modrm_sib_disp32(dst, sink);
                sib_noindex(base, sink);
                sink.put4(src.offset as u32);
                return;
            }
        }
        // Recipe RexMp2fregfill32
        133 => {
            if let InstructionData::RegFill {
                opcode,
                src,
                dst,
                arg,
                ..
            } = func.dfg[inst]
            {
                divert.regfill(arg, src, dst);
                let src = StackRef::sp(src, &func.stack_slots);
                let base = stk_base(src.base);
                put_rexmp2(bits, rex2(base, dst), sink);
                modrm_sib_disp32(dst, sink);
                sib_noindex(base, sink);
                sink.put4(src.offset as u32);
                return;
            }
        }
        // Recipe Mp2fspillSib32
        134 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_stk0 = StackRef::masked(
                    divert.stack(results[0], &func.locations),
                    StackBaseMask(1),
                    &func.stack_slots,
                )
                .unwrap();
                sink.trap(TrapCode::StackOverflow, func.srclocs[inst]);
                let base = stk_base(out_stk0.base);
                put_mp2(bits, rex2(base, in_reg0), sink);
                modrm_sib_disp32(in_reg0, sink);
                sib_noindex(base, sink);
                sink.put4(out_stk0.offset as u32);
                return;
            }
        }
        // Recipe RexMp2fspillSib32
        135 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_stk0 = StackRef::masked(
                    divert.stack(results[0], &func.locations),
                    StackBaseMask(1),
                    &func.stack_slots,
                )
                .unwrap();
                sink.trap(TrapCode::StackOverflow, func.srclocs[inst]);
                let base = stk_base(out_stk0.base);
                put_rexmp2(bits, rex2(base, in_reg0), sink);
                modrm_sib_disp32(in_reg0, sink);
                sib_noindex(base, sink);
                sink.put4(out_stk0.offset as u32);
                return;
            }
        }
        // Recipe Mp2fregspill32
        136 => {
            if let InstructionData::RegSpill {
                opcode,
                src,
                dst,
                arg,
                ..
            } = func.dfg[inst]
            {
                divert.regspill(arg, src, dst);
                sink.trap(TrapCode::StackOverflow, func.srclocs[inst]);
                let dst = StackRef::sp(dst, &func.stack_slots);
                let base = stk_base(dst.base);
                put_mp2(bits, rex2(base, src), sink);
                modrm_sib_disp32(src, sink);
                sib_noindex(base, sink);
                sink.put4(dst.offset as u32);
                return;
            }
        }
        // Recipe RexMp2fregspill32
        137 => {
            if let InstructionData::RegSpill {
                opcode,
                src,
                dst,
                arg,
                ..
            } = func.dfg[inst]
            {
                divert.regspill(arg, src, dst);
                sink.trap(TrapCode::StackOverflow, func.srclocs[inst]);
                let dst = StackRef::sp(dst, &func.stack_slots);
                let base = stk_base(dst.base);
                put_rexmp2(bits, rex2(base, src), sink);
                modrm_sib_disp32(src, sink);
                sib_noindex(base, sink);
                sink.put4(dst.offset as u32);
                return;
            }
        }
        // Recipe Op1fnaddr4
        138 => {
            if let InstructionData::FuncAddr {
                opcode, func_ref, ..
            } = func.dfg[inst]
            {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_op1(bits | (out_reg0 & 7), rex1(out_reg0), sink);
                sink.reloc_external(Reloc::Abs4, &func.dfg.ext_funcs[func_ref].name, 0);
                sink.put4(0);
                return;
            }
        }
        // Recipe RexOp1fnaddr8
        139 => {
            if let InstructionData::FuncAddr {
                opcode, func_ref, ..
            } = func.dfg[inst]
            {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_rexop1(bits | (out_reg0 & 7), rex1(out_reg0), sink);
                sink.reloc_external(Reloc::Abs8, &func.dfg.ext_funcs[func_ref].name, 0);
                sink.put8(0);
                return;
            }
        }
        // Recipe Op1allones_fnaddr4
        140 => {
            if let InstructionData::FuncAddr {
                opcode, func_ref, ..
            } = func.dfg[inst]
            {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_op1(bits | (out_reg0 & 7), rex1(out_reg0), sink);
                sink.reloc_external(Reloc::Abs4, &func.dfg.ext_funcs[func_ref].name, 0);
                // Write the immediate as `!0` for the benefit of BaldrMonkey.
                sink.put4(!0);
                return;
            }
        }
        // Recipe RexOp1allones_fnaddr8
        141 => {
            if let InstructionData::FuncAddr {
                opcode, func_ref, ..
            } = func.dfg[inst]
            {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_rexop1(bits | (out_reg0 & 7), rex1(out_reg0), sink);
                sink.reloc_external(Reloc::Abs8, &func.dfg.ext_funcs[func_ref].name, 0);
                // Write the immediate as `!0` for the benefit of BaldrMonkey.
                sink.put8(!0);
                return;
            }
        }
        // Recipe RexOp1pcrel_fnaddr8
        142 => {
            if let InstructionData::FuncAddr {
                opcode, func_ref, ..
            } = func.dfg[inst]
            {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_rexop1(bits, rex2(0, out_reg0), sink);
                modrm_riprel(out_reg0, sink);
                // The addend adjusts for the difference between the end of the
                // instruction and the beginning of the immediate field.
                sink.reloc_external(Reloc::X86PCRel4, &func.dfg.ext_funcs[func_ref].name, -4);
                sink.put4(0);
                return;
            }
        }
        // Recipe RexOp1got_fnaddr8
        143 => {
            if let InstructionData::FuncAddr {
                opcode, func_ref, ..
            } = func.dfg[inst]
            {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_rexop1(bits, rex2(0, out_reg0), sink);
                modrm_riprel(out_reg0, sink);
                // The addend adjusts for the difference between the end of the
                // instruction and the beginning of the immediate field.
                sink.reloc_external(Reloc::X86GOTPCRel4, &func.dfg.ext_funcs[func_ref].name, -4);
                sink.put4(0);
                return;
            }
        }
        // Recipe Op1gvaddr4
        144 => {
            if let InstructionData::UnaryGlobalValue {
                opcode,
                global_value,
                ..
            } = func.dfg[inst]
            {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_op1(bits | (out_reg0 & 7), rex1(out_reg0), sink);
                sink.reloc_external(
                    Reloc::Abs4,
                    &func.global_values[global_value].symbol_name(),
                    0,
                );
                sink.put4(0);
                return;
            }
        }
        // Recipe RexOp1gvaddr8
        145 => {
            if let InstructionData::UnaryGlobalValue {
                opcode,
                global_value,
                ..
            } = func.dfg[inst]
            {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_rexop1(bits | (out_reg0 & 7), rex1(out_reg0), sink);
                sink.reloc_external(
                    Reloc::Abs8,
                    &func.global_values[global_value].symbol_name(),
                    0,
                );
                sink.put8(0);
                return;
            }
        }
        // Recipe RexOp1pcrel_gvaddr8
        146 => {
            if let InstructionData::UnaryGlobalValue {
                opcode,
                global_value,
                ..
            } = func.dfg[inst]
            {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_rexop1(bits, rex2(0, out_reg0), sink);
                modrm_rm(5, out_reg0, sink);
                // The addend adjusts for the difference between the end of the
                // instruction and the beginning of the immediate field.
                sink.reloc_external(
                    Reloc::X86PCRel4,
                    &func.global_values[global_value].symbol_name(),
                    -4,
                );
                sink.put4(0);
                return;
            }
        }
        // Recipe RexOp1got_gvaddr8
        147 => {
            if let InstructionData::UnaryGlobalValue {
                opcode,
                global_value,
                ..
            } = func.dfg[inst]
            {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_rexop1(bits, rex2(0, out_reg0), sink);
                modrm_rm(5, out_reg0, sink);
                // The addend adjusts for the difference between the end of the
                // instruction and the beginning of the immediate field.
                sink.reloc_external(
                    Reloc::X86GOTPCRel4,
                    &func.global_values[global_value].symbol_name(),
                    -4,
                );
                sink.put4(0);
                return;
            }
        }
        // Recipe Op1spaddr4_id
        148 => {
            if let InstructionData::StackLoad {
                opcode,
                stack_slot,
                offset,
                ..
            } = func.dfg[inst]
            {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                let sp = StackRef::sp(stack_slot, &func.stack_slots);
                let base = stk_base(sp.base);
                put_op1(bits, rex2(out_reg0, base), sink);
                modrm_sib_disp8(out_reg0, sink);
                sib_noindex(base, sink);
                let imm: i32 = offset.into();
                sink.put4(sp.offset.checked_add(imm).unwrap() as u32);
                return;
            }
        }
        // Recipe RexOp1spaddr8_id
        149 => {
            if let InstructionData::StackLoad {
                opcode,
                stack_slot,
                offset,
                ..
            } = func.dfg[inst]
            {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                let sp = StackRef::sp(stack_slot, &func.stack_slots);
                let base = stk_base(sp.base);
                put_rexop1(bits, rex2(base, out_reg0), sink);
                modrm_sib_disp32(out_reg0, sink);
                sib_noindex(base, sink);
                let imm: i32 = offset.into();
                sink.put4(sp.offset.checked_add(imm).unwrap() as u32);
                return;
            }
        }
        // Recipe Op1call_id
        150 => {
            if let InstructionData::Call {
                opcode, func_ref, ..
            } = func.dfg[inst]
            {
                sink.trap(TrapCode::StackOverflow, func.srclocs[inst]);
                put_op1(bits, BASE_REX, sink);
                // The addend adjusts for the difference between the end of the
                // instruction and the beginning of the immediate field.
                sink.reloc_external(Reloc::X86CallPCRel4, &func.dfg.ext_funcs[func_ref].name, -4);
                sink.put4(0);
                return;
            }
        }
        // Recipe Op1call_plt_id
        151 => {
            if let InstructionData::Call {
                opcode, func_ref, ..
            } = func.dfg[inst]
            {
                sink.trap(TrapCode::StackOverflow, func.srclocs[inst]);
                put_op1(bits, BASE_REX, sink);
                sink.reloc_external(
                    Reloc::X86CallPLTRel4,
                    &func.dfg.ext_funcs[func_ref].name,
                    -4,
                );
                sink.put4(0);
                return;
            }
        }
        // Recipe Op1call_r
        152 => {
            if let InstructionData::CallIndirect {
                opcode,
                sig_ref,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                sink.trap(TrapCode::StackOverflow, func.srclocs[inst]);
                put_op1(bits, rex1(in_reg0), sink);
                modrm_r_bits(in_reg0, bits, sink);
                return;
            }
        }
        // Recipe RexOp1call_r
        153 => {
            if let InstructionData::CallIndirect {
                opcode,
                sig_ref,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                sink.trap(TrapCode::StackOverflow, func.srclocs[inst]);
                put_rexop1(bits, rex1(in_reg0), sink);
                modrm_r_bits(in_reg0, bits, sink);
                return;
            }
        }
        // Recipe Op1ret
        154 => {
            if let InstructionData::MultiAry { opcode, .. } = func.dfg[inst] {
                put_op1(bits, BASE_REX, sink);
                return;
            }
        }
        // Recipe Op1jmpb
        155 => {
            if let InstructionData::Jump {
                opcode,
                destination,
                ..
            } = func.dfg[inst]
            {
                put_op1(bits, BASE_REX, sink);
                disp1(destination, func, sink);
                return;
            }
        }
        // Recipe Op1jmpd
        156 => {
            if let InstructionData::Jump {
                opcode,
                destination,
                ..
            } = func.dfg[inst]
            {
                put_op1(bits, BASE_REX, sink);
                disp4(destination, func, sink);
                return;
            }
        }
        // Recipe Op1brib
        157 => {
            if let InstructionData::BranchInt {
                opcode,
                cond,
                destination,
                ..
            } = func.dfg[inst]
            {
                put_op1(bits | icc2opc(cond), BASE_REX, sink);
                disp1(destination, func, sink);
                return;
            }
        }
        // Recipe RexOp1brib
        158 => {
            if let InstructionData::BranchInt {
                opcode,
                cond,
                destination,
                ..
            } = func.dfg[inst]
            {
                put_rexop1(bits | icc2opc(cond), BASE_REX, sink);
                disp1(destination, func, sink);
                return;
            }
        }
        // Recipe Op2brid
        159 => {
            if let InstructionData::BranchInt {
                opcode,
                cond,
                destination,
                ..
            } = func.dfg[inst]
            {
                put_op2(bits | icc2opc(cond), BASE_REX, sink);
                disp4(destination, func, sink);
                return;
            }
        }
        // Recipe RexOp2brid
        160 => {
            if let InstructionData::BranchInt {
                opcode,
                cond,
                destination,
                ..
            } = func.dfg[inst]
            {
                put_rexop2(bits | icc2opc(cond), BASE_REX, sink);
                disp4(destination, func, sink);
                return;
            }
        }
        // Recipe Op1brfb
        161 => {
            if let InstructionData::BranchFloat {
                opcode,
                cond,
                destination,
                ..
            } = func.dfg[inst]
            {
                put_op1(bits | fcc2opc(cond), BASE_REX, sink);
                disp1(destination, func, sink);
                return;
            }
        }
        // Recipe RexOp1brfb
        162 => {
            if let InstructionData::BranchFloat {
                opcode,
                cond,
                destination,
                ..
            } = func.dfg[inst]
            {
                put_rexop1(bits | fcc2opc(cond), BASE_REX, sink);
                disp1(destination, func, sink);
                return;
            }
        }
        // Recipe Op2brfd
        163 => {
            if let InstructionData::BranchFloat {
                opcode,
                cond,
                destination,
                ..
            } = func.dfg[inst]
            {
                put_op2(bits | fcc2opc(cond), BASE_REX, sink);
                disp4(destination, func, sink);
                return;
            }
        }
        // Recipe RexOp2brfd
        164 => {
            if let InstructionData::BranchFloat {
                opcode,
                cond,
                destination,
                ..
            } = func.dfg[inst]
            {
                put_rexop2(bits | fcc2opc(cond), BASE_REX, sink);
                disp4(destination, func, sink);
                return;
            }
        }
        // Recipe Op1tjccb
        165 => {
            if let InstructionData::Branch {
                opcode,
                destination,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                // test r, r.
                put_op1((bits & 0xff00) | 0x85, rex2(in_reg0, in_reg0), sink);
                modrm_rr(in_reg0, in_reg0, sink);
                // Jcc instruction.
                sink.put1(bits as u8);
                disp1(destination, func, sink);
                return;
            }
        }
        // Recipe RexOp1tjccb
        166 => {
            if let InstructionData::Branch {
                opcode,
                destination,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                // test r, r.
                put_rexop1((bits & 0xff00) | 0x85, rex2(in_reg0, in_reg0), sink);
                modrm_rr(in_reg0, in_reg0, sink);
                // Jcc instruction.
                sink.put1(bits as u8);
                disp1(destination, func, sink);
                return;
            }
        }
        // Recipe Op1tjccd
        167 => {
            if let InstructionData::Branch {
                opcode,
                destination,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                // test r, r.
                put_op1((bits & 0xff00) | 0x85, rex2(in_reg0, in_reg0), sink);
                modrm_rr(in_reg0, in_reg0, sink);
                // Jcc instruction.
                sink.put1(0x0f);
                sink.put1(bits as u8);
                disp4(destination, func, sink);
                return;
            }
        }
        // Recipe RexOp1tjccd
        168 => {
            if let InstructionData::Branch {
                opcode,
                destination,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                // test r, r.
                put_rexop1((bits & 0xff00) | 0x85, rex2(in_reg0, in_reg0), sink);
                modrm_rr(in_reg0, in_reg0, sink);
                // Jcc instruction.
                sink.put1(0x0f);
                sink.put1(bits as u8);
                disp4(destination, func, sink);
                return;
            }
        }
        // Recipe Op1t8jccd_long
        169 => {
            if let InstructionData::Branch {
                opcode,
                destination,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                // test32 r, 0xff.
                put_op1((bits & 0xff00) | 0xf7, rex1(in_reg0), sink);
                modrm_r_bits(in_reg0, bits, sink);
                sink.put4(0xff);
                // Jcc instruction.
                sink.put1(0x0f);
                sink.put1(bits as u8);
                disp4(destination, func, sink);
                return;
            }
        }
        // Recipe Op1t8jccb_abcd
        170 => {
            if let InstructionData::Branch {
                opcode,
                destination,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                // test8 r, r.
                put_op1((bits & 0xff00) | 0x84, rex2(in_reg0, in_reg0), sink);
                modrm_rr(in_reg0, in_reg0, sink);
                // Jcc instruction.
                sink.put1(bits as u8);
                disp1(destination, func, sink);
                return;
            }
        }
        // Recipe RexOp1t8jccb
        171 => {
            if let InstructionData::Branch {
                opcode,
                destination,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                // test8 r, r.
                put_rexop1((bits & 0xff00) | 0x84, rex2(in_reg0, in_reg0), sink);
                modrm_rr(in_reg0, in_reg0, sink);
                // Jcc instruction.
                sink.put1(bits as u8);
                disp1(destination, func, sink);
                return;
            }
        }
        // Recipe Op1t8jccd_abcd
        172 => {
            if let InstructionData::Branch {
                opcode,
                destination,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                // test8 r, r.
                put_op1((bits & 0xff00) | 0x84, rex2(in_reg0, in_reg0), sink);
                modrm_rr(in_reg0, in_reg0, sink);
                // Jcc instruction.
                sink.put1(0x0f);
                sink.put1(bits as u8);
                disp4(destination, func, sink);
                return;
            }
        }
        // Recipe RexOp1t8jccd
        173 => {
            if let InstructionData::Branch {
                opcode,
                destination,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let args = args.as_slice(&func.dfg.value_lists);
                let in_reg0 = divert.reg(args[0], &func.locations);
                // test8 r, r.
                put_rexop1((bits & 0xff00) | 0x84, rex2(in_reg0, in_reg0), sink);
                modrm_rr(in_reg0, in_reg0, sink);
                // Jcc instruction.
                sink.put1(0x0f);
                sink.put1(bits as u8);
                disp4(destination, func, sink);
                return;
            }
        }
        // Recipe RexOp1jt_entry
        174 => {
            if let InstructionData::BranchTableEntry {
                opcode,
                imm,
                table,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_rexop1(bits, rex3(in_reg1, out_reg0, in_reg0), sink);
                if needs_offset(in_reg1) {
                    modrm_sib_disp8(out_reg0, sink);
                    sib(imm.trailing_zeros() as u8, in_reg0, in_reg1, sink);
                    sink.put1(0);
                } else {
                    modrm_sib(out_reg0, sink);
                    sib(imm.trailing_zeros() as u8, in_reg0, in_reg1, sink);
                }
                return;
            }
        }
        // Recipe Op1jt_entry
        175 => {
            if let InstructionData::BranchTableEntry {
                opcode,
                imm,
                table,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_op1(bits, rex3(in_reg1, out_reg0, in_reg0), sink);
                if needs_offset(in_reg1) {
                    modrm_sib_disp8(out_reg0, sink);
                    sib(imm.trailing_zeros() as u8, in_reg0, in_reg1, sink);
                    sink.put1(0);
                } else {
                    modrm_sib(out_reg0, sink);
                    sib(imm.trailing_zeros() as u8, in_reg0, in_reg1, sink);
                }
                return;
            }
        }
        // Recipe RexOp1jt_base
        176 => {
            if let InstructionData::BranchTableBase { opcode, table, .. } = func.dfg[inst] {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_rexop1(bits, rex2(0, out_reg0), sink);
                modrm_riprel(out_reg0, sink);

                // No reloc is needed here as the jump table is emitted directly after
                // the function body.
                jt_disp4(table, func, sink);
                return;
            }
        }
        // Recipe Op1jt_base
        177 => {
            if let InstructionData::BranchTableBase { opcode, table, .. } = func.dfg[inst] {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_op1(bits, rex2(0, out_reg0), sink);
                modrm_riprel(out_reg0, sink);

                // No reloc is needed here as the jump table is emitted directly after
                // the function body.
                jt_disp4(table, func, sink);
                return;
            }
        }
        // Recipe RexOp1indirect_jmp
        178 => {
            if let InstructionData::IndirectJump {
                opcode, table, arg, ..
            } = func.dfg[inst]
            {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                put_rexop1(bits, rex1(in_reg0), sink);
                modrm_r_bits(in_reg0, bits, sink);
                return;
            }
        }
        // Recipe Op1indirect_jmp
        179 => {
            if let InstructionData::IndirectJump {
                opcode, table, arg, ..
            } = func.dfg[inst]
            {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                put_op1(bits, rex1(in_reg0), sink);
                modrm_r_bits(in_reg0, bits, sink);
                return;
            }
        }
        // Recipe Op2trap
        180 => {
            if let InstructionData::Trap { opcode, code, .. } = func.dfg[inst] {
                sink.trap(code, func.srclocs[inst]);
                put_op2(bits, BASE_REX, sink);
                return;
            }
        }
        // Recipe debugtrap
        181 => {
            if let InstructionData::NullAry { opcode, .. } = func.dfg[inst] {
                sink.put1(0xcc);
                return;
            }
        }
        // Recipe trapif
        182 => {
            if let InstructionData::IntCondTrap {
                opcode, cond, code, ..
            } = func.dfg[inst]
            {
                // Jump over a 2-byte ud2.
                sink.put1(0x70 | (icc2opc(cond.inverse()) as u8));
                sink.put1(2);
                // ud2.
                sink.trap(code, func.srclocs[inst]);
                sink.put1(0x0f);
                sink.put1(0x0b);
                return;
            }
        }
        // Recipe trapff
        183 => {
            if let InstructionData::FloatCondTrap {
                opcode, cond, code, ..
            } = func.dfg[inst]
            {
                // Jump over a 2-byte ud2.
                sink.put1(0x70 | (fcc2opc(cond.inverse()) as u8));
                sink.put1(2);
                // ud2.
                sink.trap(code, func.srclocs[inst]);
                sink.put1(0x0f);
                sink.put1(0x0b);
                return;
            }
        }
        // Recipe Op1icscc
        184 => {
            if let InstructionData::IntCompare {
                opcode,
                cond,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                // Comparison instruction.
                put_op1(bits, rex2(in_reg0, in_reg1), sink);
                modrm_rr(in_reg0, in_reg1, sink);
                // `setCC` instruction, no REX.
                use crate::ir::condcodes::IntCC::*;
                let setcc = match cond {
                    Equal => 0x94,
                    NotEqual => 0x95,
                    SignedLessThan => 0x9c,
                    SignedGreaterThanOrEqual => 0x9d,
                    SignedGreaterThan => 0x9f,
                    SignedLessThanOrEqual => 0x9e,
                    UnsignedLessThan => 0x92,
                    UnsignedGreaterThanOrEqual => 0x93,
                    UnsignedGreaterThan => 0x97,
                    UnsignedLessThanOrEqual => 0x96,
                };
                sink.put1(0x0f);
                sink.put1(setcc);
                modrm_rr(out_reg0, 0, sink);
                return;
            }
        }
        // Recipe RexOp1icscc
        185 => {
            if let InstructionData::IntCompare {
                opcode,
                cond,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                // Comparison instruction.
                put_rexop1(bits, rex2(in_reg0, in_reg1), sink);
                modrm_rr(in_reg0, in_reg1, sink);
                // `setCC` instruction, no REX.
                use crate::ir::condcodes::IntCC::*;
                let setcc = match cond {
                    Equal => 0x94,
                    NotEqual => 0x95,
                    SignedLessThan => 0x9c,
                    SignedGreaterThanOrEqual => 0x9d,
                    SignedGreaterThan => 0x9f,
                    SignedLessThanOrEqual => 0x9e,
                    UnsignedLessThan => 0x92,
                    UnsignedGreaterThanOrEqual => 0x93,
                    UnsignedGreaterThan => 0x97,
                    UnsignedLessThanOrEqual => 0x96,
                };
                sink.put1(0x0f);
                sink.put1(setcc);
                modrm_rr(out_reg0, 0, sink);
                return;
            }
        }
        // Recipe Op1icscc_ib
        186 => {
            if let InstructionData::IntCompareImm {
                opcode,
                cond,
                imm,
                arg,
                ..
            } = func.dfg[inst]
            {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                // Comparison instruction.
                put_op1(bits, rex1(in_reg0), sink);
                modrm_r_bits(in_reg0, bits, sink);
                let imm: i64 = imm.into();
                sink.put1(imm as u8);
                // `setCC` instruction, no REX.
                use crate::ir::condcodes::IntCC::*;
                let setcc = match cond {
                    Equal => 0x94,
                    NotEqual => 0x95,
                    SignedLessThan => 0x9c,
                    SignedGreaterThanOrEqual => 0x9d,
                    SignedGreaterThan => 0x9f,
                    SignedLessThanOrEqual => 0x9e,
                    UnsignedLessThan => 0x92,
                    UnsignedGreaterThanOrEqual => 0x93,
                    UnsignedGreaterThan => 0x97,
                    UnsignedLessThanOrEqual => 0x96,
                };
                sink.put1(0x0f);
                sink.put1(setcc);
                modrm_rr(out_reg0, 0, sink);
                return;
            }
        }
        // Recipe RexOp1icscc_ib
        187 => {
            if let InstructionData::IntCompareImm {
                opcode,
                cond,
                imm,
                arg,
                ..
            } = func.dfg[inst]
            {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                // Comparison instruction.
                put_rexop1(bits, rex1(in_reg0), sink);
                modrm_r_bits(in_reg0, bits, sink);
                let imm: i64 = imm.into();
                sink.put1(imm as u8);
                // `setCC` instruction, no REX.
                use crate::ir::condcodes::IntCC::*;
                let setcc = match cond {
                    Equal => 0x94,
                    NotEqual => 0x95,
                    SignedLessThan => 0x9c,
                    SignedGreaterThanOrEqual => 0x9d,
                    SignedGreaterThan => 0x9f,
                    SignedLessThanOrEqual => 0x9e,
                    UnsignedLessThan => 0x92,
                    UnsignedGreaterThanOrEqual => 0x93,
                    UnsignedGreaterThan => 0x97,
                    UnsignedLessThanOrEqual => 0x96,
                };
                sink.put1(0x0f);
                sink.put1(setcc);
                modrm_rr(out_reg0, 0, sink);
                return;
            }
        }
        // Recipe Op1icscc_id
        188 => {
            if let InstructionData::IntCompareImm {
                opcode,
                cond,
                imm,
                arg,
                ..
            } = func.dfg[inst]
            {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                // Comparison instruction.
                put_op1(bits, rex1(in_reg0), sink);
                modrm_r_bits(in_reg0, bits, sink);
                let imm: i64 = imm.into();
                sink.put4(imm as u32);
                // `setCC` instruction, no REX.
                use crate::ir::condcodes::IntCC::*;
                let setcc = match cond {
                    Equal => 0x94,
                    NotEqual => 0x95,
                    SignedLessThan => 0x9c,
                    SignedGreaterThanOrEqual => 0x9d,
                    SignedGreaterThan => 0x9f,
                    SignedLessThanOrEqual => 0x9e,
                    UnsignedLessThan => 0x92,
                    UnsignedGreaterThanOrEqual => 0x93,
                    UnsignedGreaterThan => 0x97,
                    UnsignedLessThanOrEqual => 0x96,
                };
                sink.put1(0x0f);
                sink.put1(setcc);
                modrm_rr(out_reg0, 0, sink);
                return;
            }
        }
        // Recipe RexOp1icscc_id
        189 => {
            if let InstructionData::IntCompareImm {
                opcode,
                cond,
                imm,
                arg,
                ..
            } = func.dfg[inst]
            {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                // Comparison instruction.
                put_rexop1(bits, rex1(in_reg0), sink);
                modrm_r_bits(in_reg0, bits, sink);
                let imm: i64 = imm.into();
                sink.put4(imm as u32);
                // `setCC` instruction, no REX.
                use crate::ir::condcodes::IntCC::*;
                let setcc = match cond {
                    Equal => 0x94,
                    NotEqual => 0x95,
                    SignedLessThan => 0x9c,
                    SignedGreaterThanOrEqual => 0x9d,
                    SignedGreaterThan => 0x9f,
                    SignedLessThanOrEqual => 0x9e,
                    UnsignedLessThan => 0x92,
                    UnsignedGreaterThanOrEqual => 0x93,
                    UnsignedGreaterThan => 0x97,
                    UnsignedLessThanOrEqual => 0x96,
                };
                sink.put1(0x0f);
                sink.put1(setcc);
                modrm_rr(out_reg0, 0, sink);
                return;
            }
        }
        // Recipe Op1rcmp
        190 => {
            if let InstructionData::Binary {
                opcode, ref args, ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                put_op1(bits, rex2(in_reg0, in_reg1), sink);
                modrm_rr(in_reg0, in_reg1, sink);
                return;
            }
        }
        // Recipe RexOp1rcmp
        191 => {
            if let InstructionData::Binary {
                opcode, ref args, ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                put_rexop1(bits, rex2(in_reg0, in_reg1), sink);
                modrm_rr(in_reg0, in_reg1, sink);
                return;
            }
        }
        // Recipe Op1rcmp_ib
        192 => {
            if let InstructionData::BinaryImm {
                opcode, imm, arg, ..
            } = func.dfg[inst]
            {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                put_op1(bits, rex1(in_reg0), sink);
                modrm_r_bits(in_reg0, bits, sink);
                let imm: i64 = imm.into();
                sink.put1(imm as u8);
                return;
            }
        }
        // Recipe RexOp1rcmp_ib
        193 => {
            if let InstructionData::BinaryImm {
                opcode, imm, arg, ..
            } = func.dfg[inst]
            {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                put_rexop1(bits, rex1(in_reg0), sink);
                modrm_r_bits(in_reg0, bits, sink);
                let imm: i64 = imm.into();
                sink.put1(imm as u8);
                return;
            }
        }
        // Recipe Op1rcmp_id
        194 => {
            if let InstructionData::BinaryImm {
                opcode, imm, arg, ..
            } = func.dfg[inst]
            {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                put_op1(bits, rex1(in_reg0), sink);
                modrm_r_bits(in_reg0, bits, sink);
                let imm: i64 = imm.into();
                sink.put4(imm as u32);
                return;
            }
        }
        // Recipe RexOp1rcmp_id
        195 => {
            if let InstructionData::BinaryImm {
                opcode, imm, arg, ..
            } = func.dfg[inst]
            {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                put_rexop1(bits, rex1(in_reg0), sink);
                modrm_r_bits(in_reg0, bits, sink);
                let imm: i64 = imm.into();
                sink.put4(imm as u32);
                return;
            }
        }
        // Recipe Op1rcmp_sp
        196 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                put_op1(bits, rex2(in_reg0, RU::rsp.into()), sink);
                modrm_rr(in_reg0, RU::rsp.into(), sink);
                return;
            }
        }
        // Recipe RexOp1rcmp_sp
        197 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                put_rexop1(bits, rex2(in_reg0, RU::rsp.into()), sink);
                modrm_rr(in_reg0, RU::rsp.into(), sink);
                return;
            }
        }
        // Recipe Op2seti_abcd
        198 => {
            if let InstructionData::IntCond { opcode, cond, .. } = func.dfg[inst] {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_op2(bits | icc2opc(cond), rex1(out_reg0), sink);
                modrm_r_bits(out_reg0, bits, sink);
                return;
            }
        }
        // Recipe RexOp2seti
        199 => {
            if let InstructionData::IntCond { opcode, cond, .. } = func.dfg[inst] {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_rexop2(bits | icc2opc(cond), rex1(out_reg0), sink);
                modrm_r_bits(out_reg0, bits, sink);
                return;
            }
        }
        // Recipe Op2setf_abcd
        200 => {
            if let InstructionData::FloatCond { opcode, cond, .. } = func.dfg[inst] {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_op2(bits | fcc2opc(cond), rex1(out_reg0), sink);
                modrm_r_bits(out_reg0, bits, sink);
                return;
            }
        }
        // Recipe RexOp2setf
        201 => {
            if let InstructionData::FloatCond { opcode, cond, .. } = func.dfg[inst] {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_rexop2(bits | fcc2opc(cond), rex1(out_reg0), sink);
                modrm_r_bits(out_reg0, bits, sink);
                return;
            }
        }
        // Recipe Op2cmov
        202 => {
            if let InstructionData::IntSelect {
                opcode,
                cond,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let in_reg1 = divert.reg(args[1], &func.locations);
                let in_reg2 = divert.reg(args[2], &func.locations);
                put_op2(bits | icc2opc(cond), rex2(in_reg1, in_reg2), sink);
                modrm_rr(in_reg1, in_reg2, sink);
                return;
            }
        }
        // Recipe RexOp2cmov
        203 => {
            if let InstructionData::IntSelect {
                opcode,
                cond,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let in_reg1 = divert.reg(args[1], &func.locations);
                let in_reg2 = divert.reg(args[2], &func.locations);
                put_rexop2(bits | icc2opc(cond), rex2(in_reg1, in_reg2), sink);
                modrm_rr(in_reg1, in_reg2, sink);
                return;
            }
        }
        // Recipe Op2bsf_and_bsr
        204 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = func.dfg.inst_results(inst);
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_op2(bits, rex2(in_reg0, out_reg0), sink);
                modrm_rr(in_reg0, out_reg0, sink);
                return;
            }
        }
        // Recipe RexOp2bsf_and_bsr
        205 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = func.dfg.inst_results(inst);
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_rexop2(bits, rex2(in_reg0, out_reg0), sink);
                modrm_rr(in_reg0, out_reg0, sink);
                return;
            }
        }
        // Recipe Op2urm_noflags_abcd
        206 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_op2(bits, rex2(in_reg0, out_reg0), sink);
                modrm_rr(in_reg0, out_reg0, sink);
                return;
            }
        }
        // Recipe RexOp2urm_noflags
        207 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_rexop2(bits, rex2(in_reg0, out_reg0), sink);
                modrm_rr(in_reg0, out_reg0, sink);
                return;
            }
        }
        // Recipe null
        208 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                return;
            }
        }
        // Recipe Op2urm_noflags
        209 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_op2(bits, rex2(in_reg0, out_reg0), sink);
                modrm_rr(in_reg0, out_reg0, sink);
                return;
            }
        }
        // Recipe RexOp1urm_noflags
        210 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_rexop1(bits, rex2(in_reg0, out_reg0), sink);
                modrm_rr(in_reg0, out_reg0, sink);
                return;
            }
        }
        // Recipe Op2f32imm_z
        211 => {
            if let InstructionData::UnaryIeee32 { opcode, imm, .. } = func.dfg[inst] {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_op2(bits, rex2(out_reg0, out_reg0), sink);
                modrm_rr(out_reg0, out_reg0, sink);
                return;
            }
        }
        // Recipe Mp2f64imm_z
        212 => {
            if let InstructionData::UnaryIeee64 { opcode, imm, .. } = func.dfg[inst] {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_mp2(bits, rex2(out_reg0, out_reg0), sink);
                modrm_rr(out_reg0, out_reg0, sink);
                return;
            }
        }
        // Recipe RexOp2f32imm_z
        213 => {
            if let InstructionData::UnaryIeee32 { opcode, imm, .. } = func.dfg[inst] {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_rexop2(bits, rex2(out_reg0, out_reg0), sink);
                modrm_rr(out_reg0, out_reg0, sink);
                return;
            }
        }
        // Recipe RexMp2f64imm_z
        214 => {
            if let InstructionData::UnaryIeee64 { opcode, imm, .. } = func.dfg[inst] {
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_rexmp2(bits, rex2(out_reg0, out_reg0), sink);
                modrm_rr(out_reg0, out_reg0, sink);
                return;
            }
        }
        // Recipe Mp2frurm
        215 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_mp2(bits, rex2(in_reg0, out_reg0), sink);
                modrm_rr(in_reg0, out_reg0, sink);
                return;
            }
        }
        // Recipe RexMp2frurm
        216 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_rexmp2(bits, rex2(in_reg0, out_reg0), sink);
                modrm_rr(in_reg0, out_reg0, sink);
                return;
            }
        }
        // Recipe Mp2rfumr
        217 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_mp2(bits, rex2(out_reg0, in_reg0), sink);
                modrm_rr(out_reg0, in_reg0, sink);
                return;
            }
        }
        // Recipe RexMp2rfumr
        218 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_rexmp2(bits, rex2(out_reg0, in_reg0), sink);
                modrm_rr(out_reg0, in_reg0, sink);
                return;
            }
        }
        // Recipe Op2furm
        219 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_op2(bits, rex2(in_reg0, out_reg0), sink);
                modrm_rr(in_reg0, out_reg0, sink);
                return;
            }
        }
        // Recipe RexOp2furm
        220 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_rexop2(bits, rex2(in_reg0, out_reg0), sink);
                modrm_rr(in_reg0, out_reg0, sink);
                return;
            }
        }
        // Recipe Op2frmov
        221 => {
            if let InstructionData::RegMove {
                opcode,
                src,
                dst,
                arg,
                ..
            } = func.dfg[inst]
            {
                divert.regmove(arg, src, dst);
                put_op2(bits, rex2(src, dst), sink);
                modrm_rr(src, dst, sink);
                return;
            }
        }
        // Recipe RexOp2frmov
        222 => {
            if let InstructionData::RegMove {
                opcode,
                src,
                dst,
                arg,
                ..
            } = func.dfg[inst]
            {
                divert.regmove(arg, src, dst);
                put_rexop2(bits, rex2(src, dst), sink);
                modrm_rr(src, dst, sink);
                return;
            }
        }
        // Recipe Mp2furm
        223 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_mp2(bits, rex2(in_reg0, out_reg0), sink);
                modrm_rr(in_reg0, out_reg0, sink);
                return;
            }
        }
        // Recipe RexMp2furm
        224 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_rexmp2(bits, rex2(in_reg0, out_reg0), sink);
                modrm_rr(in_reg0, out_reg0, sink);
                return;
            }
        }
        // Recipe Mp2rfurm
        225 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_mp2(bits, rex2(in_reg0, out_reg0), sink);
                modrm_rr(in_reg0, out_reg0, sink);
                return;
            }
        }
        // Recipe RexMp2rfurm
        226 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_rexmp2(bits, rex2(in_reg0, out_reg0), sink);
                modrm_rr(in_reg0, out_reg0, sink);
                return;
            }
        }
        // Recipe Mp3furmi_rnd
        227 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_mp3(bits, rex2(in_reg0, out_reg0), sink);
                modrm_rr(in_reg0, out_reg0, sink);
                sink.put1(match opcode {
                    Opcode::Nearest => 0b00,
                    Opcode::Floor => 0b01,
                    Opcode::Ceil => 0b10,
                    Opcode::Trunc => 0b11,
                    x => panic!("{} unexpected for furmi_rnd", opcode),
                });
                return;
            }
        }
        // Recipe RexMp3furmi_rnd
        228 => {
            if let InstructionData::Unary { opcode, arg, .. } = func.dfg[inst] {
                let args = [arg];
                let in_reg0 = divert.reg(args[0], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                put_rexmp3(bits, rex2(in_reg0, out_reg0), sink);
                modrm_rr(in_reg0, out_reg0, sink);
                sink.put1(match opcode {
                    Opcode::Nearest => 0b00,
                    Opcode::Floor => 0b01,
                    Opcode::Ceil => 0b10,
                    Opcode::Trunc => 0b11,
                    x => panic!("{} unexpected for furmi_rnd", opcode),
                });
                return;
            }
        }
        // Recipe Mp2fa
        229 => {
            if let InstructionData::Binary {
                opcode, ref args, ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                put_mp2(bits, rex2(in_reg1, in_reg0), sink);
                modrm_rr(in_reg1, in_reg0, sink);
                return;
            }
        }
        // Recipe RexMp2fa
        230 => {
            if let InstructionData::Binary {
                opcode, ref args, ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                put_rexmp2(bits, rex2(in_reg1, in_reg0), sink);
                modrm_rr(in_reg1, in_reg0, sink);
                return;
            }
        }
        // Recipe Op2fa
        231 => {
            if let InstructionData::Binary {
                opcode, ref args, ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                put_op2(bits, rex2(in_reg1, in_reg0), sink);
                modrm_rr(in_reg1, in_reg0, sink);
                return;
            }
        }
        // Recipe RexOp2fa
        232 => {
            if let InstructionData::Binary {
                opcode, ref args, ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                put_rexop2(bits, rex2(in_reg1, in_reg0), sink);
                modrm_rr(in_reg1, in_reg0, sink);
                return;
            }
        }
        // Recipe Op2fax
        233 => {
            if let InstructionData::Binary {
                opcode, ref args, ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                put_op2(bits, rex2(in_reg0, in_reg1), sink);
                modrm_rr(in_reg0, in_reg1, sink);
                return;
            }
        }
        // Recipe RexOp2fax
        234 => {
            if let InstructionData::Binary {
                opcode, ref args, ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                put_rexop2(bits, rex2(in_reg0, in_reg1), sink);
                modrm_rr(in_reg0, in_reg1, sink);
                return;
            }
        }
        // Recipe Op2fcscc
        235 => {
            if let InstructionData::FloatCompare {
                opcode,
                cond,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                // Comparison instruction.
                put_op2(bits, rex2(in_reg1, in_reg0), sink);
                modrm_rr(in_reg1, in_reg0, sink);
                // `setCC` instruction, no REX.
                use crate::ir::condcodes::FloatCC::*;
                let setcc = match cond {
                    Ordered                    => 0x9b, // EQ|LT|GT => setnp (P=0)
                    Unordered                  => 0x9a, // UN       => setp  (P=1)
                    OrderedNotEqual            => 0x95, // LT|GT    => setne (Z=0),
                    UnorderedOrEqual           => 0x94, // UN|EQ    => sete  (Z=1)
                    GreaterThan                => 0x97, // GT       => seta  (C=0&Z=0)
                    GreaterThanOrEqual         => 0x93, // GT|EQ    => setae (C=0)
                    UnorderedOrLessThan        => 0x92, // UN|LT    => setb  (C=1)
                    UnorderedOrLessThanOrEqual => 0x96, // UN|LT|EQ => setbe (Z=1|C=1)
                    Equal |                       // EQ
                    NotEqual |                    // UN|LT|GT
                    LessThan |                    // LT
                    LessThanOrEqual |             // LT|EQ
                    UnorderedOrGreaterThan |      // UN|GT
                    UnorderedOrGreaterThanOrEqual // UN|GT|EQ
                    => panic!("{} not supported by fcscc", cond),
                };
                sink.put1(0x0f);
                sink.put1(setcc);
                modrm_rr(out_reg0, 0, sink);
                return;
            }
        }
        // Recipe RexOp2fcscc
        236 => {
            if let InstructionData::FloatCompare {
                opcode,
                cond,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                // Comparison instruction.
                put_rexop2(bits, rex2(in_reg1, in_reg0), sink);
                modrm_rr(in_reg1, in_reg0, sink);
                // `setCC` instruction, no REX.
                use crate::ir::condcodes::FloatCC::*;
                let setcc = match cond {
                    Ordered                    => 0x9b, // EQ|LT|GT => setnp (P=0)
                    Unordered                  => 0x9a, // UN       => setp  (P=1)
                    OrderedNotEqual            => 0x95, // LT|GT    => setne (Z=0),
                    UnorderedOrEqual           => 0x94, // UN|EQ    => sete  (Z=1)
                    GreaterThan                => 0x97, // GT       => seta  (C=0&Z=0)
                    GreaterThanOrEqual         => 0x93, // GT|EQ    => setae (C=0)
                    UnorderedOrLessThan        => 0x92, // UN|LT    => setb  (C=1)
                    UnorderedOrLessThanOrEqual => 0x96, // UN|LT|EQ => setbe (Z=1|C=1)
                    Equal |                       // EQ
                    NotEqual |                    // UN|LT|GT
                    LessThan |                    // LT
                    LessThanOrEqual |             // LT|EQ
                    UnorderedOrGreaterThan |      // UN|GT
                    UnorderedOrGreaterThanOrEqual // UN|GT|EQ
                    => panic!("{} not supported by fcscc", cond),
                };
                sink.put1(0x0f);
                sink.put1(setcc);
                modrm_rr(out_reg0, 0, sink);
                return;
            }
        }
        // Recipe Mp2fcscc
        237 => {
            if let InstructionData::FloatCompare {
                opcode,
                cond,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                // Comparison instruction.
                put_mp2(bits, rex2(in_reg1, in_reg0), sink);
                modrm_rr(in_reg1, in_reg0, sink);
                // `setCC` instruction, no REX.
                use crate::ir::condcodes::FloatCC::*;
                let setcc = match cond {
                    Ordered                    => 0x9b, // EQ|LT|GT => setnp (P=0)
                    Unordered                  => 0x9a, // UN       => setp  (P=1)
                    OrderedNotEqual            => 0x95, // LT|GT    => setne (Z=0),
                    UnorderedOrEqual           => 0x94, // UN|EQ    => sete  (Z=1)
                    GreaterThan                => 0x97, // GT       => seta  (C=0&Z=0)
                    GreaterThanOrEqual         => 0x93, // GT|EQ    => setae (C=0)
                    UnorderedOrLessThan        => 0x92, // UN|LT    => setb  (C=1)
                    UnorderedOrLessThanOrEqual => 0x96, // UN|LT|EQ => setbe (Z=1|C=1)
                    Equal |                       // EQ
                    NotEqual |                    // UN|LT|GT
                    LessThan |                    // LT
                    LessThanOrEqual |             // LT|EQ
                    UnorderedOrGreaterThan |      // UN|GT
                    UnorderedOrGreaterThanOrEqual // UN|GT|EQ
                    => panic!("{} not supported by fcscc", cond),
                };
                sink.put1(0x0f);
                sink.put1(setcc);
                modrm_rr(out_reg0, 0, sink);
                return;
            }
        }
        // Recipe RexMp2fcscc
        238 => {
            if let InstructionData::FloatCompare {
                opcode,
                cond,
                ref args,
                ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                let results = [func.dfg.first_result(inst)];
                let out_reg0 = divert.reg(results[0], &func.locations);
                // Comparison instruction.
                put_rexmp2(bits, rex2(in_reg1, in_reg0), sink);
                modrm_rr(in_reg1, in_reg0, sink);
                // `setCC` instruction, no REX.
                use crate::ir::condcodes::FloatCC::*;
                let setcc = match cond {
                    Ordered                    => 0x9b, // EQ|LT|GT => setnp (P=0)
                    Unordered                  => 0x9a, // UN       => setp  (P=1)
                    OrderedNotEqual            => 0x95, // LT|GT    => setne (Z=0),
                    UnorderedOrEqual           => 0x94, // UN|EQ    => sete  (Z=1)
                    GreaterThan                => 0x97, // GT       => seta  (C=0&Z=0)
                    GreaterThanOrEqual         => 0x93, // GT|EQ    => setae (C=0)
                    UnorderedOrLessThan        => 0x92, // UN|LT    => setb  (C=1)
                    UnorderedOrLessThanOrEqual => 0x96, // UN|LT|EQ => setbe (Z=1|C=1)
                    Equal |                       // EQ
                    NotEqual |                    // UN|LT|GT
                    LessThan |                    // LT
                    LessThanOrEqual |             // LT|EQ
                    UnorderedOrGreaterThan |      // UN|GT
                    UnorderedOrGreaterThanOrEqual // UN|GT|EQ
                    => panic!("{} not supported by fcscc", cond),
                };
                sink.put1(0x0f);
                sink.put1(setcc);
                modrm_rr(out_reg0, 0, sink);
                return;
            }
        }
        // Recipe Op2fcmp
        239 => {
            if let InstructionData::Binary {
                opcode, ref args, ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                put_op2(bits, rex2(in_reg1, in_reg0), sink);
                modrm_rr(in_reg1, in_reg0, sink);
                return;
            }
        }
        // Recipe RexOp2fcmp
        240 => {
            if let InstructionData::Binary {
                opcode, ref args, ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                put_rexop2(bits, rex2(in_reg1, in_reg0), sink);
                modrm_rr(in_reg1, in_reg0, sink);
                return;
            }
        }
        // Recipe Mp2fcmp
        241 => {
            if let InstructionData::Binary {
                opcode, ref args, ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                put_mp2(bits, rex2(in_reg1, in_reg0), sink);
                modrm_rr(in_reg1, in_reg0, sink);
                return;
            }
        }
        // Recipe RexMp2fcmp
        242 => {
            if let InstructionData::Binary {
                opcode, ref args, ..
            } = func.dfg[inst]
            {
                let in_reg0 = divert.reg(args[0], &func.locations);
                let in_reg1 = divert.reg(args[1], &func.locations);
                put_rexmp2(bits, rex2(in_reg1, in_reg0), sink);
                modrm_rr(in_reg1, in_reg0, sink);
                return;
            }
        }
        _ => {}
    }
    if encoding.is_legal() {
        bad_encoding(func, inst);
    }
}

//clude!(concat!(env!("OUT_DIR"), "/binemit-x86.rs"));

// Convert a stack base to the corresponding register.
fn stk_base(base: StackBase) -> RegUnit {
    let ru = match base {
        StackBase::SP => RU::rsp,
        StackBase::FP => RU::rbp,
        StackBase::Zone => unimplemented!(),
    };
    ru as RegUnit
}

// Mandatory prefix bytes for Mp* opcodes.
const PREFIX: [u8; 3] = [0x66, 0xf3, 0xf2];

// Second byte for three-byte opcodes for mm=0b10 and mm=0b11.
const OP3_BYTE2: [u8; 2] = [0x38, 0x3a];

// A REX prefix with no bits set: 0b0100WRXB.
const BASE_REX: u8 = 0b0100_0000;

// Create a single-register REX prefix, setting the B bit to bit 3 of the register.
// This is used for instructions that encode a register in the low 3 bits of the opcode and for
// instructions that use the ModR/M `reg` field for something else.
fn rex1(reg_b: RegUnit) -> u8 {
    let b = ((reg_b >> 3) & 1) as u8;
    BASE_REX | b
}

// Create a dual-register REX prefix, setting:
//
// REX.B = bit 3 of r/m register, or SIB base register when a SIB byte is present.
// REX.R = bit 3 of reg register.
fn rex2(rm: RegUnit, reg: RegUnit) -> u8 {
    let b = ((rm >> 3) & 1) as u8;
    let r = ((reg >> 3) & 1) as u8;
    BASE_REX | b | (r << 2)
}

// Create a three-register REX prefix, setting:
//
// REX.B = bit 3 of r/m register, or SIB base register when a SIB byte is present.
// REX.R = bit 3 of reg register.
// REX.X = bit 3 of SIB index register.
fn rex3(rm: RegUnit, reg: RegUnit, index: RegUnit) -> u8 {
    let b = ((rm >> 3) & 1) as u8;
    let r = ((reg >> 3) & 1) as u8;
    let x = ((index >> 3) & 1) as u8;
    BASE_REX | b | (x << 1) | (r << 2)
}

// Emit a REX prefix.
//
// The R, X, and B bits are computed from registers using the functions above. The W bit is
// extracted from `bits`.
fn rex_prefix<CS: CodeSink + ?Sized>(bits: u16, rex: u8, sink: &mut CS) {
    debug_assert_eq!(rex & 0xf8, BASE_REX);
    let w = ((bits >> 15) & 1) as u8;
    sink.put1(rex | (w << 3));
}

// Emit a single-byte opcode with no REX prefix.
fn put_op1<CS: CodeSink + ?Sized>(bits: u16, rex: u8, sink: &mut CS) {
    debug_assert_eq!(bits & 0x8f00, 0, "Invalid encoding bits for Op1*");
    debug_assert_eq!(rex, BASE_REX, "Invalid registers for REX-less Op1 encoding");
    sink.put1(bits as u8);
}

// Emit a single-byte opcode with REX prefix.
fn put_rexop1<CS: CodeSink + ?Sized>(bits: u16, rex: u8, sink: &mut CS) {
    debug_assert_eq!(bits & 0x0f00, 0, "Invalid encoding bits for Op1*");
    rex_prefix(bits, rex, sink);
    sink.put1(bits as u8);
}

// Emit two-byte opcode: 0F XX
fn put_op2<CS: CodeSink + ?Sized>(bits: u16, rex: u8, sink: &mut CS) {
    debug_assert_eq!(bits & 0x8f00, 0x0400, "Invalid encoding bits for Op2*");
    debug_assert_eq!(rex, BASE_REX, "Invalid registers for REX-less Op2 encoding");
    sink.put1(0x0f);
    sink.put1(bits as u8);
}

// Emit two-byte opcode: 0F XX with REX prefix.
fn put_rexop2<CS: CodeSink + ?Sized>(bits: u16, rex: u8, sink: &mut CS) {
    debug_assert_eq!(bits & 0x0f00, 0x0400, "Invalid encoding bits for RexOp2*");
    rex_prefix(bits, rex, sink);
    sink.put1(0x0f);
    sink.put1(bits as u8);
}

// Emit single-byte opcode with mandatory prefix.
fn put_mp1<CS: CodeSink + ?Sized>(bits: u16, rex: u8, sink: &mut CS) {
    debug_assert_eq!(bits & 0x8c00, 0, "Invalid encoding bits for Mp1*");
    let pp = (bits >> 8) & 3;
    sink.put1(PREFIX[(pp - 1) as usize]);
    debug_assert_eq!(rex, BASE_REX, "Invalid registers for REX-less Mp1 encoding");
    sink.put1(bits as u8);
}

// Emit single-byte opcode with mandatory prefix and REX.
fn put_rexmp1<CS: CodeSink + ?Sized>(bits: u16, rex: u8, sink: &mut CS) {
    debug_assert_eq!(bits & 0x0c00, 0, "Invalid encoding bits for Mp1*");
    let pp = (bits >> 8) & 3;
    sink.put1(PREFIX[(pp - 1) as usize]);
    rex_prefix(bits, rex, sink);
    sink.put1(bits as u8);
}

// Emit two-byte opcode (0F XX) with mandatory prefix.
fn put_mp2<CS: CodeSink + ?Sized>(bits: u16, rex: u8, sink: &mut CS) {
    debug_assert_eq!(bits & 0x8c00, 0x0400, "Invalid encoding bits for Mp2*");
    let pp = (bits >> 8) & 3;
    sink.put1(PREFIX[(pp - 1) as usize]);
    debug_assert_eq!(rex, BASE_REX, "Invalid registers for REX-less Mp2 encoding");
    sink.put1(0x0f);
    sink.put1(bits as u8);
}

// Emit two-byte opcode (0F XX) with mandatory prefix and REX.
fn put_rexmp2<CS: CodeSink + ?Sized>(bits: u16, rex: u8, sink: &mut CS) {
    debug_assert_eq!(bits & 0x0c00, 0x0400, "Invalid encoding bits for Mp2*");
    let pp = (bits >> 8) & 3;
    sink.put1(PREFIX[(pp - 1) as usize]);
    rex_prefix(bits, rex, sink);
    sink.put1(0x0f);
    sink.put1(bits as u8);
}

// Emit three-byte opcode (0F 3[8A] XX) with mandatory prefix.
fn put_mp3<CS: CodeSink + ?Sized>(bits: u16, rex: u8, sink: &mut CS) {
    debug_assert_eq!(bits & 0x8800, 0x0800, "Invalid encoding bits for Mp3*");
    let pp = (bits >> 8) & 3;
    sink.put1(PREFIX[(pp - 1) as usize]);
    debug_assert_eq!(rex, BASE_REX, "Invalid registers for REX-less Mp3 encoding");
    let mm = (bits >> 10) & 3;
    sink.put1(0x0f);
    sink.put1(OP3_BYTE2[(mm - 2) as usize]);
    sink.put1(bits as u8);
}

// Emit three-byte opcode (0F 3[8A] XX) with mandatory prefix and REX
fn put_rexmp3<CS: CodeSink + ?Sized>(bits: u16, rex: u8, sink: &mut CS) {
    debug_assert_eq!(bits & 0x0800, 0x0800, "Invalid encoding bits for Mp3*");
    let pp = (bits >> 8) & 3;
    sink.put1(PREFIX[(pp - 1) as usize]);
    rex_prefix(bits, rex, sink);
    let mm = (bits >> 10) & 3;
    sink.put1(0x0f);
    sink.put1(OP3_BYTE2[(mm - 2) as usize]);
    sink.put1(bits as u8);
}

/// Emit a ModR/M byte for reg-reg operands.
fn modrm_rr<CS: CodeSink + ?Sized>(rm: RegUnit, reg: RegUnit, sink: &mut CS) {
    let reg = reg as u8 & 7;
    let rm = rm as u8 & 7;
    let mut b = 0b11000000;
    b |= reg << 3;
    b |= rm;
    sink.put1(b);
}

/// Emit a ModR/M byte where the reg bits are part of the opcode.
fn modrm_r_bits<CS: CodeSink + ?Sized>(rm: RegUnit, bits: u16, sink: &mut CS) {
    let reg = (bits >> 12) as u8 & 7;
    let rm = rm as u8 & 7;
    let mut b = 0b11000000;
    b |= reg << 3;
    b |= rm;
    sink.put1(b);
}

/// Emit a mode 00 ModR/M byte. This is a register-indirect addressing mode with no offset.
/// Registers %rsp and %rbp are invalid for `rm`, %rsp indicates a SIB byte, and %rbp indicates an
/// absolute immediate 32-bit address.
fn modrm_rm<CS: CodeSink + ?Sized>(rm: RegUnit, reg: RegUnit, sink: &mut CS) {
    let reg = reg as u8 & 7;
    let rm = rm as u8 & 7;
    let mut b = 0b00000000;
    b |= reg << 3;
    b |= rm;
    sink.put1(b);
}

/// Emit a mode 00 Mod/RM byte, with a rip-relative displacement in 64-bit mode. Effective address
/// is calculated by adding displacement to 64-bit rip of next instruction. See intel Sw dev manual
/// section 2.2.1.6.
fn modrm_riprel<CS: CodeSink + ?Sized>(reg: RegUnit, sink: &mut CS) {
    modrm_rm(0b101, reg, sink)
}

/// Emit a mode 01 ModR/M byte. This is a register-indirect addressing mode with 8-bit
/// displacement.
/// Register %rsp is invalid for `rm`. It indicates the presence of a SIB byte.
fn modrm_disp8<CS: CodeSink + ?Sized>(rm: RegUnit, reg: RegUnit, sink: &mut CS) {
    let reg = reg as u8 & 7;
    let rm = rm as u8 & 7;
    let mut b = 0b01000000;
    b |= reg << 3;
    b |= rm;
    sink.put1(b);
}

/// Emit a mode 10 ModR/M byte. This is a register-indirect addressing mode with 32-bit
/// displacement.
/// Register %rsp is invalid for `rm`. It indicates the presence of a SIB byte.
fn modrm_disp32<CS: CodeSink + ?Sized>(rm: RegUnit, reg: RegUnit, sink: &mut CS) {
    let reg = reg as u8 & 7;
    let rm = rm as u8 & 7;
    let mut b = 0b10000000;
    b |= reg << 3;
    b |= rm;
    sink.put1(b);
}

/// Emit a mode 00 ModR/M with a 100 RM indicating a SIB byte is present.
fn modrm_sib<CS: CodeSink + ?Sized>(reg: RegUnit, sink: &mut CS) {
    modrm_rm(0b100, reg, sink);
}

/// Emit a mode 01 ModR/M with a 100 RM indicating a SIB byte and 8-bit
/// displacement are present.
fn modrm_sib_disp8<CS: CodeSink + ?Sized>(reg: RegUnit, sink: &mut CS) {
    modrm_disp8(0b100, reg, sink);
}

/// Emit a mode 10 ModR/M with a 100 RM indicating a SIB byte and 32-bit
/// displacement are present.
fn modrm_sib_disp32<CS: CodeSink + ?Sized>(reg: RegUnit, sink: &mut CS) {
    modrm_disp32(0b100, reg, sink);
}

/// Emit a SIB byte with a base register and no scale+index.
fn sib_noindex<CS: CodeSink + ?Sized>(base: RegUnit, sink: &mut CS) {
    let base = base as u8 & 7;
    // SIB        SS_III_BBB.
    let mut b = 0b00_100_000;
    b |= base;
    sink.put1(b);
}

/// Emit a SIB byte with a scale, base, and index.
fn sib<CS: CodeSink + ?Sized>(scale: u8, index: RegUnit, base: RegUnit, sink: &mut CS) {
    // SIB        SS_III_BBB.
    debug_assert_eq!(scale & !0x03, 0, "Scale out of range");
    let scale = scale & 3;
    let index = index as u8 & 7;
    let base = base as u8 & 7;
    let b: u8 = (scale << 6) | (index << 3) | base;
    sink.put1(b);
}

/// Get the low 4 bits of an opcode for an integer condition code.
///
/// Add this offset to a base opcode for:
///
/// ---- 0x70: Short conditional branch.
/// 0x0f 0x80: Long conditional branch.
/// 0x0f 0x90: SetCC.
///
fn icc2opc(cond: IntCC) -> u16 {
    use crate::ir::condcodes::IntCC::*;
    match cond {
        // 0x0 = Overflow.
        // 0x1 = !Overflow.
        UnsignedLessThan => 0x2,
        UnsignedGreaterThanOrEqual => 0x3,
        Equal => 0x4,
        NotEqual => 0x5,
        UnsignedLessThanOrEqual => 0x6,
        UnsignedGreaterThan => 0x7,
        // 0x8 = Sign.
        // 0x9 = !Sign.
        // 0xa = Parity even.
        // 0xb = Parity odd.
        SignedLessThan => 0xc,
        SignedGreaterThanOrEqual => 0xd,
        SignedLessThanOrEqual => 0xe,
        SignedGreaterThan => 0xf,
    }
}

/// Get the low 4 bits of an opcode for a floating point condition code.
///
/// The ucomiss/ucomisd instructions set the FLAGS bits CF/PF/CF like this:
///
///    ZPC OSA
/// UN 111 000
/// GT 000 000
/// LT 001 000
/// EQ 100 000
///
/// Not all floating point condition codes are supported.
fn fcc2opc(cond: FloatCC) -> u16 {
    use crate::ir::condcodes::FloatCC::*;
    match cond {
        Ordered                    => 0xb, // EQ|LT|GT => *np (P=0)
        Unordered                  => 0xa, // UN       => *p  (P=1)
        OrderedNotEqual            => 0x5, // LT|GT    => *ne (Z=0),
        UnorderedOrEqual           => 0x4, // UN|EQ    => *e  (Z=1)
        GreaterThan                => 0x7, // GT       => *a  (C=0&Z=0)
        GreaterThanOrEqual         => 0x3, // GT|EQ    => *ae (C=0)
        UnorderedOrLessThan        => 0x2, // UN|LT    => *b  (C=1)
        UnorderedOrLessThanOrEqual => 0x6, // UN|LT|EQ => *be (Z=1|C=1)
        Equal |                            // EQ
        NotEqual |                         // UN|LT|GT
        LessThan |                         // LT
        LessThanOrEqual |                  // LT|EQ
        UnorderedOrGreaterThan |           // UN|GT
        UnorderedOrGreaterThanOrEqual      // UN|GT|EQ
        => panic!("{} not supported", cond),
    }
}

/// Emit a single-byte branch displacement to `destination`.
fn disp1<CS: CodeSink + ?Sized>(destination: Ebb, func: &Function, sink: &mut CS) {
    let delta = func.offsets[destination].wrapping_sub(sink.offset() + 1);
    sink.put1(delta as u8);
}

/// Emit a four-byte branch displacement to `destination`.
fn disp4<CS: CodeSink + ?Sized>(destination: Ebb, func: &Function, sink: &mut CS) {
    let delta = func.offsets[destination].wrapping_sub(sink.offset() + 4);
    sink.put4(delta);
}

/// Emit a four-byte displacement to jump table `jt`.
fn jt_disp4<CS: CodeSink + ?Sized>(jt: JumpTable, func: &Function, sink: &mut CS) {
    let delta = func.jt_offsets[jt].wrapping_sub(sink.offset() + 4);
    sink.put4(delta);
    sink.reloc_jt(Reloc::X86PCRelRodata4, jt);
}
