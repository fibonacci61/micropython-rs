use std::path::PathBuf;

use micropython_manifest::{ManifestPaths, find_manifest_from, parse_manifest};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cargo_manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());

    let ManifestPaths {
        path: manifest_path,
        dir: manifest_dir,
    } = find_manifest_from(cargo_manifest_dir)?;
    let manifest = parse_manifest(&manifest_path)?;

    let mp_dir = manifest.micropython.path_canonicalized(&manifest_path)?;
    let port_dir = manifest.port.path_canonicalized(&manifest_path)?;

    micropython_build::bindings::generate("wrapper.h", &manifest_dir, &mp_dir, &port_dir)?;
    Ok(())
}
