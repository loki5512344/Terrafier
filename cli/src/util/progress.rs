use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::time::Duration;

pub fn new_spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

pub fn new_bar(total: u64, msg: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("##-"),
    );
    pb.set_message(msg.to_string());
    pb
}

#[allow(dead_code)]
pub fn new_multi() -> MultiProgress {
    MultiProgress::new()
}

pub fn finish_with(msg: &str, pb: ProgressBar) {
    pb.finish_with_message(msg.to_string());
}
