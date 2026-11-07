pub fn cpu_busy(previous: [u64; 3], current: [u64; 3]) -> Option<f64> {
    let idle = current[0].checked_sub(previous[0])?;
    let total = current[1]
        .checked_sub(previous[1])?
        .checked_add(current[2].checked_sub(previous[2])?)?;
    if total == 0 || idle > total {
        None
    } else {
        Some((total - idle) as f64 * 100.0 / total as f64)
    }
}
pub fn transfer_rate(previous: u64, current: u64, seconds: f64) -> Option<f64> {
    if !seconds.is_finite() || seconds <= 0.0 {
        return None;
    }
    Some(current.checked_sub(previous)? as f64 / seconds)
}
pub fn busiest_engine(readings: &[(String, f64)]) -> Option<f64> {
    let mut engines = std::collections::BTreeMap::<&str, f64>::new();
    for (engine, value) in readings {
        if value.is_finite() && *value >= 0.0 {
            *engines.entry(engine.as_str()).or_default() += value;
        }
    }
    engines
        .values()
        .copied()
        .reduce(f64::max)
        .map(|v| v.min(100.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cpu_kernel_includes_idle() {
        assert_eq!(cpu_busy([100, 200, 100], [120, 260, 140]), Some(80.0));
    }
    #[test]
    fn resets_and_invalid_intervals_are_not_zero() {
        assert_eq!(cpu_busy([100, 200, 100], [1, 2, 1]), None);
        assert_eq!(transfer_rate(20, 10, 1.0), None);
        assert_eq!(transfer_rate(0, 10, 0.0), None);
    }
    #[test]
    fn network_uses_actual_elapsed_time() {
        assert_eq!(transfer_rate(100, 1100, 2.0), Some(500.0));
        assert_eq!(transfer_rate(100, 100, 1.0), Some(0.0));
    }
    #[test]
    fn gpu_sums_processes_within_engine_not_across_engines() {
        let rows = vec![
            ("luid_x_phys_0_eng_0_engtype_3D".into(), 30.0),
            ("luid_x_phys_0_eng_0_engtype_3D".into(), 40.0),
            ("luid_x_phys_0_eng_1_engtype_Copy".into(), 50.0),
        ];
        assert_eq!(busiest_engine(&rows), Some(70.0));
    }
}
