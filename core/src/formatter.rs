pub fn format_size(size: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    let size = size as f64;

    match size {
        s if s >= TB => format!("{:.2} TB", s / TB),
        s if s >= GB => format!("{:.2} GB", s / GB),
        s if s >= MB => format!("{:.2} MB", s / MB),
        s if s >= KB => format!("{:.2} KB", s / KB),
        _ => format!("{} B", size as u64),
    }
}
