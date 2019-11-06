//! Shared settings module.
//!
//! This module defines data structures to access the settings defined in the meta language.
//!
//! Each settings group is translated to a `Flags` struct either in this module or in its
//! ISA-specific `settings` module. The struct provides individual getter methods for all of the
//! settings as well as computed predicate flags.
//!
//! The `Flags` struct is immutable once it has been created. A `Builder` instance is used to
//! create it.
//!
//! # Example
//! ```
//! use cranelift_codegen::settings::{self, Configurable};
//!
//! let mut b = settings::builder();
//! b.set("opt_level", "speed_and_size");
//!
//! let f = settings::Flags::new(b);
//! assert_eq!(f.opt_level(), settings::OptLevel::SpeedAndSize);
//! ```

use crate::constant_hash::{probe, simple_hash};
use crate::isa::TargetIsa;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use core::fmt;
use core::str;
use thiserror::Error;

/// A string-based configurator for settings groups.
///
/// The `Configurable` protocol allows settings to be modified by name before a finished `Flags`
/// struct is created.
pub trait Configurable {
    /// Set the string value of any setting by name.
    ///
    /// This can set any type of setting whether it is numeric, boolean, or enumerated.
    fn set(&mut self, name: &str, value: &str) -> SetResult<()>;

    /// Enable a boolean setting or apply a preset.
    ///
    /// If the identified setting isn't a boolean or a preset, a `BadType` error is returned.
    fn enable(&mut self, name: &str) -> SetResult<()>;
}

/// Collect settings values based on a template.
#[derive(Clone)]
pub struct Builder {
    template: &'static detail::Template,
    bytes: Box<[u8]>,
}

impl Builder {
    /// Create a new builder with defaults and names from the given template.
    pub fn new(tmpl: &'static detail::Template) -> Self {
        Self {
            template: tmpl,
            bytes: tmpl.defaults.into(),
        }
    }

    /// Extract contents of builder once everything is configured.
    pub fn state_for(self, name: &str) -> Box<[u8]> {
        assert_eq!(name, self.template.name);
        self.bytes
    }

    /// Set the value of a single bit.
    fn set_bit(&mut self, offset: usize, bit: u8, value: bool) {
        let byte = &mut self.bytes[offset];
        let mask = 1 << bit;
        if value {
            *byte |= mask;
        } else {
            *byte &= !mask;
        }
    }

    /// Apply a preset. The argument is a slice of (mask, value) bytes.
    fn apply_preset(&mut self, values: &[(u8, u8)]) {
        for (byte, &(mask, value)) in self.bytes.iter_mut().zip(values) {
            *byte = (*byte & !mask) | value;
        }
    }

    /// Look up a descriptor by name.
    fn lookup(&self, name: &str) -> SetResult<(usize, detail::Detail)> {
        match probe(self.template, name, simple_hash(name)) {
            Err(_) => Err(SetError::BadName(name.to_string())),
            Ok(entry) => {
                let d = &self.template.descriptors[self.template.hash_table[entry] as usize];
                Ok((d.offset as usize, d.detail))
            }
        }
    }
}

fn parse_bool_value(value: &str) -> SetResult<bool> {
    match value {
        "true" | "on" | "yes" | "1" => Ok(true),
        "false" | "off" | "no" | "0" => Ok(false),
        _ => Err(SetError::BadValue("bool".to_string())),
    }
}

fn parse_enum_value(value: &str, choices: &[&str]) -> SetResult<u8> {
    match choices.iter().position(|&tag| tag == value) {
        Some(idx) => Ok(idx as u8),
        None => {
            // TODO: Use `join` instead of this code, once
            // https://github.com/rust-lang/rust/issues/27747 is resolved.
            let mut all_choices = String::new();
            let mut first = true;
            for choice in choices {
                if first {
                    first = false
                } else {
                    all_choices += ", ";
                }
                all_choices += choice;
            }
            Err(SetError::BadValue(format!("any among {}", all_choices)))
        }
    }
}

impl Configurable for Builder {
    fn enable(&mut self, name: &str) -> SetResult<()> {
        use self::detail::Detail;
        let (offset, detail) = self.lookup(name)?;
        match detail {
            Detail::Bool { bit } => {
                self.set_bit(offset, bit, true);
                Ok(())
            }
            Detail::Preset => {
                self.apply_preset(&self.template.presets[offset..]);
                Ok(())
            }
            _ => Err(SetError::BadType),
        }
    }

    fn set(&mut self, name: &str, value: &str) -> SetResult<()> {
        use self::detail::Detail;
        let (offset, detail) = self.lookup(name)?;
        match detail {
            Detail::Bool { bit } => {
                self.set_bit(offset, bit, parse_bool_value(value)?);
            }
            Detail::Num => {
                self.bytes[offset] = value
                    .parse()
                    .map_err(|_| SetError::BadValue("number".to_string()))?;
            }
            Detail::Enum { last, enumerators } => {
                self.bytes[offset] =
                    parse_enum_value(value, self.template.enums(last, enumerators))?;
            }
            Detail::Preset => return Err(SetError::BadName(name.to_string())),
        }
        Ok(())
    }
}

/// An error produced when changing a setting.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum SetError {
    /// No setting by this name exists.
    #[error("No existing setting named '{0}'")]
    BadName(String),

    /// Type mismatch for setting (e.g., setting an enum setting as a bool).
    #[error("Trying to set a setting with the wrong type")]
    BadType,

    /// This is not a valid value for this setting.
    #[error("Unexpected value for a setting, expected {0}")]
    BadValue(String),
}

/// A result returned when changing a setting.
pub type SetResult<T> = Result<T, SetError>;

/// A reference to just the boolean predicates of a settings object.
///
/// The settings objects themselves are generated and appear in the `isa/*/settings.rs` modules.
/// Each settings object provides a `predicate_view()` method that makes it possible to query
/// ISA predicates by number.
#[derive(Clone, Copy)]
pub struct PredicateView<'a>(&'a [u8]);

impl<'a> PredicateView<'a> {
    /// Create a new view of a precomputed predicate vector.
    ///
    /// See the `predicate_view()` method on the various `Flags` types defined for each ISA.
    pub fn new(bits: &'a [u8]) -> Self {
        PredicateView(bits)
    }

    /// Check a numbered predicate.
    pub fn test(self, p: usize) -> bool {
        self.0[p / 8] & (1 << (p % 8)) != 0
    }
}

/// Implementation details for generated code.
///
/// This module holds definitions that need to be public so the can be instantiated by generated
/// code in other modules.
pub mod detail {
    use crate::constant_hash;
    use core::fmt;

    /// An instruction group template.
    pub struct Template {
        /// Name of the instruction group.
        pub name: &'static str,
        /// List of setting descriptors.
        pub descriptors: &'static [Descriptor],
        /// Union of all enumerators.
        pub enumerators: &'static [&'static str],
        /// Hash table of settings.
        pub hash_table: &'static [u16],
        /// Default values.
        pub defaults: &'static [u8],
        /// Pairs of (mask, value) for presets.
        pub presets: &'static [(u8, u8)],
    }

    impl Template {
        /// Get enumerators corresponding to a `Details::Enum`.
        pub fn enums(&self, last: u8, enumerators: u16) -> &[&'static str] {
            let from = enumerators as usize;
            let len = usize::from(last) + 1;
            &self.enumerators[from..from + len]
        }

        /// Format a setting value as a TOML string. This is mostly for use by the generated
        /// `Display` implementation.
        pub fn format_toml_value(
            &self,
            detail: Detail,
            byte: u8,
            f: &mut fmt::Formatter,
        ) -> fmt::Result {
            match detail {
                Detail::Bool { bit } => write!(f, "{}", (byte & (1 << bit)) != 0),
                Detail::Num => write!(f, "{}", byte),
                Detail::Enum { last, enumerators } => {
                    if byte <= last {
                        let tags = self.enums(last, enumerators);
                        write!(f, "\"{}\"", tags[usize::from(byte)])
                    } else {
                        write!(f, "{}", byte)
                    }
                }
                // Presets aren't printed. They are reflected in the other settings.
                Detail::Preset { .. } => Ok(()),
            }
        }
    }

    /// The template contains a hash table for by-name lookup.
    impl<'a> constant_hash::Table<&'a str> for Template {
        fn len(&self) -> usize {
            self.hash_table.len()
        }

        fn key(&self, idx: usize) -> Option<&'a str> {
            let e = self.hash_table[idx] as usize;
            if e < self.descriptors.len() {
                Some(self.descriptors[e].name)
            } else {
                None
            }
        }
    }

    /// A setting descriptor holds the information needed to generically set and print a setting.
    ///
    /// Each settings group will be represented as a constant DESCRIPTORS array.
    pub struct Descriptor {
        /// Lower snake-case name of setting as defined in meta.
        pub name: &'static str,

        /// Offset of byte containing this setting.
        pub offset: u32,

        /// Additional details, depending on the kind of setting.
        pub detail: Detail,
    }

    /// The different kind of settings along with descriptor bits that depend on the kind.
    #[derive(Clone, Copy)]
    pub enum Detail {
        /// A boolean setting only uses one bit, numbered from LSB.
        Bool {
            /// 0-7.
            bit: u8,
        },

        /// A numerical setting uses the whole byte.
        Num,

        /// An Enum setting uses a range of enumerators.
        Enum {
            /// Numerical value of last enumerator, allowing for 1-256 enumerators.
            last: u8,

            /// First enumerator in the ENUMERATORS table.
            enumerators: u16,
        },

        /// A preset is not an individual setting, it is a collection of settings applied at once.
        ///
        /// The `Descriptor::offset` field refers to the `PRESETS` table.
        Preset,
    }

    impl Detail {
        /// Check if a detail is a Detail::Preset. Useful because the Descriptor
        /// offset field has a different meaning when the detail is a preset.
        pub fn is_preset(self) -> bool {
            match self {
                Self::Preset => true,
                _ => false,
            }
        }
    }
}

// Include code generated by `meta/gen_settings.rs`. This file contains a public `Flags` struct
// with an implementation for all of the settings defined in
// `cranelift-codegen/meta/src/shared/settings.rs`.
 
#[derive(Clone)]
/// Flags group `shared`.
pub struct Flags {
    bytes: [u8; 6],
}
impl Flags {
    /// Create flags shared settings group.
    #[allow(unused_variables)]
    pub fn new(builder: Builder) -> Self {
        let bvec = builder.state_for("shared");
        let mut shared = Self { bytes: [0; 6] };
        debug_assert_eq!(bvec.len(), 6);
        shared.bytes[0..6].copy_from_slice(&bvec);
        shared
    }
}
/// Values for `shared.opt_level`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum OptLevel {
    /// `none`.
    None,
    /// `speed`.
    Speed,
    /// `speed_and_size`.
    SpeedAndSize,
}
impl fmt::Display for OptLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            Self::None => "none",
            Self::Speed => "speed",
            Self::SpeedAndSize => "speed_and_size",
        })
    }
}
impl str::FromStr for OptLevel {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Self::None),
            "speed" => Ok(Self::Speed),
            "speed_and_size" => Ok(Self::SpeedAndSize),
            _ => Err(()),
        }
    }
}
/// Values for `shared.libcall_call_conv`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum LibcallCallConv {
    /// `isa_default`.
    IsaDefault,
    /// `fast`.
    Fast,
    /// `cold`.
    Cold,
    /// `system_v`.
    SystemV,
    /// `windows_fastcall`.
    WindowsFastcall,
    /// `baldrdash_system_v`.
    BaldrdashSystemV,
    /// `baldrdash_windows`.
    BaldrdashWindows,
    /// `probestack`.
    Probestack,
}
impl fmt::Display for LibcallCallConv {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            Self::IsaDefault => "isa_default",
            Self::Fast => "fast",
            Self::Cold => "cold",
            Self::SystemV => "system_v",
            Self::WindowsFastcall => "windows_fastcall",
            Self::BaldrdashSystemV => "baldrdash_system_v",
            Self::BaldrdashWindows => "baldrdash_windows",
            Self::Probestack => "probestack",
        })
    }
}
impl str::FromStr for LibcallCallConv {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "isa_default" => Ok(Self::IsaDefault),
            "fast" => Ok(Self::Fast),
            "cold" => Ok(Self::Cold),
            "system_v" => Ok(Self::SystemV),
            "windows_fastcall" => Ok(Self::WindowsFastcall),
            "baldrdash_system_v" => Ok(Self::BaldrdashSystemV),
            "baldrdash_windows" => Ok(Self::BaldrdashWindows),
            "probestack" => Ok(Self::Probestack),
            _ => Err(()),
        }
    }
}
/// User-defined settings.
#[allow(dead_code)]
impl Flags {
    /// Get a view of the boolean predicates.
    pub fn predicate_view(&self) -> crate::settings::PredicateView {
        crate::settings::PredicateView::new(&self.bytes[4..])
    }
    /// Dynamic numbered predicate getter.
    fn numbered_predicate(&self, p: usize) -> bool {
        self.bytes[4 + p / 8] & (1 << (p % 8)) != 0
    }
    /// Optimization level:
    ///
    /// - none: Minimise compile time by disabling most optimizations.
    /// - speed: Generate the fastest possible code
    /// - speed_and_size: like "speed", but also perform transformations
    ///   aimed at reducing code size.
    pub fn opt_level(&self) -> OptLevel {
        match self.bytes[0] {
            0 => {
                OptLevel::None
            }
            1 => {
                OptLevel::Speed
            }
            2 => {
                OptLevel::SpeedAndSize
            }
            _ => {
                panic!("Invalid enum value")
            }
        }
    }
    /// Defines the calling convention to use for LibCalls call expansion,
    /// since it may be different from the ISA default calling convention.
    ///
    /// The default value is to use the same calling convention as the ISA
    /// default calling convention.
    ///
    /// This list should be kept in sync with the list of calling
    /// conventions available in isa/call_conv.rs.
    pub fn libcall_call_conv(&self) -> LibcallCallConv {
        match self.bytes[1] {
            5 => {
                LibcallCallConv::BaldrdashSystemV
            }
            6 => {
                LibcallCallConv::BaldrdashWindows
            }
            2 => {
                LibcallCallConv::Cold
            }
            1 => {
                LibcallCallConv::Fast
            }
            0 => {
                LibcallCallConv::IsaDefault
            }
            7 => {
                LibcallCallConv::Probestack
            }
            3 => {
                LibcallCallConv::SystemV
            }
            4 => {
                LibcallCallConv::WindowsFastcall
            }
            _ => {
                panic!("Invalid enum value")
            }
        }
    }
    /// Number of pointer-sized words pushed by the baldrdash prologue.
    ///
    /// Functions with the `baldrdash` calling convention don't generate their
    /// own prologue and epilogue. They depend on externally generated code
    /// that pushes a fixed number of words in the prologue and restores them
    /// in the epilogue.
    ///
    /// This setting configures the number of pointer-sized words pushed on the
    /// stack when the Cranelift-generated code is entered. This includes the
    /// pushed return address on x86.
    pub fn baldrdash_prologue_words(&self) -> u8 {
        self.bytes[2]
    }
    /// The log2 of the size of the stack guard region.
    ///
    /// Stack frames larger than this size will have stack overflow checked
    /// by calling the probestack function.
    ///
    /// The default is 12, which translates to a size of 4096.
    pub fn probestack_size_log2(&self) -> u8 {
        self.bytes[3]
    }
    /// Run the Cranelift IR verifier at strategic times during compilation.
    ///
    /// This makes compilation slower but catches many bugs. The verifier is
    /// disabled by default, except when reading Cranelift IR from a text file.
    pub fn enable_verifier(&self) -> bool {
        self.numbered_predicate(0)
    }
    /// Enable Position-Independent Code generation
    pub fn is_pic(&self) -> bool {
        self.numbered_predicate(1)
    }
    /// Use colocated libcalls.
    ///
    /// Generate code that assumes that libcalls can be declared "colocated",
    /// meaning they will be defined along with the current function, such that
    /// they can use more efficient addressing.
    pub fn colocated_libcalls(&self) -> bool {
        self.numbered_predicate(2)
    }
    /// Generate explicit checks around native division instructions to avoid
    /// their trapping.
    ///
    /// This is primarily used by SpiderMonkey which doesn't install a signal
    /// handler for SIGFPE, but expects a SIGILL trap for division by zero.
    ///
    /// On ISAs like ARM where the native division instructions don't trap,
    /// this setting has no effect - explicit checks are always inserted.
    pub fn avoid_div_traps(&self) -> bool {
        self.numbered_predicate(3)
    }
    /// Enable the use of floating-point instructions
    ///
    /// Disabling use of floating-point instructions is not yet implemented.
    pub fn enable_float(&self) -> bool {
        self.numbered_predicate(4)
    }
    /// Enable NaN canonicalization
    ///
    /// This replaces NaNs with a single canonical value, for users requiring
    /// entirely deterministic WebAssembly computation. This is not required
    /// by the WebAssembly spec, so it is not enabled by default.
    pub fn enable_nan_canonicalization(&self) -> bool {
        self.numbered_predicate(5)
    }
    /// Enable the use of the pinned register.
    ///
    /// This register is excluded from register allocation, and is completely under the control of
    /// the end-user. It is possible to read it via the get_pinned_reg instruction, and to set it
    /// with the set_pinned_reg instruction.
    pub fn enable_pinned_reg(&self) -> bool {
        self.numbered_predicate(6)
    }
    /// Use the pinned register as the heap base.
    ///
    /// Enabling this requires the enable_pinned_reg setting to be set to true. It enables a custom
    /// legalization of the `heap_addr` instruction so it will use the pinned register as the heap
    /// base, instead of fetching it from a global value.
    ///
    /// Warning! Enabling this means that the pinned register *must* be maintained to contain the
    /// heap base address at all times, during the lifetime of a function. Using the pinned
    /// register for other purposes when this is set is very likely to cause crashes.
    pub fn use_pinned_reg_as_heap_base(&self) -> bool {
        self.numbered_predicate(7)
    }
    /// Enable the use of SIMD instructions.
    pub fn enable_simd(&self) -> bool {
        self.numbered_predicate(8)
    }
    /// Enable the use of atomic instructions
    pub fn enable_atomics(&self) -> bool {
        self.numbered_predicate(9)
    }
    /// Enable safepoint instruction insertions.
    ///
    /// This will allow the emit_stackmaps() function to insert the safepoint
    /// instruction on top of calls and interrupt traps in order to display the
    /// live reference values at that point in the program.
    pub fn enable_safepoints(&self) -> bool {
        self.numbered_predicate(10)
    }
    /// Emit not-yet-relocated function addresses as all-ones bit patterns.
    pub fn allones_funcaddrs(&self) -> bool {
        self.numbered_predicate(11)
    }
    /// Enable the use of stack probes, for calling conventions which support this
    /// functionality.
    pub fn probestack_enabled(&self) -> bool {
        self.numbered_predicate(12)
    }
    /// Set this to true of the stack probe function modifies the stack pointer
    /// itself.
    pub fn probestack_func_adjusts_sp(&self) -> bool {
        self.numbered_predicate(13)
    }
    /// Enable the use of jump tables in generated machine code.
    pub fn jump_tables_enabled(&self) -> bool {
        self.numbered_predicate(14)
    }
}
static DESCRIPTORS: [detail::Descriptor; 19] = [
    detail::Descriptor {
        name: "opt_level",
        offset: 0,
        detail: detail::Detail::Enum { last: 2, enumerators: 0 },
    },
    detail::Descriptor {
        name: "libcall_call_conv",
        offset: 1,
        detail: detail::Detail::Enum { last: 7, enumerators: 3 },
    },
    detail::Descriptor {
        name: "baldrdash_prologue_words",
        offset: 2,
        detail: detail::Detail::Num,
    },
    detail::Descriptor {
        name: "probestack_size_log2",
        offset: 3,
        detail: detail::Detail::Num,
    },
    detail::Descriptor {
        name: "enable_verifier",
        offset: 4,
        detail: detail::Detail::Bool { bit: 0 },
    },
    detail::Descriptor {
        name: "is_pic",
        offset: 4,
        detail: detail::Detail::Bool { bit: 1 },
    },
    detail::Descriptor {
        name: "colocated_libcalls",
        offset: 4,
        detail: detail::Detail::Bool { bit: 2 },
    },
    detail::Descriptor {
        name: "avoid_div_traps",
        offset: 4,
        detail: detail::Detail::Bool { bit: 3 },
    },
    detail::Descriptor {
        name: "enable_float",
        offset: 4,
        detail: detail::Detail::Bool { bit: 4 },
    },
    detail::Descriptor {
        name: "enable_nan_canonicalization",
        offset: 4,
        detail: detail::Detail::Bool { bit: 5 },
    },
    detail::Descriptor {
        name: "enable_pinned_reg",
        offset: 4,
        detail: detail::Detail::Bool { bit: 6 },
    },
    detail::Descriptor {
        name: "use_pinned_reg_as_heap_base",
        offset: 4,
        detail: detail::Detail::Bool { bit: 7 },
    },
    detail::Descriptor {
        name: "enable_simd",
        offset: 5,
        detail: detail::Detail::Bool { bit: 0 },
    },
    detail::Descriptor {
        name: "enable_atomics",
        offset: 5,
        detail: detail::Detail::Bool { bit: 1 },
    },
    detail::Descriptor {
        name: "enable_safepoints",
        offset: 5,
        detail: detail::Detail::Bool { bit: 2 },
    },
    detail::Descriptor {
        name: "allones_funcaddrs",
        offset: 5,
        detail: detail::Detail::Bool { bit: 3 },
    },
    detail::Descriptor {
        name: "probestack_enabled",
        offset: 5,
        detail: detail::Detail::Bool { bit: 4 },
    },
    detail::Descriptor {
        name: "probestack_func_adjusts_sp",
        offset: 5,
        detail: detail::Detail::Bool { bit: 5 },
    },
    detail::Descriptor {
        name: "jump_tables_enabled",
        offset: 5,
        detail: detail::Detail::Bool { bit: 6 },
    },
];
static ENUMERATORS: [&str; 11] = [
    "none",
    "speed",
    "speed_and_size",
    "isa_default",
    "fast",
    "cold",
    "system_v",
    "windows_fastcall",
    "baldrdash_system_v",
    "baldrdash_windows",
    "probestack",
];
static HASH_TABLE: [u16; 32] = [
    0xffff,
    15,
    0xffff,
    9,
    0xffff,
    4,
    13,
    11,
    12,
    17,
    0xffff,
    18,
    3,
    0xffff,
    14,
    0xffff,
    10,
    0xffff,
    0xffff,
    6,
    7,
    16,
    0xffff,
    1,
    0xffff,
    0xffff,
    0xffff,
    0xffff,
    0,
    2,
    8,
    5,
];
static PRESETS: [(u8, u8); 0] = [
];
static TEMPLATE: detail::Template = detail::Template {
    name: "shared",
    descriptors: &DESCRIPTORS,
    enumerators: &ENUMERATORS,
    hash_table: &HASH_TABLE,
    defaults: &[0x00, 0x00, 0x00, 0x0c, 0x11, 0x52],
    presets: &PRESETS,
};
/// Create a `settings::Builder` for the shared settings group.
pub fn builder() -> Builder {
    Builder::new(&TEMPLATE)
}
impl fmt::Display for Flags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "[shared]")?;
        for d in &DESCRIPTORS {
            if !d.detail.is_preset() {
                write!(f, "{} = ", d.name)?;
                TEMPLATE.format_toml_value(d.detail, self.bytes[d.offset as usize], f)?;
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

 //clude!(concat!(env!("OUT_DIR"), "/settings.rs"));

/// Wrapper containing flags and optionally a `TargetIsa` trait object.
///
/// A few passes need to access the flags but only optionally a target ISA. The `FlagsOrIsa`
/// wrapper can be used to pass either, and extract the flags so they are always accessible.
#[derive(Clone, Copy)]
pub struct FlagsOrIsa<'a> {
    /// Flags are always present.
    pub flags: &'a Flags,

    /// The ISA may not be present.
    pub isa: Option<&'a dyn TargetIsa>,
}

impl<'a> From<&'a Flags> for FlagsOrIsa<'a> {
    fn from(flags: &'a Flags) -> FlagsOrIsa {
        FlagsOrIsa { flags, isa: None }
    }
}

impl<'a> From<&'a dyn TargetIsa> for FlagsOrIsa<'a> {
    fn from(isa: &'a dyn TargetIsa) -> FlagsOrIsa {
        FlagsOrIsa {
            flags: isa.flags(),
            isa: Some(isa),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Configurable;
    use super::SetError::*;
    use super::{builder, Flags};
    use alloc::string::ToString;

    #[test]
    fn display_default() {
        let b = builder();
        let f = Flags::new(b);
        assert_eq!(
            f.to_string(),
            "[shared]\n\
             opt_level = \"none\"\n\
             libcall_call_conv = \"isa_default\"\n\
             baldrdash_prologue_words = 0\n\
             probestack_size_log2 = 12\n\
             enable_verifier = true\n\
             is_pic = false\n\
             colocated_libcalls = false\n\
             avoid_div_traps = false\n\
             enable_float = true\n\
             enable_nan_canonicalization = false\n\
             enable_pinned_reg = false\n\
             use_pinned_reg_as_heap_base = false\n\
             enable_simd = false\n\
             enable_atomics = true\n\
             enable_safepoints = false\n\
             allones_funcaddrs = false\n\
             probestack_enabled = true\n\
             probestack_func_adjusts_sp = false\n\
             jump_tables_enabled = true\n"
        );
        assert_eq!(f.opt_level(), super::OptLevel::None);
        assert_eq!(f.enable_simd(), false);
        assert_eq!(f.baldrdash_prologue_words(), 0);
    }

    #[test]
    fn modify_bool() {
        let mut b = builder();
        assert_eq!(b.enable("not_there"), Err(BadName("not_there".to_string())));
        assert_eq!(b.enable("enable_simd"), Ok(()));
        assert_eq!(b.set("enable_simd", "false"), Ok(()));

        let f = Flags::new(b);
        assert_eq!(f.enable_simd(), false);
    }

    #[test]
    fn modify_string() {
        let mut b = builder();
        assert_eq!(
            b.set("not_there", "true"),
            Err(BadName("not_there".to_string()))
        );
        assert_eq!(b.set("enable_simd", ""), Err(BadValue("bool".to_string())));
        assert_eq!(
            b.set("enable_simd", "best"),
            Err(BadValue("bool".to_string()))
        );
        assert_eq!(
            b.set("opt_level", "true"),
            Err(BadValue(
                "any among none, speed, speed_and_size".to_string()
            ))
        );
        assert_eq!(b.set("opt_level", "speed"), Ok(()));
        assert_eq!(b.set("enable_simd", "0"), Ok(()));

        let f = Flags::new(b);
        assert_eq!(f.enable_simd(), false);
        assert_eq!(f.opt_level(), super::OptLevel::Speed);
    }
}
