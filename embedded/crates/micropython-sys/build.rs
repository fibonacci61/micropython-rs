fn main() -> Result<(), Box<dyn std::error::Error>> {
    micropython_build::config::process_mp_config()?.emit_cargo_directives();
    Ok(())
}
