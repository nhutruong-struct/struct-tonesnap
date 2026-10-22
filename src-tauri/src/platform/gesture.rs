//! Tells a selection gesture (a drag, or a double/triple click, over text) apart from an
//! ordinary click, from the raw left-button events the mouse hook (Windows) or event monitor (macOS) sees.

use std::time::{Duration, Instant};

/// Mouse travel (physical px on Windows, points on macOS) that counts as a drag-selection rather than a click.
const DRAG_DISTANCE: i32 = 6;

/// The system's double-click limits: the longest pause and the largest move between clicks.
#[derive(Clone, Copy, Debug)]
pub struct DoubleClick {
    pub time: Duration,
    pub dx: i32,
    pub dy: i32,
}

/// Gesture state between a press and its release. Only touched on the hook thread.
#[derive(Default)]
pub struct Gesture {
    down_at: Option<(i32, i32)>,
    down_on_icon: bool,
    ibeam_at_down: bool,
    last_up: Option<((i32, i32), Instant)>,
    clicks: u32,
}

impl Gesture {
    /// The left button went down. `ibeam` says whether the pointer is the text I-beam.
    pub fn press(
        &mut self,
        at: (i32, i32),
        now: Instant,
        on_icon: bool,
        ibeam: bool,
        double: DoubleClick,
    ) {
        self.down_on_icon = on_icon;
        if on_icon {
            return;
        }
        let repeat = self.last_up.is_some_and(|(p, up)| {
            now.duration_since(up) <= double.time
                && (p.0 - at.0).abs() <= double.dx
                && (p.1 - at.1).abs() <= double.dy
        });
        self.clicks = if repeat { self.clicks + 1 } else { 1 };
        self.down_at = Some(at);
        self.ibeam_at_down = ibeam;
    }

    /// The left button went up. True when the press and this release selected text.
    /// `ibeam_now` is only asked when the press did not happen over text.
    pub fn release(
        &mut self,
        at: (i32, i32),
        now: Instant,
        ibeam_now: impl FnOnce() -> bool,
    ) -> bool {
        if self.down_on_icon {
            return false;
        }
        self.last_up = Some((at, now));
        let Some(down) = self.down_at.take() else {
            return false;
        };
        let dragged =
            (down.0 - at.0).abs() > DRAG_DISTANCE || (down.1 - at.1).abs() > DRAG_DISTANCE;
        (dragged || self.clicks >= 2) && (self.ibeam_at_down || ibeam_now())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOUBLE: DoubleClick = DoubleClick {
        time: Duration::from_millis(500),
        dx: 4,
        dy: 4,
    };

    fn click(g: &mut Gesture, at: (i32, i32), now: Instant, ibeam: bool) -> bool {
        g.press(at, now, false, ibeam, DOUBLE);
        g.release(at, now + Duration::from_millis(60), || ibeam)
    }

    #[test]
    fn a_drag_over_text_selects() {
        let mut g = Gesture::default();
        let now = Instant::now();
        g.press((100, 100), now, false, true, DOUBLE);
        assert!(g.release((180, 102), now, || false));
    }

    #[test]
    fn a_single_click_does_not_select() {
        let mut g = Gesture::default();
        assert!(!click(&mut g, (100, 100), Instant::now(), true));
    }

    #[test]
    fn a_double_click_selects_only_within_the_system_limits() {
        let start = Instant::now();
        let mut g = Gesture::default();
        click(&mut g, (100, 100), start, true);
        assert!(click(
            &mut g,
            (102, 101),
            start + Duration::from_millis(200),
            true
        ));

        let mut slow = Gesture::default();
        click(&mut slow, (100, 100), start, true);
        assert!(!click(
            &mut slow,
            (100, 100),
            start + Duration::from_secs(2),
            true
        ));

        let mut far = Gesture::default();
        click(&mut far, (100, 100), start, true);
        assert!(!click(
            &mut far,
            (120, 100),
            start + Duration::from_millis(200),
            true
        ));
    }

    #[test]
    fn a_drag_away_from_text_does_not_select() {
        let mut g = Gesture::default();
        let now = Instant::now();
        g.press((100, 100), now, false, false, DOUBLE);
        assert!(!g.release((300, 100), now, || false));
    }

    #[test]
    fn a_drag_that_ends_over_text_selects() {
        let mut g = Gesture::default();
        let now = Instant::now();
        g.press((100, 100), now, false, false, DOUBLE);
        assert!(g.release((300, 100), now, || true));
    }

    #[test]
    fn clicks_on_the_icon_are_ignored() {
        let mut g = Gesture::default();
        let now = Instant::now();
        g.press((100, 100), now, true, true, DOUBLE);
        assert!(!g.release((300, 100), now, || true));
        assert!(!g.release((300, 100), now, || true));
    }
}
