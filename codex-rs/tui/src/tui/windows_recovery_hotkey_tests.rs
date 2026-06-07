use super::windows_recovery_hotkey::RecoveryShortcutState;

#[test]
fn recovery_shortcut_state_emits_once_per_press() {
    let mut state = RecoveryShortcutState::default();

    assert!(!state.observe(
        /*ctrl_down*/ false, /*alt_down*/ false, /*r_down*/ false
    ));
    assert!(state.observe(
        /*ctrl_down*/ true, /*alt_down*/ true, /*r_down*/ true
    ));
    assert!(!state.observe(
        /*ctrl_down*/ true, /*alt_down*/ true, /*r_down*/ true
    ));
    assert!(!state.observe(
        /*ctrl_down*/ true, /*alt_down*/ true, /*r_down*/ false
    ));
    assert!(state.observe(
        /*ctrl_down*/ true, /*alt_down*/ true, /*r_down*/ true
    ));
}
