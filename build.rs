use vergen_gix::{Build, Emitter, Gix};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Emitter::default()
        .default_on_error()
        .add_instructions(&Build::all_build())?
        .add_instructions(&Gix::all_git())?
        .emit()?;
    Ok(())
}
