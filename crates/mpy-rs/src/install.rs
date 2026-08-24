use crate::manifest::{ManifestPaths, find_manifest};

const EMBEDDED_TAR_ZST: &[u8] = include_bytes!(env!("EMBEDDED_TAR_ZST"));

pub fn install() -> anyhow::Result<()> {
    let ManifestPaths { dir, .. } = find_manifest()?;

    let zstd_decoder = zstd::Decoder::new(EMBEDDED_TAR_ZST)?;
    let mut archive = tar::Archive::new(zstd_decoder);
    archive.unpack(dir)?;

    Ok(())
}
