use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // ── Always download the OpenAPI Generator CLI JAR ──────────────
    download_openapi_generator_jar(&out_dir);

    // ── Conditionally download the JRE (bundled-jre feature) ───────
    if env::var("CARGO_FEATURE_BUNDLED_JRE").is_ok() {
        download_jre(&out_dir);
    }

    // Re-run build.rs only if it changes
    println!("cargo:rerun-if-changed=build.rs");
}

/// Download the latest OpenAPI Generator CLI JAR from Maven Central.
fn download_openapi_generator_jar(out_dir: &Path) {
    let version = get_latest_openapi_generator_version();
    println!("cargo:warning=Latest OpenAPI Generator version: {version}");

    let jar_name = format!("openapi-generator-cli-{version}.jar");
    let jar_path = out_dir.join(&jar_name);

    // Skip download if already cached
    if jar_path.exists() {
        println!("cargo:warning=OpenAPI Generator JAR already cached at: {}", jar_path.display());
        println!("cargo:rustc-env=OPENAPI_GENERATOR_JAR={}", jar_path.display());
        return;
    }

    let url = format!(
        "https://repo1.maven.org/maven2/org/openapitools/openapi-generator-cli/{version}/{jar_name}"
    );

    println!("cargo:warning=Downloading OpenAPI Generator JAR from Maven Central...");
    download_file(&url, &jar_path);

    // Also write a symlink-friendly "latest" name
    let latest_link = out_dir.join("openapi-generator-cli.jar");
    let _ = fs::remove_file(&latest_link);
    fs::copy(&jar_path, &latest_link).ok();

    println!("cargo:warning=OpenAPI Generator JAR downloaded to: {}", jar_path.display());
    println!("cargo:rustc-env=OPENAPI_GENERATOR_JAR={}", jar_path.display());
}

/// Get the latest OpenAPI Generator version from the Maven metadata XML.
fn get_latest_openapi_generator_version() -> String {
    let url = "https://repo1.maven.org/maven2/org/openapitools/openapi-generator-cli/maven-metadata.xml";
    let output = Command::new("curl")
        .args(["-sS", "--fail", url])
        .output()
        .expect("Failed to execute curl — is it installed?");

    if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            panic!("Failed to fetch Maven metadata from {url}: {stderr}");
        }

    let body = String::from_utf8(output.stdout).expect("Maven metadata is not valid UTF-8");

    // Simple XML scan: find <latest>X.Y.Z</latest>
    let open_tag = "<latest>";
    let close_tag = "</latest>";
    let start = body
        .find(open_tag)
        .unwrap_or_else(|| panic!("Could not find {open_tag} in Maven metadata"));
    let end = body
        .find(close_tag)
        .unwrap_or_else(|| panic!("Could not find {close_tag} in Maven metadata"));

    body[start + open_tag.len()..end].to_string()
}

/// Download the latest JRE from Adoptium.
fn download_jre(out_dir: &Path) {
    println!("cargo:warning=Downloading latest JRE (bundled-jre feature enabled)...");

    let (adopt_os, adopt_arch) = detect_platform();

    let version = get_latest_jre_version();
    println!("cargo:warning=Latest JRE feature release: {version}");

    let url = format!(
        "https://api.adoptium.net/v3/binary/latest/{version}/ga/{adopt_os}/{adopt_arch}/jre/hotspot/normal/eclipse"
    );

    let ext = if adopt_os == "windows" { "zip" } else { "tar.gz" };
    let archive_path = out_dir.join(format!("jre.{ext}"));

    download_file(&url, &archive_path);

    println!("cargo:rustc-env=JRE_PATH={}", archive_path.display());
}

/// Detect the OS and architecture strings used by Adoptium's API.
fn detect_platform() -> (&'static str, &'static str) {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    match (os, arch) {
        ("linux", "x86_64") => ("linux", "x64"),
        ("linux", "aarch64") => ("linux", "aarch64"),
        ("macos", "x86_64") => ("mac", "x64"),
        ("macos", "aarch64") => ("mac", "aarch64"),
        ("windows", "x86_64") => ("windows", "x64"),
        ("windows", "aarch64") => ("windows", "aarch64"),
        _ => panic!("Unsupported platform: {os}/{arch}"),
    }
}

/// Fetch the most recent feature release version from the Adoptium API.
fn get_latest_jre_version() -> u32 {
    let url = "https://api.adoptium.net/v3/info/available_releases";
    let output = Command::new("curl")
        .args(["-sS", "--fail", url])
        .output()
        .expect("Failed to execute curl — is it installed?");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("Failed to fetch available releases from {url}: {stderr}");
    }

    let body = String::from_utf8(output.stdout).expect("API response is not valid UTF-8");

    // Simple JSON scan without pulling in serde as a build dependency.
    let key = "\"most_recent_feature_release\":";
    let pos = body
        .find(key)
        .unwrap_or_else(|| panic!("Could not find {key} in API response"));

    let after_key = &body[pos + key.len()..];
    let num_str: String = after_key
        .chars()
        .skip_while(|c| c.is_ascii_whitespace()) // skip space/colon whitespace
        .take_while(|c| c.is_ascii_digit())
        .collect();

    num_str
        .parse::<u32>()
        .expect("Failed to parse version number from API response")
}

/// Download a file from `url` and save it to `dest`.
fn download_file(url: &str, dest: &Path) {
    let status = Command::new("curl")
        .args([
            "-sS",
            "--fail",
            "-L",
            "-o",
            &dest.to_string_lossy(),
            url,
        ])
        .status()
        .expect("Failed to execute curl — is it installed?");

    if !status.success() {
        panic!("Failed to download JRE from {url}");
    }
}




