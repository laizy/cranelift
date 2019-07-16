//! x86 register descriptions.

use crate::isa::registers::{RegBank, RegClass, RegClassData, RegInfo, RegUnit};

pub static INFO: RegInfo = RegInfo {
    banks: &[
        RegBank {
            name: "IntRegs",
            first_unit: 0,
            units: 16,
            names: &["rax", "rcx", "rdx", "rbx", "rsp", "rbp", "rsi", "rdi"],
            prefix: "r",
            first_toprc: 0,
            num_toprcs: 1,
            pressure_tracking: true,
        },
        RegBank {
            name: "FloatRegs",
            first_unit: 16,
            units: 16,
            names: &[],
            prefix: "xmm",
            first_toprc: 1,
            num_toprcs: 1,
            pressure_tracking: true,
        },
        RegBank {
            name: "FlagRegs",
            first_unit: 32,
            units: 1,
            names: &["rflags"],
            prefix: "",
            first_toprc: 2,
            num_toprcs: 1,
            pressure_tracking: false,
        },
    ],
    classes: &[
        &GPR_DATA, &FPR_DATA, &FLAG_DATA, &GPR8_DATA, &ABCD_DATA, &FPR8_DATA,
    ],
};
pub static GPR_DATA: RegClassData = RegClassData {
    name: "GPR",
    index: 0,
    width: 1,
    bank: 0,
    toprc: 0,
    first: 0,
    subclasses: 0x19,
    mask: [0x0000ffff, 0x00000000, 0x00000000],
    info: &INFO,
};
#[allow(dead_code)]
pub static GPR: RegClass = &GPR_DATA;
pub static FPR_DATA: RegClassData = RegClassData {
    name: "FPR",
    index: 1,
    width: 1,
    bank: 1,
    toprc: 1,
    first: 16,
    subclasses: 0x22,
    mask: [0xffff0000, 0x00000000, 0x00000000],
    info: &INFO,
};
#[allow(dead_code)]
pub static FPR: RegClass = &FPR_DATA;
pub static FLAG_DATA: RegClassData = RegClassData {
    name: "FLAG",
    index: 2,
    width: 1,
    bank: 2,
    toprc: 2,
    first: 32,
    subclasses: 0x4,
    mask: [0x00000000, 0x00000001, 0x00000000],
    info: &INFO,
};
#[allow(dead_code)]
pub static FLAG: RegClass = &FLAG_DATA;
pub static GPR8_DATA: RegClassData = RegClassData {
    name: "GPR8",
    index: 3,
    width: 1,
    bank: 0,
    toprc: 0,
    first: 0,
    subclasses: 0x18,
    mask: [0x000000ff, 0x00000000, 0x00000000],
    info: &INFO,
};
#[allow(dead_code)]
pub static GPR8: RegClass = &GPR8_DATA;
pub static ABCD_DATA: RegClassData = RegClassData {
    name: "ABCD",
    index: 4,
    width: 1,
    bank: 0,
    toprc: 0,
    first: 0,
    subclasses: 0x10,
    mask: [0x0000000f, 0x00000000, 0x00000000],
    info: &INFO,
};
#[allow(dead_code)]
pub static ABCD: RegClass = &ABCD_DATA;
pub static FPR8_DATA: RegClassData = RegClassData {
    name: "FPR8",
    index: 5,
    width: 1,
    bank: 1,
    toprc: 1,
    first: 16,
    subclasses: 0x20,
    mask: [0x00ff0000, 0x00000000, 0x00000000],
    info: &INFO,
};
#[allow(dead_code)]
pub static FPR8: RegClass = &FPR8_DATA;
#[allow(dead_code, non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum RU {
    rax = 0,
    rcx = 1,
    rdx = 2,
    rbx = 3,
    rsp = 4,
    rbp = 5,
    rsi = 6,
    rdi = 7,
    r8 = 8,
    r9 = 9,
    r10 = 10,
    r11 = 11,
    r12 = 12,
    r13 = 13,
    r14 = 14,
    r15 = 15,
    xmm0 = 16,
    xmm1 = 17,
    xmm2 = 18,
    xmm3 = 19,
    xmm4 = 20,
    xmm5 = 21,
    xmm6 = 22,
    xmm7 = 23,
    xmm8 = 24,
    xmm9 = 25,
    xmm10 = 26,
    xmm11 = 27,
    xmm12 = 28,
    xmm13 = 29,
    xmm14 = 30,
    xmm15 = 31,
    rflags = 32,
}
impl Into<RegUnit> for RU {
    fn into(self) -> RegUnit {
        self as RegUnit
    }
}

//clude!(concat!(env!("OUT_DIR"), "/registers-x86.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use crate::isa::RegUnit;
    use std::string::{String, ToString};

    #[test]
    fn unit_encodings() {
        // The encoding of integer registers is not alphabetical.
        assert_eq!(INFO.parse_regunit("rax"), Some(0));
        assert_eq!(INFO.parse_regunit("rbx"), Some(3));
        assert_eq!(INFO.parse_regunit("rcx"), Some(1));
        assert_eq!(INFO.parse_regunit("rdx"), Some(2));
        assert_eq!(INFO.parse_regunit("rsi"), Some(6));
        assert_eq!(INFO.parse_regunit("rdi"), Some(7));
        assert_eq!(INFO.parse_regunit("rbp"), Some(5));
        assert_eq!(INFO.parse_regunit("rsp"), Some(4));
        assert_eq!(INFO.parse_regunit("r8"), Some(8));
        assert_eq!(INFO.parse_regunit("r15"), Some(15));

        assert_eq!(INFO.parse_regunit("xmm0"), Some(16));
        assert_eq!(INFO.parse_regunit("xmm15"), Some(31));
    }

    #[test]
    fn unit_names() {
        fn uname(ru: RegUnit) -> String {
            INFO.display_regunit(ru).to_string()
        }

        assert_eq!(uname(0), "%rax");
        assert_eq!(uname(3), "%rbx");
        assert_eq!(uname(1), "%rcx");
        assert_eq!(uname(2), "%rdx");
        assert_eq!(uname(6), "%rsi");
        assert_eq!(uname(7), "%rdi");
        assert_eq!(uname(5), "%rbp");
        assert_eq!(uname(4), "%rsp");
        assert_eq!(uname(8), "%r8");
        assert_eq!(uname(15), "%r15");
        assert_eq!(uname(16), "%xmm0");
        assert_eq!(uname(31), "%xmm15");
    }

    #[test]
    fn regclasses() {
        assert_eq!(GPR.intersect_index(GPR), Some(GPR.into()));
        assert_eq!(GPR.intersect_index(ABCD), Some(ABCD.into()));
        assert_eq!(GPR.intersect_index(FPR), None);
        assert_eq!(ABCD.intersect_index(GPR), Some(ABCD.into()));
        assert_eq!(ABCD.intersect_index(ABCD), Some(ABCD.into()));
        assert_eq!(ABCD.intersect_index(FPR), None);
        assert_eq!(FPR.intersect_index(FPR), Some(FPR.into()));
        assert_eq!(FPR.intersect_index(GPR), None);
        assert_eq!(FPR.intersect_index(ABCD), None);
    }
}
