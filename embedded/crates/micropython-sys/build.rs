use std::io;
use std::path::PathBuf;

use micropython_build::config::Config;

fn integer_config(config: &Config, name: &str) -> io::Result<i128> {
    config
        .definitions()
        .get(name)
        .and_then(|value| value.integer())
        .ok_or_else(|| io::Error::other(format!("configuration `{name}` is not an integer")))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = micropython_build::config::process_mp_config()?;

    let hash_type = match integer_config(&config, "MICROPY_QSTR_BYTES_IN_HASH")? {
        0 => "",
        1 => "pub type qstr_hash_t = u8;\n",
        2 => "pub type qstr_hash_t = u16;\n",
        value => {
            return Err(format!("unsupported MICROPY_QSTR_BYTES_IN_HASH value {value}").into());
        }
    };
    let len_type = match integer_config(&config, "MICROPY_QSTR_BYTES_IN_LEN")? {
        1 => "pub type qstr_len_t = u8;\n",
        2 => "pub type qstr_len_t = u16;\n",
        value => return Err(format!("unsupported MICROPY_QSTR_BYTES_IN_LEN value {value}").into()),
    };

    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").ok_or("OUT_DIR is not set")?);
    std::fs::write(
        out_dir.join("qstr_types.rs"),
        format!("{hash_type}{len_type}"),
    )?;

    config.emit_cargo_directives();
    Ok(())
}
