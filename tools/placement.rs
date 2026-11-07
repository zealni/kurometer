pub fn find_gap(left: i32, right: i32, width: i32, occupied: &[(i32, i32)]) -> Option<i32> {
    if width <= 0 || right <= left {
        return None;
    }
    let mut regions = occupied.to_vec();
    regions.sort_unstable();
    let mut cursor = i64::from(left) + 8;
    let width = i64::from(width);
    for (start, end) in regions {
        if end <= left || start >= right || end <= start {
            continue;
        }
        if cursor + width + 8 <= i64::from(start) {
            return i32::try_from(cursor).ok();
        }
        cursor = cursor.max(i64::from(end) + 8);
    }
    if cursor + width + 8 <= i64::from(right) {
        i32::try_from(cursor).ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn centered_taskbar_uses_empty_left_region() {
        assert_eq!(
            find_gap(0, 1920, 390, &[(602, 1318), (1573, 1920)]),
            Some(8)
        );
    }
    #[test]
    fn left_alignment_uses_gap_before_tray() {
        assert_eq!(find_gap(0, 1920, 390, &[(0, 720), (1573, 1920)]), Some(728));
    }
    #[test]
    fn crowded_taskbar_hides_instead_of_covering_controls() {
        assert_eq!(find_gap(0, 1920, 390, &[(0, 1550), (1573, 1920)]), None);
    }
    #[test]
    fn unsorted_overlapping_regions_are_merged() {
        assert_eq!(
            find_gap(-1920, 0, 390, &[(-600, 0), (-1900, -1200), (-1400, -900)]),
            None
        );
    }
}
