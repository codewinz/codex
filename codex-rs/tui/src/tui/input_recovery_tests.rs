use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;

use super::input_recovery::InputRecoverySource;
use super::input_recovery::TerminalInputRecoveryDetector;

fn plain_char(ch: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE)
}

fn feed_sequence(
    detector: &mut TerminalInputRecoveryDetector,
    sequence: &str,
) -> Option<InputRecoverySource> {
    sequence
        .chars()
        .filter_map(|ch| detector.observe_key(plain_char(ch)))
        .last()
}

#[test]
fn ctrl_alt_r_key_event_triggers_manual_recovery() {
    let mut detector = TerminalInputRecoveryDetector::default();

    let source = detector.observe_key(KeyEvent::new(
        KeyCode::Char('r'),
        KeyModifiers::CONTROL | KeyModifiers::ALT,
    ));

    assert_eq!(source, Some(InputRecoverySource::Shortcut));
}

#[test]
fn leaked_ctrl_alt_r_csi_triggers_manual_recovery() {
    for sequence in ["[114;7u", "[82;7u", "[27;7;114~", "[27;7;82~"] {
        let mut detector = TerminalInputRecoveryDetector::default();

        let source = feed_sequence(&mut detector, sequence);

        assert_eq!(source, Some(InputRecoverySource::Shortcut));
    }
}

#[test]
fn repeated_arrow_fragments_trigger_leaked_sequence_recovery() {
    let mut detector = TerminalInputRecoveryDetector::default();

    let source = feed_sequence(&mut detector, "[A[A[B[D[C");

    assert_eq!(source, Some(InputRecoverySource::LeakedSequence));
}

#[test]
fn repeated_shift_tab_csi_fragments_trigger_leaked_sequence_recovery() {
    let mut detector = TerminalInputRecoveryDetector::default();

    let source = feed_sequence(&mut detector, "[1;2Z[1;2Z");

    assert_eq!(source, Some(InputRecoverySource::LeakedSequence));
}

#[test]
fn regular_text_does_not_trigger_recovery() {
    let mut detector = TerminalInputRecoveryDetector::default();

    let source = feed_sequence(&mut detector, "array[index] and [A once");

    assert_eq!(source, None);
}
