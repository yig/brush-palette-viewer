fn main() {
    println!("cargo:rerun-if-env-changed=BRUSH_PYTHON_HOME");
    println!("cargo:rerun-if-changed=python/constraint_optimizer.py");

    let Ok(home) = std::env::var("BRUSH_PYTHON_HOME") else {
        return;
    };

    let home = std::path::Path::new(&home);
    if !home.exists() {
        // Point the user at the setup script rather than printing a raw path error.
        panic!(
            "\n\nStandalone Python not found at '{}'.\n\
             Run the setup script once to download it:\n\n  \
             ./scripts/setup_python_standalone.sh\n\n\
             After that, `cargo build` will work without any extra exports.\n",
            home.display()
        );
    }

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    match target_os.as_str() {
        "macos" | "linux" => {
            let lib_dir = home.join("lib");
            // Help the linker find libpython inside the standalone distribution.
            println!("cargo:rustc-link-search=native={}", lib_dir.display());

            if target_os == "macos" {
                // Dev/test builds: find libpython in the vendor directory directly.
                println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir.display());
                // Distribution builds: find libpython relative to the installed binary.
                println!(
                    "cargo:rustc-link-arg=-Wl,-rpath,@executable_path/python-runtime/lib"
                );
            } else {
                println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir.display());
                // $ORIGIN is resolved by the dynamic linker at load time.
                println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/python-runtime/lib");
            }
        }
        "windows" => {
            // The import library (.lib) lives in libs/ on Windows.
            println!(
                "cargo:rustc-link-search=native={}",
                home.join("libs").display()
            );
            // Delay-load python3XX.dll so init_bundled_python() can call
            // SetDllDirectoryW before the DLL is first loaded.  Without
            // delay-load the PE loader would look for the DLL at process
            // start in the standard search path (exe dir, System32, PATH)
            // and not find it inside python-runtime/.
            let dll = python_dll_name(home);
            println!("cargo:rustc-link-arg=/DELAYLOAD:{}", dll);
            println!("cargo:rustc-link-lib=delayimp");
        }
        _ => {}
    }
}

/// Finds the versioned Python DLL (e.g. `python312.dll`) at the root of the
/// standalone distribution.  Falls back to `python312.dll` if detection fails.
fn python_dll_name(home: &std::path::Path) -> String {
    let Ok(entries) = std::fs::read_dir(home) else {
        return "python312.dll".to_string();
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        // Match python3XX.dll (version-specific) but not python3.dll (stable-ABI shim).
        if name.starts_with("python3")
            && name.ends_with(".dll")
            && name != "python3.dll"
        {
            return name;
        }
    }
    "python312.dll".to_string()
}
