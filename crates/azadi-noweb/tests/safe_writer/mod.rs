use azadi_noweb::safe_writer::SafeFileWriter;
use azadi_noweb::AzadiError;
use std::fs;
use std::io::Write;
use tempfile::TempDir;

#[test]
fn test_modification_detection() {
    let tmp = TempDir::new().unwrap();
    let gen = tmp.path().join("gen");
    let wrk = tmp.path().join("wrk");
    {
        let mut writer = SafeFileWriter::new(&gen, &wrk, true);
        let f = "detect.txt";
        let priv_file = writer.before_write(f).unwrap();
        fs::write(&priv_file, "original text").unwrap();
        writer.after_write(f).unwrap();
    }
    let final_file = gen.join("detect.txt");
    {
        let mut ext = fs::OpenOptions::new()
            .append(true)
            .open(&final_file)
            .unwrap();
        writeln!(ext, "\nEXTERNAL CHANGE").unwrap();
    }
    {
        let mut writer2 = SafeFileWriter::new(&gen, &wrk, true);
        let result = writer2.before_write("detect.txt");
        assert!(result.is_err(), "Should fail with ModifiedExternally");
        if let Err(AzadiError::ModifiedExternally(msg)) = result {
            assert!(msg.contains("detect.txt"));
        } else {
            panic!("Expected ModifiedExternally error");
        }
    }
}

#[test]
fn test_nested_directory_creation() {
    let tmp = TempDir::new().unwrap();
    let gen = tmp.path().join("gen");
    let wrk = tmp.path().join("wrk");
    let mut writer = SafeFileWriter::new(&gen, &wrk, true);
    let nested = "deep/dir/test.txt";
    let privp = writer.before_write(nested).unwrap();
    fs::write(&privp, "some nested text").unwrap();
    writer.after_write(nested).unwrap();
    let final_file = gen.join(nested);
    assert!(final_file.exists());
    let c = fs::read_to_string(final_file).unwrap();
    assert_eq!(c, "some nested text");
}
