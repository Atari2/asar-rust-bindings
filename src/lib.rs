#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub(crate) mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[cfg(test)]
mod test;

pub mod asar {
    extern crate asar_rust_bindings_proc_macros;
    pub use asar_rust_bindings_proc_macros::use_asar_global_lock;

    use parking_lot::ReentrantMutex;
    use std::{
        ffi::{CStr, CString},
        os::raw::{c_char, c_int, c_void},
        ptr,
        sync::OnceLock,
    };

    use crate::bindings::{
        asar_apiversion, asar_getalldefines, asar_getalllabels, asar_getdefine, asar_geterrors,
        asar_getlabelval, asar_getmapper, asar_getprints, asar_getsymbolsfile, asar_getwarnings,
        asar_getwrittenblocks, asar_math, asar_maxromsize, asar_patch, asar_patch_ex, asar_reset,
        asar_resolvedefines, asar_version, definedata, errordata, labeldata, mappertype,
        memoryfile, patchparams, warnsetting, writtenblockdata,
    };

    fn global_asar_lock() -> &'static ReentrantMutex<()> {
        static LOCK: OnceLock<ReentrantMutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| ReentrantMutex::new(()))
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
    /// use asar_rust_bindings::asar;
    /// use asar_rust_bindings::asar::with_asar_lock;
    /// use asar_rust_bindings::asar::BasicPatchOptions;
    /// // thread 1
    /// let result = with_asar_lock(|| {
    ///     asar::patch(BasicPatchOptions::new(vec![0x00, 0x00, 0x00, 0x00], "test.asm".into()))
    /// });
    ///
    /// // thread 2
    /// let (result, labels) = with_asar_lock(|| {
    ///     let result = asar::patch(BasicPatchOptions::new(vec![0x00, 0x00, 0x00, 0x00], "test2.asm".into()));
    ///     let labels = asar::labels();
    ///     (result, labels)
    /// });
    /// ```
    ///
    /// A lot of functions already use this lock internally, but if you are calling multiple functions in a row, it is recommended to call it manually since other threads might interfere between the calls.
    pub fn with_asar_lock<F, R>(f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let _lock = global_asar_lock().lock();
        f()
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
        romdata: Vec<u8>,
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

    /// Represents the memory file for a patch operation, with the filename and the data.
    #[derive(Debug, Clone)]
    pub struct MemoryFile {
        pub filename: String,
        pub data: MemoryFileData,
    }

    /// Represents the advanced options for a patch operation, requiring, at least, the ROM data and the patch location.
    /// Several options can be added to the patch operation, like include paths, defines, warning settings, memory files, etc.
    /// See the [`PatchOption`] enum for all the available options.
    /// Creation of this struct should be done with the [`AdvancedPatchOptions::new`](crate::asar::AdvancedPatchOptions::new) method.
    #[derive(Debug, Clone)]
    pub struct AdvancedPatchOptions {
        patchloc: String,
        romdata: Vec<u8>,
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
        Success(Vec<u8>, Vec<WarningData>),
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
        pub fn new(romdata: Vec<u8>, patchloc: String) -> BasicPatchOptions {
            BasicPatchOptions { romdata, patchloc }
        }
    }

    impl AdvancedPatchOptions {
        /// Creates a new AdvancedPatchOptions with the ROM data and the patch location.
        pub fn new(romdata: Vec<u8>, patchloc: String) -> AdvancedPatchOptions {
            AdvancedPatchOptions {
                patchloc,
                romdata,
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
    }

    /// Returns the API version of Asar.
    pub fn api_version() -> i32 {
        unsafe { asar_apiversion() }
    }

    /// Returns the version of Asar, in the format Major * 10000 + Minor * 100 + Revision.
    pub fn version() -> i32 {
        unsafe { asar_version() }
    }

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
        let romdata = options.romdata.as_mut_ptr() as *mut c_char;
        let mut romsize = options.romdata.len() as c_int;
        let patchloc = CString::new(options.patchloc).unwrap();
        let romlen: *mut c_int = &mut romsize;
        let result = unsafe { asar_patch(patchloc.as_ptr(), romdata, romsize, romlen) };
        let mut count: c_int = 0;
        let warnings = unsafe { asar_getwarnings(&mut count) };
        let warnings = unsafe { std::slice::from_raw_parts(warnings, count as usize) };
        let warnings = warnings.iter().map(ErrorData::from_raw).collect();
        if result {
            PatchResult::Success(options.romdata, warnings)
        } else {
            let mut count: c_int = 0;
            let errors = unsafe { asar_geterrors(&mut count) };
            let errors = unsafe { std::slice::from_raw_parts(errors, count as usize) };
            let errors = errors.iter().map(ErrorData::from_raw).collect();
            PatchResult::Failure(errors)
        }
    }

    /// Patches the ROM data with the patch provided in the [`AdvancedPatchOptions`].
    ///
    /// Returns a [`PatchResult`] with the result of the patch operation.
    ///
    /// remarks: This function uses the global lock.
    #[use_asar_global_lock]
    pub fn patch_ex(mut options: AdvancedPatchOptions) -> PatchResult {
        let romdata = options.romdata.as_mut_ptr() as *mut c_char;
        let mut romsize = options.romdata.len() as c_int;
        let patchloc = CString::new(options.patchloc).unwrap();
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
            buflen: romsize,
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

        unsafe {
            options.romdata.set_len(*romlen as usize);
        }
        let mut count: c_int = 0;
        let warnings = unsafe { asar_getwarnings(&mut count) };
        let warnings = unsafe { std::slice::from_raw_parts(warnings, count as usize) };
        let warnings = warnings.iter().map(ErrorData::from_raw).collect();

        if result {
            PatchResult::Success(options.romdata, warnings)
        } else {
            let mut count: c_int = 0;
            let errors = unsafe { asar_geterrors(&mut count) };
            let errors = unsafe { std::slice::from_raw_parts(errors, count as usize) };
            let errors = errors.iter().map(ErrorData::from_raw).collect();
            PatchResult::Failure(errors)
        }
    }

    /// Returns the maximum ROM size that Asar can handle in bytes
    ///
    /// This should normally be 16*1024*1024
    pub fn max_rom_size() -> i32 {
        unsafe { asar_maxromsize() }
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
    pub fn symbols_file(symboltype: SymbolType) -> String {
        let symboltype = match symboltype {
            SymbolType::WLA => "wla",
            SymbolType::NoCash => "nocash",
        };
        let symboltype = CString::new(symboltype).unwrap();
        unsafe {
            let file = asar_getsymbolsfile(symboltype.as_ptr());
            CStr::from_ptr(file).to_string_lossy().into_owned()
        }
    }
}
