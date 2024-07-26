use crate::asar::{
    self, AdvancedPatchOptions, MemoryFileData, PatchOption, PatchResult
};

#[test]
fn test_api_version() {
    let apiversion = asar::api_version();
    assert_eq!(apiversion, 303);
}

#[test]
fn test_version() {
    let version = asar::version();
    assert_eq!(version, 10901);
}

#[test]
fn test_math() {
    let result = asar::math("1+1");
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result, 2f64);
}

#[test]
fn test_maxromsize() {
    let result = asar::max_rom_size();
    assert_eq!(result, 16*1024*1024);
}

#[test]
fn test_patch() {
    let romdata = vec![0x00, 0x00, 0x00, 0x00];
    let patchdata = r#"incsrc "include.asm"
org $008000
lda !test
%testmacro()"#
        .into();
    let includedata = r#"macro testmacro()
    sta $19
endmacro"#
        .into();
    let options = AdvancedPatchOptions::new(romdata, "test.asm".into())
        .option(PatchOption::Include("includefiles".into()))
        .option(PatchOption::Define("test".into(), "$18".into()))
        .option(PatchOption::Warning("Wrelative_path_used".into(), false))
        .option(PatchOption::MemoryFile(
            "test.asm".into(),
            MemoryFileData::Text(patchdata),
        ))
        .option(PatchOption::MemoryFile(
            "includefiles/include.asm".into(),
            MemoryFileData::Text(includedata),
        ));
    let result = asar::patch_ex(options);
    assert!(matches!(result, PatchResult::Success(_, _)));
    let expected: [u8; 4] = [0xA5, 0x18, 0x85, 0x19];
    match result {
        PatchResult::Success(data, warnings) => {
            assert_eq!(data[0..4], expected);
            assert_eq!(warnings.len(), 1);
            assert_eq!(warnings[0].errid, 1001);
        }
        _ => panic!("Expected success"),
    }
}

#[test]
fn test_get_labels() {
    let romdata = vec![];
    let patchdata = "org $008000\nlabel:";
    let options = AdvancedPatchOptions::new(romdata, "test.asm".into()).option(
        PatchOption::MemoryFile("test.asm".into(), MemoryFileData::Text(patchdata.into())),
    );
    let (result, labels) = asar::with_asar_lock(|| {
        let result = asar::patch_ex(options);
        let labels = asar::labels();
        (result, labels)
    });
    assert!(matches!(result, PatchResult::Success(_, _)));
    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0].name, "label");
    assert_eq!(labels[0].location, 0x8000);
}

#[test]
fn test_proc_macro() {
    use asar::use_asar_global_lock;
    #[use_asar_global_lock]
    fn test() {
        assert!(asar::reset());
    }
    test();
}
