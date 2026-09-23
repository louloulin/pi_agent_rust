//! ftui adapter for ChatInput: bridges terminal events to ChatInput actions.
//!
//! This module provides the integration layer between ftui's event system
//! and the ChatInput editor, handling key event classification and
//! translation to editor actions.

use crate::chat_input::ChatInput;
use ftui::core::event::KeyEvent;
use ftui::KeyCode;

/// Actions that can be performed on the chat input
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatInputAction {
    /// Insert a character
    Insert(char),
    /// Move cursor left
    CursorLeft,
    /// Move cursor right
    CursorRight,
    /// Move cursor up (visual line)
    CursorUp,
    /// Move cursor down (visual line)
    CursorDown,
    /// Move to start of line
    LineStart,
    /// Move to end of line
    LineEnd,
    /// Delete character before cursor
    Backspace,
    /// Delete character after cursor
    DeleteForward,
    /// Delete word before cursor
    DeleteWordBack,
    /// Kill to end of line
    KillToEnd,
    /// Kill to start of line
    KillToStart,
    /// Yank (paste) from kill ring
    Yank,
    /// Submit the input
    Submit,
    /// Insert newline
    Newline,
    /// Go to previous history entry
    HistoryPrev,
    /// Go to next history entry
    HistoryNext,
    /// Move to start of document
    DocumentStart,
    /// Move to end of document
    DocumentEnd,
    /// Move word left
    WordLeft,
    /// Move word right
    WordRight,
    /// Escape/cancel
    Escape,
    /// No-op (key not handled)
    Ignore,
}

/// Classify a key event into a ChatInput action
pub fn classify_key(key: &KeyEvent) -> ChatInputAction {
    use ChatInputAction::*;
    let ctrl = key.modifiers.contains(ftui::Modifiers::CTRL);
    let alt = key.modifiers.contains(ftui::Modifiers::ALT);
    let shift = key.modifiers.contains(ftui::Modifiers::SHIFT);

    match key.code {
        // Enter = submit
        KeyCode::Enter if !shift => Submit,
        // Shift+Enter = newline
        KeyCode::Enter if shift => Newline,
        KeyCode::Escape => Escape,

        // Control chords
        KeyCode::Char('a') if ctrl => LineStart,
        KeyCode::Char('e') if ctrl => LineEnd,
        KeyCode::Char('b') if ctrl => CursorLeft,
        KeyCode::Char('f') if ctrl => CursorRight,
        KeyCode::Char('p') if ctrl => HistoryPrev,
        KeyCode::Char('n') if ctrl => HistoryNext,
        KeyCode::Char('d') if ctrl => DeleteForward,
        KeyCode::Char('k') if ctrl => KillToEnd,
        KeyCode::Char('u') if ctrl => KillToStart,
        KeyCode::Char('w') if ctrl => DeleteWordBack,
        KeyCode::Char('y') if ctrl => Yank,

        // Alt chords (word movement on macOS)
        KeyCode::Char('b') if alt => WordLeft,
        KeyCode::Char('f') if alt => WordRight,

        // Arrow keys
        KeyCode::Left if alt || ctrl => WordLeft,
        KeyCode::Right if alt || ctrl => WordRight,
        KeyCode::Left => CursorLeft,
        KeyCode::Right => CursorRight,
        KeyCode::Up => CursorUp,
        KeyCode::Down => CursorDown,

        // Home/End
        KeyCode::Home => LineStart,
        KeyCode::End => LineEnd,

        // Backspace/Delete
        KeyCode::Backspace if alt || ctrl => DeleteWordBack,
        KeyCode::Backspace => Backspace,
        KeyCode::Delete => DeleteForward,

        // Tab for completion (handled elsewhere)
        KeyCode::Tab => Ignore,
        KeyCode::BackTab => Ignore,

        // Character insertion
        KeyCode::Char(ch) if !ctrl && !alt => Insert(ch),

        _ => Ignore,
    }
}

/// Apply an action to the ChatInput
pub fn apply_action(input: &mut ChatInput, action: ChatInputAction, wrap_width: usize) {
    match action {
        ChatInputAction::Insert(ch) => input.insert(ch),
        ChatInputAction::CursorLeft => input.move_left(),
        ChatInputAction::CursorRight => input.move_right(),
        ChatInputAction::CursorUp => input.move_vertical(wrap_width, -1),
        ChatInputAction::CursorDown => input.move_vertical(wrap_width, 1),
        ChatInputAction::LineStart => input.move_to_visual_line_start(wrap_width),
        ChatInputAction::LineEnd => input.move_to_visual_line_end(wrap_width),
        ChatInputAction::Backspace => input.backspace(),
        ChatInputAction::DeleteForward => input.delete_forward(),
        ChatInputAction::DeleteWordBack => input.delete_word_back(),
        ChatInputAction::KillToEnd => input.kill_to_end(),
        ChatInputAction::KillToStart => input.kill_to_start(),
        ChatInputAction::Yank => {
            let _ = input.yank();
        }
        ChatInputAction::DocumentStart => input.move_to_start(),
        ChatInputAction::DocumentEnd => input.move_to_end(),
        ChatInputAction::WordLeft => input.cursor = input.prev_word(),
        ChatInputAction::WordRight => input.cursor = input.next_word(),
        ChatInputAction::HistoryPrev => {
            let _ = input.history_prev();
        }
        ChatInputAction::HistoryNext => {
            let _ = input.history_next();
        }
        ChatInputAction::Newline => input.insert('\n'),
        ChatInputAction::Submit | ChatInputAction::Escape | ChatInputAction::Ignore => {}
    }
}

/// Handle a key event and update the input
/// Returns true if the event was handled
pub fn handle_key(input: &mut ChatInput, key: &KeyEvent, wrap_width: usize) -> bool {
    let action = classify_key(key);
    if action != ChatInputAction::Ignore {
        apply_action(input, action, wrap_width);
        true
    } else {
        false
    }
}

/// Cursor position info for rendering
#[derive(Debug, Clone, Copy)]
pub struct CursorPosition {
    pub row: usize,
    pub col: usize,
    pub visual_row: usize,
}

/// Get the cursor position info for rendering
pub fn get_cursor_position(input: &ChatInput, wrap_width: usize) -> CursorPosition {
    let (visual_row, col) = input.visual_cursor(wrap_width);
    // Calculate absolute row from cursor position
    let absolute_row = input
        .buf
        .chars()
        .take(input.cursor)
        .filter(|&c| c == '\n')
        .count();

    CursorPosition {
        row: absolute_row,
        col,
        visual_row,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_classification() {
        // Basic characters
        let key = KeyEvent::new(ftui::KeyCode::Char('x'));
        assert_eq!(classify_key(&key), ChatInputAction::Insert('x'));

        // Control chords
        let key = KeyEvent::new(ftui::KeyCode::Char('a')).with_modifiers(ftui::Modifiers::CTRL);
        assert_eq!(classify_key(&key), ChatInputAction::LineStart);

        // Arrow keys
        let key = KeyEvent::new(ftui::KeyCode::Left);
        assert_eq!(classify_key(&key), ChatInputAction::CursorLeft);

        // Alt+arrow = word movement
        let key = KeyEvent::new(ftui::KeyCode::Left).with_modifiers(ftui::Modifiers::ALT);
        assert_eq!(classify_key(&key), ChatInputAction::WordLeft);

        // Enter = submit
        let key = KeyEvent::new(ftui::KeyCode::Enter);
        assert_eq!(classify_key(&key), ChatInputAction::Submit);

        // Shift+Enter = newline
        let key = KeyEvent::new(ftui::KeyCode::Enter).with_modifiers(ftui::Modifiers::SHIFT);
        assert_eq!(classify_key(&key), ChatInputAction::Newline);
    }

    #[test]
    fn test_action_application() {
        let mut input = ChatInput::new();
        input.set("hello world".into());
        // Reset cursor to start
        input.cursor = 0;

        // Move to word boundary (skip 'hello', land after it)
        apply_action(&mut input, ChatInputAction::WordRight, 80);
        assert_eq!(input.cursor, 5, "should land after 'hello'");

        // Move back to start of line
        apply_action(&mut input, ChatInputAction::LineStart, 80);
        assert_eq!(input.cursor, 0);

        // Delete word back from start (no-op)
        apply_action(&mut input, ChatInputAction::DeleteWordBack, 80);
        assert_eq!(input.buf, "hello world");

        // Move to end and kill to start
        apply_action(&mut input, ChatInputAction::DocumentEnd, 80);
        apply_action(&mut input, ChatInputAction::KillToStart, 80);
        assert_eq!(input.buf, "");

        // Test insert
        input.set("test".into());
        input.cursor = 0;
        apply_action(&mut input, ChatInputAction::Insert('X'), 80);
        assert_eq!(input.buf, "Xtest");
    }

    #[test]
    fn test_handle_key() {
        let mut input = ChatInput::new();

        let key = KeyEvent::new(ftui::KeyCode::Char('h'));
        assert!(handle_key(&mut input, &key, 80));
        assert_eq!(input.buf, "h");

        let key = KeyEvent::new(ftui::KeyCode::Char('i'));
        assert!(handle_key(&mut input, &key, 80));
        assert_eq!(input.buf, "hi");

        // Unhandled key (e.g., function keys) - F1 not supported
        let key = KeyEvent::new(ftui::KeyCode::Null);
        assert!(!handle_key(&mut input, &key, 80));
    }
}
