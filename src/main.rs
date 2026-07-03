
use std::path::PathBuf;
use std::process::Command;

#[cfg(feature = "bundled-jre")]
const JRE_BYTES: &[u8] = include_bytes!(env!("JRE_PATH"));

const GENERATOR_JAR_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/openapi-generator-cli.jar"));

/// Directory next to the executable where extracted assets live.
fn oas_gen_dir() -> PathBuf {
    let exe = std::env::current_exe().expect("failed to get executable path");
    exe.parent().expect("executable has no parent").join("oas-gen")
}

#[cfg(feature = "bundled-jre")]
fn extract_archive() -> String {
    use std::io::Cursor;

    let base = oas_gen_dir();
    let java_path = base.join("jre/bin/java");

    // Already extracted on a previous run — skip rebuilding
    if java_path.exists() {
        return java_path.to_string_lossy().into_owned();
    }

    let jre_path = base.join("jre");

    if cfg!(target_os = "windows") {
        // ZIP archive — iterate entries and strip the top-level wrapper directory
        let cursor = Cursor::new(JRE_BYTES);
        let mut zip = zip::ZipArchive::new(cursor).expect("Failed to read JRE ZIP archive");

        for i in 0..zip.len() {
            let mut entry = zip.by_index(i).expect("Failed to read ZIP entry");
            let Some(path) = entry.enclosed_name() else { continue };

            // Strip the top-level directory component (e.g. "jdk-26.0.1+8-jre/")
            let stripped: PathBuf = path.components().skip(1).collect();
            if stripped.as_os_str().is_empty() {
                continue; // skip directory entries themselves
            }

            let target = jre_path.join(&stripped);
            if entry.is_dir() {
                std::fs::create_dir_all(&target).ok();
            } else {
                if let Some(parent) = target.parent() {
                    std::fs::create_dir_all(parent).ok();
                }
                let mut out = std::fs::File::create(&target)
                    .expect("Failed to create output file during JRE extraction");
                std::io::copy(&mut entry, &mut out)
                    .expect("Failed to write file during JRE extraction");
            }
        }
    } else {
        // TAR.GZ archive — iterate entries and strip the top-level wrapper directory
        use std::os::unix::fs::PermissionsExt;

        let cursor = Cursor::new(JRE_BYTES);
        let tar = flate2::read::GzDecoder::new(cursor);
        let mut archive = tar::Archive::new(tar);

        std::fs::create_dir_all(&jre_path).ok();

        for entry_result in archive.entries().expect("Failed to read TAR archive entries") {
            let mut entry = entry_result.expect("Failed to read TAR entry");
            let path = entry.path().expect("Failed to read entry path").into_owned();

            // Strip the top-level directory component (e.g. "jdk-26.0.1+8-jre/")
            let stripped: PathBuf = path.components().skip(1).collect();
            if stripped.as_os_str().is_empty() {
                continue;
            }

            let target = jre_path.join(&stripped);
            if entry.header().entry_type().is_dir() {
                std::fs::create_dir_all(&target).ok();
            } else {
                if let Some(parent) = target.parent() {
                    std::fs::create_dir_all(parent).ok();
                }
                let mut out = std::fs::File::create(&target)
                    .expect("Failed to create output file during JRE extraction");
                std::io::copy(&mut entry, &mut out)
                    .expect("Failed to write file during JRE extraction");
                // Preserve executable permissions from the archive entry
                if let Ok(mode) = entry.header().mode() {
                    if mode & 0o111 != 0 {
                        std::fs::set_permissions(&target, std::fs::Permissions::from_mode(mode))
                            .ok();
                    }
                }
            }
        }
    }

    java_path.to_string_lossy().into_owned()
}


fn write_generator_jar() -> String {
    use std::fs;

    let jar_path = oas_gen_dir().join("openapi-generator-cli.jar");

    if !jar_path.exists() {
        fs::write(&jar_path, GENERATOR_JAR_BYTES).expect("Failed to write OpenAPI Generator JAR");
    }

    jar_path.to_string_lossy().into_owned()
}

fn main() {
    let base = oas_gen_dir();
    std::fs::create_dir_all(&base).ok();

    let (java, jar) = if cfg!(feature = "bundled-jre") {
        (extract_archive(), write_generator_jar())
    } else {
        ("java".into(), write_generator_jar())
    };

    let args: Vec<_> = std::env::args_os().skip(1).collect();

    let status = Command::new(&java)
        .arg("-jar")
        .arg(&jar)
        .args(&args)
        .status()
        .expect("Failed to launch Java — is it installed?");

    std::process::exit(status.code().unwrap_or(1));
}