use crate::hardware::{clock::Clock, ram::RAM64K};

use super::cpu::{MemoryResponse, CPU};
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind};
use std::io::{self, Write};

pub const SYSTEM_RESERVED_START: u16 = 0;
pub const SYSTEM_RESERVED_BYTES: u16 = 100;
pub const KEY_EVENT_TYPE: u16 = SYSTEM_RESERVED_START;
pub const KEY_CODE: u16 = SYSTEM_RESERVED_START + 1;
pub const KEY_MODIFIERS: u16 = SYSTEM_RESERVED_START + 2;
pub const MOUSE_EVENT_TYPE: u16 = SYSTEM_RESERVED_START + 3;
pub const MOUSE_BUTTONS: u16 = SYSTEM_RESERVED_START + 4;
pub const MOUSE_X_LOW: u16 = SYSTEM_RESERVED_START + 5;
pub const MOUSE_X_HIGH: u16 = SYSTEM_RESERVED_START + 6;
pub const MOUSE_Y_LOW: u16 = SYSTEM_RESERVED_START + 7;
pub const MOUSE_Y_HIGH: u16 = SYSTEM_RESERVED_START + 8;
pub const BOARD_X: u16 = SYSTEM_RESERVED_START + 9;
pub const BOARD_Y: u16 = SYSTEM_RESERVED_START + 10;

pub const KEY_EVENT_PRESSED: u8 = 1;
pub const KEY_EVENT_RELEASED: u8 = 2;
pub const KEY_EVENT_REPEAT: u8 = 3;
pub const MOUSE_EVENT_PRESSED: u8 = 1;
pub const MOUSE_EVENT_RELEASED: u8 = 2;
pub const MOUSE_EVENT_MOVED: u8 = 3;
pub const MOUSE_EVENT_SCROLL_UP: u8 = 4;
pub const MOUSE_EVENT_SCROLL_DOWN: u8 = 5;

pub const SCREEN_START: u16 = SYSTEM_RESERVED_START + SYSTEM_RESERVED_BYTES;
pub const SCREEN_WIDTH: usize = 150;
pub const SCREEN_HEIGHT: usize = 40;
pub const SCREEN_BYTES_PER_PIXEL: usize = 3;
pub const SCREEN_BYTES: usize = SCREEN_WIDTH * SCREEN_HEIGHT * SCREEN_BYTES_PER_PIXEL;
pub const SCREEN_END: u16 = SCREEN_START + SCREEN_BYTES as u16;
pub const GENERAL_PURPOSE_START: u16 = SCREEN_END;
pub const GENERAL_PURPOSE_END: u32 = u16::MAX as u32 + 1;

pub struct Computer {
    pub cpu: CPU,
    pub ram: RAM64K,
    pub clock: Clock,
}

impl Computer {
    pub fn new() -> Self {
        Self {
            cpu: CPU::new(),
            ram: RAM64K::new(),
            clock: Clock::new(),
        }
    }

    pub fn tick(&mut self) -> bool {
        // Advance the clock exactly once.
        let rising_edge = self.clock.tick();

        // Only process memory operations on rising edges.
        if !rising_edge {
            return false;
        }

        let request = self.cpu.memory_request;

        // A request should not read and write simultaneously.
        debug_assert!(
            !(request.read && request.write),
            "CPU cannot read and write simultaneously"
        );

        if request.write {
            self.ram
                .update(request.data, true, request.address, rising_edge);

            self.cpu.memory_ready = true;
        }

        if request.read {
            let data = self.ram.output(request.address);

            self.cpu
                .receive_memory_response(MemoryResponse { data }, rising_edge);

            self.cpu.memory_ready = true;
        }

        self.cpu.clear_memory_request();

        true
    }

    /// Store the latest terminal keyboard or mouse event in system-reserved RAM.
    pub fn handle_input_event(&mut self, event: Event) {
        match event {
            Event::Key(key) => {
                let event_type = match key.kind {
                    KeyEventKind::Press => KEY_EVENT_PRESSED,
                    KeyEventKind::Release => KEY_EVENT_RELEASED,
                    KeyEventKind::Repeat => KEY_EVENT_REPEAT,
                };
                self.write_system_byte(KEY_EVENT_TYPE, event_type);
                self.write_system_byte(KEY_CODE, encode_key_code(key.code));
                self.write_system_byte(KEY_MODIFIERS, encode_modifiers(key.modifiers));
            }
            Event::Mouse(mouse) => {
                let (event_type, button) = match mouse.kind {
                    MouseEventKind::Down(button) => (MOUSE_EVENT_PRESSED, Some((button, true))),
                    MouseEventKind::Up(button) => (MOUSE_EVENT_RELEASED, Some((button, false))),
                    MouseEventKind::Drag(button) => (MOUSE_EVENT_MOVED, Some((button, true))),
                    MouseEventKind::Moved => (MOUSE_EVENT_MOVED, None),
                    MouseEventKind::ScrollUp => (MOUSE_EVENT_SCROLL_UP, None),
                    MouseEventKind::ScrollDown => (MOUSE_EVENT_SCROLL_DOWN, None),
                    MouseEventKind::ScrollLeft | MouseEventKind::ScrollRight => return,
                };

                let mut buttons = self.ram.output(MOUSE_BUTTONS);
                if let Some((button, pressed)) = button {
                    let mask = mouse_button_mask(button);
                    if pressed {
                        buttons |= mask;
                    } else {
                        buttons &= !mask;
                    }
                }

                self.write_system_byte(MOUSE_EVENT_TYPE, event_type);
                self.write_system_byte(MOUSE_BUTTONS, buttons);
                self.write_system_byte(MOUSE_X_LOW, mouse.column as u8);
                self.write_system_byte(MOUSE_X_HIGH, (mouse.column >> 8) as u8);
                self.write_system_byte(MOUSE_Y_LOW, mouse.row as u8);
                self.write_system_byte(MOUSE_Y_HIGH, (mouse.row >> 8) as u8);
            }
            _ => {}
        }
    }

    /// Set one RGB pixel in the screen framebuffer.
    pub fn set_pixel(&mut self, x: usize, y: usize, color: [u8; 3]) -> io::Result<()> {
        if x >= SCREEN_WIDTH || y >= SCREEN_HEIGHT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("pixel ({x}, {y}) is outside the {SCREEN_WIDTH}x{SCREEN_HEIGHT} screen"),
            ));
        }

        let address = SCREEN_START as usize + (y * SCREEN_WIDTH + x) * SCREEN_BYTES_PER_PIXEL;
        for (channel, value) in color.into_iter().enumerate() {
            self.ram
                .update(value, true, (address + channel) as u16, true);
        }
        Ok(())
    }

    fn write_system_byte(&mut self, address: u16, value: u8) {
        self.ram.update(value, true, address, true);
    }

    /// Render the RGB framebuffer in RAM to the terminal using ANSI colors.
    pub fn update_screen(&self) -> io::Result<()> {
        let stdout = io::stdout();
        let mut output = stdout.lock();
        self.write_screen(&mut output)?;
        output.flush()
    }

    fn write_screen<W: Write>(&self, output: &mut W) -> io::Result<()> {
        output.write_all(b"\x1b[H")?;

        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                let pixel = y * SCREEN_WIDTH + x;
                let address = SCREEN_START as usize + pixel * SCREEN_BYTES_PER_PIXEL;
                let red = self.ram.output(address as u16);
                let green = self.ram.output((address + 1) as u16);
                let blue = self.ram.output((address + 2) as u16);

                write!(output, "\x1b[48;2;{red};{green};{blue}m ")?;
            }
            output.write_all(b"\r\n")?;
        }

        output.write_all(b"\x1b[0m")
    }
}

fn encode_key_code(code: KeyCode) -> u8 {
    match code {
        KeyCode::Char(character) if character.is_ascii() => character as u8,
        KeyCode::Backspace => 0x80,
        KeyCode::Enter => 0x81,
        KeyCode::Left => 0x82,
        KeyCode::Right => 0x83,
        KeyCode::Up => 0x84,
        KeyCode::Down => 0x85,
        KeyCode::Home => 0x86,
        KeyCode::End => 0x87,
        KeyCode::PageUp => 0x88,
        KeyCode::PageDown => 0x89,
        KeyCode::Tab => 0x8A,
        KeyCode::BackTab => 0x8B,
        KeyCode::Delete => 0x8C,
        KeyCode::Insert => 0x8D,
        KeyCode::Esc => 0x8E,
        KeyCode::Null => 0x8F,
        KeyCode::F(number @ 1..=24) => 0x8F + number,
        _ => 0,
    }
}

fn encode_modifiers(modifiers: KeyModifiers) -> u8 {
    u8::from(modifiers.contains(KeyModifiers::SHIFT))
        | (u8::from(modifiers.contains(KeyModifiers::CONTROL)) << 1)
        | (u8::from(modifiers.contains(KeyModifiers::ALT)) << 2)
        | (u8::from(modifiers.contains(KeyModifiers::SUPER)) << 3)
        | (u8::from(modifiers.contains(KeyModifiers::HYPER)) << 4)
        | (u8::from(modifiers.contains(KeyModifiers::META)) << 5)
}

fn mouse_button_mask(button: MouseButton) -> u8 {
    match button {
        MouseButton::Left => 0b001,
        MouseButton::Right => 0b010,
        MouseButton::Middle => 0b100,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEvent, MouseEvent};

    fn set_byte(computer: &mut Computer, address: u16, value: u8) {
        computer.ram.update(value, true, address, true);
    }

    #[test]
    fn screen_layout_uses_three_bytes_per_pixel() {
        assert_eq!(SCREEN_START, 100);
        assert_eq!(SCREEN_BYTES, 18_000);
        assert_eq!(SCREEN_END, 18_100);
        assert_eq!(GENERAL_PURPOSE_START, SCREEN_END);
        assert_eq!(GENERAL_PURPOSE_END, 65_536);
    }

    #[test]
    fn keyboard_events_are_stored_in_reserved_ram() {
        let mut computer = Computer::new();
        computer.handle_input_event(Event::Key(KeyEvent::new(
            KeyCode::Char('A'),
            KeyModifiers::SHIFT | KeyModifiers::CONTROL,
        )));

        assert_eq!(computer.ram.output(KEY_EVENT_TYPE), KEY_EVENT_PRESSED);
        assert_eq!(computer.ram.output(KEY_CODE), b'A');
        assert_eq!(computer.ram.output(KEY_MODIFIERS), 0b0000_0011);
    }

    #[test]
    fn mouse_events_store_position_and_pressed_button_state() {
        let mut computer = Computer::new();
        computer.handle_input_event(Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 0x123,
            row: 0x045,
            modifiers: KeyModifiers::NONE,
        }));

        assert_eq!(computer.ram.output(MOUSE_EVENT_TYPE), MOUSE_EVENT_PRESSED);
        assert_eq!(computer.ram.output(MOUSE_BUTTONS), 0b001);
        assert_eq!(computer.ram.output(MOUSE_X_LOW), 0x23);
        assert_eq!(computer.ram.output(MOUSE_X_HIGH), 0x01);
        assert_eq!(computer.ram.output(MOUSE_Y_LOW), 0x45);
        assert_eq!(computer.ram.output(MOUSE_Y_HIGH), 0x00);

        computer.handle_input_event(Event::Mouse(MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: 0x123,
            row: 0x045,
            modifiers: KeyModifiers::NONE,
        }));
        assert_eq!(computer.ram.output(MOUSE_BUTTONS), 0);
    }

    #[test]
    fn screen_update_renders_rgb_pixels_in_row_major_order() {
        let mut computer = Computer::new();
        set_byte(&mut computer, SCREEN_START, 1);
        set_byte(&mut computer, SCREEN_START + 1, 2);
        set_byte(&mut computer, SCREEN_START + 2, 3);
        set_byte(&mut computer, SCREEN_START + 3, 4);
        set_byte(&mut computer, SCREEN_START + 4, 5);
        set_byte(&mut computer, SCREEN_START + 5, 6);

        let mut output = Vec::new();
        computer.write_screen(&mut output).unwrap();
        let output = String::from_utf8(output).unwrap();

        assert!(output.starts_with("\x1b[H\x1b[48;2;1;2;3m \x1b[48;2;4;5;6m \x1b[48;2;0;0;0m "));
        assert_eq!(
            output.matches("\x1b[48;2;").count(),
            SCREEN_WIDTH * SCREEN_HEIGHT
        );
        assert_eq!(output.matches("\r\n").count(), SCREEN_HEIGHT);
        assert!(output.ends_with("\x1b[0m"));
    }

    #[test]
    fn set_pixel_writes_rgb_channels_to_the_framebuffer() {
        let mut computer = Computer::new();
        computer.set_pixel(2, 1, [10, 20, 30]).unwrap();
        let address = SCREEN_START + (SCREEN_WIDTH as u16) * 3 + 2 * 3;

        assert_eq!(computer.ram.output(address), 10);
        assert_eq!(computer.ram.output(address + 1), 20);
        assert_eq!(computer.ram.output(address + 2), 30);
        assert!(computer.set_pixel(SCREEN_WIDTH, 0, [1, 2, 3]).is_err());
    }
}
