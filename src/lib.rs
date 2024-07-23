#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod test;

pub mod asar {
    use crate::{asar_apiversion, asar_getalldefines, asar_getalllabels, asar_getdefine, asar_geterrors, asar_getlabelval, asar_getmapper, asar_getprints, asar_getsymbolsfile, asar_getwarnings, asar_getwrittenblocks, asar_math, asar_maxromsize, asar_patch, asar_patch_ex, asar_reset, asar_resolvedefines, asar_version, definedata, errordata, labeldata, mappertype_bigsa1rom, mappertype_exhirom, mappertype_exlorom, mappertype_hirom, mappertype_lorom, mappertype_norom, mappertype_sa1rom, mappertype_sfxrom, memoryfile, patchparams, warnsetting, writtenblockdata};

    pub struct ErrorData {
        pub fullerrdata: String,
        pub rawerrdata: String,
        pub block: String,
        pub line: i32,
        pub callerfilename: String,
        pub callerline: i32,
        pub errid: i32
    }

    pub struct DefineData {
        pub name: String,
        pub contents: String
    }

    pub struct WrittenBlockData {
        pub pcoffset: i32,
        pub snesoffset: i32,
        pub numbytes: i32
    }

    pub struct LabelData {
        pub name: String,
        pub location: i32
    }

    pub struct BasicPatchOptions {
        pub romdata: Vec<u8>,
        pub patchloc: String
    }

    pub struct WarnSetting {
        pub warnid: String,
        pub enabled: bool
    }

    pub struct MemoryFile {
        pub filename: String,
        pub data: Vec<u8>
    }

    pub struct AdvancedPatchOptions {
        pub patchloc: String,
        pub romdata: Vec<u8>,
        pub includepaths: Vec<String>,
        pub should_reset: bool,
        pub additional_defines: Vec<DefineData>,
        pub stdincludesfile: String,
        pub stddefinesfile: String,
        pub warning_settings: Vec<WarnSetting>,
        pub memory_files: Vec<MemoryFile>,
        pub override_checksum_gen: bool,
        pub generate_checksum: bool
    }

    pub enum MapperType {
        Lorom,
        Hirom,
        Sa1rom,
        BigSa1rom,
        Sfxrom,
        Exlorom,
        Exhirom,
        Norom
    }

    pub enum SymbolType {
        WLA,
        NoCash
    }

    pub enum PatchResult {
        Success(Vec<u8>),
        Failure(Vec<ErrorData>)
    }

    impl MemoryFile {
        pub fn as_raw(&self) -> memoryfile {
            let filename = std::ffi::CString::new(self.filename.clone()).unwrap();
            let data = self.data.as_ptr() as *mut std::os::raw::c_void;
            let size = self.data.len();
            memoryfile {
                path: filename.into_raw(),
                buffer: data,
                length: size
            }
        }
    }

    impl WarnSetting {
        pub fn as_raw(&self) -> warnsetting {
            let warnid = std::ffi::CString::new(self.warnid.clone()).unwrap();
            warnsetting {
                warnid: warnid.into_raw(),
                enabled: self.enabled
            }
        }
    }

    impl ErrorData {
        fn from_raw(raw: &errordata) -> ErrorData {
            ErrorData {
                fullerrdata: unsafe { std::ffi::CStr::from_ptr(raw.fullerrdata) }.to_string_lossy().into_owned(),
                rawerrdata: unsafe { std::ffi::CStr::from_ptr(raw.rawerrdata) }.to_string_lossy().into_owned(),
                block: unsafe { std::ffi::CStr::from_ptr(raw.block) }.to_string_lossy().into_owned(),
                line: raw.line,
                callerfilename: unsafe { std::ffi::CStr::from_ptr(raw.callerfilename) }.to_string_lossy().into_owned(),
                callerline: raw.callerline,
                errid: raw.errid
            }
        }
    }

    impl DefineData {
        fn from_raw(raw: &definedata) -> DefineData {
            DefineData {
                name: unsafe { std::ffi::CStr::from_ptr(raw.name) }.to_string_lossy().into_owned(),
                contents: unsafe { std::ffi::CStr::from_ptr(raw.contents) }.to_string_lossy().into_owned()
            }
        }
        fn as_raw(&self) -> definedata {
            let name = std::ffi::CString::new(self.name.clone()).unwrap();
            let contents = std::ffi::CString::new(self.contents.clone()).unwrap();
            definedata {
                name: name.into_raw(),
                contents: contents.into_raw()
            }
        }
    }

    impl WrittenBlockData {
        fn from_raw(raw: &writtenblockdata) -> WrittenBlockData {
            WrittenBlockData {
                pcoffset: raw.pcoffset,
                snesoffset: raw.snesoffset,
                numbytes: raw.numbytes
            }
        }
    }

    impl LabelData {
        fn from_raw(raw: &labeldata) -> LabelData {
            LabelData {
                name: unsafe { std::ffi::CStr::from_ptr(raw.name) }.to_string_lossy().into_owned(),
                location: raw.location
            }
        }
    }

    impl MapperType {
        fn from_raw(raw: std::os::raw::c_int) -> Option<MapperType> {
            match raw {
                mappertype_lorom => Some(MapperType::Lorom),
                mappertype_hirom => Some(MapperType::Hirom),
                mappertype_sa1rom => Some(MapperType::Sa1rom),
                mappertype_bigsa1rom => Some(MapperType::BigSa1rom),
                mappertype_sfxrom => Some(MapperType::Sfxrom),
                mappertype_exlorom => Some(MapperType::Exlorom),
                mappertype_exhirom => Some(MapperType::Exhirom),
                mappertype_norom => Some(MapperType::Norom),
                _ => None
            }
        }
    }

    pub fn api_version() -> i32 {
        unsafe { asar_apiversion() }
    }

    pub fn version() -> i32 {
        unsafe { asar_version() }
    }

    pub fn reset() -> bool {
        unsafe { asar_reset() }
    }

    pub fn patch(mut options: BasicPatchOptions) -> PatchResult {
        let romdata = options.romdata.as_mut_ptr() as *mut std::os::raw::c_char;
        let mut romsize = options.romdata.len() as std::os::raw::c_int;
        let patchloc = std::ffi::CString::new(options.patchloc).unwrap();
        let romlen: *mut std::os::raw::c_int = &mut romsize;
        let result = unsafe { asar_patch(patchloc.as_ptr(), romdata, romsize, romlen) };
        if result {
            PatchResult::Success(options.romdata)
        } else {
            let mut count: std::os::raw::c_int = 0;
            let errors = unsafe { asar_geterrors(&mut count) };
            let errors = unsafe { std::slice::from_raw_parts(errors, count as usize) };
            let errors = errors.iter().map(ErrorData::from_raw).collect();
            PatchResult::Failure(errors)
        }
    }

    pub fn patch_ex(mut options: AdvancedPatchOptions) -> PatchResult {
        let romdata = options.romdata.as_mut_ptr() as *mut std::os::raw::c_char;
        let mut romsize = options.romdata.len() as std::os::raw::c_int;
        let patchloc = std::ffi::CString::new(options.patchloc).unwrap();
        let romlen: *mut std::os::raw::c_int = &mut romsize;

        let mut definedata = options.additional_defines.iter().map(DefineData::as_raw).collect::<Vec<definedata>>();
        let mut warning_settings = options.warning_settings.iter().map(WarnSetting::as_raw).collect::<Vec<warnsetting>>();
        let mut memory_files = options.memory_files.iter().map(MemoryFile::as_raw).collect::<Vec<memoryfile>>();
        let mut includepaths = options.includepaths.iter().map(|p| std::ffi::CString::new(p.clone()).unwrap().into_raw() as *const i8).collect::<Vec<_>>();


        let stdincludesfile = std::ffi::CString::new(options.stdincludesfile).unwrap();
        let stddefinesfile = std::ffi::CString::new(options.stddefinesfile).unwrap();

        let params = patchparams {
            structsize: std::mem::size_of::<patchparams>() as std::os::raw::c_int,
            buflen: romsize,
            patchloc: patchloc.as_ptr(),
            romdata,
            romlen,
            includepaths: includepaths.as_mut_ptr(),
            numincludepaths: includepaths.len() as std::os::raw::c_int,
            should_reset: options.should_reset,
            additional_defines: definedata.as_mut_ptr(),
            additional_define_count: definedata.len() as std::os::raw::c_int,
            stdincludesfile: stdincludesfile.as_ptr(),
            stddefinesfile: stddefinesfile.as_ptr(),
            warning_settings: warning_settings.as_mut_ptr(),
            warning_setting_count: warning_settings.len() as std::os::raw::c_int,
            memory_files: memory_files.as_mut_ptr(),
            memory_file_count: memory_files.len() as std::os::raw::c_int,
            override_checksum_gen: options.override_checksum_gen,
            generate_checksum: options.generate_checksum
        };
        let result = unsafe { asar_patch_ex(&params) };

        for define in definedata {
            unsafe {
                drop(std::ffi::CString::from_raw(define.name as *mut i8));
                drop(std::ffi::CString::from_raw(define.contents as *mut i8));
            }
        }

        for path in includepaths {
            unsafe {
                drop(std::ffi::CString::from_raw(path as *mut i8));
            }
        }

        if result {
            PatchResult::Success(options.romdata)
        } else {
            let mut count: std::os::raw::c_int = 0;
            let errors = unsafe { asar_geterrors(&mut count) };
            let errors = unsafe { std::slice::from_raw_parts(errors, count as usize) };
            let errors = errors.iter().map(ErrorData::from_raw).collect();
            PatchResult::Failure(errors)
        }
    }

    pub fn max_rom_size() -> i32 {
        unsafe { asar_maxromsize() }
    }

    pub fn errors() -> Vec<ErrorData> {
        let mut count: std::os::raw::c_int = 0;
        let errors = unsafe { asar_geterrors(&mut count) };
        let errors = unsafe { std::slice::from_raw_parts(errors, count as usize) };
        errors.iter().map(ErrorData::from_raw).collect()
    }

    pub fn warnings() -> Vec<ErrorData> {
        let mut count: std::os::raw::c_int = 0;
        let errors = unsafe { asar_getwarnings(&mut count) };
        let errors = unsafe { std::slice::from_raw_parts(errors, count as usize) };
        errors.iter().map(ErrorData::from_raw).collect()
    }

    pub fn prints() -> Vec<String> {
        let mut count: std::os::raw::c_int = 0;
        let prints = unsafe { asar_getprints(&mut count) };
        let prints = unsafe { std::slice::from_raw_parts(prints, count as usize) };
        prints.iter().map(|p| unsafe { std::ffi::CStr::from_ptr(*p) }.to_string_lossy().into_owned()).collect()
    }

    pub fn labels() -> Vec<LabelData> {
        let mut count: std::os::raw::c_int = 0;
        let labels = unsafe { asar_getalllabels(&mut count) };
        let labels = unsafe { std::slice::from_raw_parts(labels, count as usize) };
        labels.iter().map(LabelData::from_raw).collect()
    }

    pub fn label_value(name: &str) -> Option<i32> {
        let name = std::ffi::CString::new(name).unwrap();
        let value = unsafe { asar_getlabelval(name.as_ptr()) };
        if value == -1 {
            None
        } else {
            Some(value)
        }
    }

    pub fn define(name: &str) -> Option<String> {
        let name = std::ffi::CString::new(name).unwrap();
        let def = unsafe { asar_getdefine(name.as_ptr()) };
        if def.is_null() {
            None
        } else {
            Some(unsafe { std::ffi::CStr::from_ptr(def) }.to_string_lossy().into_owned())
        }
    }

    pub fn defines() -> Vec<DefineData> {
        let mut count: std::os::raw::c_int = 0;
        let defines = unsafe { asar_getalldefines(&mut count) };
        let defines = unsafe { std::slice::from_raw_parts(defines, count as usize) };
        defines.iter().map(DefineData::from_raw).collect()
    }

    pub fn resolve_defines(data: &str, learn_new: bool) -> String {
        unsafe {
            let data = std::ffi::CString::new(data).unwrap();
            let resolved = asar_resolvedefines(data.as_ptr(), learn_new);
            std::ffi::CStr::from_ptr(resolved).to_string_lossy().into_owned()
        }
    }

    pub fn math(math: &str) -> Result<f64, String> {
        let math = std::ffi::CString::new(math).unwrap();
        let err = std::ptr::null_mut();
        let result = unsafe { asar_math(math.as_ptr(), err) };
        if err.is_null() {
            Ok(result)
        } else {
            Err(unsafe { std::ffi::CStr::from_ptr(*err) }.to_string_lossy().into_owned())
        }
    }

    pub fn written_blocks() -> Vec<WrittenBlockData> {
        let mut count: std::os::raw::c_int = 0;
        let blocks = unsafe { asar_getwrittenblocks(&mut count) };
        let blocks = unsafe { std::slice::from_raw_parts(blocks, count as usize) };
        blocks.iter().map(WrittenBlockData::from_raw).collect()
    }

    pub fn mapper_type() -> Option<MapperType> {
        let raw = unsafe { asar_getmapper() };
        MapperType::from_raw(raw)
    }

    pub fn symbols_file(symboltype: SymbolType) -> String {
        let symboltype = match symboltype {
            SymbolType::WLA => "wla",
            SymbolType::NoCash => "nocash"
        };
        let symboltype = std::ffi::CString::new(symboltype).unwrap();
        unsafe {
            let file = asar_getsymbolsfile(symboltype.as_ptr());
            std::ffi::CStr::from_ptr(file).to_string_lossy().into_owned()
        }
    }
    
}