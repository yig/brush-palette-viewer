fn main() {
    println!("cargo:rerun-if-env-changed=BRUSH_PYTHON_HOME");
    println!("cargo:rerun-if-changed=python/constraint_optimizer.py");

    let Ok(home) = std::env::var("BRUSH_PYTHON_HOME") else {
        // Dev mode: PyO3 finds Python normally via PYO3_PYTHON / system python3.
        return;
    };

    let home = std::path::Path::new(&home);
    assert!(
        home.exists(),
        "BRUSH_PYTHON_HOME does not exist: {}",
        home.display()
    );

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
