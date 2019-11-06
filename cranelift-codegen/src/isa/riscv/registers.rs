//! RISC-V register descriptions.

use crate::isa::registers::{RegBank, RegClass, RegClassData, RegInfo, RegUnit};

 
pub static INFO: RegInfo = RegInfo {
    banks: &[
        RegBank {
            name: "IntRegs",
            first_unit: 0,
            units: 32,
            names: &[],
            prefix: "x",
            first_toprc: 0,
            num_toprcs: 1,
            pressure_tracking: true,
        },
        RegBank {
            name: "FloatRegs",
            first_unit: 32,
            units: 32,
            names: &[],
            prefix: "f",
            first_toprc: 1,
            num_toprcs: 1,
            pressure_tracking: true,
        },
    ],
    classes: &[
        &GPR_DATA,
        &FPR_DATA,
    ],
};
pub static GPR_DATA: RegClassData = RegClassData {
    name: "GPR",
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
pub static GPR: RegClass = &GPR_DATA;
pub static FPR_DATA: RegClassData = RegClassData {
    name: "FPR",
    index: 1,
    width: 1,
    bank: 1,
    toprc: 1,
    first: 32,
    subclasses: 0x2,
    mask: [0x00000000, 0xffffffff, 0x00000000],
    pinned_reg: None,
    info: &INFO,
};
#[allow(dead_code)]
pub static FPR: RegClass = &FPR_DATA;
#[allow(dead_code, non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum RU {
    x0 = 0,
    x1 = 1,
    x2 = 2,
    x3 = 3,
    x4 = 4,
    x5 = 5,
    x6 = 6,
    x7 = 7,
    x8 = 8,
    x9 = 9,
    x10 = 10,
    x11 = 11,
    x12 = 12,
    x13 = 13,
    x14 = 14,
    x15 = 15,
    x16 = 16,
    x17 = 17,
    x18 = 18,
    x19 = 19,
    x20 = 20,
    x21 = 21,
    x22 = 22,
    x23 = 23,
    x24 = 24,
    x25 = 25,
    x26 = 26,
    x27 = 27,
    x28 = 28,
    x29 = 29,
    x30 = 30,
    x31 = 31,
    f0 = 32,
    f1 = 33,
    f2 = 34,
    f3 = 35,
    f4 = 36,
    f5 = 37,
    f6 = 38,
    f7 = 39,
    f8 = 40,
    f9 = 41,
    f10 = 42,
    f11 = 43,
    f12 = 44,
    f13 = 45,
    f14 = 46,
    f15 = 47,
    f16 = 48,
    f17 = 49,
    f18 = 50,
    f19 = 51,
    f20 = 52,
    f21 = 53,
    f22 = 54,
    f23 = 55,
    f24 = 56,
    f25 = 57,
    f26 = 58,
    f27 = 59,
    f28 = 60,
    f29 = 61,
    f30 = 62,
    f31 = 63,
}
impl Into<RegUnit> for RU {
    fn into(self) -> RegUnit {
        self as RegUnit
    }
}

 //clude!(concat!(env!("OUT_DIR"), "/registers-riscv.rs"));

#[cfg(test)]
mod tests {
    use super::{FPR, GPR, INFO};
    use crate::isa::RegUnit;
    use alloc::string::{String, ToString};

    #[test]
    fn unit_encodings() {
        assert_eq!(INFO.parse_regunit("x0"), Some(0));
        assert_eq!(INFO.parse_regunit("x31"), Some(31));
        assert_eq!(INFO.parse_regunit("f0"), Some(32));
        assert_eq!(INFO.parse_regunit("f31"), Some(63));

        assert_eq!(INFO.parse_regunit("x32"), None);
        assert_eq!(INFO.parse_regunit("f32"), None);
    }

    #[test]
    fn unit_names() {
        fn uname(ru: RegUnit) -> String {
            INFO.display_regunit(ru).to_string()
        }

        assert_eq!(uname(0), "%x0");
        assert_eq!(uname(1), "%x1");
        assert_eq!(uname(31), "%x31");
        assert_eq!(uname(32), "%f0");
        assert_eq!(uname(33), "%f1");
        assert_eq!(uname(63), "%f31");
        assert_eq!(uname(64), "%INVALID64");
    }

    #[test]
    fn classes() {
        assert!(GPR.contains(GPR.unit(0)));
        assert!(GPR.contains(GPR.unit(31)));
        assert!(!FPR.contains(GPR.unit(0)));
        assert!(!FPR.contains(GPR.unit(31)));
        assert!(!GPR.contains(FPR.unit(0)));
        assert!(!GPR.contains(FPR.unit(31)));
        assert!(FPR.contains(FPR.unit(0)));
        assert!(FPR.contains(FPR.unit(31)));
    }
}
