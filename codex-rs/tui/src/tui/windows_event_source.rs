use std::io;
use std::pin::Pin;
use std::ptr;
use std::task::Context;
use std::task::Poll;
use std::thread;
use std::thread::JoinHandle;

use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use tokio::sync::mpsc;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
use windows_sys::Win32::Foundation::WAIT_FAILED;
use windows_sys::Win32::Foundation::WAIT_OBJECT_0;
use windows_sys::Win32::System::Console::CAPSLOCK_ON;
use windows_sys::Win32::System::Console::FOCUS_EVENT;
use windows_sys::Win32::System::Console::FOCUS_EVENT_RECORD;
use windows_sys::Win32::System::Console::GetStdHandle;
use windows_sys::Win32::System::Console::INPUT_RECORD;
use windows_sys::Win32::System::Console::KEY_EVENT;
use windows_sys::Win32::System::Console::KEY_EVENT_RECORD;
use windows_sys::Win32::System::Console::LEFT_ALT_PRESSED;
use windows_sys::Win32::System::Console::LEFT_CTRL_PRESSED;
use windows_sys::Win32::System::Console::RIGHT_ALT_PRESSED;
use windows_sys::Win32::System::Console::RIGHT_CTRL_PRESSED;
use windows_sys::Win32::System::Console::ReadConsoleInputW;
use windows_sys::Win32::System::Console::SHIFT_PRESSED;
use windows_sys::Win32::System::Console::STD_INPUT_HANDLE;
use windows_sys::Win32::System::Console::WINDOW_BUFFER_SIZE_EVENT;
use windows_sys::Win32::System::Console::WINDOW_BUFFER_SIZE_RECORD;
use windows_sys::Win32::System::Threading::CreateEventW;
use windows_sys::Win32::System::Threading::INFINITE;
use windows_sys::Win32::System::Threading::SetEvent;
use windows_sys::Win32::System::Threading::WaitForMultipleObjects;

use super::event_stream::EventResult;
use super::event_stream::EventSource;

const VK_BACK: u16 = 0x08;
const VK_TAB: u16 = 0x09;
const VK_RETURN: u16 = 0x0d;
const VK_SHIFT: u16 = 0x10;
const VK_CONTROL: u16 = 0x11;
const VK_MENU: u16 = 0x12;
const VK_ESCAPE: u16 = 0x1b;
const VK_SPACE: u16 = 0x20;
const VK_0: u16 = b'0' as u16;
const VK_9: u16 = b'9' as u16;
const VK_A: u16 = b'A' as u16;
const VK_Z: u16 = b'Z' as u16;
const VK_PRIOR: u16 = 0x21;
const VK_NEXT: u16 = 0x22;
const VK_END: u16 = 0x23;
const VK_HOME: u16 = 0x24;
const VK_LEFT: u16 = 0x25;
const VK_UP: u16 = 0x26;
const VK_RIGHT: u16 = 0x27;
const VK_DOWN: u16 = 0x28;
const VK_INSERT: u16 = 0x2d;
const VK_DELETE: u16 = 0x2e;
const VK_NUMPAD0: u16 = 0x60;
const VK_NUMPAD9: u16 = 0x69;
const VK_F1: u16 = 0x70;
const VK_F24: u16 = 0x87;

const READ_BATCH_SIZE: usize = 32;

/// Windows console event source with an explicitly restartable reader thread.
pub(crate) struct WindowsEventSource {
    rx: mpsc::UnboundedReceiver<EventResult>,
    _shutdown: Option<ReaderShutdown>,
}

impl WindowsEventSource {
    fn new() -> io::Result<Self> {
        let stdin_handle = stdin_handle()?;
        let stop_event = WinHandle::create_manual_reset_event()?;
        let (tx, rx) = mpsc::unbounded_channel();
        let reader_thread = spawn_reader_thread(stdin_handle, stop_event.raw(), tx)?;

        Ok(Self {
            rx,
            _shutdown: Some(ReaderShutdown {
                stop_event,
                reader_thread: Some(reader_thread),
            }),
        })
    }

    fn from_startup_error(err: io::Error) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let _ = tx.send(Err(err));
        Self {
            rx,
            _shutdown: None,
        }
    }
}

impl Default for WindowsEventSource {
    fn default() -> Self {
        Self::new().unwrap_or_else(Self::from_startup_error)
    }
}

impl EventSource for WindowsEventSource {
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<EventResult>> {
        Pin::new(&mut self.get_mut().rx).poll_recv(cx)
    }
}

struct ReaderShutdown {
    stop_event: WinHandle,
    reader_thread: Option<JoinHandle<()>>,
}

impl Drop for ReaderShutdown {
    fn drop(&mut self) {
        // Safety: stop_event is an owned manual-reset event handle that remains valid until
        // after the reader thread has joined.
        if unsafe { SetEvent(self.stop_event.raw()) } == 0 {
            tracing::warn!(
                "failed to signal Windows input reader shutdown: {}",
                io::Error::last_os_error()
            );
        }

        if let Some(reader_thread) = self.reader_thread.take()
            && reader_thread.join().is_err()
        {
            tracing::warn!("Windows input reader thread panicked during shutdown");
        }
    }
}

struct WinHandle(HANDLE);

impl WinHandle {
    fn create_manual_reset_event() -> io::Result<Self> {
        // Safety: null security attributes and name are accepted by CreateEventW. The returned
        // handle is owned by WinHandle and closed in Drop.
        let handle = unsafe { CreateEventW(ptr::null(), 1, 0, ptr::null()) };
        if handle == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self(handle))
    }

    fn raw(&self) -> HANDLE {
        self.0
    }
}

impl Drop for WinHandle {
    fn drop(&mut self) {
        // Safety: WinHandle owns this handle and closes it exactly once.
        let _ = unsafe { CloseHandle(self.0) };
    }
}

fn stdin_handle() -> io::Result<HANDLE> {
    // Safety: GetStdHandle does not transfer ownership.
    let handle = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
    if handle == 0 || handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    Ok(handle)
}

fn spawn_reader_thread(
    stdin_handle: HANDLE,
    stop_handle: HANDLE,
    tx: mpsc::UnboundedSender<EventResult>,
) -> io::Result<JoinHandle<()>> {
    thread::Builder::new()
        .name("codex-windows-input-reader".to_string())
        .spawn(move || read_console_events(stdin_handle, stop_handle, tx))
}

enum WaitOutcome {
    Input,
    Stop,
}

fn wait_for_input_or_stop(stdin_handle: HANDLE, stop_handle: HANDLE) -> io::Result<WaitOutcome> {
    let handles = [stdin_handle, stop_handle];
    // Safety: both handles are valid for the lifetime of this wait; we wait on either one.
    let result =
        unsafe { WaitForMultipleObjects(handles.len() as u32, handles.as_ptr(), 0, INFINITE) };

    match result {
        WAIT_OBJECT_0 => Ok(WaitOutcome::Input),
        result if result == WAIT_OBJECT_0 + 1 => Ok(WaitOutcome::Stop),
        WAIT_FAILED => Err(io::Error::last_os_error()),
        other => Err(io::Error::other(format!(
            "WaitForMultipleObjects returned unexpected result {other}"
        ))),
    }
}

fn read_console_events(
    stdin_handle: HANDLE,
    stop_handle: HANDLE,
    tx: mpsc::UnboundedSender<EventResult>,
) {
    let mut parser_state = ParserState::default();

    loop {
        match wait_for_input_or_stop(stdin_handle, stop_handle) {
            Ok(WaitOutcome::Input) => {}
            Ok(WaitOutcome::Stop) => return,
            Err(err) => {
                let _ = tx.send(Err(err));
                return;
            }
        }

        let mut records: [INPUT_RECORD; READ_BATCH_SIZE] = unsafe { std::mem::zeroed() };
        let mut records_read = 0;
        // Safety: records points to a valid writable buffer and records_read is a valid out param.
        let ok = unsafe {
            ReadConsoleInputW(
                stdin_handle,
                records.as_mut_ptr(),
                records.len() as u32,
                &mut records_read,
            )
        };
        if ok == 0 {
            let _ = tx.send(Err(io::Error::last_os_error()));
            return;
        }

        for record in records.into_iter().take(records_read as usize) {
            // Safety: the INPUT_RECORD union variant is selected by EventType.
            let raw_record = unsafe { raw_input_record(record) };
            if let Some(event) = parse_raw_input_record(raw_record, &mut parser_state)
                && tx.send(Ok(event)).is_err()
            {
                return;
            }
        }
    }
}

#[derive(Default)]
pub(super) struct ParserState {
    surrogate_buffer: Option<u16>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RawInputRecord {
    Key(RawKeyEvent),
    Resize { width: i16, height: i16 },
    Focus(bool),
    Unsupported,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RawKeyEvent {
    pub(super) key_down: bool,
    pub(super) repeat_count: u16,
    pub(super) virtual_key_code: u16,
    pub(super) virtual_scan_code: u16,
    pub(super) unicode_char: u16,
    pub(super) control_key_state: u32,
}

unsafe fn raw_input_record(record: INPUT_RECORD) -> RawInputRecord {
    match record.EventType as u32 {
        KEY_EVENT => {
            // Safety: EventType identifies the active INPUT_RECORD union field.
            let key_event = unsafe { record.Event.KeyEvent };
            RawInputRecord::Key(raw_key_event(key_event))
        }
        WINDOW_BUFFER_SIZE_EVENT => {
            // Safety: EventType identifies the active INPUT_RECORD union field.
            let resize_event = unsafe { record.Event.WindowBufferSizeEvent };
            raw_resize_event(resize_event)
        }
        FOCUS_EVENT => {
            // Safety: EventType identifies the active INPUT_RECORD union field.
            let focus_event = unsafe { record.Event.FocusEvent };
            raw_focus_event(focus_event)
        }
        _ => RawInputRecord::Unsupported,
    }
}

fn raw_key_event(key_event: KEY_EVENT_RECORD) -> RawKeyEvent {
    // Safety: UnicodeChar is the console input variant used by ReadConsoleInputW.
    let unicode_char = unsafe { key_event.uChar.UnicodeChar };
    RawKeyEvent {
        key_down: key_event.bKeyDown != 0,
        repeat_count: key_event.wRepeatCount,
        virtual_key_code: key_event.wVirtualKeyCode,
        virtual_scan_code: key_event.wVirtualScanCode,
        unicode_char,
        control_key_state: key_event.dwControlKeyState,
    }
}

fn raw_resize_event(resize_event: WINDOW_BUFFER_SIZE_RECORD) -> RawInputRecord {
    RawInputRecord::Resize {
        width: resize_event.dwSize.X,
        height: resize_event.dwSize.Y,
    }
}

fn raw_focus_event(focus_event: FOCUS_EVENT_RECORD) -> RawInputRecord {
    RawInputRecord::Focus(focus_event.bSetFocus != 0)
}

pub(super) fn parse_raw_input_record(
    record: RawInputRecord,
    parser_state: &mut ParserState,
) -> Option<Event> {
    match record {
        RawInputRecord::Key(key_event) => parse_key_event(key_event, parser_state),
        RawInputRecord::Resize { width, height } => Some(Event::Resize(
            windows_coord_to_crossterm_size(width),
            windows_coord_to_crossterm_size(height),
        )),
        RawInputRecord::Focus(true) => Some(Event::FocusGained),
        RawInputRecord::Focus(false) => Some(Event::FocusLost),
        RawInputRecord::Unsupported => None,
    }
}

fn windows_coord_to_crossterm_size(value: i16) -> u16 {
    u16::try_from(value).unwrap_or_default().saturating_add(1)
}

fn parse_key_event(key_event: RawKeyEvent, parser_state: &mut ParserState) -> Option<Event> {
    let modifiers = modifiers_from_control_key_state(key_event.control_key_state);
    let key_code = match key_code_from_raw_key(key_event, modifiers, parser_state)? {
        ParsedKeyCode::PendingSurrogate => return None,
        ParsedKeyCode::KeyCode(key_code) => key_code,
    };
    parser_state.surrogate_buffer = None;

    let kind = if key_event.key_down {
        KeyEventKind::Press
    } else {
        KeyEventKind::Release
    };
    Some(Event::Key(KeyEvent::new_with_kind(
        key_code, modifiers, kind,
    )))
}

enum ParsedKeyCode {
    KeyCode(KeyCode),
    PendingSurrogate,
}

fn key_code_from_raw_key(
    key_event: RawKeyEvent,
    modifiers: KeyModifiers,
    parser_state: &mut ParserState,
) -> Option<ParsedKeyCode> {
    let _ = key_event.repeat_count;
    let _ = key_event.virtual_scan_code;

    let is_alt_code =
        key_event.virtual_key_code == VK_MENU && !key_event.key_down && key_event.unicode_char != 0;
    if is_alt_code {
        return parsed_key_code_from_utf16(key_event.unicode_char, parser_state);
    }

    let is_numpad_numeric_key = (VK_NUMPAD0..=VK_NUMPAD9).contains(&key_event.virtual_key_code);
    let is_only_alt_modifier = modifiers.contains(KeyModifiers::ALT)
        && !modifiers.contains(KeyModifiers::SHIFT | KeyModifiers::CONTROL);
    if is_only_alt_modifier && is_numpad_numeric_key {
        return None;
    }

    match key_event.virtual_key_code {
        VK_SHIFT | VK_CONTROL | VK_MENU => None,
        VK_BACK => Some(ParsedKeyCode::KeyCode(KeyCode::Backspace)),
        VK_ESCAPE => Some(ParsedKeyCode::KeyCode(KeyCode::Esc)),
        VK_RETURN => Some(ParsedKeyCode::KeyCode(KeyCode::Enter)),
        VK_F1..=VK_F24 => Some(ParsedKeyCode::KeyCode(KeyCode::F(
            (key_event.virtual_key_code - VK_F1 + 1) as u8,
        ))),
        VK_LEFT => Some(ParsedKeyCode::KeyCode(KeyCode::Left)),
        VK_UP => Some(ParsedKeyCode::KeyCode(KeyCode::Up)),
        VK_RIGHT => Some(ParsedKeyCode::KeyCode(KeyCode::Right)),
        VK_DOWN => Some(ParsedKeyCode::KeyCode(KeyCode::Down)),
        VK_PRIOR => Some(ParsedKeyCode::KeyCode(KeyCode::PageUp)),
        VK_NEXT => Some(ParsedKeyCode::KeyCode(KeyCode::PageDown)),
        VK_HOME => Some(ParsedKeyCode::KeyCode(KeyCode::Home)),
        VK_END => Some(ParsedKeyCode::KeyCode(KeyCode::End)),
        VK_DELETE => Some(ParsedKeyCode::KeyCode(KeyCode::Delete)),
        VK_INSERT => Some(ParsedKeyCode::KeyCode(KeyCode::Insert)),
        VK_TAB if modifiers.contains(KeyModifiers::SHIFT) => {
            Some(ParsedKeyCode::KeyCode(KeyCode::BackTab))
        }
        VK_TAB => Some(ParsedKeyCode::KeyCode(KeyCode::Tab)),
        _ => key_code_from_character_key(key_event, parser_state),
    }
}

fn key_code_from_character_key(
    key_event: RawKeyEvent,
    parser_state: &mut ParserState,
) -> Option<ParsedKeyCode> {
    match key_event.unicode_char {
        0x00..=0x1f => fallback_char_from_virtual_key(key_event)
            .map(|ch| ParsedKeyCode::KeyCode(KeyCode::Char(ch))),
        utf16 @ 0xd800..=0xdfff => parsed_key_code_from_utf16(utf16, parser_state),
        utf16 => char::from_u32(utf16 as u32)
            .map(KeyCode::Char)
            .map(ParsedKeyCode::KeyCode),
    }
}

fn parsed_key_code_from_utf16(utf16: u16, parser_state: &mut ParserState) -> Option<ParsedKeyCode> {
    if !(0xd800..=0xdfff).contains(&utf16) {
        return char::from_u32(utf16 as u32)
            .map(KeyCode::Char)
            .map(ParsedKeyCode::KeyCode);
    }

    match parser_state.surrogate_buffer {
        Some(buffered) => {
            parser_state.surrogate_buffer = None;
            char::decode_utf16([buffered, utf16])
                .next()
                .and_then(Result::ok)
                .map(KeyCode::Char)
                .map(ParsedKeyCode::KeyCode)
        }
        None => {
            parser_state.surrogate_buffer = Some(utf16);
            Some(ParsedKeyCode::PendingSurrogate)
        }
    }
}

fn fallback_char_from_virtual_key(key_event: RawKeyEvent) -> Option<char> {
    match key_event.virtual_key_code {
        VK_0..=VK_9 => Some(key_event.virtual_key_code as u8 as char),
        VK_A..=VK_Z => {
            let ch = key_event.virtual_key_code as u8 as char;
            let shifted = key_event.control_key_state & SHIFT_PRESSED != 0;
            let caps_locked = key_event.control_key_state & CAPSLOCK_ON != 0;
            if shifted ^ caps_locked {
                Some(ch)
            } else {
                Some(ch.to_ascii_lowercase())
            }
        }
        VK_SPACE => Some(' '),
        _ => None,
    }
}

fn modifiers_from_control_key_state(control_key_state: u32) -> KeyModifiers {
    let mut modifiers = KeyModifiers::empty();
    if control_key_state & SHIFT_PRESSED != 0 {
        modifiers |= KeyModifiers::SHIFT;
    }
    if control_key_state & (LEFT_CTRL_PRESSED | RIGHT_CTRL_PRESSED) != 0 {
        modifiers |= KeyModifiers::CONTROL;
    }
    if control_key_state & (LEFT_ALT_PRESSED | RIGHT_ALT_PRESSED) != 0 {
        modifiers |= KeyModifiers::ALT;
    }
    modifiers
}
