use std::path::PathBuf;

use micropython_build::config::process_mp_config;
use micropython_manifest::{ManifestPaths, find_manifest_from, parse_manifest};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cargo_manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());

    let ManifestPaths {
        path: manifest_path,
        ..
    } = find_manifest_from(cargo_manifest_dir)?;
    let manifest = parse_manifest(&manifest_path)?;

    let mp_dir = manifest.micropython.path_canonicalized(&manifest_path)?;
    let port_dir = manifest.port.path_canonicalized(&manifest_path)?;

    process_mp_config(&mp_dir, &port_dir)?.emit_cargo_directives();
    Ok(())
}
