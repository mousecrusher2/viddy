use vergen_gix::{Build, Emitter, Gix};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env_prefix = std::env::var("CARGO_PKG_NAME")?.to_ascii_uppercase();
    println!("cargo:rustc-env=PACKAGE_ENV_PREFIX={env_prefix}");

    Emitter::default()
        .default_on_error()
        .add_instructions(&Build::all_build())?
        .add_instructions(&Gix::all_git())?
        .emit()?;
    Ok(())
}
