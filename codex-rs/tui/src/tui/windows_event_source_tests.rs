use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use pretty_assertions::assert_eq;
use windows_sys::Win32::System::Console::LEFT_ALT_PRESSED;
use windows_sys::Win32::System::Console::LEFT_CTRL_PRESSED;
use windows_sys::Win32::System::Console::RIGHT_ALT_PRESSED;
use windows_sys::Win32::System::Console::RIGHT_CTRL_PRESSED;

use super::windows_event_source::ParserState;
use super::windows_event_source::RawInputRecord;
use super::windows_event_source::RawKeyEvent;
use super::windows_event_source::parse_raw_input_record;

const VK_C: u16 = b'C' as u16;
const VK_ESCAPE: u16 = 0x1b;
const VK_R: u16 = b'R' as u16;

fn parse(raw: RawInputRecord) -> Option<Event> {
    parse_raw_input_record(raw, &mut ParserState::default())
}

#[test]
fn ctrl_alt_r_without_unicode_char_maps_to_recovery_shortcut_shape() {
    let event = parse(RawInputRecord::Key(RawKeyEvent {
        key_down: true,
        repeat_count: 1,
        virtual_key_code: VK_R,
        virtual_scan_code: 0,
        unicode_char: 0,
        control_key_state: LEFT_CTRL_PRESSED | LEFT_ALT_PRESSED,
    }));

    assert_eq!(
        event,
        Some(Event::Key(KeyEvent::new_with_kind(
            KeyCode::Char('r'),
            KeyModifiers::CONTROL | KeyModifiers::ALT,
            KeyEventKind::Press,
        )))
    );
}

#[test]
fn ctrl_c_with_control_char_maps_to_regular_char_key() {
    let event = parse(RawInputRecord::Key(RawKeyEvent {
        key_down: true,
        repeat_count: 1,
        virtual_key_code: VK_C,
        virtual_scan_code: 0,
        unicode_char: 0x03,
        control_key_state: RIGHT_CTRL_PRESSED,
    }));

    assert_eq!(
        event,
        Some(Event::Key(KeyEvent::new_with_kind(
            KeyCode::Char('c'),
            KeyModifiers::CONTROL,
            KeyEventKind::Press,
        )))
    );
}

#[test]
fn non_modifier_key_release_is_preserved() {
    let event = parse(RawInputRecord::Key(RawKeyEvent {
        key_down: false,
        repeat_count: 1,
        virtual_key_code: VK_ESCAPE,
        virtual_scan_code: 0,
        unicode_char: 0,
        control_key_state: 0,
    }));

    assert_eq!(
        event,
        Some(Event::Key(KeyEvent::new_with_kind(
            KeyCode::Esc,
            KeyModifiers::NONE,
            KeyEventKind::Release,
        )))
    );
}

#[test]
fn resize_adds_one_for_crossterm_compatibility() {
    let event = parse(RawInputRecord::Resize {
        width: 79,
        height: 23,
    });

    assert_eq!(event, Some(Event::Resize(80, 24)));
}

#[test]
fn focus_records_map_to_focus_events() {
    assert_eq!(parse(RawInputRecord::Focus(true)), Some(Event::FocusGained));
    assert_eq!(parse(RawInputRecord::Focus(false)), Some(Event::FocusLost));
}

#[test]
fn unsupported_records_are_ignored() {
    assert_eq!(parse(RawInputRecord::Unsupported), None);
}

#[test]
fn either_side_alt_ctrl_are_merged_into_modifiers() {
    let event = parse(RawInputRecord::Key(RawKeyEvent {
        key_down: true,
        repeat_count: 1,
        virtual_key_code: VK_R,
        virtual_scan_code: 0,
        unicode_char: 0,
        control_key_state: RIGHT_CTRL_PRESSED | RIGHT_ALT_PRESSED,
    }));

    assert_eq!(
        event,
        Some(Event::Key(KeyEvent::new_with_kind(
            KeyCode::Char('r'),
            KeyModifiers::CONTROL | KeyModifiers::ALT,
            KeyEventKind::Press,
        )))
    );
}
