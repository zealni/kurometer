pub struct MeterLayout {
    pub cells: Vec<(i32, i32, i32, i32)>,
    pub font_px: i32,
}

pub fn responsive_meter(width: i32, height: i32, dpi: u32, metrics: &[usize], ram_both: bool, requested_columns: u32) -> MeterLayout {
    let mut layout = MeterLayout { cells: vec![(0, 0, 0, 0); metrics.len()], font_px: 0 };
    if width <= 0 || height <= 0 || metrics.is_empty() {
        return layout;
    }

    let dpi = if dpi == 0 { 96 } else { dpi.min(768) } as i64;
    let scale = |value: i64| (value * dpi / 96).max(1) as i32;
    let inset_x = scale(8).min((width - 1) / 2);
    let inset_y = scale(2).min((height - 1) / 2);
    let available_width = width - inset_x * 2;
    let available_height = height - inset_y * 2;
    let count = metrics.len();
    let mut best_columns = 0;
    let mut best_font = 0;
    for columns in 1..=count {
        if requested_columns != 0 && columns != (requested_columns as usize).min(count) { continue; }
        let rows = (count + columns - 1) / columns;
        let cell_width = available_width / columns as i32;
        let cell_height = available_height / rows as i32;
        if cell_width == 0 || cell_height == 0 || (rows > 1 && cell_height < scale(12)) {
            continue;
        }
        let mut font = scale(16).min(cell_height);
        for &metric in metrics {
            let preferred = if metric == 2 && ram_both { 150 } else if metric == usize::MAX - 1 || metric == usize::MAX - 2 { 120 } else if metric < 3 || metric >= 5 { 88 } else { 120 };
            font = font.min((i64::from(cell_width) * 16 / preferred) as i32);
        }
        if font >= best_font && font > 0 {
            best_font = font;
            best_columns = columns;
        }
    }
    if best_columns == 0 {
        return layout;
    }
    let rows = (count + best_columns - 1) / best_columns;
    for slot in 0..count {
        let column = slot % best_columns;
        let row = slot / best_columns;
        let x = |part: usize| inset_x + (i64::from(available_width) * part as i64 / best_columns as i64) as i32;
        let y = |part: usize| inset_y + (i64::from(available_height) * part as i64 / rows as i64) as i32;
        layout.cells[slot] = (x(column), y(row), x(column + 1), y(row + 1));
    }
    layout.font_px = best_font;
    layout
}

pub const PRIMARY_DISPLAY: &str = "__primary__";
pub const ALL_DISPLAYS: &str = "__all__";
pub const SECONDARY_DISPLAYS: &str = "__secondary__";
pub fn display_targets(choice: &str, displays: &[(&str, bool)]) -> Vec<usize> {
    if choice == ALL_DISPLAYS {
        return (0..displays.len()).collect();
    }
    if choice == SECONDARY_DISPLAYS {
        return displays.iter().enumerate().filter_map(|(i, (_, primary))| (!primary).then_some(i)).collect();
    }
    displays.iter().position(|(id, primary)| {
        if choice.is_empty() || choice == PRIMARY_DISPLAY { *primary } else { *id == choice }
    }).or_else(|| displays.iter().position(|(_, primary)| *primary)).into_iter().collect()
}
pub fn settings_bounds(
    meter: (i32, i32, i32, i32),
    size: (i32, i32),
    work: (i32, i32, i32, i32),
) -> (i32, i32, i32, i32) {
    let (left, top, right, bottom) = work;
    let width = size.0.clamp(1, (right - left).max(1));
    let height = size.1.clamp(1, (bottom - top).max(1));
    let (x, y) = popup_position(meter, (width, height), work);
    (x, y, width, height)
}
pub fn keep_meter_visible_during_refresh(_hosted: bool, taskbar_alive: bool, last_position_safe: bool) -> bool {
    taskbar_alive && last_position_safe
}
pub fn intersects_taskbar(rect: (i32, i32, i32, i32), bar: (i32, i32, i32, i32)) -> bool {
    rect.0 < rect.2 && rect.1 < rect.3
        && rect.0 < bar.2 && rect.2 > bar.0
        && rect.1 < bar.3 && rect.3 > bar.1
}
pub fn taskbar_bounds_match(previous: Option<(i32, i32, i32, i32)>, current: (i32, i32, i32, i32)) -> bool {
    previous == Some(current)
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
pub fn recreate_meter(attached_parent: usize, current_parent: usize, meter_alive: bool, rejected_parent: usize) -> bool {
    !meter_alive || (attached_parent != current_parent && rejected_parent != current_parent)
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
    fn responsive_cells_fit_at_common_scales_and_taskbar_sizes() {
        let metrics = [0, 1, 2, 3, 4];
        for dpi in [96, 120, 144, 192] {
            for height in [24, 40, 48, 72] {
                for width in [260, 340, 480, 800, 1200] {
                    for count in 1..=5 {
                        for ram_both in [false, true] {
                            let layout = responsive_meter(width, height, dpi, &metrics[..count], ram_both, 0);
                            assert!(layout.font_px > 0);
                            for (slot, &(left, top, right, bottom)) in layout.cells[..count].iter().enumerate() {
                                assert!(0 <= left && left < right && right <= width);
                                assert!(0 <= top && top < bottom && bottom <= height);
                                assert!(layout.font_px <= bottom - top);
                                for &(l, t, r, b) in &layout.cells[..slot] {
                                    assert!(right <= l || r <= left || bottom <= t || b <= top);
                                }
                            }
                            assert!(layout.cells[count..].iter().all(|cell| *cell == (0, 0, 0, 0)));
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn every_metric_combination_stays_inside_taskbar() {
        for mask in 1..32 {
            let metrics: Vec<_> = (0..5).filter(|i| mask & (1 << i) != 0).collect();
            for (width, height, dpi) in [(260, 24, 96), (571, 44, 120), (900, 72, 192)] {
                let layout = responsive_meter(width, height, dpi, &metrics, true, 0);
                for &(left, top, right, bottom) in &layout.cells[..metrics.len()] {
                    assert!(left >= 0 && top >= 0 && right <= width && bottom <= height);
                    assert!(left < right && top < bottom);
                }
            }
        }
    }

    #[test]
    fn responsive_layout_uses_height_and_width_to_choose_rows() {
        let metrics = [0, 1, 2, 3, 4];
        let short = responsive_meter(260, 24, 96, &metrics, false, 0);
        let tall = responsive_meter(260, 72, 96, &metrics, false, 0);
        let wide = responsive_meter(1200, 72, 96, &metrics, false, 0);
        assert!(short.cells.iter().all(|cell| cell.1 == short.cells[0].1));
        assert!(tall.cells.iter().any(|cell| cell.1 > tall.cells[0].1));
        assert!(wide.cells.iter().all(|cell| cell.1 == wide.cells[0].1));
    }

    #[test]
    fn requested_columns_control_grid_placement() {
        let layout = responsive_meter(600, 80, 96, &[0, 1, 2, 3], false, 2);
        assert_eq!(layout.cells[0].1, layout.cells[1].1);
        assert!(layout.cells[2].1 > layout.cells[0].1);
        assert!(layout.cells[1].0 > layout.cells[0].0);
    }

    #[test]
    fn responsive_layout_rejects_unusable_dimensions() {
        for (width, height) in [(0, 40), (260, 0), (-1, 40), (260, -1), (1, 1)] {
            let layout = responsive_meter(width, height, 96, &[0, 1, 2, 3, 4], true, 0);
            assert_eq!(layout.font_px, 0);
            assert_eq!(layout.cells, vec![(0, 0, 0, 0); 5]);
        }
        let empty = responsive_meter(260, 40, 96, &[], false, 0);
        assert_eq!(empty.font_px, 0);
        assert!(empty.cells.is_empty());
    }
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
        assert!(recreate_meter(0, 42, false, 0));
        assert!(recreate_meter(41, 42, true, 0));
        assert!(recreate_meter(42, 42, false, 0));
        assert!(!recreate_meter(42, 42, true, 0));
    }
    #[test]
    fn refused_parent_keeps_overlay_until_retry_or_taskbar_change() {
        assert!(!recreate_meter(0, 42, true, 42));
        assert!(recreate_meter(0, 42, true, 0));
        assert!(recreate_meter(0, 43, true, 42));
        assert!(recreate_meter(0, 42, false, 42));
    }
    #[test]
    fn popup_buttons_do_not_count_as_taskbar_obstacles() {
        let bar = (0, 1032, 1920, 1080);
        assert!(!intersects_taskbar((0, 800, 300, 1032), bar));
        assert!(intersects_taskbar((700, 1036, 740, 1076), bar));
    }
    #[test]
    fn details_follow_meter_and_stay_on_screen() {
        assert_eq!(popup_position((10, 1034, 350, 1078), (450, 370), (0, 0, 1920, 1080)), (10, 656));
        assert_eq!(popup_position((1850, 1034, 1920, 1078), (450, 370), (0, 0, 1920, 1080)), (1470, 656));
        assert_eq!(popup_position((-1800, 0, -1500, 44), (450, 370), (-1920, 0, 0, 1080)), (-1800, 52));
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
    #[test]
    fn display_choices_resolve_primary_all_specific_and_stale_ids() {
        let displays = [("DISPLAY1", true), ("DISPLAY2", false)];
        assert_eq!(display_targets("", &displays), [0]);
        assert_eq!(display_targets(PRIMARY_DISPLAY, &displays), [0]);
        assert_eq!(display_targets(ALL_DISPLAYS, &displays), [0, 1]);
        assert_eq!(display_targets("DISPLAY2", &displays), [1]);
        assert_eq!(display_targets("removed", &displays), [0]);
        assert_eq!(display_targets("__secondary__", &displays), [1]);
        assert!(display_targets("__secondary__", &[("DISPLAY1", true)]).is_empty());
        assert!(display_targets(ALL_DISPLAYS, &[]).is_empty());
    }
    #[test]
    fn settings_panel_sits_low_and_clamps_to_negative_origin_work_area() {
        assert_eq!(settings_bounds((-1750, 12, -1300, 56), (520, 560), (-1920, 0, 0, 1020)), (-1750, 64, 520, 560));
        assert_eq!(settings_bounds((1500, 20, 1800, 64), (520, 560), (0, 0, 1920, 550)), (1400, 0, 520, 550));
    }
    #[test]
    fn all_displays_uses_only_taskbar_backed_entries() {
        let taskbar_backed = [("DISPLAY5", true)];
        assert_eq!(display_targets(ALL_DISPLAYS, &taskbar_backed), [0]);
        assert!(display_targets(ALL_DISPLAYS, &[]).is_empty());
    }
    #[test]
    fn taskbar_bounds_match_is_required_to_keep_last_position_safe() {
        assert!(taskbar_bounds_match(Some((0, 1032, 1920, 1080)), (0, 1032, 1920, 1080)));
        assert!(!taskbar_bounds_match(Some((0, 1032, 1920, 1080)), (0, 1030, 1920, 1080)));
        assert!(!taskbar_bounds_match(None, (0, 1032, 1920, 1080)));
    }
    #[test]
    fn visible_overlay_during_refresh_requires_live_matching_taskbar() {
        assert!(keep_meter_visible_during_refresh(false, true, true));
        assert!(!keep_meter_visible_during_refresh(false, true, false));
        assert!(!keep_meter_visible_during_refresh(false, false, true));
    }
}
