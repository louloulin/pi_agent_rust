//! The composer line editor: buffer + cursor (char-indexed), prompt
//! history, and the word/kill motions the keymap dispatches. No app or
//! terminal knowledge — pure string surgery, unit-tested.
//!
//! This is a port of Martty's Input implementation to Rust, adapted for pi's
//! TUI architecture with visual cursor tracking across soft-wrapped lines.

use std::collections::VecDeque;
use unicode_width::UnicodeWidthChar;

/// Editable prompt state. `cursor` is a char index (not bytes).
#[derive(Debug, Clone)]
pub struct ChatInput {
    /// The text buffer
    pub buf: String,
    /// Cursor position in characters
    pub cursor: usize,
    /// Command history
    history: VecDeque<String>,
    /// Current history position (None = not browsing history)
    hist_pos: Option<usize>,
    /// Stash for interrupted edit (used for history browsing)
    stash: String,
    /// Preferred visual column when moving vertically
    preferred_visual_col: Option<usize>,
    /// Whether cursor is at the end of a soft-wrapped line
    cursor_at_wrap_end: bool,
    /// Kill ring for yanked text
    kill_ring: VecDeque<String>,
}

impl Default for ChatInput {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatInput {
    /// Create a new empty ChatInput
    pub fn new() -> Self {
        ChatInput {
            buf: String::new(),
            cursor: 0,
            history: VecDeque::new(),
            hist_pos: None,
            stash: String::new(),
            preferred_visual_col: None,
            cursor_at_wrap_end: false,
            kill_ring: VecDeque::new(),
        }
    }

    /// Get the byte offset for a character index
    fn byte_at(&self, char_idx: usize) -> usize {
        self.buf
            .char_indices()
            .nth(char_idx)
            .map(|(b, _)| b)
            .unwrap_or(self.buf.len())
    }

    /// Character count of the buffer (cursor upper bound).
    pub fn len_chars(&self) -> usize {
        self.buf.chars().count()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Get the current text
    pub fn text(&self) -> &str {
        &self.buf
    }

    /// Insert a character at cursor position
    pub fn insert(&mut self, ch: char) {
        let at = self.byte_at(self.cursor);
        self.buf.insert(at, ch);
        self.cursor += 1;
        self.hist_pos = None;
        self.preferred_visual_col = None;
        self.cursor_at_wrap_end = false;
    }

    /// Insert a string at cursor position
    pub fn insert_str(&mut self, s: &str) {
        let at = self.byte_at(self.cursor);
        self.buf.insert_str(at, s);
        self.cursor += s.chars().count();
        self.hist_pos = None;
        self.preferred_visual_col = None;
        self.cursor_at_wrap_end = false;
    }

    /// Delete character before cursor
    pub fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let start = self.byte_at(self.cursor - 1);
        let end = self.byte_at(self.cursor);
        self.buf.replace_range(start..end, "");
        self.cursor -= 1;
        self.preferred_visual_col = None;
        self.cursor_at_wrap_end = false;
    }

    /// Delete character after cursor (forward delete)
    pub fn delete_forward(&mut self) {
        if self.cursor >= self.len_chars() {
            return;
        }
        let start = self.byte_at(self.cursor);
        let end = self.byte_at(self.cursor + 1);
        self.buf.replace_range(start..end, "");
        self.preferred_visual_col = None;
        self.cursor_at_wrap_end = false;
    }

    /// Delete word before cursor
    pub fn delete_word_back(&mut self) {
        let i = self.prev_word();
        let start = self.byte_at(i);
        let end = self.byte_at(self.cursor);
        self.buf.replace_range(start..end, "");
        self.cursor = i;
        self.preferred_visual_col = None;
        self.cursor_at_wrap_end = false;
    }

    /// Kill (delete) from cursor to end of line, saving to kill ring
    pub fn kill_to_end(&mut self) {
        let at = self.byte_at(self.cursor);
        let killed = self.buf[at..].to_string();
        if !killed.is_empty() {
            self.kill_ring.push_back(killed);
            if self.kill_ring.len() > 20 {
                self.kill_ring.pop_front();
            }
        }
        self.buf.truncate(at);
        self.preferred_visual_col = None;
        self.cursor_at_wrap_end = false;
    }

    /// Kill (delete) from start of line to cursor, saving to kill ring
    pub fn kill_to_start(&mut self) {
        let at = self.byte_at(self.cursor);
        let killed = self.buf[..at].to_string();
        if !killed.is_empty() {
            self.kill_ring.push_back(killed);
            if self.kill_ring.len() > 20 {
                self.kill_ring.pop_front();
            }
        }
        self.buf.replace_range(..at, "");
        self.cursor = 0;
        self.preferred_visual_col = None;
        self.cursor_at_wrap_end = false;
    }

    /// Yank (paste) from kill ring
    pub fn yank(&mut self) -> bool {
        if let Some(text) = self.kill_ring.back().cloned() {
            self.insert_str(&text);
            true
        } else {
            false
        }
    }

    /// Get position of previous word boundary
    pub fn prev_word(&self) -> usize {
        let chars: Vec<char> = self.buf.chars().collect();
        let mut i = self.cursor.min(chars.len());
        while i > 0 && chars[i - 1].is_whitespace() {
            i -= 1;
        }
        while i > 0 && !chars[i - 1].is_whitespace() {
            i -= 1;
        }
        i
    }

    /// Get position of next word boundary
    pub fn next_word(&self) -> usize {
        let chars: Vec<char> = self.buf.chars().collect();
        let n = chars.len();
        let mut i = self.cursor.min(n);
        while i < n && chars[i].is_whitespace() {
            i += 1;
        }
        while i < n && !chars[i].is_whitespace() {
            i += 1;
        }
        i
    }

    /// Clear the input buffer
    pub fn clear(&mut self) {
        self.buf.clear();
        self.cursor = 0;
        self.hist_pos = None;
        self.preferred_visual_col = None;
        self.cursor_at_wrap_end = false;
    }

    /// Set the buffer content
    pub fn set(&mut self, s: String) {
        self.cursor = s.chars().count();
        self.buf = s;
        self.preferred_visual_col = None;
        self.cursor_at_wrap_end = false;
    }

    /// Reset the preferred visual column (used after horizontal movement)
    pub fn reset_vertical_goal(&mut self) {
        self.preferred_visual_col = None;
        self.cursor_at_wrap_end = false;
    }

    /// Number of hard/soft-wrapped visual rows at `width` display cells.
    pub fn visual_row_count(&self, width: usize) -> usize {
        let (_, _, rows) = self.visual_layout(width.max(1));
        rows
    }

    /// Move cursor one character left
    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.preferred_visual_col = None;
            self.cursor_at_wrap_end = false;
        }
    }

    /// Move cursor one character right
    pub fn move_right(&mut self) {
        if self.cursor < self.len_chars() {
            self.cursor += 1;
            self.preferred_visual_col = None;
            self.cursor_at_wrap_end = false;
        }
    }

    /// Move to start of line (document)
    pub fn move_to_start(&mut self) {
        self.cursor = 0;
        self.preferred_visual_col = None;
        self.cursor_at_wrap_end = false;
    }

    /// Move to end of line (document)
    pub fn move_to_end(&mut self) {
        self.cursor = self.len_chars();
        self.preferred_visual_col = None;
        self.cursor_at_wrap_end = false;
    }

    /// Move by one visual row while preserving the original display column.
    pub fn move_vertical(&mut self, width: usize, direction: i8) {
        let width = width.max(1);
        let (candidates, rows) = self.visual_candidates(width);
        let (row, col) = self.visual_cursor(width);
        let goal = *self.preferred_visual_col.get_or_insert(col);
        let target = if direction < 0 {
            row.checked_sub(1)
        } else if direction > 0 && row + 1 < rows {
            Some(row + 1)
        } else {
            None
        };
        let Some(target) = target else {
            return;
        };

        let mut best: Option<(usize, usize, bool)> = None;
        for &(index, candidate_row, candidate_col, wrap_end) in &candidates {
            if candidate_row != target {
                continue;
            }
            let distance = candidate_col.abs_diff(goal);
            if best.is_none_or(|(_, best_distance, _)| distance < best_distance) {
                best = Some((index, distance, wrap_end));
            }
        }
        if let Some((index, _, wrap_end)) = best {
            self.cursor = index;
            self.cursor_at_wrap_end = wrap_end;
        }
    }

    /// Move to the beginning of the current rendered row
    pub fn move_to_visual_line_start(&mut self, width: usize) {
        let width = width.max(1);
        let row = self.visual_cursor(width).0;
        if let Some(&(index, _, _, wrap_end)) = self
            .visual_candidates(width)
            .0
            .iter()
            .filter(|&&(_, candidate_row, _, _)| candidate_row == row)
            .min_by_key(|&&(_, _, col, _)| col)
        {
            self.cursor = index;
            self.cursor_at_wrap_end = wrap_end;
        }
        self.preferred_visual_col = None;
    }

    /// Move to the end of the current rendered row
    pub fn move_to_visual_line_end(&mut self, width: usize) {
        let width = width.max(1);
        let row = self.visual_cursor(width).0;
        if let Some(&(index, _, _, wrap_end)) = self
            .visual_candidates(width)
            .0
            .iter()
            .filter(|&&(_, candidate_row, _, _)| candidate_row == row)
            .max_by_key(|&&(_, _, col, _)| col)
        {
            self.cursor = index;
            self.cursor_at_wrap_end = wrap_end;
        }
        self.preferred_visual_col = None;
    }

    /// Current rendered caret position, including upstream affinity at a
    /// soft-wrap line end.
    pub fn visual_cursor(&self, width: usize) -> (usize, usize) {
        let width = width.max(1);
        let (chars, carets, _) = self.visual_layout(width);
        let index = self.cursor.min(carets.len().saturating_sub(1));
        if self.cursor_at_wrap_end {
            if let Some(position) = Self::wrap_end_position(index, &chars, &carets) {
                return position;
            }
        }
        carets[index]
    }

    /// All visual affinities for character boundaries.
    fn visual_candidates(&self, width: usize) -> (Vec<(usize, usize, usize, bool)>, usize) {
        let (chars, carets, rows) = self.visual_layout(width);
        let mut candidates = Vec::with_capacity(carets.len() * 2);
        for (index, &(row, col)) in carets.iter().enumerate() {
            candidates.push((index, row, col, false));
            if let Some((upstream_row, upstream_col)) =
                Self::wrap_end_position(index, &chars, &carets)
            {
                candidates.push((index, upstream_row, upstream_col, true));
            }
        }
        (candidates, rows)
    }

    fn wrap_end_position(
        index: usize,
        chars: &[char],
        carets: &[(usize, usize)],
    ) -> Option<(usize, usize)> {
        let previous = *chars.get(index.checked_sub(1)?)?;
        if previous == '\n' {
            return None;
        }
        let (row, col) = carets[index - 1];
        let end = col + UnicodeWidthChar::width(previous).unwrap_or(0).max(1);
        (carets[index].0 > row).then_some((row, end))
    }

    /// Characters plus caret (row, display-column) for every boundary.
    fn visual_layout(&self, width: usize) -> (Vec<char>, Vec<(usize, usize)>, usize) {
        let chars: Vec<char> = self.buf.chars().collect();
        let mut carets = vec![(0, 0); chars.len() + 1];
        let (mut row, mut col) = (0usize, 0usize);
        for (index, ch) in chars.iter().copied().enumerate() {
            if ch == '\n' {
                carets[index] = (row, col);
                row += 1;
                col = 0;
                carets[index + 1] = (row, col);
                continue;
            }
            let char_width = UnicodeWidthChar::width(ch).unwrap_or(0).max(1);
            if col + char_width > width {
                row += 1;
                col = 0;
            }
            carets[index] = (row, col);
            col += char_width;
            carets[index + 1] = (row, col);
        }
        if !chars.is_empty() && chars.last() != Some(&'\n') && col >= width {
            row += 1;
            carets[chars.len()] = (row, 0);
        }
        (chars, carets, row + 1)
    }

    // --- History management ---

    /// Add current buffer to history
    pub fn add_to_history(&mut self) {
        if !self.buf.trim().is_empty() {
            // Don't add duplicates at the end
            if self.history.back() != Some(&self.buf) {
                self.history.push_back(self.buf.clone());
                if self.history.len() > 100 {
                    self.history.pop_front();
                }
            }
        }
        self.hist_pos = None;
    }

    /// Move to previous history entry
    pub fn history_prev(&mut self) -> bool {
        if self.history.is_empty() {
            return false;
        }

        // Stash current content if not already browsing
        if self.hist_pos.is_none() && !self.buf.is_empty() {
            self.stash = self.buf.clone();
        }

        let new_pos = match self.hist_pos {
            None => self.history.len().saturating_sub(1),
            Some(p) => {
                if p == 0 {
                    // Already at the oldest entry, can't go back further
                    return false;
                }
                p - 1
            }
        };

        if let Some(entry) = self.history.get(new_pos) {
            self.buf = entry.clone();
            self.cursor = entry.chars().count();
            self.hist_pos = Some(new_pos);
            self.preferred_visual_col = None;
            self.cursor_at_wrap_end = false;
            true
        } else {
            false
        }
    }

    /// Move to next history entry
    pub fn history_next(&mut self) -> bool {
        let Some(pos) = self.hist_pos else {
            // Never started browsing history
            return false;
        };

        // Check if we're at or past the end of history
        if pos >= self.history.len() {
            // We've gone past the newest entry, restore stash and exit browsing
            self.buf = self.stash.clone();
            self.cursor = self.buf.chars().count();
            self.hist_pos = None;
            self.stash = String::new(); // Clear stash after restoring
            self.preferred_visual_col = None;
            self.cursor_at_wrap_end = false;
            return true;
        }

        // Normal case: move to next entry
        if let Some(entry) = self.history.get(pos + 1) {
            self.buf = entry.clone();
            self.cursor = entry.chars().count();
            self.hist_pos = Some(pos + 1);
            self.preferred_visual_col = None;
            self.cursor_at_wrap_end = false;
            true
        } else {
            // Past the end, restore stash
            self.buf = self.stash.clone();
            self.cursor = self.buf.chars().count();
            self.hist_pos = None;
            self.stash = String::new(); // Clear stash after restoring
            self.preferred_visual_col = None;
            self.cursor_at_wrap_end = false;
            true
        }
    }

    /// Check if currently browsing history
    pub fn is_browsing_history(&self) -> bool {
        self.hist_pos.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn editor(text: &str) -> ChatInput {
        let mut i = ChatInput::new();
        i.set(text.into());
        i
    }

    #[test]
    fn word_motions_hop_whitespace_delimited_words() {
        let mut i = editor("hello brave world");
        assert_eq!(i.next_word(), 17, "already at end");
        i.cursor = 0;
        assert_eq!(i.next_word(), 5);
        i.cursor = 5;
        assert_eq!(i.next_word(), 11);
        i.cursor = 17;
        assert_eq!(i.prev_word(), 12);
        i.cursor = 12;
        assert_eq!(i.prev_word(), 6);
    }

    #[test]
    fn kill_commands_split_at_cursor() {
        let mut i = editor("hello world");
        i.cursor = 6;
        i.kill_to_start();
        assert_eq!(i.buf, "world");
        assert_eq!(i.cursor, 0);

        let mut i = editor("hello world");
        i.cursor = 5;
        i.kill_to_end();
        assert_eq!(i.buf, "hello");
    }

    #[test]
    fn delete_forward_and_multibyte_safety() {
        let mut i = editor("a中b");
        i.cursor = 1;
        i.delete_forward();
        assert_eq!(i.buf, "ab");
        assert_eq!(i.cursor, 1);
        i.delete_forward();
        assert_eq!(i.buf, "a");
        i.delete_forward();
        assert_eq!(i.buf, "a", "at end: no-op");
    }

    #[test]
    fn delete_word_back_eats_trailing_whitespace_then_word() {
        let mut i = editor("one two   ");
        i.delete_word_back();
        assert_eq!(i.buf, "one ");
        assert_eq!(i.cursor, 4);
    }

    #[test]
    fn vertical_motion_preserves_the_display_column_across_hard_lines() {
        let mut i = editor("abcd\nefgh\nij");

        i.move_vertical(20, -1);
        assert_eq!(i.cursor, 7, "row 2, display column 2");
        i.move_vertical(20, -1);
        assert_eq!(i.cursor, 2, "row 1, display column 2");
        i.move_vertical(20, 1);
        i.move_vertical(20, 1);
        assert_eq!(i.cursor, 12, "the preferred column survives round-trips");
    }

    #[test]
    fn vertical_motion_uses_soft_wrapped_visual_rows() {
        let mut i = editor("abcdefghij");

        i.move_vertical(4, -1);
        assert_eq!(i.cursor, 6, "wrapped row 2, display column 2");
        i.move_vertical(4, -1);
        assert_eq!(i.cursor, 2, "wrapped row 1, display column 2");
    }

    #[test]
    fn history_navigation() {
        let mut i = ChatInput::new();
        i.set("first".into());
        i.add_to_history();
        i.set("second".into());
        i.add_to_history();
        i.set("third".into());
        i.add_to_history();

        // Navigate to previous
        assert!(i.history_prev());
        assert_eq!(i.buf, "third");

        assert!(i.history_prev());
        assert_eq!(i.buf, "second");

        assert!(i.history_prev());
        assert_eq!(i.buf, "first");

        // At the oldest, should stay
        assert!(!i.history_prev());

        // Navigate forward
        assert!(i.history_next());
        assert_eq!(i.buf, "second");

        assert!(i.history_next());
        assert_eq!(i.buf, "third");

        // After latest, should restore stash (the buffer before browsing started)
        assert!(i.history_next());
        assert_eq!(i.buf, "third"); // Stash was set to "third" when browsing started

        // Further calls should do nothing (exited browsing mode)
        assert!(!i.history_next());
    }

    #[test]
    fn visual_layout_with_multibyte_chars() {
        let mut i = editor("a中bcd");
        assert_eq!(i.visual_row_count(3), 3, "中 takes 2 cells, wraps");
        assert_eq!(i.visual_row_count(10), 1, "fits on one line");
    }
}
