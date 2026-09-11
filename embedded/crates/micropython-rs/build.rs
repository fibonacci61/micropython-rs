use std::path::{Path, PathBuf};

use micropython_build::config::process_mp_config;
use micropython_manifest::{ManifestPaths, find_manifest_from, parse_manifest};

fn native_sources(src_dir: &Path) -> Result<(Vec<PathBuf>, Vec<PathBuf>), std::io::Error> {
    let mut sources = Vec::new();
    let mut inputs = Vec::new();

    for entry in src_dir.read_dir()? {
        let path = entry?.path();
        if !path.is_file() {
            continue;
        }

        match path.extension().and_then(|extension| extension.to_str()) {
            Some("c") => {
                sources.push(path.clone());
                inputs.push(path);
            }
            Some("h") => inputs.push(path),
            _ => {}
        }
    }

    // deterministic
    sources.sort();
    inputs.sort();
    Ok((sources, inputs))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cargo_manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let src_dir = cargo_manifest_dir.join("src");

    let wrapper_path = src_dir.join("wrapper.h");
    let (c_sources, native_inputs) = native_sources(&src_dir)?;

    let ManifestPaths {
        path: manifest_path,
        dir: manifest_dir,
    } = find_manifest_from(cargo_manifest_dir)?;
    let manifest = parse_manifest(&manifest_path)?;

    let mp_dir = manifest.micropython.path_canonicalized(&manifest_path)?;
    let port_dir = manifest.port.path_canonicalized(&manifest_path)?;

    process_mp_config(&mp_dir, &port_dir)?.emit_cargo_directives();

    let header_dir = manifest_dir.join("micropython-rs/generated");

    // TODO: process depfile
    cc::Build::new()
        .includes([&port_dir, &mp_dir, &header_dir, &src_dir])
        .files(&c_sources)
        .warnings(true)
        .compile("mprsshims");

    println!("cargo::rerun-if-changed={}", port_dir.display());
    println!("cargo::rerun-if-changed={}", mp_dir.display());
    println!("cargo::rerun-if-changed={}", header_dir.display());
    for path in native_inputs {
        println!("cargo::rerun-if-changed={}", path.display());
    }

    let bindings = bindgen::builder()
        .header(wrapper_path.into_string().unwrap())
        .use_core()
        .wrap_unsafe_ops(true)
        .allowlist_type("mprs_.*")
        .allowlist_function("mprs_.*")
        .allowlist_var("mprs_.*")
        .clang_arg("-I")
        .clang_arg(mp_dir.into_string().unwrap())
        .clang_arg("-I")
        .clang_arg(port_dir.into_string().unwrap())
        .clang_arg("-I")
        .clang_arg(header_dir.into_string().unwrap())
        .generate()?;
    bindings.write_to_file(PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("shims.rs"))?;

    micropython_build::scan::emit_scan_cfgs(&manifest_dir)?;

    Ok(())
}
