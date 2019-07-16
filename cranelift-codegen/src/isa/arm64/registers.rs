//! ARM64 register descriptions.

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
            prefix: "v",
            first_toprc: 1,
            num_toprcs: 1,
            pressure_tracking: true,
        },
        RegBank {
            name: "FlagRegs",
            first_unit: 64,
            units: 1,
            names: &["nzcv"],
            prefix: "",
            first_toprc: 2,
            num_toprcs: 1,
            pressure_tracking: false,
        },
    ],
    classes: &[&GPR_DATA, &FPR_DATA, &FLAG_DATA],
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
    first: 64,
    subclasses: 0x4,
    mask: [0x00000000, 0x00000000, 0x00000001],
    info: &INFO,
};
#[allow(dead_code)]
pub static FLAG: RegClass = &FLAG_DATA;
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
    v0 = 32,
    v1 = 33,
    v2 = 34,
    v3 = 35,
    v4 = 36,
    v5 = 37,
    v6 = 38,
    v7 = 39,
    v8 = 40,
    v9 = 41,
    v10 = 42,
    v11 = 43,
    v12 = 44,
    v13 = 45,
    v14 = 46,
    v15 = 47,
    v16 = 48,
    v17 = 49,
    v18 = 50,
    v19 = 51,
    v20 = 52,
    v21 = 53,
    v22 = 54,
    v23 = 55,
    v24 = 56,
    v25 = 57,
    v26 = 58,
    v27 = 59,
    v28 = 60,
    v29 = 61,
    v30 = 62,
    v31 = 63,
    nzcv = 64,
}
impl Into<RegUnit> for RU {
    fn into(self) -> RegUnit {
        self as RegUnit
    }
}

//clude!(concat!(env!("OUT_DIR"), "/registers-arm64.rs"));

#[cfg(test)]
mod tests {
    use super::INFO;
    use crate::isa::RegUnit;
    use std::string::{String, ToString};

    #[test]
    fn unit_encodings() {
        assert_eq!(INFO.parse_regunit("x0"), Some(0));
        assert_eq!(INFO.parse_regunit("x31"), Some(31));
        assert_eq!(INFO.parse_regunit("v0"), Some(32));
        assert_eq!(INFO.parse_regunit("v31"), Some(63));

        assert_eq!(INFO.parse_regunit("x32"), None);
        assert_eq!(INFO.parse_regunit("v32"), None);
    }

    #[test]
    fn unit_names() {
        fn uname(ru: RegUnit) -> String {
            INFO.display_regunit(ru).to_string()
        }

        assert_eq!(uname(0), "%x0");
        assert_eq!(uname(1), "%x1");
        assert_eq!(uname(31), "%x31");
        assert_eq!(uname(32), "%v0");
        assert_eq!(uname(33), "%v1");
        assert_eq!(uname(63), "%v31");
        assert_eq!(uname(64), "%nzcv");
        assert_eq!(uname(65), "%INVALID65");
    }
}
