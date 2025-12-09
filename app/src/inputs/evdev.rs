use crate::button::InputEvent;
use crossterm::event::{
    KeyCode as CrosstermKeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers,
};
use evdev::{Device, EventSummary, KeyCode};
use std::io;
use std::sync::mpsc::Sender;
use std::thread;

/// Spawns a thread that listens for events from the physical keyboard device
/// and sends them to the main application loop via the provided Sender.
pub fn spawn_listener(tx: Sender<InputEvent>) -> anyhow::Result<()> {
    thread::spawn(move || {
        let mut adapter = match EvdevAdapter::new() {
            Ok(a) => a,
            Err(e) => {
                tracing::warn!("Failed to initialize evdev keyboard: {}", e);
                return;
            }
        };

        loop {
            match adapter.next_key_event() {
                Ok(Some(key_event)) => {
                    if tx.send(InputEvent::Key(key_event)).is_err() {
                        break;
                    }
                }
                Ok(None) => {}
                Err(_) => break, // Device lost or error
            }
        }
    });
    Ok(())
}

/// Helper to manage input state (modifiers) and device access
struct EvdevAdapter {
    device: Device,
    modifiers: KeyModifiers,
}

impl EvdevAdapter {
    /// Attempts to open the first available keyboard device.
    pub fn new() -> io::Result<Self> {
        let devices = evdev::enumerate().map(|t| t.1).collect::<Vec<_>>();

        // Find a device that supports keys
        let device = devices
            .into_iter()
            .find(|d| {
                d.supported_keys()
                    .map_or(false, |keys| keys.contains(KeyCode::KEY_ENTER))
            })
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No keyboard device found"))?;

        Ok(Self {
            device,
            modifiers: KeyModifiers::empty(),
        })
    }

    /// Blocks until the next key event occurs, then returns it as a Crossterm KeyEvent
    pub fn next_key_event(&mut self) -> io::Result<Option<KeyEvent>> {
        loop {
            // fetch_events blocks, but returns an iterator of multiple events
            for ev in self.device.fetch_events()? {
                match ev.destructure() {
                    EventSummary::Key(_, key, value) => {
                        // value: 0 = Release, 1 = Press, 2 = Repeat
                        match key {
                            KeyCode::KEY_LEFTSHIFT | KeyCode::KEY_RIGHTSHIFT => {
                                if value == 1 {
                                    self.modifiers.insert(KeyModifiers::SHIFT);
                                } else if value == 0 {
                                    self.modifiers.remove(KeyModifiers::SHIFT);
                                }
                            }
                            KeyCode::KEY_LEFTCTRL | KeyCode::KEY_RIGHTCTRL => {
                                if value == 1 {
                                    self.modifiers.insert(KeyModifiers::CONTROL);
                                } else if value == 0 {
                                    self.modifiers.remove(KeyModifiers::CONTROL);
                                }
                            }
                            KeyCode::KEY_LEFTALT | KeyCode::KEY_RIGHTALT => {
                                if value == 1 {
                                    self.modifiers.insert(KeyModifiers::ALT);
                                } else if value == 0 {
                                    self.modifiers.remove(KeyModifiers::ALT);
                                }
                            }
                            _ => {}
                        }

                        // Only emit events on Press (1) or Repeat (2)
                        if value == 0 {
                            continue;
                        }

                        if let Some(crossterm_code) =
                            evdev_key_to_crossterm_keycode(key, self.modifiers)
                        {
                            return Ok(Some(KeyEvent {
                                code: crossterm_code,
                                modifiers: self.modifiers,
                                kind: if value == 2 {
                                    KeyEventKind::Repeat
                                } else {
                                    KeyEventKind::Press
                                },
                                state: KeyEventState::empty(),
                            }));
                        }
                    }
                    _ => {} // Ignore non-key events
                }
            }
        }
    }
}

/// Translates a Linux evdev KeyCode to a Crossterm KeyCode.
fn evdev_key_to_crossterm_keycode(
    key: KeyCode,
    modifiers: KeyModifiers,
) -> Option<CrosstermKeyCode> {
    let is_shifted = modifiers.contains(KeyModifiers::SHIFT);

    match key {
        // --- Control Keys ---
        KeyCode::KEY_ESC => Some(CrosstermKeyCode::Esc),
        KeyCode::KEY_ENTER => Some(CrosstermKeyCode::Enter),
        KeyCode::KEY_BACKSPACE => Some(CrosstermKeyCode::Backspace),
        KeyCode::KEY_TAB => Some(CrosstermKeyCode::Tab),
        KeyCode::KEY_DELETE => Some(CrosstermKeyCode::Delete),
        KeyCode::KEY_HOME => Some(CrosstermKeyCode::Home),
        KeyCode::KEY_END => Some(CrosstermKeyCode::End),
        KeyCode::KEY_PAGEUP => Some(CrosstermKeyCode::PageUp),
        KeyCode::KEY_PAGEDOWN => Some(CrosstermKeyCode::PageDown),
        KeyCode::KEY_UP => Some(CrosstermKeyCode::Up),
        KeyCode::KEY_DOWN => Some(CrosstermKeyCode::Down),
        KeyCode::KEY_LEFT => Some(CrosstermKeyCode::Left),
        KeyCode::KEY_RIGHT => Some(CrosstermKeyCode::Right),
        KeyCode::KEY_F1 => Some(CrosstermKeyCode::F(1)),
        KeyCode::KEY_F2 => Some(CrosstermKeyCode::F(2)),
        KeyCode::KEY_F3 => Some(CrosstermKeyCode::F(3)),
        KeyCode::KEY_F4 => Some(CrosstermKeyCode::F(4)),
        KeyCode::KEY_F5 => Some(CrosstermKeyCode::F(5)),
        KeyCode::KEY_F6 => Some(CrosstermKeyCode::F(6)),
        KeyCode::KEY_F7 => Some(CrosstermKeyCode::F(7)),
        KeyCode::KEY_F8 => Some(CrosstermKeyCode::F(8)),
        KeyCode::KEY_F9 => Some(CrosstermKeyCode::F(9)),
        KeyCode::KEY_F10 => Some(CrosstermKeyCode::F(10)),
        KeyCode::KEY_F11 => Some(CrosstermKeyCode::F(11)),
        KeyCode::KEY_F12 => Some(CrosstermKeyCode::F(12)),

        // --- Alphanumeric ---
        KeyCode::KEY_A => Some(CrosstermKeyCode::Char(if is_shifted { 'A' } else { 'a' })),
        KeyCode::KEY_B => Some(CrosstermKeyCode::Char(if is_shifted { 'B' } else { 'b' })),
        KeyCode::KEY_C => Some(CrosstermKeyCode::Char(if is_shifted { 'C' } else { 'c' })),
        KeyCode::KEY_D => Some(CrosstermKeyCode::Char(if is_shifted { 'D' } else { 'd' })),
        KeyCode::KEY_E => Some(CrosstermKeyCode::Char(if is_shifted { 'E' } else { 'e' })),
        KeyCode::KEY_F => Some(CrosstermKeyCode::Char(if is_shifted { 'F' } else { 'f' })),
        KeyCode::KEY_G => Some(CrosstermKeyCode::Char(if is_shifted { 'G' } else { 'g' })),
        KeyCode::KEY_H => Some(CrosstermKeyCode::Char(if is_shifted { 'H' } else { 'h' })),
        KeyCode::KEY_I => Some(CrosstermKeyCode::Char(if is_shifted { 'I' } else { 'i' })),
        KeyCode::KEY_J => Some(CrosstermKeyCode::Char(if is_shifted { 'J' } else { 'j' })),
        KeyCode::KEY_K => Some(CrosstermKeyCode::Char(if is_shifted { 'K' } else { 'k' })),
        KeyCode::KEY_L => Some(CrosstermKeyCode::Char(if is_shifted { 'L' } else { 'l' })),
        KeyCode::KEY_M => Some(CrosstermKeyCode::Char(if is_shifted { 'M' } else { 'm' })),
        KeyCode::KEY_N => Some(CrosstermKeyCode::Char(if is_shifted { 'N' } else { 'n' })),
        KeyCode::KEY_O => Some(CrosstermKeyCode::Char(if is_shifted { 'O' } else { 'o' })),
        KeyCode::KEY_P => Some(CrosstermKeyCode::Char(if is_shifted { 'P' } else { 'p' })),
        KeyCode::KEY_Q => Some(CrosstermKeyCode::Char(if is_shifted { 'Q' } else { 'q' })),
        KeyCode::KEY_R => Some(CrosstermKeyCode::Char(if is_shifted { 'R' } else { 'r' })),
        KeyCode::KEY_S => Some(CrosstermKeyCode::Char(if is_shifted { 'S' } else { 's' })),
        KeyCode::KEY_T => Some(CrosstermKeyCode::Char(if is_shifted { 'T' } else { 't' })),
        KeyCode::KEY_U => Some(CrosstermKeyCode::Char(if is_shifted { 'U' } else { 'u' })),
        KeyCode::KEY_V => Some(CrosstermKeyCode::Char(if is_shifted { 'V' } else { 'v' })),
        KeyCode::KEY_W => Some(CrosstermKeyCode::Char(if is_shifted { 'W' } else { 'w' })),
        KeyCode::KEY_X => Some(CrosstermKeyCode::Char(if is_shifted { 'X' } else { 'x' })),
        KeyCode::KEY_Y => Some(CrosstermKeyCode::Char(if is_shifted { 'Y' } else { 'y' })),
        KeyCode::KEY_Z => Some(CrosstermKeyCode::Char(if is_shifted { 'Z' } else { 'z' })),
        KeyCode::KEY_SPACE => Some(CrosstermKeyCode::Char(' ')),

        // --- Numbers ---
        KeyCode::KEY_1 => Some(CrosstermKeyCode::Char(if is_shifted { '!' } else { '1' })),
        KeyCode::KEY_2 => Some(CrosstermKeyCode::Char(if is_shifted { '@' } else { '2' })),
        KeyCode::KEY_3 => Some(CrosstermKeyCode::Char(if is_shifted { '#' } else { '3' })),
        KeyCode::KEY_4 => Some(CrosstermKeyCode::Char(if is_shifted { '$' } else { '4' })),
        KeyCode::KEY_5 => Some(CrosstermKeyCode::Char(if is_shifted { '%' } else { '5' })),
        KeyCode::KEY_6 => Some(CrosstermKeyCode::Char(if is_shifted { '^' } else { '6' })),
        KeyCode::KEY_7 => Some(CrosstermKeyCode::Char(if is_shifted { '&' } else { '7' })),
        KeyCode::KEY_8 => Some(CrosstermKeyCode::Char(if is_shifted { '*' } else { '8' })),
        KeyCode::KEY_9 => Some(CrosstermKeyCode::Char(if is_shifted { '(' } else { '9' })),
        KeyCode::KEY_0 => Some(CrosstermKeyCode::Char(if is_shifted { ')' } else { '0' })),

        _ => None,
    }
}
