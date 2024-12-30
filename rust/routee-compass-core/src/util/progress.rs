use kdam::{Bar, BarBuilder};

/// environment variable used to denote if the progress bar should be used.
/// if COMPASS_PROGRESS=false, the bar is deactivated, otherwise it runs.
const COMPASS_PROGRESS: &str = "COMPASS_PROGRESS";

/// helper function for building a progress bar.
/// a progress bar is created only if:
///   - the `progress` argument is not None
///   - the logging system is set to DEBUG or INFO
///   - the COMPASS_PROGRESS environment variable is not set to "false"
///
/// # Arguments
///
/// * `progress` - progress bar configuration
///
/// # Returns
///
/// Some progress bar if it should be built, else None
pub fn build_progress_bar(progress: BarBuilder) -> Option<Bar> {
    let progress_disabled = std::env::var(COMPASS_PROGRESS)
        .ok()
        .map(|v| v.to_lowercase() == "false")
        .unwrap_or_default();
    let log_info_enabled = log::log_enabled!(log::Level::Info);
    if !progress_disabled && log_info_enabled {
        progress.build().ok()
    } else {
        None
    }
}
