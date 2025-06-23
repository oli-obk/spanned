use ui_test::{
    custom_flags::run::Run, dependencies::DependencyBuilder, run_tests, spanned::Spanned, Config,
};

fn main() -> ui_test::color_eyre::Result<()> {
    let mut config = Config::rustc("examples");
    config
        .comment_defaults
        .base()
        .set_custom("dependencies", DependencyBuilder::default());
    config.comment_defaults.base().set_custom(
        "run",
        Run {
            exit_code: 1,
            output_conflict_handling: None,
        },
    );
    config.comment_defaults.base().exit_status = Spanned::dummy(0_i32).into();
    config.comment_defaults.base().require_annotations = Spanned::dummy(false).into();
    let abort_check = config.abort_check.clone();
    ctrlc::set_handler(move || abort_check.abort())?;

    // Compile all `.rs` files in the given directory (relative to your
    // Cargo.toml) and compare their output against the corresponding
    // `.stderr` files.
    run_tests(config)
}
