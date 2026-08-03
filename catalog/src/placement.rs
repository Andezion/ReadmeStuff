pub const SNAP_TOLERANCE: i32 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    pub fn left(&self) -> i32 {
        self.x
    }

    pub fn right(&self) -> i32 {
        self.x + self.w as i32
    }

    pub fn top(&self) -> i32 {
        self.y
    }

    pub fn bottom(&self) -> i32 {
        self.y + self.h as i32
    }
}

pub fn overlaps(a: Rect, b: Rect) -> bool {
    a.left() < b.right() && b.left() < a.right() && a.top() < b.bottom() && b.top() < a.bottom()
}

pub fn fits_canvas(r: Rect, canvas_w: u32) -> bool {
    r.x >= 0 && r.right() <= canvas_w as i32 && r.y >= 0
}

fn best_axis_snap(near: i32, far: i32, targets: &[i32]) -> i32 {
    let mut best: Option<i32> = None;
    for &target in targets {
        for delta in [target - near, target - far] {
            if delta.abs() <= SNAP_TOLERANCE && best.is_none_or(|b: i32| delta.abs() < b.abs()) {
                best = Some(delta);
            }
        }
    }
    best.unwrap_or(0)
}

pub fn snap(candidate: Rect, others: &[Rect], canvas_w: u32) -> (i32, i32) {
    let mut x_targets = vec![0, canvas_w as i32];
    let mut y_targets = vec![0];
    for other in others {
        x_targets.push(other.left());
        x_targets.push(other.right());
        y_targets.push(other.top());
        y_targets.push(other.bottom());
    }

    let dx = best_axis_snap(candidate.left(), candidate.right(), &x_targets);
    let dy = best_axis_snap(candidate.top(), candidate.bottom(), &y_targets);
    (candidate.x + dx, candidate.y + dy)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: i32, y: i32, w: u32, h: u32) -> Rect {
        Rect { x, y, w, h }
    }

    #[test]
    fn identical_rects_overlap() {
        assert!(overlaps(rect(0, 0, 10, 10), rect(0, 0, 10, 10)));
    }

    #[test]
    fn touching_edges_do_not_overlap() {
        assert!(!overlaps(rect(0, 0, 10, 10), rect(10, 0, 10, 10)));
    }

    #[test]
    fn far_apart_rects_do_not_overlap() {
        assert!(!overlaps(rect(0, 0, 10, 10), rect(100, 100, 10, 10)));
    }

    #[test]
    fn fits_canvas_rejects_negative_origin() {
        assert!(!fits_canvas(rect(-1, 0, 10, 10), 990));
        assert!(!fits_canvas(rect(0, -1, 10, 10), 990));
    }

    #[test]
    fn fits_canvas_rejects_overflow_past_right_edge() {
        assert!(!fits_canvas(rect(985, 0, 10, 10), 990));
    }

    #[test]
    fn fits_canvas_has_no_bottom_bound() {
        assert!(fits_canvas(rect(0, 1_000_000, 10, 10), 990));
    }

    #[test]
    fn snaps_left_edge_to_canvas_left() {
        let (x, y) = snap(rect(3, 500, 100, 50), &[], 990);
        assert_eq!((x, y), (0, 500));
    }

    #[test]
    fn snaps_right_edge_to_canvas_right() {
        let (x, _) = snap(rect(886, 0, 100, 50), &[], 990);
        assert_eq!(x, 890);
    }

    #[test]
    fn snaps_to_another_widgets_right_edge() {
        let other = rect(0, 0, 100, 50);
        let (x, _) = snap(rect(105, 0, 80, 50), &[other], 990);
        assert_eq!(x, 100);
    }

    #[test]
    fn snaps_to_another_widgets_bottom_edge() {
        let other = rect(0, 0, 100, 50);
        let (_, y) = snap(rect(0, 53, 100, 50), &[other], 990);
        assert_eq!(y, 50);
    }

    #[test]
    fn does_not_snap_when_outside_tolerance() {
        let other = rect(0, 0, 100, 50);
        let candidate = rect(150, 0, 80, 50);
        assert_eq!(snap(candidate, &[other], 990), (150, 0));
    }

    #[test]
    fn snap_at_exact_tolerance_boundary_still_snaps() {
        let other = rect(0, 0, 100, 50);
        let candidate = rect(100 + SNAP_TOLERANCE, 0, 80, 50);
        let (x, _) = snap(candidate, &[other], 990);
        assert_eq!(x, 100);
    }
}
