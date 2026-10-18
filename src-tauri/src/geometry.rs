//! Screen rectangles in physical pixels, shared by capture, popup placement and the platform
//! layer.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    /// The rectangle reaching `dx` and `dy` from a point on each side.
    pub fn around((x, y): (i32, i32), dx: i32, dy: i32) -> Self {
        Self {
            left: x - dx,
            top: y - dy,
            right: x + dx,
            bottom: y + dy,
        }
    }

    pub fn inflate(self, dx: i32, dy: i32) -> Self {
        Self {
            left: self.left - dx,
            top: self.top - dy,
            right: self.right + dx,
            bottom: self.bottom + dy,
        }
    }

    /// The part of `self` inside `other`; empty (zero or negative size) when they do not meet.
    pub fn intersect(self, other: Self) -> Self {
        Self {
            left: self.left.max(other.left),
            top: self.top.max(other.top),
            right: self.right.min(other.right),
            bottom: self.bottom.min(other.bottom),
        }
    }

    /// The smallest rectangle holding both.
    pub fn union(self, other: Self) -> Self {
        Self {
            left: self.left.min(other.left),
            top: self.top.min(other.top),
            right: self.right.max(other.right),
            bottom: self.bottom.max(other.bottom),
        }
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.left && x < self.right && y >= self.top && y < self.bottom
    }

    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: Rect = Rect {
        left: 0,
        top: 0,
        right: 100,
        bottom: 50,
    };
    const B: Rect = Rect {
        left: 80,
        top: 40,
        right: 200,
        bottom: 90,
    };

    #[test]
    fn intersect_and_union() {
        assert_eq!(
            A.intersect(B),
            Rect {
                left: 80,
                top: 40,
                right: 100,
                bottom: 50
            }
        );
        assert_eq!(
            A.union(B),
            Rect {
                left: 0,
                top: 0,
                right: 200,
                bottom: 90
            }
        );
    }

    #[test]
    fn contains_excludes_the_far_edges() {
        assert!(A.contains(0, 0));
        assert!(A.contains(99, 49));
        assert!(!A.contains(100, 10));
        assert!(!A.contains(10, 50));
    }

    #[test]
    fn around_a_point() {
        assert_eq!(
            Rect::around((10, 20), 5, 2),
            Rect {
                left: 5,
                top: 18,
                right: 15,
                bottom: 22
            }
        );
        assert_eq!(Rect::around((10, 20), 5, 2).inflate(1, 1).width(), 12);
    }
}
