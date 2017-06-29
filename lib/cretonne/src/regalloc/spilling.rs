//! Spilling pass.
//!
//! The spilling pass is the first to run after the liveness analysis. Its primary function is to
//! ensure that the register pressure never exceeds the number of available registers by moving
//! some SSA values to spill slots on the stack. This is encoded in the affinity of the value's
//! live range.
//!
//! Some instruction operand constraints may require additional registers to resolve. Since this
//! can cause spilling, the spilling pass is also responsible for resolving those constraints by
//! inserting copies. The extra constraints are:
//!
//! 1. A value used by a tied operand must be killed by the instruction. This is resolved by
//!    inserting a copy to a temporary value when necessary.
//! 2. When the same value is used more than once by an instruction, the operand constraints must
//!    be compatible. Otherwise, the value must be copied into a new register for some of the
//!    operands.

use dominator_tree::DominatorTree;
use ir::{DataFlowGraph, Layout, Cursor, InstBuilder};
use ir::{Function, Ebb, Inst, Value, ValueLoc, ArgumentLoc, Signature, SigRef};
use ir::{InstEncodings, StackSlots, ValueLocations};
use isa::registers::{RegClassMask, RegClassIndex};
use isa::{TargetIsa, RegInfo, EncInfo, RecipeConstraints, ConstraintKind};
use regalloc::affinity::Affinity;
use regalloc::live_value_tracker::{LiveValue, LiveValueTracker};
use regalloc::liveness::Liveness;
use regalloc::pressure::Pressure;
use regalloc::virtregs::VirtRegs;
use topo_order::TopoOrder;

/// Persistent data structures for the spilling pass.
pub struct Spilling {
    spills: Vec<Value>,
    reg_uses: Vec<RegUse>,
}

/// Context data structure that gets instantiated once per pass.
struct Context<'a> {
    isa: &'a TargetIsa,
    // Cached ISA information.
    reginfo: RegInfo,
    encinfo: EncInfo,

    // References to parts of the current function.
    encodings: &'a mut InstEncodings,
    stack_slots: &'a mut StackSlots,
    locations: &'a mut ValueLocations,
    func_signature: &'a Signature,

    // References to contextual data structures we need.
    domtree: &'a DominatorTree,
    liveness: &'a mut Liveness,
    virtregs: &'a VirtRegs,
    topo: &'a mut TopoOrder,

    // Current register pressure.
    pressure: Pressure,

    // Values spilled for the current instruction. These values have already been removed from the
    // pressure tracker, but they are still present in the live value tracker and their affinity
    // hasn't been changed yet.
    spills: &'a mut Vec<Value>,

    // Uses of register values in the current instruction.
    reg_uses: &'a mut Vec<RegUse>,
}

impl Spilling {
    /// Create a new spilling data structure.
    pub fn new() -> Spilling {
        Spilling {
            spills: Vec::new(),
            reg_uses: Vec::new(),
        }
    }

    /// Run the spilling algorithm over `func`.
    pub fn run(&mut self,
               isa: &TargetIsa,
               func: &mut Function,
               domtree: &DominatorTree,
               liveness: &mut Liveness,
               virtregs: &VirtRegs,
               topo: &mut TopoOrder,
               tracker: &mut LiveValueTracker) {
        dbg!("Spilling for:\n{}", func.display(isa));
        let reginfo = isa.register_info();
        let usable_regs = isa.allocatable_registers(func);
        let mut ctx = Context {
            isa,
            reginfo: isa.register_info(),
            encinfo: isa.encoding_info(),
            encodings: &mut func.encodings,
            stack_slots: &mut func.stack_slots,
            locations: &mut func.locations,
            func_signature: &func.signature,
            domtree,
            liveness,
            virtregs,
            topo,
            pressure: Pressure::new(&reginfo, &usable_regs),
            spills: &mut self.spills,
            reg_uses: &mut self.reg_uses,
        };
        ctx.run(&mut func.layout, &mut func.dfg, tracker)
    }
}

impl<'a> Context<'a> {
    fn run(&mut self,
           layout: &mut Layout,
           dfg: &mut DataFlowGraph,
           tracker: &mut LiveValueTracker) {
        if let Some(entry) = layout.entry_block() {
            self.spill_entry_arguments(entry, dfg);
        }

        self.topo.reset(layout.ebbs());
        while let Some(ebb) = self.topo.next(layout, self.domtree) {
            self.visit_ebb(ebb, layout, dfg, tracker);
        }
    }

    /// Assign stack slots to incoming function arguments on the stack.
    fn spill_entry_arguments(&mut self, entry: Ebb, dfg: &DataFlowGraph) {
        for (abi, &arg) in self.func_signature
                .argument_types
                .iter()
                .zip(dfg.ebb_args(entry)) {
            if let ArgumentLoc::Stack(offset) = abi.location {
                // Function arguments passed on the stack can't be part of a virtual register. We
                // would need to write other values to the stack slot, but it belongs to the
                // caller. (Not that the caller would care, nobody depends on stack arguments being
                // preserved across calls).
                assert_eq!(self.virtregs.get(arg),
                           None,
                           "Stack argument {} can't be part of a virtual register",
                           arg);
                let ss = self.stack_slots.make_incoming_arg(abi.value_type, offset);
                *self.locations.ensure(arg) = ValueLoc::Stack(ss);
            }
        }
    }

    fn visit_ebb(&mut self,
                 ebb: Ebb,
                 layout: &mut Layout,
                 dfg: &mut DataFlowGraph,
                 tracker: &mut LiveValueTracker) {
        dbg!("Spilling {}:", ebb);
        self.visit_ebb_header(ebb, layout, dfg, tracker);
        tracker.drop_dead_args();

        let mut pos = Cursor::new(layout);
        pos.goto_top(ebb);
        while let Some(inst) = pos.next_inst() {
            if let Some(constraints) = self.encinfo.operand_constraints(self.encodings[inst]) {
                self.visit_inst(inst, constraints, &mut pos, dfg, tracker);
            } else {
                let (_throughs, kills) = tracker.process_ghost(inst);
                self.free_regs(kills);
            }
            tracker.drop_dead(inst);
            self.process_spills(tracker);
        }
    }

    // Take all live registers in `regs` from the pressure set.
    // This doesn't cause any spilling, it is assumed there are enough registers.
    fn take_live_regs(&mut self, regs: &[LiveValue]) {
        for lv in regs {
            if !lv.is_dead {
                if let Affinity::Reg(rci) = lv.affinity {
                    let rc = self.reginfo.rc(rci);
                    self.pressure.take(rc);
                }
            }
        }
    }

    // Free all registers in `kills` from the pressure set.
    fn free_regs(&mut self, kills: &[LiveValue]) {
        for lv in kills {
            if let Affinity::Reg(rci) = lv.affinity {
                let rc = self.reginfo.rc(rci);
                self.pressure.free(rc);
            }
        }
    }

    fn visit_ebb_header(&mut self,
                        ebb: Ebb,
                        layout: &mut Layout,
                        dfg: &mut DataFlowGraph,
                        tracker: &mut LiveValueTracker) {
        let (liveins, args) = tracker.ebb_top(ebb, dfg, self.liveness, layout, self.domtree);

        // Count the live-in registers. These should already fit in registers; they did at the
        // dominator.
        self.pressure.reset();
        self.take_live_regs(liveins);

        // An EBB can have an arbitrary (up to 2^16...) number of EBB arguments, so they are not
        // guaranteed to fit in registers.
        for lv in args {
            if let Affinity::Reg(rci) = lv.affinity {
                let rc = self.reginfo.rc(rci);
                'try_take: while let Err(mask) = self.pressure.take_transient(rc) {
                    dbg!("Need {} reg for EBB argument {} from {} live-ins",
                         rc,
                         lv.value,
                         liveins.len());
                    match self.spill_candidate(mask, liveins, dfg, layout) {
                        Some(cand) => {
                            dbg!("Spilling live-in {} to make room for {} EBB argument {}",
                                 cand,
                                 rc,
                                 lv.value);
                            self.spill_reg(cand, dfg);
                        }
                        None => {
                            // We can't spill any of the live-in registers, so we have to spill an
                            // EBB argument. Since the current spill metric would consider all the
                            // EBB arguments equal, just spill the present register.
                            dbg!("Spilling {} EBB argument {}", rc, lv.value);

                            // Since `spill_reg` will free a register, add the current one here.
                            self.pressure.take(rc);
                            self.spill_reg(lv.value, dfg);
                            break 'try_take;
                        }
                    }
                }
            }
        }

        // The transient pressure counts for the EBB arguments are accurate. Just preserve them.
        self.pressure.preserve_transient();
    }

    fn visit_inst(&mut self,
                  inst: Inst,
                  constraints: &RecipeConstraints,
                  pos: &mut Cursor,
                  dfg: &mut DataFlowGraph,
                  tracker: &mut LiveValueTracker) {
        dbg!("Inst {}, {}", dfg.display_inst(inst), self.pressure);

        // We may need to resolve register constraints if there are any noteworthy uses.
        assert!(self.reg_uses.is_empty());
        self.collect_reg_uses(inst, constraints, dfg);

        // Calls usually have fixed register uses.
        let call_sig = dfg.call_signature(inst);
        if let Some(sig) = call_sig {
            self.collect_abi_reg_uses(inst, sig, dfg);
        }

        if !self.reg_uses.is_empty() {
            self.process_reg_uses(inst, pos, dfg, tracker);
        }

        // Update the live value tracker with this instruction.
        let (throughs, kills, defs) = tracker.process_inst(inst, dfg, self.liveness);

        // Remove kills from the pressure tracker.
        self.free_regs(kills);

        // If inst is a call, spill all register values that are live across the call.
        // This means that we don't currently take advantage of callee-saved registers.
        // TODO: Be more sophisticated.
        if call_sig.is_some() {
            for lv in throughs {
                if lv.affinity.is_reg() && !self.spills.contains(&lv.value) {
                    self.spill_reg(lv.value, dfg);
                }
            }
        }

        // Make sure we have enough registers for the register defs.
        // Dead defs are included here. They need a register too.
        // No need to process call return values, they are in fixed registers.
        for op in constraints.outs {
            if op.kind != ConstraintKind::Stack {
                // Add register def to pressure, spill if needed.
                while let Err(mask) = self.pressure.take_transient(op.regclass) {
                    dbg!("Need {} reg from {} throughs", op.regclass, throughs.len());
                    match self.spill_candidate(mask, throughs, dfg, pos.layout) {
                        Some(cand) => self.spill_reg(cand, dfg),
                        None => {
                            panic!("Ran out of {} registers for {}",
                                   op.regclass,
                                   dfg.display_inst(inst))
                        }
                    }
                }
            }
        }
        self.pressure.reset_transient();

        // Restore pressure state, compute pressure with affinities from `defs`.
        // Exclude dead defs. Includes call return values.
        // This won't cause spilling.
        self.take_live_regs(defs);
    }

    // Collect register uses that are noteworthy in one of the following ways:
    //
    // 1. It's a fixed register constraint.
    // 2. It's a use of a spilled value.
    // 3. It's a tied register constraint and the value isn't killed.
    //
    // We are assuming here that if a value is used both by a fixed register operand and a register
    // class operand, they two are compatible. We are also assuming that two register class
    // operands are always compatible.
    fn collect_reg_uses(&mut self,
                        inst: Inst,
                        constraints: &RecipeConstraints,
                        dfg: &DataFlowGraph) {
        let args = dfg.inst_args(inst);
        for (idx, (op, &arg)) in constraints.ins.iter().zip(args).enumerate() {
            let mut reguse = RegUse::new(arg, idx, op.regclass.into());
            match op.kind {
                ConstraintKind::Stack => continue,
                ConstraintKind::FixedReg(_) => reguse.fixed = true,
                ConstraintKind::Tied(_) => {
                    // TODO: If `arg` isn't killed here, we need a copy
                }
                ConstraintKind::Reg => {}
            }
            let lr = &self.liveness[arg];
            if lr.affinity.is_stack() {
                reguse.spilled = true;
            }

            // Only collect the interesting register uses.
            if reguse.fixed || reguse.spilled {
                self.reg_uses.push(reguse);
            }
        }
    }

    // Collect register uses from the ABI input constraints.
    fn collect_abi_reg_uses(&mut self, inst: Inst, sig: SigRef, dfg: &DataFlowGraph) {
        let fixed_args = dfg[inst].opcode().constraints().fixed_value_arguments();
        let args = dfg.inst_variable_args(inst);
        for (idx, (abi, &arg)) in
            dfg.signatures[sig]
                .argument_types
                .iter()
                .zip(args)
                .enumerate() {
            if abi.location.is_reg() {
                let (rci, spilled) = match self.liveness[arg].affinity {
                    Affinity::Reg(rci) => (rci, false),
                    Affinity::Stack => {
                        (self.isa.regclass_for_abi_type(abi.value_type).into(), true)
                    }
                    Affinity::None => panic!("Missing affinity for {}", arg),
                };
                let mut reguse = RegUse::new(arg, fixed_args + idx, rci);
                reguse.fixed = true;
                reguse.spilled = spilled;
                self.reg_uses.push(reguse);
            }
        }
    }

    // Process multiple register uses to resolve potential conflicts.
    //
    // Look for multiple uses of the same value in `self.reg_uses` and insert copies as necessary.
    // Trigger spilling if any of the temporaries cause the register pressure to become too high.
    //
    // Leave `self.reg_uses` empty.
    fn process_reg_uses(&mut self,
                        inst: Inst,
                        pos: &mut Cursor,
                        dfg: &mut DataFlowGraph,
                        tracker: &LiveValueTracker) {
        // We're looking for multiple uses of the same value, so start by sorting by value. The
        // secondary `opidx` key makes it possible to use an unstable sort once that is available
        // outside nightly Rust.
        self.reg_uses.sort_by_key(|u| (u.value, u.opidx));

        for i in 0..self.reg_uses.len() {
            let ru = self.reg_uses[i];

            // Do we need to insert a copy for this use?
            let need_copy = if ru.tied {
                true
            } else if ru.fixed {
                // This is a fixed register use which doesn't necessarily require a copy.
                // Make a copy only if this is not the first use of the value.
                self.reg_uses
                    .get(i.wrapping_sub(1))
                    .map(|ru2| ru2.value == ru.value)
                    .unwrap_or(false)
            } else {
                false
            };

            if need_copy {
                let copy = self.insert_copy(ru.value, ru.rci, pos, dfg);
                dfg.inst_args_mut(inst)[ru.opidx as usize] = copy;
            }

            // Even if we don't insert a copy, we may need to account for register pressure for the
            // reload pass.
            if need_copy || ru.spilled {
                let rc = self.reginfo.rc(ru.rci);
                while let Err(mask) = self.pressure.take_transient(rc) {
                    dbg!("Copy of {} reg causes spill", rc);
                    // Spill a live register that is *not* used by the current instruction.
                    // Spilling a use wouldn't help.
                    let args = dfg.inst_args(inst);
                    match self.spill_candidate(mask,
                                               tracker.live().iter().filter(|lv| {
                        !args.contains(&lv.value)
                    }),
                                               dfg,
                                               &pos.layout) {
                        Some(cand) => self.spill_reg(cand, dfg),
                        None => {
                            panic!("Ran out of {} registers when inserting copy before {}",
                                   rc,
                                   dfg.display_inst(inst))
                        }
                    }
                }
            }
        }
        self.pressure.reset_transient();
        self.reg_uses.clear()
    }

    // Find a spill candidate from `candidates` whose top-level register class is in `mask`.
    fn spill_candidate<'ii, II>(&self,
                                mask: RegClassMask,
                                candidates: II,
                                dfg: &DataFlowGraph,
                                layout: &Layout)
                                -> Option<Value>
        where II: IntoIterator<Item = &'ii LiveValue>
    {
        // Find the best viable spill candidate.
        //
        // The very simple strategy implemented here is to spill the value with the earliest def in
        // the reverse post-order. This strategy depends on a good reload pass to generate good
        // code.
        //
        // We know that all candidate defs dominate the current instruction, so one of them will
        // dominate the others. That is the earliest def.
        candidates
            .into_iter()
            .filter_map(|lv| {
                // Viable candidates are registers in one of the `mask` classes, and not already in
                // the spill set.
                if let Affinity::Reg(rci) = lv.affinity {
                    let rc = self.reginfo.rc(rci);
                    if (mask & (1 << rc.toprc)) != 0 && !self.spills.contains(&lv.value) {
                        // Here, `lv` is a viable spill candidate.
                        return Some(lv.value);
                    }
                }
                None
            })
            .min_by(|&a, &b| {
                        // Find the minimum candidate according to the RPO of their defs.
                        self.domtree
                            .rpo_cmp(dfg.value_def(a), dfg.value_def(b), layout)
                    })
    }

    /// Spill `value` immediately by
    ///
    /// 1. Changing its affinity to `Stack` which marks the spill.
    /// 2. Removing the value from the pressure tracker.
    /// 3. Adding the value to `self.spills` for later reference by `process_spills`.
    ///
    /// Note that this does not update the cached affinity in the live value tracker. Call
    /// `process_spills` to do that.
    fn spill_reg(&mut self, value: Value, dfg: &DataFlowGraph) {
        if let Affinity::Reg(rci) = self.liveness.spill(value) {
            let rc = self.reginfo.rc(rci);
            self.pressure.free(rc);
            self.spills.push(value);
            dbg!("Spilled {}:{} -> {}", value, rc, self.pressure);
        } else {
            panic!("Cannot spill {} that was already on the stack", value);
        }

        // Assign a spill slot for the whole virtual register.
        let ss = self.stack_slots.make_spill_slot(dfg.value_type(value));
        for &v in self.virtregs.congruence_class(&value) {
            self.liveness.spill(v);
            *self.locations.ensure(v) = ValueLoc::Stack(ss);
        }
    }

    /// Process any pending spills in the `self.spills` vector.
    ///
    /// It is assumed that spills are removed from the pressure tracker immediately, see
    /// `spill_reg` above.
    ///
    /// We also need to update the live range affinity and remove spilled values from the live
    /// value tracker.
    fn process_spills(&mut self, tracker: &mut LiveValueTracker) {
        if !self.spills.is_empty() {
            tracker.process_spills(|v| self.spills.contains(&v));
            self.spills.clear()
        }
    }

    /// Insert a `copy value` before `pos` and give it a live range extending to `pos`.
    ///
    /// Returns the new local value created.
    fn insert_copy(&mut self,
                   value: Value,
                   rci: RegClassIndex,
                   pos: &mut Cursor,
                   dfg: &mut DataFlowGraph)
                   -> Value {
        let copy = dfg.ins(pos).copy(value);
        let inst = dfg.value_def(copy).unwrap_inst();
        let ty = dfg.value_type(copy);

        // Give it an encoding.
        let encoding = self.isa
            .encode(dfg, &dfg[inst], ty)
            .expect("Can't encode copy");
        *self.encodings.ensure(inst) = encoding;

        // Update live ranges.
        self.liveness.create_dead(copy, inst, Affinity::Reg(rci));
        self.liveness
            .extend_locally(copy,
                            pos.layout.pp_ebb(inst),
                            pos.current_inst().expect("must be at an instruction"),
                            pos.layout);

        copy
    }
}

// Struct representing a register use of a value.
// Used to detect multiple uses of the same value with incompatible register constraints.
#[derive(Clone, Copy)]
struct RegUse {
    value: Value,
    opidx: u16,

    // Register class required by the use.
    rci: RegClassIndex,

    // A use with a fixed register constraint.
    fixed: bool,

    // A register use of a spilled value.
    spilled: bool,

    // A use with a tied register constraint *and* the used value is not killed.
    tied: bool,
}

impl RegUse {
    fn new(value: Value, idx: usize, rci: RegClassIndex) -> RegUse {
        RegUse {
            value,
            opidx: idx as u16,
            rci,
            fixed: false,
            spilled: false,
            tied: false,
        }
    }
}