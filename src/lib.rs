//! # Asar Rust Bindings
//! This crate provides a safe wrapper around the [Asar](<https://github.com/RPGHacker/asar>) library version 1.91.
//!
//! By default this crate is not thread-safe.
//!
//! In case it is needed to use this crate in a multithreaded environment, the `thread-safe` feature should be enabled, doing so will make all the functions use a global lock to ensure that the Asar API is called in a thread-safe manner.
pub(crate) mod bindings {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[cfg(test)]
mod test;

extern crate asar_snes_proc_macros;
pub use asar_snes_proc_macros::use_asar_global_lock;

use core::fmt;
#[cfg(feature = "thread-safe")]
use parking_lot::ReentrantMutex;
#[cfg(feature = "thread-safe")]
use std::sync::OnceLock;

use std::{
    ffi::{CStr, CString},
    os::raw::{c_char, c_int, c_void},
    ptr,
};

use crate::bindings::{
    asar_apiversion, asar_getalldefines, asar_getalllabels, asar_getdefine, asar_geterrors,
    asar_getlabelval, asar_getmapper, asar_getprints, asar_getsymbolsfile, asar_getwarnings,
    asar_getwrittenblocks, asar_math, asar_maxromsize, asar_patch, asar_patch_ex, asar_reset,
    asar_resolvedefines, asar_version, definedata, errordata, labeldata, mappertype, memoryfile,
    patchparams, warnsetting, writtenblockdata,
};

#[cfg(feature = "thread-safe")]
fn global_asar_lock() -> &'static ReentrantMutex<()> {
    static LOCK: OnceLock<ReentrantMutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| ReentrantMutex::new(()))
}

#[cfg(not(feature = "thread-safe"))]
fn global_asar_lock() -> &'static FakeLock {
    &FakeLock
}

#[cfg(not(feature = "thread-safe"))]
struct FakeLock;

#[cfg(not(feature = "thread-safe"))]
impl FakeLock {
    fn lock(&self) -> FakeLock {
        FakeLock
    }
}

/// Executes the closure with the Asar global lock.
///
/// This lock is recursive, so it can be used in nested calls without issues.
///
/// This is necessary to ensure that Asar's API is called in a thread-safe manner.
///
/// It is recommended to use this function in multithreaded environments, because Asar uses a lot of global state.
///
/// e.g. these 2 calls would be unsafe without the lock because patch stores defines, labels in global state.
/// ```rust
/// use asar_snes as asar;
/// use asar_snes::with_asar_lock;
/// use asar_snes::BasicPatchOptions;
/// // thread 1
/// let result = with_asar_lock(|| {
///     asar::patching::patch(BasicPatchOptions::new(vec![0x00, 0x00, 0x00, 0x00].into(), "test.asm".into()))
/// });
///
/// // thread 2
/// let (result, labels) = with_asar_lock(|| {
///     let result = asar::patching::patch(BasicPatchOptions::new(vec![0x00, 0x00, 0x00, 0x00].into(), "test2.asm".into()));
///     let labels = asar::patching::labels();
///     (result, labels)
/// });
/// ```
///
/// A lot of functions already use this lock internally, but if you are calling multiple functions in a row, it is recommended to call it manually since other threads might interfere between the calls.
/// 
/// # Note
/// This function does something **only** if the `thread-safe` feature is **enabled**. Otherwise it is a no-op.
pub fn with_asar_lock<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let _lock = global_asar_lock().lock();
    f()
}

/// Represents the ROM data, with a byte vector and the ROM length
///
/// Note that the ROM length may not be the same as the length of the data vector, it is the actual length of the ROM.
///
/// The [`RomData::length`] parameter is updated after a patch operation to reflect the new length of the ROM if it was modified.
///
/// Note that asar will not modify the length of the data vector, if the patch does not fit in the data vector, patching will fail.
#[derive(Debug, Clone, Default)]
pub struct RomData {
    pub data: Vec<u8>,
    pub length: usize,
}

/// Represents an error message from Asar.
#[derive(Debug, Clone)]
pub struct ErrorData {
    pub fullerrdata: String,
    pub rawerrdata: String,
    pub block: String,
    pub filename: String,
    pub line: i32,
    pub callerfilename: String,
    pub callerline: i32,
    pub errid: i32,
}

/// Represents a warning message from Asar.
pub type WarningData = ErrorData;

/// Represents a define from Asar, with its name and contents.
#[derive(Debug, Clone)]
pub struct Define {
    pub name: String,
    pub contents: String,
}

/// Represents a block of data written to the ROM by Asar as a consequence of a call to asar_patch or asar_patch_ex.
/// It has the PC offset, the SNES offset and the number of bytes written.
#[derive(Debug, Clone)]
pub struct WrittenBlock {
    pub pcoffset: i32,
    pub snesoffset: i32,
    pub numbytes: i32,
}

/// Represents a label from Asar, with its name and location.
/// The location is the SNES address of the label.
#[derive(Debug, Clone)]
pub struct Label {
    pub name: String,
    pub location: i32,
}

/// Represents the basic options for a patch operation, only requiring the ROM data and the patch location.
#[derive(Debug, Clone)]
pub struct BasicPatchOptions {
    romdata: RomData,
    patchloc: String,
}

/// Represents the warn settings for a patch operation, with the warnid and whether it is enabled or not.
#[derive(Debug, Clone)]
pub struct WarnSetting {
    pub warnid: String,
    pub enabled: bool,
}

/// Represents the data for a memory file, the data can be binary or text.
#[derive(Debug, Clone)]
pub enum MemoryFileData {
    Binary(Vec<u8>),
    Text(String),
}

impl From<Vec<u8>> for MemoryFileData {
    fn from(data: Vec<u8>) -> Self {
        MemoryFileData::Binary(data)
    }
}

impl From<String> for MemoryFileData {
    fn from(data: String) -> Self {
        MemoryFileData::Text(data)
    }
}

impl From<&str> for MemoryFileData {
    fn from(data: &str) -> Self {
        MemoryFileData::Text(data.into())
    }
}

/// Represents the memory file for a patch operation, with the filename and the data.
#[derive(Debug, Clone)]
pub struct MemoryFile {
    pub filename: String,
    pub data: MemoryFileData,
}

/// Represents the advanced options for a patch operation, requiring, at least, the ROM data and the patch location.
/// Several options can be added to the patch operation, like include paths, defines, warning settings, memory files, etc.
/// See the [`PatchOption`] enum for all the available options.
/// Creation of this struct should be done with the [`AdvancedPatchOptions::new`] method.
#[derive(Debug, Clone)]
pub struct AdvancedPatchOptions {
    includepaths: Vec<String>,
    should_reset: bool,
    additional_defines: Vec<Define>,
    stdincludesfile: Option<String>,
    stddefinesfile: Option<String>,
    warning_settings: Vec<WarnSetting>,
    memory_files: Vec<MemoryFile>,
    override_checksum_gen: bool,
    generate_checksum: bool,
}

pub type MapperType = mappertype;

#[derive(Debug, Clone)]
pub enum SymbolType {
    WLA,
    NoCash,
}

#[derive(Debug, Clone)]
pub enum PatchResult {
    Success(RomData, Vec<WarningData>),
    Failure(Vec<ErrorData>),
}

/// Represents the options that can be added to a patch operation.
#[derive(Debug, Clone)]
pub enum PatchOption {
    /// Adds an include path to the patch operation.
    Include(String),
    /// Adds a define to the patch operation.
    Define(String, String),
    /// Adds a warning setting to the patch operation.
    Warning(String, bool),
    /// Adds a memory file to the patch operation.
    MemoryFile(String, MemoryFileData),
    /// Adds a standard includes file to the patch operation.
    StdIncludesFile(String),
    /// Adds a standard defines file to the patch operation.
    StdDefinesFile(String),
    /// Overrides the checksum generation.
    OverrideChecksumGen(bool),
    /// Generates the checksum.
    GenerateChecksum(bool),
    /// Sets whether the patch operation should reset.
    ShouldReset(bool),
}

impl RomData {
    /// Creates a new RomData with the data provided.
    pub fn from_vec(data: Vec<u8>) -> RomData {
        let length = data.len();
        RomData { data, length }
    }

    /// Creates a new RomData with the data provided.
    pub fn new(data: Vec<u8>, length: usize) -> RomData {
        RomData { data, length }
    }
}

impl From<Vec<u8>> for RomData {
    fn from(data: Vec<u8>) -> Self {
        RomData::from_vec(data)
    }
}

impl MemoryFile {
    fn as_raw(&self) -> memoryfile {
        let filename = CString::new(self.filename.clone()).unwrap();
        let data = match &self.data {
            MemoryFileData::Binary(d) => d.as_ptr() as *mut c_void,
            MemoryFileData::Text(d) => d.as_ptr() as *mut c_void,
        };
        let size = match &self.data {
            MemoryFileData::Binary(d) => d.len(),
            MemoryFileData::Text(d) => d.len(),
        };
        memoryfile {
            path: filename.into_raw(),
            buffer: data,
            length: size,
        }
    }
}

impl WarnSetting {
    fn as_raw(&self) -> warnsetting {
        let warnid = CString::new(self.warnid.clone()).unwrap();
        warnsetting {
            warnid: warnid.into_raw(),
            enabled: self.enabled,
        }
    }
}

impl ErrorData {
    fn from_raw(raw: &errordata) -> ErrorData {
        ErrorData {
            fullerrdata: unsafe { CStr::from_ptr(raw.fullerrdata) }
                .to_string_lossy()
                .into_owned(),
            rawerrdata: unsafe { CStr::from_ptr(raw.rawerrdata) }
                .to_string_lossy()
                .into_owned(),
            block: unsafe { CStr::from_ptr(raw.block) }
                .to_string_lossy()
                .into_owned(),
            filename: if raw.filename.is_null() {
                "".into()
            } else {
                unsafe { CStr::from_ptr(raw.filename) }
                    .to_string_lossy()
                    .into_owned()
            },
            line: raw.line,
            callerfilename: if raw.callerfilename.is_null() {
                "".into()
            } else {
                unsafe { CStr::from_ptr(raw.callerfilename) }
                    .to_string_lossy()
                    .into_owned()
            },
            callerline: raw.callerline,
            errid: raw.errid,
        }
    }
}

impl Define {
    fn from_raw(raw: &definedata) -> Define {
        Define {
            name: unsafe { CStr::from_ptr(raw.name) }
                .to_string_lossy()
                .into_owned(),
            contents: unsafe { CStr::from_ptr(raw.contents) }
                .to_string_lossy()
                .into_owned(),
        }
    }
    fn as_raw(&self) -> definedata {
        let name = std::ffi::CString::new(self.name.clone()).unwrap();
        let contents = std::ffi::CString::new(self.contents.clone()).unwrap();
        definedata {
            name: name.into_raw(),
            contents: contents.into_raw(),
        }
    }
}

impl WrittenBlock {
    fn from_raw(raw: &writtenblockdata) -> WrittenBlock {
        WrittenBlock {
            pcoffset: raw.pcoffset,
            snesoffset: raw.snesoffset,
            numbytes: raw.numbytes,
        }
    }
}

impl Label {
    fn from_raw(raw: &labeldata) -> Label {
        Label {
            name: unsafe { CStr::from_ptr(raw.name) }
                .to_string_lossy()
                .into_owned(),
            location: raw.location,
        }
    }
}

impl BasicPatchOptions {
    /// Creates a new BasicPatchOptions with the ROM data and the patch location.
    pub fn new(romdata: RomData, patchloc: String) -> BasicPatchOptions {
        BasicPatchOptions { romdata, patchloc }
    }
}

impl AdvancedPatchOptions {
    /// Creates a new AdvancedPatchOptions, with all default values.
    pub fn new() -> AdvancedPatchOptions {
        AdvancedPatchOptions {
            includepaths: Vec::new(),
            should_reset: true,
            additional_defines: Vec::new(),
            stdincludesfile: None,
            stddefinesfile: None,
            warning_settings: Vec::new(),
            memory_files: Vec::new(),
            override_checksum_gen: false,
            generate_checksum: false,
        }
    }

    /// Creates a new AdvancedPatchOptions with the options provided.
    pub fn from(options: Vec<PatchOption>) -> AdvancedPatchOptions {
        AdvancedPatchOptions::new().options(options)
    }

    /// Adds an option to the patch operation.
    pub fn option(mut self, option: PatchOption) -> AdvancedPatchOptions {
        match option {
            PatchOption::Include(path) => self.includepaths.push(path),
            PatchOption::Define(name, contents) => {
                self.additional_defines.push(Define { name, contents })
            }
            PatchOption::Warning(warnid, enabled) => {
                self.warning_settings.push(WarnSetting { warnid, enabled })
            }
            PatchOption::MemoryFile(filename, data) => {
                self.memory_files.push(MemoryFile { filename, data })
            }
            PatchOption::StdIncludesFile(filename) => self.stdincludesfile = Some(filename),
            PatchOption::StdDefinesFile(filename) => self.stddefinesfile = Some(filename),
            PatchOption::OverrideChecksumGen(override_checksum_gen) => {
                self.override_checksum_gen = override_checksum_gen
            }
            PatchOption::GenerateChecksum(generate_checksum) => {
                self.generate_checksum = generate_checksum
            }
            PatchOption::ShouldReset(should_reset) => self.should_reset = should_reset,
        };
        self
    }

    /// Adds multiple options to the patch operation.
    pub fn options(mut self, options: Vec<PatchOption>) -> AdvancedPatchOptions {
        for option in options {
            self = self.option(option);
        }
        self
    }
}

impl Default for AdvancedPatchOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns the maximum ROM size that Asar can handle in bytes
///
/// This should normally be 16*1024*1024
pub fn max_rom_size() -> i32 {
    unsafe { asar_maxromsize() }
}

/// Returns the API version of Asar.
pub fn api_version() -> i32 {
    unsafe { asar_apiversion() }
}

/// Returns the version of Asar, in the format Major * 10000 + Minor * 100 + Revision.
pub fn version() -> i32 {
    unsafe { asar_version() }
}

/// Computes a math expression.
///
/// If the math expression is invalid, it returns an error message.
pub fn math(math: &str) -> Result<f64, String> {
    let math = CString::new(math).unwrap();
    let mut err: *const i8 = std::ptr::null();
    let result = unsafe { asar_math(math.as_ptr(), &mut err) };
    if err.is_null() {
        Ok(result)
    } else {
        Err(unsafe { CStr::from_ptr(err) }
            .to_string_lossy()
            .into_owned())
    }
}

/// This is the raw patching API of asar, it is not recommended to use this directly as it does not prevent multiple calls to the API at the same time which may result in tramplings of the global state in asar's library.
///
/// e.g.
///
/// ```rust
/// /// assuming that test.asm contains:
/// /// !test = $18
/// /// and that test2.asm contains:
/// /// !test = $19
/// use asar_snes::BasicPatchOptions;
/// use asar_snes as asar;
///
/// let options1 = BasicPatchOptions::new(vec![].into(), "test.asm".into());
/// let options2 = BasicPatchOptions::new(vec![].into(), "test2.asm".into());
/// let result1 = asar::patching::patch(options1);
/// let result2 = asar::patching::patch(options2);
///
/// let define = asar::patching::define("test");
///
/// println!("{:?}", define); // this will print $19, because the second patch operation overwrote the global state of the first patch operation.
///
/// ```
///
/// For this reason, it is recommended to use [`Patcher`] instead.
///
/// This module is however provided for users that want to use the raw API directly
///
/// remarks: all functions in this module use the global lock.
pub mod patching {

    use super::*;

    /// Resets Asar, clearing all the errors, warnings and prints.
    ///
    /// Useful to clear the state of Asar between patch operations.
    ///
    /// Returns true if the reset was successful, false otherwise.
    ///
    /// If false is returned, you can check the errors with the [`errors`] function.
    ///
    /// remarks: This function uses the global lock.
    #[use_asar_global_lock]
    pub fn reset() -> bool {
        unsafe { asar_reset() }
    }

    /// Patches the ROM data with the patch provided in the [`BasicPatchOptions`].
    ///
    /// Returns a [`PatchResult`] with the result of the patch operation.
    ///
    /// remarks: This function uses the global lock.
    #[use_asar_global_lock]
    pub fn patch(mut options: BasicPatchOptions) -> PatchResult {
        let romdata = options.romdata.data.as_mut_ptr() as *mut c_char;
        let buflen = options.romdata.data.len() as c_int;
        let patchloc = CString::new(options.patchloc).unwrap();
        let mut romsize = options.romdata.length as c_int;
        let romlen: *mut c_int = &mut romsize;
        let result = unsafe { asar_patch(patchloc.as_ptr(), romdata, buflen, romlen) };
        let mut count: c_int = 0;
        let warnings = unsafe { asar_getwarnings(&mut count) };
        let warnings = unsafe { std::slice::from_raw_parts(warnings, count as usize) };
        let warnings = warnings.iter().map(ErrorData::from_raw).collect();
        if result {
            options.romdata.length = romsize as usize;
            PatchResult::Success(options.romdata, warnings)
        } else {
            let mut count: c_int = 0;
            let errors = unsafe { asar_geterrors(&mut count) };
            let errors = unsafe { std::slice::from_raw_parts(errors, count as usize) };
            let errors = errors.iter().map(ErrorData::from_raw).collect();
            PatchResult::Failure(errors)
        }
    }

    #[use_asar_global_lock]
    pub(crate) fn patch_ex_basic(
        mut rom: RomData,
        patch: String,
        options: AdvancedPatchOptions,
    ) -> (RomData, bool) {
        let romdata = rom.data.as_mut_ptr() as *mut c_char;
        let buflen = rom.data.len() as c_int;
        let patchloc = CString::new(patch).unwrap();
        let mut romsize = rom.length as c_int;
        let romlen: *mut c_int = &mut romsize;

        let mut definedata = options
            .additional_defines
            .iter()
            .map(Define::as_raw)
            .collect::<Vec<definedata>>();
        let mut warning_settings = options
            .warning_settings
            .iter()
            .map(WarnSetting::as_raw)
            .collect::<Vec<warnsetting>>();
        let mut memory_files = options
            .memory_files
            .iter()
            .map(MemoryFile::as_raw)
            .collect::<Vec<memoryfile>>();
        let mut includepaths = options
            .includepaths
            .iter()
            .map(|p| CString::new(p.clone()).unwrap().into_raw() as *const i8)
            .collect::<Vec<_>>();

        let stdincludesfile = options.stdincludesfile.map(|s| CString::new(s).unwrap());
        let stddefinesfile = options.stddefinesfile.map(|s| CString::new(s).unwrap());

        let params = patchparams {
            structsize: std::mem::size_of::<patchparams>() as c_int,
            buflen,
            patchloc: patchloc.as_ptr(),
            romdata,
            romlen,
            includepaths: includepaths.as_mut_ptr(),
            numincludepaths: includepaths.len() as c_int,
            should_reset: options.should_reset,
            additional_defines: definedata.as_mut_ptr(),
            additional_define_count: definedata.len() as c_int,
            stdincludesfile: stdincludesfile.map_or(ptr::null(), |s| s.as_ptr()),
            stddefinesfile: stddefinesfile.map_or(ptr::null(), |s| s.as_ptr()),
            warning_settings: warning_settings.as_mut_ptr(),
            warning_setting_count: warning_settings.len() as c_int,
            memory_files: memory_files.as_mut_ptr(),
            memory_file_count: memory_files.len() as c_int,
            override_checksum_gen: options.override_checksum_gen,
            generate_checksum: options.generate_checksum,
        };
        let result = unsafe { asar_patch_ex(&params) };

        for define in definedata {
            unsafe {
                drop(CString::from_raw(define.name as *mut i8));
                drop(CString::from_raw(define.contents as *mut i8));
            }
        }

        for path in includepaths {
            unsafe {
                drop(CString::from_raw(path as *mut i8));
            }
        }

        rom.length = romsize as usize;

        (rom, result)
    }

    /// Patches the ROM data with the patch provided in the [`AdvancedPatchOptions`].
    ///
    /// Returns a [`PatchResult`] with the result of the patch operation.
    ///
    /// remarks: This function uses the global lock.
    #[use_asar_global_lock]
    pub fn patch_ex<T: Into<String>>(rom: RomData, patch: T, options: AdvancedPatchOptions) -> PatchResult {
        let (romdata, result) = patch_ex_basic(rom, patch.into(), options);

        let mut count: c_int = 0;
        let warnings = unsafe { asar_getwarnings(&mut count) };
        let warnings = unsafe { std::slice::from_raw_parts(warnings, count as usize) };
        let warnings = warnings.iter().map(ErrorData::from_raw).collect();

        if result {
            PatchResult::Success(romdata, warnings)
        } else {
            let mut count: c_int = 0;
            let errors = unsafe { asar_geterrors(&mut count) };
            let errors = unsafe { std::slice::from_raw_parts(errors, count as usize) };
            let errors = errors.iter().map(ErrorData::from_raw).collect();
            PatchResult::Failure(errors)
        }
    }

    /// Returns the errors from the latest api call (usually [`patch`] or [`patch_ex`]).
    ///
    /// remarks: This function uses the global lock.
    #[use_asar_global_lock]
    pub fn errors() -> Vec<ErrorData> {
        let mut count: c_int = 0;
        let errors = unsafe { asar_geterrors(&mut count) };
        let errors = unsafe { std::slice::from_raw_parts(errors, count as usize) };
        errors.iter().map(ErrorData::from_raw).collect()
    }

    /// Returns the warnings from the latest api call (usually [`patch`] or [`patch_ex`]).
    ///
    /// remarks: This function uses the global lock.
    #[use_asar_global_lock]
    pub fn warnings() -> Vec<ErrorData> {
        let mut count: c_int = 0;
        let errors = unsafe { asar_getwarnings(&mut count) };
        let errors = unsafe { std::slice::from_raw_parts(errors, count as usize) };
        errors.iter().map(ErrorData::from_raw).collect()
    }

    /// Returns the prints from the latest api call (usually [`patch`] or [`patch_ex`]).
    ///
    /// remarks: This function uses the global lock.
    #[use_asar_global_lock]
    pub fn prints() -> Vec<String> {
        let mut count: c_int = 0;
        let prints = unsafe { asar_getprints(&mut count) };
        let prints = unsafe { std::slice::from_raw_parts(prints, count as usize) };
        prints
            .iter()
            .map(|p| unsafe { CStr::from_ptr(*p) }.to_string_lossy().into_owned())
            .collect()
    }

    /// Returns the labels from the latest api call (usually [`patch`] or [`patch_ex`]).
    ///
    /// remarks: This function uses the global lock.
    #[use_asar_global_lock]
    pub fn labels() -> Vec<Label> {
        let mut count: c_int = 0;
        let labels = unsafe { asar_getalllabels(&mut count) };
        let labels = unsafe { std::slice::from_raw_parts(labels, count as usize) };
        labels.iter().map(Label::from_raw).collect()
    }

    /// Returns the value of a label from the latest api call (usually [`patch`] or [`patch_ex`]).
    ///
    /// If the label is not found, it returns None.
    ///
    /// remarks: This function uses the global lock.
    #[use_asar_global_lock]
    pub fn label_value(name: &str) -> Option<i32> {
        let name = CString::new(name).unwrap();
        let value = unsafe { asar_getlabelval(name.as_ptr()) };
        if value == -1 {
            None
        } else {
            Some(value)
        }
    }

    /// Returns the value of a define from the latest api call (usually [`patch`] or [`patch_ex`]).
    ///
    /// If the define is not found, it returns None.
    #[use_asar_global_lock]
    pub fn define(name: &str) -> Option<String> {
        let name = CString::new(name).unwrap();
        let def = unsafe { asar_getdefine(name.as_ptr()) };
        if def.is_null() {
            None
        } else {
            Some(
                unsafe { CStr::from_ptr(def) }
                    .to_string_lossy()
                    .into_owned(),
            )
        }
    }

    /// Returns all the defines from the latest api call (usually [`patch`] or [`patch_ex`]).
    ///
    /// remarks: This function uses the global lock.
    #[use_asar_global_lock]
    pub fn defines() -> Vec<Define> {
        let mut count: c_int = 0;
        let defines = unsafe { asar_getalldefines(&mut count) };
        let defines = unsafe { std::slice::from_raw_parts(defines, count as usize) };
        defines.iter().map(Define::from_raw).collect()
    }

    /// Resolves the defines in the data provided.
    ///
    /// This function is not very useful and it has some issues, it is not recommended to use it.
    ///
    /// remarks: This function uses the global lock.
    #[use_asar_global_lock]
    pub fn resolve_defines(data: &str) -> String {
        unsafe {
            let data = CString::new(data).unwrap();
            let resolved = asar_resolvedefines(data.as_ptr(), false);
            CStr::from_ptr(resolved).to_string_lossy().into_owned()
        }
    }

    /// Returns the blocks written to the ROM by Asar as a consequence of a call to [`patch`] or [`patch_ex`].
    ///
    /// remarks: This function uses the global lock.
    #[use_asar_global_lock]
    pub fn written_blocks() -> Vec<WrittenBlock> {
        let mut count: c_int = 0;
        let blocks = unsafe { asar_getwrittenblocks(&mut count) };
        let blocks = unsafe { std::slice::from_raw_parts(blocks, count as usize) };
        blocks.iter().map(WrittenBlock::from_raw).collect()
    }

    /// Returns the mapper type used in the latest api call (usually [`patch`] or [`patch_ex`]).
    ///
    /// If the mapper type is not recognized, it returns None.
    ///
    /// remarks: This function uses the global lock.
    #[use_asar_global_lock]
    pub fn mapper_type() -> Option<MapperType> {
        let raw = unsafe { asar_getmapper() };
        match raw {
            MapperType::invalid_mapper => None,
            _ => Some(raw),
        }
    }

    /// Returns the symbols file for the specified symbol type.
    ///
    /// The symbol type can be WLA or NoCash.
    ///
    /// remarks: This function uses the global lock.
    #[use_asar_global_lock]
    pub fn symbols_file(symboltype: SymbolType) -> Option<String> {
        let symboltype = match symboltype {
            SymbolType::WLA => "wla",
            SymbolType::NoCash => "nocash",
        };
        let symboltype = CString::new(symboltype).unwrap();
        unsafe {
            let file = asar_getsymbolsfile(symboltype.as_ptr());
            if file.is_null() {
                None
            } else {
                Some(CStr::from_ptr(file).to_string_lossy().into_owned())
            }
        }
    }
}
#[cfg(feature = "thread-safe")]
use parking_lot::ReentrantMutexGuard;

/// The Patcher struct is a convenient wrapper around the [`patching`] api.
///
/// It wraps the patching functions as well as providing a way to gather all information about the result of the patch.
///
/// see [`Patcher::apply`] and [`ApplyResult`] for more information.
#[derive(Debug, Clone)]
pub struct Patcher {
    options: Option<AdvancedPatchOptions>,
}

/// This type represents the result of a patch operation.
///
/// It contains the possibly modified ROM data and a boolean indicating whether the patch was successful or not.
///
/// see [`ApplyResult::success`]
///
/// ### Notes:
///
/// The following notes apply to the functions
/// - [`ApplyResult::warnings`]
/// - [`ApplyResult::errors`]
/// - [`ApplyResult::prints`]
/// - [`ApplyResult::labels`]
/// - [`ApplyResult::label_value`]
/// - [`ApplyResult::define`]
/// - [`ApplyResult::defines`]
/// - [`ApplyResult::written_blocks`]
/// - [`ApplyResult::mapper_type`]
/// - [`ApplyResult::symbols_file`]
///
/// The other functions are not affected by these notes.
///
/// - If the patch operation was *not* successful ([`ApplyResult::success`] returns false), they will return an empty vector/None/empty string if [`PatchOption::ShouldReset`] was set to true
///   or the values from the previous patch operation if it was set to false.
///  
/// - If there were any call to [`patching::patch`] or [`patching::patch_ex`] between the [`Patcher::apply`] call that returned this [`ApplyResult`] and this call,
///   this will return the warnings from the latest call instead of ones related to this [`ApplyResult`].   
#[cfg(feature = "thread-safe")]
pub struct ApplyResult<'a> {
    romdata: RomData,
    success: bool,
    _guard: ReentrantMutexGuard<'a, ()>,
}

/// This type represents the result of a patch operation.
///
/// It contains the possibly modified ROM data and a boolean indicating whether the patch was successful or not.
///
/// see [`ApplyResult::success`] for more information.
#[cfg(not(feature = "thread-safe"))]
pub struct ApplyResult<'a> {
    romdata: RomData,
    success: bool,
    _marker: std::marker::PhantomData<&'a ()>,
}

use std::sync::atomic::{AtomicBool, Ordering};

static APPLYRESULT_ONCE_ALIVE: AtomicBool = AtomicBool::new(false);

/// This error is returned when trying to call [`Patcher::apply`] while another [`ApplyResult`] is alive.
///
/// This is to prevent multiple patch operations from happening at the same time, since Asar uses a lot of global state.
#[derive(Debug, Clone)]
pub struct ConcurrentApplyError;

impl fmt::Display for ConcurrentApplyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cannot call `Patcher::apply` while another `ApplyResult` is alive, drop() it or consume it by calling `ApplyResult::romdata()`.")
    }
}

impl Patcher {
    /// Creates a new Patcher with default options.
    pub fn new() -> Self {
        Self { options: None }
    }
    /// Adds an option to the patch operation.
    pub fn option(&mut self, option: PatchOption) {
        self.options = Some(self.options.take().unwrap_or_default().option(option));
    }
    /// Replaces the options of the patch operation.
    pub fn options(&mut self, options: AdvancedPatchOptions) {
        self.options = Some(options);
    }
    /// Applies the patch to the ROM data
    ///
    /// Multiple patch operations cannot be done at the same time, this function will return an error if another [`ApplyResult`] is alive.
    ///
    /// See [`ConcurrentApplyError`] for more information.
    ///
    /// remarks: This function uses the global lock.
    #[cfg(feature = "thread-safe")]
    pub fn apply<'a, T: Into<String>>(
        self,
        rom: RomData,
        patch: T,
    ) -> Result<ApplyResult<'a>, ConcurrentApplyError> {
        if APPLYRESULT_ONCE_ALIVE
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Err(ConcurrentApplyError);
        }

        let guard = global_asar_lock().lock();
        let (romdata, result) =
            patching::patch_ex_basic(rom, patch.into(), self.options.unwrap_or_default());

        Ok(ApplyResult {
            romdata,
            success: result,
            _guard: guard,
        })
    }

    /// Applies the patch to the ROM data
    ///
    /// Multiple patch operations cannot be done at the same time, this function will return an error if another [`ApplyResult`] is alive.
    ///
    /// See [`ConcurrentApplyError`] for more information.
    #[cfg(not(feature = "thread-safe"))]
    pub fn apply<'a, T: Into<String>>(
        self,
        rom: RomData,
        patch: T,
    ) -> Result<ApplyResult<'a>, ConcurrentApplyError> {
        if APPLYRESULT_ONCE_ALIVE
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Err(ConcurrentApplyError);
        }

        let (romdata, result) =
            patching::patch_ex_basic(rom, patch.into(), self.options.unwrap_or_default());

        Ok(ApplyResult {
            romdata,
            success: result,
            _marker: std::marker::PhantomData,
        })
    }
}

impl Default for Patcher {
    fn default() -> Self {
        Self::new()
    }
}

impl ApplyResult<'_> {
    /// Returns whether the patch operation was successful or not.
    pub fn success(&self) -> bool {
        self.success
    }

    /// Returns the warnings from the apply operation.

    pub fn warnings(&self) -> Vec<WarningData> {
        patching::warnings()
    }

    /// Returns the errors from the apply operation.
    ///
    /// See the notes in the [`ApplyResult`] type for more information.
    pub fn errors(&self) -> Vec<ErrorData> {
        patching::errors()
    }

    /// Returns the prints from the apply operation.        
    pub fn prints(&self) -> Vec<String> {
        patching::prints()
    }

    /// Returns the labels from the apply operation.        
    ///
    /// See the notes in the [`ApplyResult`] type for more information.  
    pub fn labels(&self) -> Vec<Label> {
        patching::labels()
    }

    /// Returns the value of a label from the apply operation.
    ///
    /// See the notes in the [`ApplyResult`] type for more information.
    pub fn label_value(&self, name: &str) -> Option<i32> {
        patching::label_value(name)
    }

    /// Returns the value of a define from the apply operation.
    ///
    /// See the notes in the [`ApplyResult`] type for more information.
    pub fn define(&self, name: &str) -> Option<String> {
        patching::define(name)
    }

    /// Returns the defines from the apply operation.
    ///
    /// See the notes in the [`ApplyResult`] type for more information.
    pub fn defines(&self) -> Vec<Define> {
        patching::defines()
    }

    /// Returns the written blocks from the apply operation.
    ///
    /// See the notes in the [`ApplyResult`] type for more information.
    pub fn written_blocks(&self) -> Vec<WrittenBlock> {
        patching::written_blocks()
    }

    /// Returns the mapper type from the apply operation.
    ///
    /// See the notes in the [`ApplyResult`] type for more information.
    pub fn mapper_type(&self) -> Option<MapperType> {
        patching::mapper_type()
    }

    /// Returns the symbols file from the apply operation.
    ///
    /// See the notes in the [`ApplyResult`] type for more information.
    pub fn symbols_file(&self, symboltype: SymbolType) -> Option<String> {
        patching::symbols_file(symboltype)
    }

    /// Consumes the ApplyResult and returns the ROM data.
    ///
    /// This will reset Asar, clearing all the errors, warnings and prints.
    ///
    /// Calling this method will allow another patch operation to be done with the [`Patcher::apply`] method.
    pub fn romdata(mut self) -> RomData {
        let romdata = std::mem::take(&mut self.romdata);
        APPLYRESULT_ONCE_ALIVE.store(false, Ordering::SeqCst);
        romdata
    }
}

impl Drop for ApplyResult<'_> {
    fn drop(&mut self) {
        patching::reset();
    }
}
