use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const PDFIUM_VERSION: &str = "128.0.6611.0";

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let lib_dir = out_dir.join("lib");
    fs::create_dir_all(&lib_dir).unwrap();

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    // Check if PDFium is already installed.
    if is_pdfium_installed() {
        println!("PDFium is already installed on the system.");
        return;
    }

    let (url, filename) = get_pdfium_url(&target_os, &target_arch);
    download_and_extract_pdfium(&lib_dir, &url, &filename);

    // Tell Cargo to tell rustc to link the Library
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=dylib=pdfium");

    // Tell Cargo to re-run this script if the build script changes
    println!("cargo:rerun-if-changed=build.rs");
}

/// Check common installation paths to see if PDFium is already installed.
fn is_pdfium_installed() -> bool {
    let common_paths = vec![
        "/usr/lib/libpdfium.so",
        "/usr/local/lib/libpdfium.so",
        "C:\\Program Files\\PDFium\\pdfium.dll",
        "/Library/Frameworks/PDFium.framework/PDFium",
    ];

    for path in common_paths {
        if Path::new(path).exists() {
            return true;
        }
    }

    false
}

fn get_pdfium_url(target_os: &str, target_arch: &str) -> (String, String) {
    let base_url = format!(
        "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/{}",
        PDFIUM_VERSION
    );
    match (target_os, target_arch) {
        ("windows", "x86_64") => (
            format!("{}/pdfium-win-x64.tgz", base_url),
            "pdfium-win-x64.tgz".to_string(),
        ),
        ("windows", "x86") => (
            format!("{}/pdfium-win-x86.tgz", base_url),
            "pdfium-win-x86.tgz".to_string(),
        ),
        ("windows", "aarch64") => (
            format!("{}/pdfium-win-arm64.tgz", base_url),
            "pdfium-win-arm64.tgz".to_string(),
        ),
        ("linux", "x86_64") => (
            format!("{}/pdfium-linux-x64.tgz", base_url),
            "pdfium-linux-x64.tgz".to_string(),
        ),
        ("linux", "x86") => (
            format!("{}/pdfium-linux-x86.tgz", base_url),
            "pdfium-linux-x86.tgz".to_string(),
        ),
        ("linux", "aarch64") => (
            format!("{}/pdfium-linux-arm64.tgz", base_url),
            "pdfium-linux-arm64.tgz".to_string(),
        ),
        ("macos", "x86_64") => (
            format!("{}/pdfium-mac-x64.tgz", base_url),
            "pdfium-mac-x64.tgz".to_string(),
        ),
        ("macos", "aarch64") => (
            format!("{}/pdfium-mac-arm64.tgz", base_url),
            "pdfium-mac-arm64.tgz".to_string(),
        ),
        _ => panic!("Unsupported target: {}-{}", target_os, target_arch),
    }
}

fn download_and_extract_pdfium(lib_dir: &Path, url: &str, filename: &str) {
    let output = Command::new("curl")
        .args(&["-L", "-o", &lib_dir.join(filename).to_str().unwrap(), url])
        .output()
        .expect("Failed to execute curl");

    if !output.status.success() {
        panic!("Failed to download PDFium library");
    }

    // Extract and download the archive.
    Command::new("tar")
        .args(&[
            "-xzf",
            &lib_dir.join(filename).to_str().unwrap(),
            "-C",
            lib_dir.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to extract PDFium library");

    // Clean up the archive file.
    fs::remove_file(lib_dir.join(filename)).unwrap();
}
