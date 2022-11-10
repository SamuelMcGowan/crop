use std::ops::RangeBounds;

use super::metrics::ByteMetric;
use super::utils::*;
use super::{Chunks, TextChunk, TextChunkIter};
use crate::tree::Tree;
use crate::RopeSlice;

#[cfg(not(test))]
const ROPE_FANOUT: usize = 8;

#[cfg(test)]
const ROPE_FANOUT: usize = 2;

/// TODO: docs
pub struct Rope {
    root: Tree<ROPE_FANOUT, TextChunk>,
}

impl Rope {
    /// TODO: docs
    pub fn byte_len(&self) -> usize {
        self.root.summary().bytes
    }

    /// TODO: docs
    pub fn byte_slice<R>(&self, byte_range: R) -> RopeSlice<'_>
    where
        R: RangeBounds<usize>,
    {
        let (start, end) = range_to_tuple(byte_range, 0, self.byte_len());
        RopeSlice::new(self.root.slice(ByteMetric(start)..ByteMetric(end)))
    }

    fn chunks(&self) -> Chunks<'_> {
        Chunks { chunks: self.root.leaves() }
    }

    pub(super) const fn fanout() -> usize {
        ROPE_FANOUT
    }

    /// TODO: docs
    #[inline]
    pub fn new() -> Self {
        Self::from("")
    }
}

impl std::fmt::Debug for Rope {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("Rope(\"")?;
        std::fmt::Display::fmt(self, f)?;
        f.write_str("\")")
    }
}

impl std::fmt::Display for Rope {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for chunk in self.chunks() {
            f.write_str(chunk)?;
        }
        Ok(())
    }
}

impl Default for Rope {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl From<&str> for Rope {
    #[inline]
    fn from(s: &str) -> Self {
        Rope { root: Tree::from_leaves(TextChunkIter::new(s)) }
    }
}

impl From<String> for Rope {
    #[inline]
    fn from(s: String) -> Self {
        if s.len() <= TextChunk::max_bytes() {
            // If the string fits in one chunk we can avoid the allocation.
            Rope { root: Tree::from_leaves([TextChunk::new(s)]) }
        } else {
            Rope::from(&*s)
        }
    }
}

impl<'a> From<std::borrow::Cow<'a, str>> for Rope {
    #[inline]
    fn from(moo: std::borrow::Cow<'a, str>) -> Self {
        match moo {
            std::borrow::Cow::Owned(s) => Rope::from(s),
            std::borrow::Cow::Borrowed(s) => Rope::from(s),
        }
    }
}

impl std::cmp::PartialEq<Rope> for Rope {
    #[inline]
    fn eq(&self, rhs: &Rope) -> bool {
        todo!()
    }
}

impl std::cmp::PartialEq<str> for Rope {
    #[inline]
    fn eq(&self, rhs: &str) -> bool {
        if self.byte_len() != rhs.len() {
            false
        } else {
            chunks_eq_str(self.chunks(), rhs)
        }
    }
}

impl std::cmp::PartialEq<Rope> for str {
    #[inline]
    fn eq(&self, rhs: &Rope) -> bool {
        rhs == self
    }
}

impl<'a> std::cmp::PartialEq<&'a str> for Rope {
    #[inline]
    fn eq(&self, rhs: &&'a str) -> bool {
        self == *rhs
    }
}

impl<'a> std::cmp::PartialEq<Rope> for &'a str {
    #[inline]
    fn eq(&self, rhs: &Rope) -> bool {
        rhs == self
    }
}

impl std::cmp::PartialEq<String> for Rope {
    #[inline]
    fn eq(&self, rhs: &String) -> bool {
        self == &**rhs
    }
}

impl std::cmp::PartialEq<Rope> for String {
    #[inline]
    fn eq(&self, rhs: &Rope) -> bool {
        rhs == self
    }
}

impl<'a> std::cmp::PartialEq<std::borrow::Cow<'a, str>> for Rope {
    #[inline]
    fn eq(&self, rhs: &std::borrow::Cow<'a, str>) -> bool {
        self == &**rhs
    }
}

impl<'a> std::cmp::PartialEq<Rope> for std::borrow::Cow<'a, str> {
    #[inline]
    fn eq(&self, rhs: &Rope) -> bool {
        rhs == self
    }
}

impl std::cmp::Eq for Rope {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn easy() {
        let r = Rope::from("Hello there");
        assert_eq!(11, r.byte_len());

        let r = Rope::from("🐕‍🦺");
        assert_eq!(11, r.byte_len());
    }

    #[test]
    fn slice() {
        let r = Rope::from("Hello there");

        let s = r.byte_slice(..);
        assert_eq!(11, s.byte_len());

        let s = s.byte_slice(0..5);
        assert_eq!(5, s.byte_len());

        let r = Rope::from(
            "Hello there this is a really long line that I'm gonna use to \
             test this fucking slicing methods that we got going on well \
             hope this shit works 'cause if it doesn't I'm gonna fucking \
             lose it ok bye :)",
        );

        let s = r.byte_slice(14..79);
        assert_eq!(65, s.byte_len());

        let s = r.byte_slice(0..11);
        assert_eq!(11, s.byte_len());

        let s = r.byte_slice(0..=10);
        assert_eq!(11, s.byte_len());
    }

    #[test]
    fn partial_eq() {
        let s = "This is a service dog: 🐕‍🦺";
        let r = Rope::from(s);
        assert_eq!(s, r);
        assert_eq!(s, r.byte_slice(..));
    }
}