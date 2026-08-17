use std::path::PathBuf;

use crate::manifest::find_manifest;

const EMBEDDED_TAR_ZST: &[u8] = include_bytes!(env!("EMBEDDED_TAR_ZST"));

pub fn install(dir: Option<PathBuf>) -> anyhow::Result<()> {
    let manifest_path = find_manifest(dir)?;
    let manifest_dir = manifest_path.parent().unwrap();

    let zstd_decoder = zstd::Decoder::new(EMBEDDED_TAR_ZST)?;
    let mut archive = tar::Archive::new(zstd_decoder);
    archive.unpack(manifest_dir)?;

    Ok(())
}
