//! Presentation-only capacity calculations. These never decide cleanup targets.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Usage {
    used: u64,
    total: u64,
}

impl Usage {
    /// A missing or contradictory OS reading is unknown, not a full disk.
    pub fn new(used: u64, total: u64) -> Option<Self> {
        (total > 0 && used <= total).then_some(Self { used, total })
    }

    pub fn fraction(self) -> f32 {
        (self.used as f64 / self.total as f64) as f32
    }

    pub fn percent(self) -> f64 {
        self.used as f64 / self.total as f64 * 100.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpaceLevel {
    Normal,
    Low,
    VeryLow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiskCapacity {
    total: u64,
    available: u64,
}

impl DiskCapacity {
    pub fn new(total: u64, available: u64) -> Option<Self> {
        (total > 0 && available <= total).then_some(Self { total, available })
    }

    pub fn used_fraction(self) -> f32 {
        ((self.total - self.available) as f64 / self.total as f64) as f32
    }

    pub fn used_percent(self) -> f64 {
        (self.total - self.available) as f64 / self.total as f64 * 100.0
    }

    pub fn space_level(self) -> SpaceLevel {
        // u128 avoids overflow and floating-point boundary errors for large disks.
        let available = u128::from(self.available) * 100;
        let total = u128::from(self.total);
        if available <= total * 5 {
            SpaceLevel::VeryLow
        } else if available <= total * 10 {
            SpaceLevel::Low
        } else {
            SpaceLevel::Normal
        }
    }
}

pub fn cpu_fraction(percent: f32) -> Option<f32> {
    percent
        .is_finite()
        .then(|| (percent / 100.0).clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_reports_used_fraction_not_free_fraction() {
        let usage = Usage::new(28, 32).unwrap();
        assert_eq!(usage.fraction(), 0.875);
        assert_eq!(usage.percent(), 87.5);
    }

    #[test]
    fn missing_or_contradictory_memory_is_unknown() {
        assert!(Usage::new(0, 0).is_none());
        assert!(Usage::new(2, 1).is_none());
        assert_eq!(Usage::new(0, 10).unwrap().fraction(), 0.0);
    }

    #[test]
    fn missing_or_contradictory_disk_is_unknown() {
        assert!(DiskCapacity::new(0, 0).is_none());
        assert!(DiskCapacity::new(0, 10).is_none());
        assert!(DiskCapacity::new(10, 11).is_none());
    }

    #[test]
    fn disk_bar_measures_used_space() {
        let disk = DiskCapacity::new(1_000, 400).unwrap();
        assert!((disk.used_fraction() - 0.6).abs() < 0.000_001);
        assert_eq!(disk.used_percent(), 60.0);
    }

    #[test]
    fn full_and_empty_drives_have_valid_extremes() {
        let full = DiskCapacity::new(100, 0).unwrap();
        assert_eq!(full.used_fraction(), 1.0);
        assert_eq!(full.space_level(), SpaceLevel::VeryLow);
        let empty = DiskCapacity::new(100, 100).unwrap();
        assert_eq!(empty.used_fraction(), 0.0);
        assert_eq!(empty.space_level(), SpaceLevel::Normal);
    }

    #[test]
    fn exact_low_space_thresholds_are_inclusive() {
        for (free, expected) in [
            (11, SpaceLevel::Normal),
            (10, SpaceLevel::Low),
            (6, SpaceLevel::Low),
            (5, SpaceLevel::VeryLow),
        ] {
            assert_eq!(
                DiskCapacity::new(100, free).unwrap().space_level(),
                expected
            );
        }
    }

    #[test]
    fn tiny_nonempty_drive_is_not_classified_as_full() {
        assert_eq!(
            DiskCapacity::new(3, 1).unwrap().space_level(),
            SpaceLevel::Normal
        );
    }

    #[test]
    fn near_full_large_drive_is_very_low() {
        let gib = 1_u64 << 30;
        assert_eq!(
            DiskCapacity::new(1_843 * gib, 9 * gib)
                .unwrap()
                .space_level(),
            SpaceLevel::VeryLow
        );
    }

    #[test]
    fn maximum_capacity_does_not_overflow() {
        let disk = DiskCapacity::new(u64::MAX, u64::MAX / 2).unwrap();
        assert!((disk.used_fraction() - 0.5).abs() < 0.000_001);
        assert_eq!(disk.space_level(), SpaceLevel::Normal);
        assert_eq!(
            DiskCapacity::new(u64::MAX, 0).unwrap().space_level(),
            SpaceLevel::VeryLow
        );
    }

    #[test]
    fn nonfinite_cpu_is_not_rendered_as_a_percentage() {
        assert_eq!(cpu_fraction(f32::NAN), None);
        assert_eq!(cpu_fraction(f32::INFINITY), None);
        assert_eq!(cpu_fraction(f32::NEG_INFINITY), None);
    }

    #[test]
    fn cpu_is_bounded() {
        assert_eq!(cpu_fraction(-10.0), Some(0.0));
        assert_eq!(cpu_fraction(50.0), Some(0.5));
        assert_eq!(cpu_fraction(150.0), Some(1.0));
    }
}
