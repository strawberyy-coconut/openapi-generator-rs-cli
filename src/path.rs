/// Returns the path to the bundled OpenAPI Generator CLI JAR.
/// This is always provided (the JAR is downloaded regardless of features).
pub fn openapi_generator_jar_path() -> &'static str {
    env!("OPENAPI_GENERATOR_JAR")
}

/// Returns the path to a `java` binary.  If the `bundled-jre` feature was
/// enabled at compile time this points to the bundled JRE's `java`;
/// otherwise it falls back to `"java"` (expecting it on `PATH`).
pub fn java_path() -> &'static str {
    #[cfg(feature = "bundled-jre")]
    {
        // SAFETY: bundled_jre_path() returns Some when this feature is active.
        let jre_home = bundled_jre_path().unwrap();
        // std::path::Path joining at compile time is awkward, so we use a
        // static which we initialise once at runtime.
        static JAVA: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        JAVA.get_or_init(|| {
            let p = std::path::Path::new(jre_home).join("bin/java");
            p.to_string_lossy().into_owned()
        })
    }
    #[cfg(not(feature = "bundled-jre"))]
    {
        "java"
    }
}

/// Returns the path to the bundled JRE, if the `bundled-jre` feature was
/// enabled at compile time.
pub fn bundled_jre_path() -> Option<&'static str> {
    #[cfg(feature = "bundled-jre")]
    {
        Some(env!("JRE_HOME"))
    }
    #[cfg(not(feature = "bundled-jre"))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jar_path_is_non_empty() {
        let path = openapi_generator_jar_path();
        assert!(!path.is_empty(), "JAR path should not be empty");
        eprintln!("JAR path: {path}");
    }

    #[test]
    fn jar_exists_on_disk() {
        let path = openapi_generator_jar_path();
        assert!(
            std::path::Path::new(path).exists(),
            "JAR does not exist: {path}"
        );
    }

    #[test]
    fn bundled_jre_path_is_set_when_feature_enabled() {
        #[cfg(feature = "bundled-jre")]
        {
            let path = bundled_jre_path()
                .expect("JRE path should be Some when bundled-jre is enabled");
            eprintln!("JRE path: {path}");
            assert!(
                std::path::Path::new(path).exists(),
                "JRE path does not exist: {path}"
            );
        }

        #[cfg(not(feature = "bundled-jre"))]
        {
            assert!(bundled_jre_path().is_none());
        }
    }

    #[test]
    fn java_path_points_to_bundled_jre_when_feature_enabled() {
        #[cfg(feature = "bundled-jre")]
        {
            let java = java_path();
            assert!(
                std::path::Path::new(java).exists(),
                "Java binary does not exist: {java}"
            );

            let output = std::process::Command::new(java)
                .arg("-version")
                .output()
                .expect("failed to execute java");
            assert!(output.status.success(), "java -version failed");
            let ver = String::from_utf8_lossy(&output.stderr);
            eprintln!("Java version:\n{ver}");
        }

        #[cfg(not(feature = "bundled-jre"))]
        {
            assert!(!java_path().is_empty());
        }
    }
}

