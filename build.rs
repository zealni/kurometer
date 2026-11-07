fn main() {
    println!("cargo:rerun-if-changed=app.manifest");
    println!("cargo:rerun-if-changed=kurmon.rc");
    println!("cargo:rerun-if-changed=kurmon.ico");
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
        let resource = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap())
            .join("kurmon.res");
        let status = std::process::Command::new(rc)
            .current_dir(std::env::var_os("CARGO_MANIFEST_DIR").unwrap())
            .arg("/nologo")
            .arg(format!("/fo{}", resource.display()))
            .arg("kurmon.rc")
            .status()
            .expect("failed to run Windows resource compiler");
        assert!(status.success(), "Windows resource compiler failed");
        println!(
            "cargo:rustc-link-arg-bin=taskbar-system-monitor={}",
            resource.display()
        );
        println!("cargo:rustc-link-arg-bin=taskbar-system-monitor=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg-bin=taskbar-system-monitor=/MANIFESTINPUT:{}",
            manifest.display()
        );
    }
}
