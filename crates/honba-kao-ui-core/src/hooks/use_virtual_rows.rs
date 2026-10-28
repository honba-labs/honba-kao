//! `use_virtual_rows` — DOM row virtualization for large tabular datasets (1,000+ items).
//! Ensures 60 FPS scrolling on Level-2 depth ladders and trade logs.

/// Computed slice of visible rows for virtualized tables.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VirtualSlice {
    pub start_index: usize,
    pub end_index: usize,
    pub top_pad_px: f64,
    pub bottom_pad_px: f64,
}

/// Derives a virtual row slice given total rows, row height, container height, and scroll offset.
pub fn use_virtual_rows(
    total_rows: impl Fn() -> usize + Copy + 'static,
    row_height_px: f64,
    container_height_px: impl Fn() -> f64 + Copy + 'static,
    scroll_top_px: impl Fn() -> f64 + Copy + 'static,
    overscan: usize,
) -> impl Fn() -> VirtualSlice + Copy + 'static {
    move || {
        let total = total_rows();
        let scroll = scroll_top_px().max(0.0);
        let height = container_height_px().max(1.0);

        let start = ((scroll / row_height_px).floor() as usize)
            .saturating_sub(overscan)
            .min(total);
        let visible_count = ((height / row_height_px).ceil() as usize) + (overscan * 2);
        let end = (start + visible_count).min(total);

        let top_pad_px = start as f64 * row_height_px;
        let bottom_pad_px = (total.saturating_sub(end)) as f64 * row_height_px;

        VirtualSlice {
            start_index: start,
            end_index: end,
            top_pad_px,
            bottom_pad_px,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_list_renders_nothing() {
        let slice = use_virtual_rows(|| 0usize, 40.0, || 400.0, || 0.0, 1)();
        assert_eq!(slice.start_index, 0);
        assert_eq!(slice.end_index, 0);
    }

    #[test]
    fn scroll_advances_window() {
        let slice0 = use_virtual_rows(|| 100usize, 40.0, || 400.0, || 0.0, 1)();
        assert_eq!(slice0.start_index, 0);
        assert!(slice0.end_index > slice0.start_index);

        let slice1 = use_virtual_rows(|| 100usize, 40.0, || 400.0, || 4000.0, 1)();
        assert!(slice1.start_index > slice0.start_index);
        assert_eq!(slice1.top_pad_px, slice1.start_index as f64 * 40.0);
    }

    #[test]
    fn clamps_at_list_end() {
        let slice = use_virtual_rows(|| 100usize, 40.0, || 400.0, || 1e9, 1)();
        assert_eq!(slice.end_index, 100);
        assert!(slice.start_index <= 100);
    }
}
