use std::path::PathBuf;

use micropython_manifest::{ManifestPaths, find_manifest_from};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cargo_manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let ManifestPaths { dir, .. } = find_manifest_from(cargo_manifest_dir)?;
    micropython_build::scan::emit_scan_cfgs(&dir)?;
    Ok(())
}
