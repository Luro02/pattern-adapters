use core::cmp;
use core::ops;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Range {
    start: usize,
    end: usize,
}

impl Range {
    /// Returns the start of the `Range` (inclusive bound).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use pattern_adaptors::Range;
    /// let range = Range::from(1..3);
    ///
    /// assert_eq!(range.start(), 1);
    /// ```
    ///
    /// ```ignore
    /// # use pattern_adaptors::Range;
    /// let range = Range::from(4..=5);
    ///
    /// assert_eq!(range.start(), 4);
    /// ```
    ///
    /// ```ignore
    /// # use pattern_adaptors::Range;
    /// let range = Range::from(6..3);
    ///
    /// assert_eq!(range.start(), 6);
    /// ```
    #[must_use]
    pub const fn start(self) -> usize {
        self.start
    }

    /// Returns the end of the `Range` (exclusive bound).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use pattern_adaptors::Range;
    /// let range = Range::from(1..3);
    ///
    /// assert_eq!(range.end(), 3);
    /// ```
    ///
    /// ```ignore
    /// # use pattern_adaptors::Range;
    /// let range = Range::from(1..=3);
    ///
    /// assert_eq!(range.end(), 4);
    /// ```
    ///
    /// ```ignore
    /// # use pattern_adaptors::Range;
    /// let range = Range::from(6..3);
    ///
    /// assert_eq!(range.end(), 3);
    /// ```
    #[must_use]
    pub const fn end(self) -> usize {
        self.end
    }

    /// Returns the intersection between this range and the other range.
    ///
    /// The intersection can be thought of like this:
    ///
    /// ```text
    /// self   : 1 2 3 4 5
    /// other  :     3 4 5 6 7 8
    /// result :     3 4 5
    /// ```
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use pattern_adaptors::Range;
    /// let range = Range::from(1..=5);
    /// let other_range = Range::from(3..=8);
    ///
    /// assert_eq!(range.intersect(other_range), Some(Range::from(3..=5)));
    /// ```
    ///
    /// If the ranges have no items in common, `None` is returned:
    ///
    /// ```ignore
    /// # use pattern_adaptors::Range;
    /// let range = Range::from(1..=4);
    /// let other_range = Range::from(5..=7);
    ///
    /// assert_eq!(range.intersect(other_range), None);
    /// ```
    #[must_use]
    pub fn intersect(self, other: Self) -> Option<Self> {
        // self   : 1 2 3 4 .
        // other  :         5 6 7
        //
        // self   :     3 4
        // other  : 1 2

        // if self.end <= other.start || other.end <= self.start || other.is_empty() ||
        // self.is_empty() {
        if other.start() >= self.end()
            || self.start() >= other.end()
            || other.is_empty()
            || self.is_empty()
        {
            return None; // empty range?
        }

        let start = cmp::max(self.start(), other.start());
        let end = cmp::min(self.end(), other.end());
        Some((start..end).into())
    }

    /// Returns the (absolute) length of the range.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use pattern_adaptors::Range;
    /// let range = Range::from(1..=9);
    ///
    /// assert_eq!(range.len(), 9);
    /// ```
    ///
    /// ```ignore
    /// # use pattern_adaptors::Range;
    /// let range = Range::from(9..2);
    ///
    /// assert_eq!(range.len(), 7);
    /// ```
    #[must_use]
    pub fn len(self) -> usize {
        cmp::max(self.start(), self.end()) - cmp::min(self.start(), self.end())
    }

    #[must_use]
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }
}

impl From<ops::Range<usize>> for Range {
    fn from(value: ops::Range<usize>) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

impl From<ops::RangeInclusive<usize>> for Range {
    fn from(value: ops::RangeInclusive<usize>) -> Self {
        Self {
            start: *value.start(),
            end: value.end() + 1,
        }
    }
}

// NOTE: seems to be a clippy bug (can not implement foreign traits on foreign
//       types)
#[allow(clippy::from_over_into)]
impl Into<(usize, usize)> for Range {
    fn into(self) -> (usize, usize) {
        (self.start, self.end)
    }
}

impl From<(usize, usize)> for Range {
    fn from(value: (usize, usize)) -> Self {
        (value.0..value.1).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_range_intersection() {
        assert_eq!(
            Range::from(1..5).intersect((2..3).into()),
            Some(Range::from(2..3))
        );
        assert_eq!(Range::from(1..5).intersect((5..5).into()), None);
        // self:   1 2 3 4 5
        // other:      3 4 5 6 7 8
        // result:     3 4 5
        assert_eq!(
            Range::from(1..6).intersect((3..9).into()),
            Some(Range::from(3..6))
        );
        // self   : 1 2 3 4 .
        // other  :         5 6 7
        assert_eq!(Range::from(1..5).intersect((5..7).into()), None);
        // self   :     3 4
        // other  : 1 2
        assert_eq!(Range::from(3..5).intersect((1..3).into()), None);
        assert_eq!(Range::from(1..5).intersect((0..1).into()), None);
    }
}
