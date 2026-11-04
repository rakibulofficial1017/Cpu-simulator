
use crate::hardware::{
    pc::PC,
    register::{Register, Register16},
};
#[derive(Debug, Clone, Copy, Default)]
pub struct MemoryRequest {
    pub address: u16,
    pub data: u8,
    pub read: bool,
    pub write: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MemoryResponse {
    pub data: u8,
}
#[derive(Debug)]
pub struct CPU {
    pub pc: PC,
    pub acc: Register,
    pub temp_a: Register16,
    pub temp_b: Register16,
    pub psr: Register16,
    pub sr: Register,
    pub ar: Register16,
    pub dr: Register,
    pub sp: Register16,
    pub memory_request: MemoryRequest,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            pc: PC::new(),
            acc: Register::new(),

            temp_a: Register16::new(),
            temp_b: Register16::new(),

            psr: Register16::new(),

            sr: Register::new(),

            ar: Register16::new(),
            dr: Register::new(),

            sp: Register16::new(),

            memory_request: MemoryRequest::default(),
        }
    }
    pub fn request_read(&mut self, address: u16) {
        self.memory_request = MemoryRequest {
            address,
            data: 0,
            read: true,
            write: false,
        };
    }

    pub fn request_write(&mut self, address: u16, data: u8) {
        self.memory_request = MemoryRequest {
            address,
            data,
            read: false,
            write: true,
        };
    }

    pub fn clear_memory_request(&mut self) {
        self.memory_request.read = false;
        self.memory_request.write = false;
    }
    pub fn receive_memory_response(
        &mut self,
        response: MemoryResponse,
        rising_edge: bool,
    ) {
        self.dr.update(response.data, true, rising_edge);
    }
    pub fn debug_registers(&self) {
        dbg!(
            self.pc.output(),
            self.acc.output(),
            self.temp_a.output(),
            self.temp_b.output(),
            self.psr.output(),
            self.sr.output(),
            self.ar.output(),
            self.dr.output(),
            self.sp.output(),
        );
    }
}
