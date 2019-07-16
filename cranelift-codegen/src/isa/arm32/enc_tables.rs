//! Encoding tables for ARM32 ISA.

use crate::ir;
use crate::isa;
use crate::isa::constraints::*;
use crate::isa::enc_tables::*;
use crate::isa::encoding::RecipeSizing;

pub static RECIPE_PREDICATES: [RecipePredicate; 0] = [];
pub static INST_PREDICATES: [InstPredicate; 0] = [];
pub static ENCLISTS: [u16; 0] = [];
pub static LEVEL2: [Level2Entry<u16>; 0] = [];
pub static LEVEL1_A32: [Level1Entry<u16>; 2] = [
    Level1Entry {
        ty: ir::types::INVALID,
        log2len: 0,
        offset: !0 - 1,
        legalize: 0,
    }, // narrow
    Level1Entry {
        ty: ir::types::INVALID,
        log2len: !0,
        offset: 0,
        legalize: 0,
    },
];
pub static LEVEL1_T32: [Level1Entry<u16>; 2] = [
    Level1Entry {
        ty: ir::types::INVALID,
        log2len: 0,
        offset: !0 - 1,
        legalize: 0,
    }, // narrow
    Level1Entry {
        ty: ir::types::INVALID,
        log2len: !0,
        offset: 0,
        legalize: 0,
    },
];
static RECIPE_NAMES: [&str; 0] = [];
static RECIPE_CONSTRAINTS: [RecipeConstraints; 0] = [];
static RECIPE_SIZING: [RecipeSizing; 0] = [];
pub static INFO: isa::EncInfo = isa::EncInfo {
    constraints: &RECIPE_CONSTRAINTS,
    sizing: &RECIPE_SIZING,
    names: &RECIPE_NAMES,
};

//clude!(concat!(env!("OUT_DIR"), "/encoding-arm32.rs"));

pub static LEGALIZE_ACTIONS: [isa::Legalize; 1] = [crate::legalizer::narrow];

//clude!(concat!(env!("OUT_DIR"), "/legalize-arm32.rs"));
