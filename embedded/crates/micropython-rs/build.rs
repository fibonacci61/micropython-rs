use std::path::PathBuf;

use micropython_build::config::process_mp_config;
use micropython_manifest::{ManifestPaths, find_manifest_from, parse_manifest};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cargo_manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let src_dir = cargo_manifest_dir.join("src");

    let wrapper_path = src_dir.join("wrapper.h");
    let nlrshims_c_path = src_dir.join("nlrshims.c");
    let nlrshims_h_path = src_dir.join("nlrshims.h");
    let staticshims_h_path = src_dir.join("staticshims.h");
    let staticshims_c_path = src_dir.join("staticshims.h");

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
        .files([&nlrshims_c_path, &staticshims_c_path])
        .warnings(true)
        .compile("mprsshims");

    println!("cargo::rerun-if-changed={}", port_dir.display());
    println!("cargo::rerun-if-changed={}", mp_dir.display());
    println!("cargo::rerun-if-changed={}", header_dir.display());
    println!("cargo::rerun-if-changed={}", wrapper_path.display());
    println!("cargo::rerun-if-changed={}", nlrshims_h_path.display());
    println!("cargo::rerun-if-changed={}", nlrshims_c_path.display());
    println!("cargo::rerun-if-changed={}", staticshims_h_path.display());
    println!("cargo::rerun-if-changed={}", staticshims_c_path.display());

    let bindings = bindgen::builder()
        .header(wrapper_path.into_string().unwrap())
        .use_core()
        .wrap_unsafe_ops(true)
        .allowlist_type("mprs_.*")
        .allowlist_function("mprs_.*")
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
