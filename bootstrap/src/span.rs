use std::ops::Range;

/// Byte-range span into a source file.
///
/// Spans are expressed as half-open ranges `[start, end)`, matching the layout
/// returned by `logos::Lexer::span()`.  Helper methods provide convenient access
/// to derived information such as line/column positions without duplicating the
/// conversion logic across the compiler pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[allow(dead_code)]
impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn from_range(range: Range<usize>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }

    pub fn len(self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty(self) -> bool {
        self.start >= self.end
    }

    /// Clamp the span to the provided source length.
    pub fn clamp(self, source_len: usize) -> Self {
        Self {
            start: self.start.min(source_len),
            end: self.end.min(source_len),
        }
    }

    /// Slice the underlying source string with this span.
    pub fn snippet<'a>(self, source: &'a str) -> &'a str {
        let clamped = self.clamp(source.len());
        &source[clamped.start..clamped.end]
    }

    /// Line/column for the starting byte of the span.
    pub fn start_position(self, map: &SourceMap) -> (usize, usize) {
        map.line_col(self.start)
    }

    /// Line/column for the last byte inside the span.
    pub fn end_position(self, map: &SourceMap) -> (usize, usize) {
        if self.end == 0 {
            (1, 1)
        } else {
            map.line_col(self.end - 1)
        }
    }
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self {
        Span::from_range(range)
    }
}

/// Pre-computed line start indexes for a source file.
///
/// This provides O(log n) lookups for converting byte offsets into line/column
/// diagnostics, which keeps downstream passes cheap even when they need to map
/// spans repeatedly.
#[derive(Debug, Clone)]
pub struct SourceMap {
    line_starts: Vec<usize>,
    source_len: usize,
}

#[allow(dead_code)]
impl SourceMap {
    pub fn new(source: &str) -> Self {
        let mut line_starts = Vec::with_capacity(source.lines().count() + 1);
        line_starts.push(0);

        for (idx, ch) in source.char_indices() {
            if ch == '\n' {
                let next = idx + ch.len_utf8();
                if next <= source.len() {
                    line_starts.push(next);
                }
            }
        }

        if *line_starts.last().unwrap() != source.len() {
            line_starts.push(source.len());
        }

        Self {
            line_starts,
            source_len: source.len(),
        }
    }

    pub fn source_len(&self) -> usize {
        self.source_len
    }

    pub fn line_count(&self) -> usize {
        // `line_starts` always has a sentinel equal to the source length.
        self.line_starts.len().saturating_sub(1).max(1)
    }

    pub fn line_col(&self, offset: usize) -> (usize, usize) {
        if self.line_starts.is_empty() {
            return (1, offset.saturating_add(1));
        }

        let offset = offset.min(self.source_len);
        let line_idx = match self.line_starts.binary_search(&offset) {
            Ok(idx) => {
                // Exact match means the offset is at the start of line `idx`.
                if idx == self.line_starts.len() - 1 {
                    idx.saturating_sub(1)
                } else {
                    idx
                }
            }
            Err(0) => 0,
            Err(idx) => idx - 1,
        };

        let line_start = self.line_starts[line_idx];
        let column = offset.saturating_sub(line_start) + 1;
        (line_idx + 1, column)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_snippet_clamps() {
        let source = "hello\nworld";
        let span = Span::new(0, 5);
        assert_eq!(span.snippet(source), "hello");

        let span = Span::new(0, 64);
        assert_eq!(span.snippet(source), "hello\nworld");

        let span = Span::new(6, 64);
        assert_eq!(span.snippet(source), "world");
    }

    #[test]
    fn source_map_line_col() {
        let source = "line1\nline2\nline3";
        let map = SourceMap::new(source);

        assert_eq!(map.line_col(0), (1, 1));
        assert_eq!(map.line_col(5), (1, 6)); // '\n'
        assert_eq!(map.line_col(6), (2, 1));
        assert_eq!(map.line_col(10), (2, 5));
        assert_eq!(map.line_col(16), (3, 5));
        assert_eq!(map.line_col(64), (3, 6));
    }

    #[test]
    fn span_positions_use_map() {
        let source = "foo\nbar\nbaz";
        let map = SourceMap::new(source);
        let span = Span::new(4, 7); // "bar"

        assert_eq!(span.start_position(&map), (2, 1));
        assert_eq!(span.end_position(&map), (2, 3));
    }
}
