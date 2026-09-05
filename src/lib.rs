#![forbid(unsafe_code)]

pub mod engine;
pub mod platform;

pub fn human_bytes(bytes: u64) -> String {
    let units = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < units.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", units[unit])
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn byte_format_is_bounded_and_binary() {
        assert_eq!(super::human_bytes(0), "0 B");
        assert_eq!(super::human_bytes(1024), "1.0 KiB");
        assert_eq!(super::human_bytes(u64::MAX), "16.0 EiB");
    }
}
