pub fn columns(width: i32) -> usize {
    if width >= 480 {
        5
    } else if width < 320 {
        2
    } else {
        3
    }
}
pub fn row_widths(width: i32, metrics: &[usize]) -> [i32; 5] {
    let mut widths = [0; 5];
    for (slot, metric) in metrics.iter().enumerate() {
        widths[slot] = if *metric < 3 { 88 } else { 120 };
    }
    if widths.iter().sum::<i32>() > width - 16 {
        widths[..metrics.len()].fill((width - 16) / 5);
    }
    widths
}
pub fn intersects_taskbar(rect: (i32, i32, i32, i32), bar: (i32, i32, i32, i32)) -> bool {
    rect.0 < rect.2 && rect.1 < rect.3
        && rect.0 < bar.2 && rect.2 > bar.0
        && rect.1 < bar.3 && rect.3 > bar.1
}
pub fn inset_bounds(bounds: (i32, i32, i32, i32), inset: i32) -> (i32, i32, i32, i32) {
    let (l, t, r, b) = bounds;
    (l, t + inset, r, b - inset)
}
pub fn fit_position(
    x: i32,
    width: i32,
    height: i32,
    bounds: (i32, i32, i32, i32),
) -> Option<(i32, i32)> {
    let (l, t, r, b) = bounds;
    if width <= 0 || height <= 0 || r <= l || b <= t || width > r - l || height > b - t {
        return None;
    }
    Some((x.clamp(l, r - width), b - height))
}
pub fn popup_position(
    meter: (i32, i32, i32, i32),
    size: (i32, i32),
    screen: (i32, i32, i32, i32),
) -> (i32, i32) {
    let (left, top, right, bottom) = screen;
    let x = meter.0.clamp(left, (right - size.0).max(left));
    let above = meter.1 - size.1 - 8;
    let y = if above >= top { above } else { meter.3 + 8 };
    (x, y.clamp(top, (bottom - size.1).max(top)))
}
pub fn relevant_shell_event(event: u32, target_is_taskbar: bool) -> bool {
    event == 3 || ((0x8000..=0x800c).contains(&event) && target_is_taskbar)
}
pub fn replace_meter(attached_parent: usize, current_parent: usize, meter_alive: bool) -> bool {
    !meter_alive || attached_parent != current_parent
}
pub fn meter_position(
    bar: (i32, i32, i32, i32),
    width: i32,
    position: i32,
    offset: i32,
    occupied: &[(i32, i32)],
) -> Option<(i32, i32, i32)> {
    let (left, top, right, bottom) = bar;
    let height = bottom - top - 4;
    if right - left <= bottom - top || height <= 0 || width <= 0 {
        return None;
    }
    let x = match position {
        0 => left + 8,
        1 => left + (right - left - width) / 2,
        2 => right - width - 8,
        3 if offset >= 0 => left + offset,
        _ => crate::placement::find_gap(left, right, width, occupied)?,
    };
    if x < left || x + width > right
        || occupied.iter().any(|(l, r)| x < *r && x + width > *l)
    {
        return None;
    }
    let (x, y) = fit_position(x, width, height, inset_bounds(bar, 2))?;
    Some((x, y, height))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn child_placement_uses_screen_coordinates_on_negative_origin_display() {
        assert_eq!(meter_position((-1920, 1032, 0, 1080), 500, 0, -1, &[]), Some((-1912, 1034, 44)));
        assert_eq!(meter_position((-1920, 1032, 0, 1080), 500, 3, 100, &[]), Some((-1820, 1034, 44)));
    }
    #[test]
    fn occupied_or_unsupported_taskbar_never_places_meter() {
        assert_eq!(meter_position((0, 1032, 1920, 1080), 500, 0, -1, &[(0, 600)]), None);
        assert_eq!(meter_position((0, 0, 48, 1080), 500, -1, -1, &[]), None);
        assert_eq!(meter_position((0, 0, 1920, 3), 500, -1, -1, &[]), None);
    }
    #[test]
    fn missing_or_replaced_parent_requires_recreating_meter_only() {
        assert!(replace_meter(0, 42, false));
        assert!(replace_meter(41, 42, true));
        assert!(replace_meter(42, 42, false));
        assert!(!replace_meter(42, 42, true));
    }
    #[test]
    fn popup_buttons_do_not_count_as_taskbar_obstacles() {
        let bar = (0, 1032, 1920, 1080);
        assert!(!intersects_taskbar((0, 800, 300, 1032), bar));
        assert!(intersects_taskbar((700, 1036, 740, 1076), bar));
    }
    #[test]
    fn wide_single_row_keeps_metrics_compact() {
        assert_eq!(row_widths(800, &[0, 1, 2, 3, 4]), [88, 88, 88, 120, 120]);
        assert_eq!(row_widths(480, &[0, 1, 2, 3, 4]), [92; 5]);
        assert_eq!(row_widths(800, &[3, 0]), [120, 88, 0, 0, 0]);
    }
    #[test]
    fn details_follow_meter_and_stay_on_screen() {
        assert_eq!(popup_position((10, 1034, 350, 1078), (450, 370), (0, 0, 1920, 1080)), (10, 656));
        assert_eq!(popup_position((1850, 1034, 1920, 1078), (450, 370), (0, 0, 1920, 1080)), (1470, 656));
        assert_eq!(popup_position((-1800, 0, -1500, 44), (450, 370), (-1920, 0, 0, 1080)), (-1800, 52));
    }
    #[test]
    fn width_changes_grid_density() {
        assert_eq!(columns(260), 2);
        assert_eq!(columns(340), 3);
        assert_eq!(columns(480), 5);
    }
    #[test]
    fn meter_is_clamped_inside_actual_taskbar() {
        assert_eq!(
            fit_position(950, 340, 48, (0, 1020, 1920, 1080)),
            Some((950, 1032))
        );
        assert_eq!(
            fit_position(-50, 340, 48, (0, 1020, 1920, 1080)),
            Some((0, 1032))
        );
        assert_eq!(fit_position(0, 2000, 48, (0, 1020, 1920, 1080)), None);
        let b = inset_bounds((0, 1020, 1920, 1080), 2);
        assert_eq!(b, (0, 1022, 1920, 1078));
        assert_eq!(fit_position(8, 500, 44, b), Some((8, 1034)));
    }
    #[test]
    fn unrelated_explorer_events_do_not_trigger_meter_hide() {
        assert!(!relevant_shell_event(0x8002, false));
        assert!(relevant_shell_event(0x8002, true));
        assert!(relevant_shell_event(3, false));
    }
}
