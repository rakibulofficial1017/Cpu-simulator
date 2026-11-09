use crate::hardware::{clock::Clock, ram::RAM64K};

use super::cpu::{MemoryResponse, CPU};
use std::io::{self, Write};

pub const SYSTEM_RESERVED_START: u16 = 0;
pub const SYSTEM_RESERVED_BYTES: u16 = 100;
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
            output.write_all(b"\n")?;
        }

        output.write_all(b"\x1b[0m")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(output.ends_with("\x1b[0m"));
    }
}
