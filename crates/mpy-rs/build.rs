#[cfg(feature = "install")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use ignore::WalkBuilder;
    use std::{
        fs::File,
        path::{Path, PathBuf},
    };

    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());

    let workspace_dir = manifest_dir.parent().and_then(Path::parent).unwrap();
    let embedded_dir = workspace_dir.join("embedded");

    let archive_path = out_dir.join("embedded.tar.zst");
    let file = File::create(&archive_path)?;

    let zstd_encoder = zstd::Encoder::new(file, 3)?;
    let mut tar = tar::Builder::new(zstd_encoder);

    let mut walker = WalkBuilder::new(&embedded_dir);
    walker.hidden(true);
    walker.add_ignore(workspace_dir.join("install.ignore"));
    for entry in walker.build() {
        let entry = entry?;
        let path = entry.path();

        if path == embedded_dir {
            continue;
        }

        let archive_path = Path::new("micropython-rs").join(path.strip_prefix(&embedded_dir)?);
        if path.is_dir() {
            tar.append_dir(archive_path, path)?;
        } else {
            println!("cargo::rerun-if-changed={}", path.display());
            tar.append_path_with_name(path, archive_path)?;
        }
    }

    let zstd_encoder = tar.into_inner()?;
    zstd_encoder.finish()?;

    println!(
        "cargo::rustc-env=EMBEDDED_TAR_ZST={}",
        archive_path.display()
    );

    Ok(())
}

#[cfg(not(feature = "install"))]
fn main() {}
