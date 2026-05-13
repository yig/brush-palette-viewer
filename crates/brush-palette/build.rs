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

    // Help the linker find libpython inside the standalone distribution.
    println!(
        "cargo:rustc-link-search=native={}",
        home.join("lib").display()
    );

    let lib_dir = home.join("lib");
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    match target_os.as_str() {
        "macos" => {
            // Dev/test builds: find libpython in the vendor directory directly.
            println!(
                "cargo:rustc-link-arg=-Wl,-rpath,{}",
                lib_dir.display()
            );
            // Distribution builds: find libpython relative to the installed binary.
            println!(
                "cargo:rustc-link-arg=-Wl,-rpath,@executable_path/python-runtime/lib"
            );
        }
        "linux" => {
            // Dev/test builds.
            println!(
                "cargo:rustc-link-arg=-Wl,-rpath,{}",
                lib_dir.display()
            );
            // Distribution builds ($ORIGIN is resolved by the dynamic linker).
            println!(
                "cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/python-runtime/lib"
            );
        }
        _ => {}
    }
}
