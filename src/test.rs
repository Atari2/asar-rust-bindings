use crate::{AdvancedPatchOptions, PatchOption, PatchResult};

use crate as asar;

#[cfg(feature = "thread-safe")]
use crate::{Patcher, RomData};

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
    assert_eq!(result, 16 * 1024 * 1024);
}

#[test]
fn test_patch() {
    let romdata = vec![0x00, 0x00, 0x00, 0x00].into();
    let patchdata = r#"incsrc "include.asm"
org $008000
lda !test
%testmacro()"#;
    let includedata = r#"macro testmacro()
    sta $19
endmacro"#;
    let options = AdvancedPatchOptions::new()
        .option(PatchOption::Include("includefiles".into()))
        .option(PatchOption::Define("test".into(), "$18".into()))
        .option(PatchOption::Warning("Wrelative_path_used".into(), false))
        .option(PatchOption::MemoryFile(
            "test.asm".into(),
            patchdata.into(),
        ))
        .option(PatchOption::MemoryFile(
            "includefiles/include.asm".into(),
            includedata.into(),
        ));
    let result = asar::patching::patch_ex(romdata, "test.asm", options);
    assert!(matches!(result, PatchResult::Success(_, _)));
    let expected: [u8; 4] = [0xA5, 0x18, 0x85, 0x19];
    match result {
        PatchResult::Success(data, warnings) => {
            assert_eq!(data.data[0..4], expected);
            assert_eq!(warnings.len(), 1);
            assert_eq!(warnings[0].errid, 1001);
        }
        _ => panic!("Expected success"),
    }
}

#[test]
#[cfg(feature = "thread-safe")]
fn test_get_labels() {
    let romdata = vec![].into();
    let patchdata = "org $008000\nlabel:";
    let options = AdvancedPatchOptions::new().option(PatchOption::MemoryFile(
        "test.asm".into(),
        patchdata.into(),
    ));
    let (result, labels) = asar::with_asar_lock(|| {
        let result = asar::patching::patch_ex(romdata, "test.asm", options);
        let labels = asar::patching::labels();
        (result, labels)
    });
    assert!(matches!(result, PatchResult::Success(_, _)));
    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0].name, "label");
    assert_eq!(labels[0].location, 0x8000);
}

#[test]
#[cfg(feature = "thread-safe")]
fn test_proc_macro() {
    use asar::use_asar_global_lock;
    #[use_asar_global_lock]
    fn test() {
        assert!(asar::patching::reset());
    }
    test();
}

#[test]
#[cfg(feature = "thread-safe")]
fn test_interface() {
    let max_size = asar::max_rom_size() as usize;
    // we give the buffer "MAX_ROM_SIZE" bytes of space but tell asar that the ROM is 0 bytes long.
    // this is to test that asar will correctly resize the rom length to 5.
    let romdata = RomData::new(vec![0x00; max_size], 0);
    let mut patcher = Patcher::new();
    // $0D8000 maps to 0x68000 on a headered rom
    let pcaddress = 0x68000;
    let patchdata = r#"
org $008000
    db $00, $01, $02, $03
    nop
org $0D8000
    label:
    nop  
"#;
    patcher.option(PatchOption::MemoryFile(
        "test.asm".into(),
        patchdata.into(),
    ));
    let patcher2 = patcher.clone();
    let patcher3 = patcher.clone();

    // first application should succeed
    let result = patcher.apply(romdata.clone(), "test.asm");
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.success());

    // if we try to apply while the first application is still not marked as done, it should fail
    let result2 = patcher2.apply(romdata, "test.asm");
    assert!(result2.is_err());

    let labels = result.labels();
    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0].name, "label");
    assert_eq!(labels[0].location, 0x0D8000);
    assert_eq!(result.label_value("label"), Some(0x0D8000));
    assert_eq!(result.mapper_type(), Some(asar::MapperType::lorom));
    let written_blocks = result.written_blocks();
    assert_eq!(written_blocks.len(), 3);
    assert_eq!(written_blocks[0].snesoffset, 0x808000);

    // consume the result
    let romdata = result.romdata();
    assert_eq!(romdata.data[0..5], [0x00, 0x01, 0x02, 0x03, 0xEA]);
    assert_eq!(romdata.data[pcaddress], 0xEA);
    assert_eq!(romdata.length, pcaddress + 1);

    // after consuming the result, we should be able to apply again
    let result3 = patcher3.apply(romdata, "test2.asm");
    assert!(result3.is_ok());
}
