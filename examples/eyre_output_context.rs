use color_eyre::eyre::{eyre, Context};
use spanned::Spanned;

fn main() -> color_eyre::eyre::Result<()> {
    parse().with_context(|| eyre!("kawoosh"))?;
    Ok(())
}

fn parse() -> color_eyre::eyre::Result<()> {
    Err(Spanned::here("kaboom"))?;
    Ok(())
}
