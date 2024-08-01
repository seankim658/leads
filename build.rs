use core::panic;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const PDFIUM_VERSION: &str = "6569";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Starting build script...");

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    eprintln!("Output directory: {}", out_dir.display());

    let target_os = env::var("CARGO_CFG_TARGET_OS")?;
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH")?;
    eprintln!("Target OS: {}, Target Arch: {}", target_os, target_arch);

    let (url, filename) = get_pdfium_url(&target_os, &target_arch);
    eprintln!("PDFium URL: {}", url);
    eprintln!("PDFium filename: {}", filename);

    download_and_extract_pdfium(&out_dir, &url, &filename)?;

    // Tell Cargo to tell rustc to link the Library
    let lib_name = get_lib_name(&target_os);
    println!("cargo:rustc-env=PDFIUM_LIB_NAME={}", lib_name);
    let lib_dir = out_dir.join("lib");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    eprintln!("Library search path: {}", lib_dir.display());

    let lib_path = lib_dir.join(&lib_name);
    // Verify library extraction.
    if !lib_path.exists() {
        panic!(
            "Library not found at expected location: {}",
            lib_path.display()
        );
    }

    println!("cargo:rustc-link-lib=dylib=pdfium");

    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", out_dir.display());

    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}

fn get_pdfium_url(target_os: &str, target_arch: &str) -> (String, String) {
    let base_url = format!(
        "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F{}",
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

fn download_and_extract_pdfium(
    lib_dir: &Path,
    url: &str,
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Downloading from URL: {}", url);
    eprintln!("Saving to: {}", lib_dir.join(filename).display());

    // Download the file.
    let output = Command::new("curl")
        .args(&["-L", "-o", &lib_dir.join(filename).to_str().unwrap(), url])
        .output()?;

    if !output.status.success() {
        eprintln!("curl stderr: {}", String::from_utf8_lossy(&output.stderr));
        let error_message = format!(
            "Failed to download PDFium library. Curl exit status: {}. Stderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(error_message.into());
    }

    eprintln!("Download completed. Extracting...");

    // Extract the archive.
    let output = Command::new("tar")
        .args(&[
            "-xzf",
            &lib_dir.join(filename).to_str().unwrap(),
            "-C",
            lib_dir.to_str().unwrap(),
        ])
        .output()?;

    if !output.status.success() {
        eprintln!("tar stderr: {}", String::from_utf8_lossy(&output.stderr));
        return Err("Failed to extract PDFium library".into());
    }

    eprintln!("Extraction completed. Cleaning up...");

    // Clean up the archive file.
    fs::remove_file(lib_dir.join(filename))?;

    eprintln!("Cleanup completed.\nCurrent files:");
    for entry in fs::read_dir(lib_dir)? {
        let entry = entry?;
        eprintln!("  {}", entry.path().display());
    }

    Ok(())
}

fn get_lib_name(target_os: &str) -> String {
    match target_os {
        "windows" => "pdfium.dll".to_string(),
        "macos" => "libpdfium.dylib".to_string(),
        "linux" => "libpdfium.so".to_string(),
        _ => panic!("Unsupported target OS: {}", target_os),
    }
}
