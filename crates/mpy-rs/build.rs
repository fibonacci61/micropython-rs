use std::{
    fs::File,
    path::{Path, PathBuf},
};

fn main() -> std::io::Result<()> {
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());

    let workspace_dir = manifest_dir.parent().and_then(Path::parent).unwrap();
    let embedded_dir = workspace_dir.join("embedded");
    println!("cargo::rerun-if-changed={}", embedded_dir.display());

    let archive_path = out_dir.join("embedded.tar.zst");
    let file = File::create(&archive_path)?;

    let zstd_encoder = zstd::Encoder::new(file, 3)?;
    let mut tar = tar::Builder::new(zstd_encoder);

    tar.append_dir_all("micropython-rs", &embedded_dir)?;
    let zstd_encoder = tar.into_inner()?;
    zstd_encoder.finish()?;

    println!(
        "cargo::rustc-env=EMBEDDED_TAR_ZST={}",
        archive_path.display()
    );

    Ok(())
}
