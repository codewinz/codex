#[cfg(not(test))]
use std::thread;
#[cfg(not(test))]
use std::time::Duration;

#[cfg(not(test))]
use tokio::sync::mpsc;
#[cfg(not(test))]
use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;

#[cfg(not(test))]
use super::InputRecoverySource;

#[cfg(not(test))]
const POLL_INTERVAL: Duration = Duration::from_millis(25);
#[cfg(not(test))]
const VK_CONTROL: i32 = 0x11;
#[cfg(not(test))]
const VK_MENU: i32 = 0x12;
#[cfg(not(test))]
const VK_R: i32 = 0x52;

#[cfg(not(test))]
pub(crate) fn spawn_recovery_hotkey_monitor() -> mpsc::UnboundedReceiver<InputRecoverySource> {
    let (tx, rx) = mpsc::unbounded_channel();
    if let Err(err) = thread::Builder::new()
        .name("codex-windows-recovery-hotkey".to_string())
        .spawn(move || poll_recovery_hotkey(tx))
    {
        tracing::warn!(error = %err, "failed to spawn Windows recovery hotkey monitor");
    }
    rx
}

#[cfg(not(test))]
fn poll_recovery_hotkey(tx: mpsc::UnboundedSender<InputRecoverySource>) {
    let mut state = RecoveryShortcutState::default();
    while !tx.is_closed() {
        if state.observe(
            async_key_down(VK_CONTROL),
            async_key_down(VK_MENU),
            async_key_down(VK_R),
        ) && tx.send(InputRecoverySource::Shortcut).is_err()
        {
            return;
        }

        thread::sleep(POLL_INTERVAL);
    }
}

#[cfg(not(test))]
fn async_key_down(virtual_key: i32) -> bool {
    // Safety: GetAsyncKeyState only observes process-global keyboard state for a virtual-key code.
    unsafe { GetAsyncKeyState(virtual_key) as u16 & 0x8000 != 0 }
}

#[derive(Default)]
pub(super) struct RecoveryShortcutState {
    was_pressed: bool,
}

impl RecoveryShortcutState {
    pub(super) fn observe(&mut self, ctrl_down: bool, alt_down: bool, r_down: bool) -> bool {
        let pressed = ctrl_down && alt_down && r_down;
        let should_emit = pressed && !self.was_pressed;
        self.was_pressed = pressed;
        should_emit
    }
}
