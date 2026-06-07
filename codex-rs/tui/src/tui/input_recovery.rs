//! Terminal input recovery detection for Windows terminals that leak VT key fragments.

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;

const MAX_PENDING_LEN: usize = 16;
const LEAK_SEQUENCE_THRESHOLD: u8 = 2;

const KNOWN_LEAK_SEQUENCES: &[&str] = &[
    "[A", "[B", "[C", "[D", "[H", "[F", "[I", "[O", "[Z", "[2~", "[3~", "[5~", "[6~",
];

const ENCODED_CTRL_ALT_R_SEQUENCES: &[&str] = &["[114;7u", "[82;7u", "[27;7;114~", "[27;7;82~"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputRecoverySource {
    Shortcut,
    LeakedSequence,
}

impl InputRecoverySource {
    pub(crate) fn label(self) -> &'static str {
        match self {
            InputRecoverySource::Shortcut => "Ctrl+Alt+R",
            InputRecoverySource::LeakedSequence => "leaked key sequence",
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct TerminalInputRecoveryDetector {
    pending: String,
    consecutive_leak_sequences: u8,
}

impl TerminalInputRecoveryDetector {
    pub(crate) fn observe_key(&mut self, key_event: KeyEvent) -> Option<InputRecoverySource> {
        if recovery_shortcut_matches(key_event) {
            self.reset();
            return Some(InputRecoverySource::Shortcut);
        }

        if !matches!(key_event.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
            self.reset();
            return None;
        }

        let KeyCode::Char(ch) = key_event.code else {
            self.reset();
            return None;
        };

        if key_event
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
            || !ch.is_ascii()
        {
            self.reset();
            return None;
        }

        self.pending.push(ch);
        if self.pending.len() > MAX_PENDING_LEN {
            let keep_from = self.pending.len() - MAX_PENDING_LEN;
            self.pending = self.pending[keep_from..].to_string();
        }

        if let Some(sequence) = complete_terminal_sequence_suffix(&self.pending) {
            let source = if ENCODED_CTRL_ALT_R_SEQUENCES.contains(&sequence) {
                Some(InputRecoverySource::Shortcut)
            } else {
                self.consecutive_leak_sequences = self.consecutive_leak_sequences.saturating_add(1);
                (self.consecutive_leak_sequences >= LEAK_SEQUENCE_THRESHOLD)
                    .then_some(InputRecoverySource::LeakedSequence)
            };
            self.pending.clear();
            if source.is_some() {
                self.reset();
            }
            return source;
        }

        if let Some(start) = self.pending.rfind('[') {
            let suffix = self.pending[start..].to_string();
            if is_potential_terminal_sequence_prefix(&suffix) {
                self.pending = suffix;
                return None;
            }
        }

        self.reset();
        None
    }

    fn reset(&mut self) {
        self.pending.clear();
        self.consecutive_leak_sequences = 0;
    }
}

pub(crate) fn recovery_shortcut_matches(key_event: KeyEvent) -> bool {
    matches!(key_event.kind, KeyEventKind::Press | KeyEventKind::Repeat)
        && matches!(key_event.code, KeyCode::Char('r') | KeyCode::Char('R'))
        && key_event
            .modifiers
            .contains(KeyModifiers::CONTROL | KeyModifiers::ALT)
}

fn complete_terminal_sequence_suffix(text: &str) -> Option<&str> {
    let start = text.rfind('[')?;
    let suffix = &text[start..];
    if KNOWN_LEAK_SEQUENCES.contains(&suffix) || is_generic_csi_tail(suffix) {
        return Some(suffix);
    }
    None
}

fn is_potential_terminal_sequence_prefix(text: &str) -> bool {
    KNOWN_LEAK_SEQUENCES
        .iter()
        .chain(ENCODED_CTRL_ALT_R_SEQUENCES)
        .any(|sequence| sequence.starts_with(text))
        || is_generic_csi_prefix(text)
}

fn is_generic_csi_tail(text: &str) -> bool {
    let Some(body) = text.strip_prefix('[') else {
        return false;
    };
    let Some(first) = body.bytes().next() else {
        return false;
    };
    if !is_csi_parameter_start(first) {
        return false;
    }
    let Some((&final_byte, parameter_bytes)) = body.as_bytes().split_last() else {
        return false;
    };
    !parameter_bytes.is_empty()
        && parameter_bytes.iter().copied().all(is_csi_parameter_byte)
        && is_csi_final_byte(final_byte)
}

fn is_generic_csi_prefix(text: &str) -> bool {
    let Some(body) = text.strip_prefix('[') else {
        return false;
    };
    let Some(first) = body.bytes().next() else {
        return true;
    };
    is_csi_parameter_start(first) && body.bytes().all(is_csi_parameter_byte)
}

fn is_csi_parameter_start(byte: u8) -> bool {
    byte.is_ascii_digit() || byte == b'?'
}

fn is_csi_parameter_byte(byte: u8) -> bool {
    byte.is_ascii_digit() || matches!(byte, b';' | b':' | b'?')
}

fn is_csi_final_byte(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'~'
}
