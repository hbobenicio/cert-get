/// new_clock_spinner returns a new progress bar with a clock styling.
#[allow(dead_code)]
pub fn new_clock_spinner(initial_message: &str) -> indicatif::ProgressBar {
    let spinner = indicatif::ProgressBar::new_spinner();

    // For more spinners check out the cli-spinners project:
    // https://github.com/sindresorhus/cli-spinners/blob/master/spinners.json
    let clock_style = indicatif::ProgressStyle::default_spinner()
        .tick_strings(&[
            "🕛", "🕐", "🕑", "🕒", "🕓", "🕔", "🕕", "🕖", "🕗", "🕘", "🕙", "🎉",
        ])
        .template("{spinner:.blue} {msg}");

    spinner.enable_steady_tick(100);
    spinner.set_message(initial_message);
    spinner.set_style(clock_style);

    spinner
}
