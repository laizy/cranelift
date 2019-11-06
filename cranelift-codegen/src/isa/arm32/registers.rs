//! ARM32 register descriptions.

use crate::isa::registers::{RegBank, RegClass, RegClassData, RegInfo, RegUnit};

 
pub static INFO: RegInfo = RegInfo {
    banks: &[
        RegBank {
            name: "FloatRegs",
            first_unit: 0,
            units: 64,
            names: &[],
            prefix: "s",
            first_toprc: 0,
            num_toprcs: 3,
            pressure_tracking: true,
        },
        RegBank {
            name: "IntRegs",
            first_unit: 64,
            units: 16,
            names: &[],
            prefix: "r",
            first_toprc: 3,
            num_toprcs: 1,
            pressure_tracking: true,
        },
        RegBank {
            name: "FlagRegs",
            first_unit: 80,
            units: 1,
            names: &["nzcv"],
            prefix: "",
            first_toprc: 4,
            num_toprcs: 1,
            pressure_tracking: false,
        },
    ],
    classes: &[
        &S_DATA,
        &D_DATA,
        &Q_DATA,
        &GPR_DATA,
        &FLAG_DATA,
    ],
};
pub static S_DATA: RegClassData = RegClassData {
    name: "S",
    index: 0,
    width: 1,
    bank: 0,
    toprc: 0,
    first: 0,
    subclasses: 0x1,
    mask: [0xffffffff, 0x00000000, 0x00000000],
    pinned_reg: None,
    info: &INFO,
};
#[allow(dead_code)]
pub static S: RegClass = &S_DATA;
pub static D_DATA: RegClassData = RegClassData {
    name: "D",
    index: 1,
    width: 2,
    bank: 0,
    toprc: 1,
    first: 0,
    subclasses: 0x2,
    mask: [0x55555555, 0x55555555, 0x00000000],
    pinned_reg: None,
    info: &INFO,
};
#[allow(dead_code)]
pub static D: RegClass = &D_DATA;
pub static Q_DATA: RegClassData = RegClassData {
    name: "Q",
    index: 2,
    width: 4,
    bank: 0,
    toprc: 2,
    first: 0,
    subclasses: 0x4,
    mask: [0x11111111, 0x11111111, 0x00000000],
    pinned_reg: None,
    info: &INFO,
};
#[allow(dead_code)]
pub static Q: RegClass = &Q_DATA;
pub static GPR_DATA: RegClassData = RegClassData {
    name: "GPR",
    index: 3,
    width: 1,
    bank: 1,
    toprc: 3,
    first: 64,
    subclasses: 0x8,
    mask: [0x00000000, 0x00000000, 0x0000ffff],
    pinned_reg: None,
    info: &INFO,
};
#[allow(dead_code)]
pub static GPR: RegClass = &GPR_DATA;
pub static FLAG_DATA: RegClassData = RegClassData {
    name: "FLAG",
    index: 4,
    width: 1,
    bank: 2,
    toprc: 4,
    first: 80,
    subclasses: 0x10,
    mask: [0x00000000, 0x00000000, 0x00010000],
    pinned_reg: None,
    info: &INFO,
};
#[allow(dead_code)]
pub static FLAG: RegClass = &FLAG_DATA;
#[allow(dead_code, non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum RU {
    s0 = 0,
    s1 = 1,
    s2 = 2,
    s3 = 3,
    s4 = 4,
    s5 = 5,
    s6 = 6,
    s7 = 7,
    s8 = 8,
    s9 = 9,
    s10 = 10,
    s11 = 11,
    s12 = 12,
    s13 = 13,
    s14 = 14,
    s15 = 15,
    s16 = 16,
    s17 = 17,
    s18 = 18,
    s19 = 19,
    s20 = 20,
    s21 = 21,
    s22 = 22,
    s23 = 23,
    s24 = 24,
    s25 = 25,
    s26 = 26,
    s27 = 27,
    s28 = 28,
    s29 = 29,
    s30 = 30,
    s31 = 31,
    s32 = 32,
    s33 = 33,
    s34 = 34,
    s35 = 35,
    s36 = 36,
    s37 = 37,
    s38 = 38,
    s39 = 39,
    s40 = 40,
    s41 = 41,
    s42 = 42,
    s43 = 43,
    s44 = 44,
    s45 = 45,
    s46 = 46,
    s47 = 47,
    s48 = 48,
    s49 = 49,
    s50 = 50,
    s51 = 51,
    s52 = 52,
    s53 = 53,
    s54 = 54,
    s55 = 55,
    s56 = 56,
    s57 = 57,
    s58 = 58,
    s59 = 59,
    s60 = 60,
    s61 = 61,
    s62 = 62,
    s63 = 63,
    r0 = 64,
    r1 = 65,
    r2 = 66,
    r3 = 67,
    r4 = 68,
    r5 = 69,
    r6 = 70,
    r7 = 71,
    r8 = 72,
    r9 = 73,
    r10 = 74,
    r11 = 75,
    r12 = 76,
    r13 = 77,
    r14 = 78,
    r15 = 79,
    nzcv = 80,
}
impl Into<RegUnit> for RU {
    fn into(self) -> RegUnit {
        self as RegUnit
    }
}

 //clude!(concat!(env!("OUT_DIR"), "/registers-arm32.rs"));

#[cfg(test)]
mod tests {
    use super::{D, GPR, INFO, S};
    use crate::isa::RegUnit;
    use alloc::string::{String, ToString};

    #[test]
    fn unit_encodings() {
        assert_eq!(INFO.parse_regunit("s0"), Some(0));
        assert_eq!(INFO.parse_regunit("s31"), Some(31));
        assert_eq!(INFO.parse_regunit("s32"), Some(32));
        assert_eq!(INFO.parse_regunit("r0"), Some(64));
        assert_eq!(INFO.parse_regunit("r15"), Some(79));
    }

    #[test]
    fn unit_names() {
        fn uname(ru: RegUnit) -> String {
            INFO.display_regunit(ru).to_string()
        }

        assert_eq!(uname(0), "%s0");
        assert_eq!(uname(1), "%s1");
        assert_eq!(uname(31), "%s31");
        assert_eq!(uname(64), "%r0");
    }

    #[test]
    fn overlaps() {
        // arm32 has the most interesting register geometries, so test `regs_overlap()` here.
        use crate::isa::regs_overlap;

        let r0 = GPR.unit(0);
        let r1 = GPR.unit(1);
        let r2 = GPR.unit(2);

        assert!(regs_overlap(GPR, r0, GPR, r0));
        assert!(regs_overlap(GPR, r2, GPR, r2));
        assert!(!regs_overlap(GPR, r0, GPR, r1));
        assert!(!regs_overlap(GPR, r1, GPR, r0));
        assert!(!regs_overlap(GPR, r2, GPR, r1));
        assert!(!regs_overlap(GPR, r1, GPR, r2));

        let s0 = S.unit(0);
        let s1 = S.unit(1);
        let s2 = S.unit(2);
        let s3 = S.unit(3);
        let d0 = D.unit(0);
        let d1 = D.unit(1);

        assert!(regs_overlap(S, s0, D, d0));
        assert!(regs_overlap(S, s1, D, d0));
        assert!(!regs_overlap(S, s0, D, d1));
        assert!(!regs_overlap(S, s1, D, d1));
        assert!(regs_overlap(S, s2, D, d1));
        assert!(regs_overlap(S, s3, D, d1));
        assert!(!regs_overlap(D, d1, S, s1));
        assert!(regs_overlap(D, d1, S, s2));
        assert!(!regs_overlap(D, d0, D, d1));
        assert!(regs_overlap(D, d1, D, d1));
    }
}
