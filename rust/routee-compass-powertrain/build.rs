use std::{
    env,
    path::{Path, PathBuf},
};

#[cfg(feature = "onnx")]
const ORT_EXTRACT_DIR: &str = "onnx-runtime";

#[cfg(feature = "onnx")]
fn fetch_file(source_url: &str) -> Vec<u8> {
    let resp = ureq::get(source_url)
        .timeout(std::time::Duration::from_secs(1800))
        .call()
        .unwrap_or_else(|err| panic!("failed to download {source_url}: {err:?}"));

    let len = resp
        .header("Content-Length")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap();
    let mut reader = resp.into_reader();
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).unwrap();
    assert_eq!(buffer.len(), len);
    buffer
}

#[cfg(feature = "onnx")]
fn extract_tgz(buf: &[u8], output: &Path) {
    let buf: std::io::BufReader<&[u8]> = std::io::BufReader::new(buf);
    let tar = flate2::read::GzDecoder::new(buf);
    let mut archive = tar::Archive::new(tar);
    archive.unpack(output).unwrap();
}

#[cfg(feature = "onnx")]
fn extract_zip(buf: &[u8], output: &Path) {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(buf)).unwrap();
    archive.extract(output).unwrap();
}

enum ArchiveType {
    Tgz,
    Zip,
}

fn main() {
    if !cfg!(feature = "onnx") {
        return;
    }

    let (archive_type, url) = match env::var("TARGET").unwrap().as_str() {
        "x86_64-apple-darwin" => (ArchiveType::Tgz, "https://github.com/supertone-inc/onnxruntime-build/releases/download/v1.15.1/onnxruntime-osx-universal2-static_lib-1.15.1.tgz"),
        "x86_64-unknown-linux-gnu" => (ArchiveType::Tgz, "https://github.com/supertone-inc/onnxruntime-build/releases/download/v1.15.1/onnxruntime-linux-x64-static_lib-1.15.1.tgz"),
        "aarch64-unknown-linux-gnu" => (ArchiveType::Tgz, "https://github.com/supertone-inc/onnxruntime-build/releases/download/v1.15.1/onnxruntime-linux-aarch64-static_lib-1.15.1.tgz"),
        "x86_64-pc-windows-msvc" => (ArchiveType::Zip,"https://github.com/supertone-inc/onnxruntime-build/releases/download/v1.15.1/onnxruntime-win-x64-static_lib-1.15.1.zip"),
        "x86_64-pc-windows-gnu" => (ArchiveType::Zip, "https://github.com/supertone-inc/onnxruntime-build/releases/download/v1.15.1/onnxruntime-win-x64-static_lib-1.15.1.zip"),
        t => panic!("Unsupported target:  {t}"),
    };

    let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(ORT_EXTRACT_DIR);
    if !out_dir.exists() {
        let buf = fetch_file(url);
        match archive_type {
            ArchiveType::Tgz => extract_tgz(&buf, &out_dir),
            ArchiveType::Zip => extract_zip(&buf, &out_dir),
        }
    }
}
