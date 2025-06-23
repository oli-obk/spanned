use spanned::Spanned;

fn main() -> color_eyre::eyre::Result<()> {
    Err(Spanned::here("kaboom"))?;
    Ok(())
}
