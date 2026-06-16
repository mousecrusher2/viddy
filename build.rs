use vergen_gix::{BuildBuilder, Emitter, GixBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Emitter::default()
        .add_instructions(&BuildBuilder::all_build()?)?
        .add_instructions(&GixBuilder::all_git()?)?
        .emit()?;
    Ok(())
}
