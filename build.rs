fn git_info() -> (String, u32, bool, String) {
    let git = |args: &[&str]| {
        std::process::Command::new("git")
            .args(args)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
    };
    let commit = git(&["rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "unknown".into());
    let count = git(&["rev-list", "--count", "HEAD"])
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let dirty = git(&["status", "--porcelain"])
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    let date = git(&["log", "-1", "--format=%cs"]).unwrap_or_default();
    (commit, count, dirty, date)
}

fn main() {
    println!("cargo:rerun-if-changed=app.manifest");
    println!("cargo:rerun-if-changed=kurmon.ico");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
    if std::path::Path::new(".git/HEAD").exists() {
        println!("cargo:rerun-if-changed=.git/HEAD");
        println!("cargo:rerun-if-changed=.git/index");
    }

    let pkg_version = std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.1.0".into());
    let (commit, count, dirty, date) = git_info();
    let dirty_suffix = if dirty { "*" } else { "" };
    let full = if date.is_empty() {
        format!("{pkg_version} (build {count} · {commit}{dirty_suffix})")
    } else {
        format!("{pkg_version} (build {count} · {commit}{dirty_suffix} · {date})")
    };
    let short = format!("v{pkg_version} (b{count} · {commit}{dirty_suffix})");
    println!("cargo:rustc-env=KURMON_VERSION={full}");
    println!("cargo:rustc-env=KURMON_SHORT_VERSION={short}");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let manifest = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap())
            .join("app.manifest");
        let sdk = std::env::var_os("WindowsSdkDir")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(std::env::var_os("ProgramFiles(x86)").unwrap())
                    .join("Windows Kits/10")
            });
        let host = match std::env::consts::ARCH {
            "x86_64" => "x64",
            "aarch64" => "arm64",
            _ => "x86",
        };
        let rc = std::fs::read_dir(sdk.join("bin"))
            .expect("Windows SDK bin directory not found")
            .filter_map(Result::ok)
            .map(|entry| entry.path().join(host).join("rc.exe"))
            .filter(|path| path.is_file())
            .max()
            .expect("Windows SDK resource compiler not found");

        let parts: Vec<u32> = pkg_version
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        let major = parts.first().copied().unwrap_or(0);
        let minor = parts.get(1).copied().unwrap_or(1);
        let patch = parts.get(2).copied().unwrap_or(0);

        let out_dir = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
        let rc_path = out_dir.join("kurmon.rc");
        let rc_content = format!(
            r#"101 ICON "kurmon.ico"

1 VERSIONINFO
 FILEVERSION {major},{minor},{patch},{count}
 PRODUCTVERSION {major},{minor},{patch},{count}
 FILEFLAGSMASK 0x3fL
 FILEFLAGS 0
 FILEOS 0x40004L
 FILETYPE 0x1L
 FILESUBTYPE 0
BEGIN
    BLOCK "StringFileInfo"
    BEGIN
        BLOCK "040904b0"
        BEGIN
            VALUE "FileDescription", "Kurmon"
            VALUE "FileVersion", "{major}.{minor}.{patch}.{count}"
            VALUE "InternalName", "Kurmon"
            VALUE "OriginalFilename", "Kurmon.exe"
            VALUE "ProductName", "Kurmon"
            VALUE "ProductVersion", "{pkg_version}+{commit}{dirty_suffix}"
        END
    END
    BLOCK "VarFileInfo"
    BEGIN
        VALUE "Translation", 0x0409, 1200
    END
END
"#
        );
        std::fs::write(&rc_path, rc_content).expect("failed to write generated kurmon.rc");

        let resource = out_dir.join("kurmon.res");
        let status = std::process::Command::new(rc)
            .current_dir(std::env::var_os("CARGO_MANIFEST_DIR").unwrap())
            .arg("/nologo")
            .arg(format!("/fo{}", resource.display()))
            .arg(&rc_path)
            .status()
            .expect("failed to run Windows resource compiler");
        assert!(status.success(), "Windows resource compiler failed");
        println!(
            "cargo:rustc-link-arg-bin=Kurmon={}",
            resource.display()
        );
        println!("cargo:rustc-link-arg-bin=Kurmon=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg-bin=Kurmon=/MANIFESTINPUT:{}",
            manifest.display()
        );
    }
}
