fn main() -> Result<(), Box<dyn std::error::Error>> {
    micropython_build::bindings::generate("wrapper.h")?;
    Ok(())
}
