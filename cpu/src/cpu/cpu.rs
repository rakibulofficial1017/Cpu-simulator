use super::micro_op::{MicroOp, Register8 as R8, Register16 as R16};

use crate::{cpu::micro_op::{ALUDestination, ALUInput}, hardware::{
    alu::alu, pc::PC, register::{Register, Register16},
}};
#[derive(Debug, Clone, Copy, Default)]
pub struct MemoryRequest {
    pub address: u16,
    pub data: u8,
    pub read: bool,
    pub write: bool,
}
const FLAG_ZR: u8 = 1 << 7;
const FLAG_NG: u8 = 1 << 6;
const FLAG_CY: u8 = 1 << 5;
const FLAG_OV: u8 = 1 << 4;
const FLAG_EQ: u8 = 1 << 3;
const FLAG_GT: u8 = 1 << 2;
const FLAG_LT: u8 = 1 << 1;

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
    pub ir: Register,
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
            ir: Register::new(),
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
        println!(
"
Reg     Binary              Hex             Decimal 
PC    = {:016b}, : {:04x},         : {}  
ACC   = {:08b},         : {:02x},           : {}
TempA = {:016b}, : {:04x},         : {}
TempB = {:016b}, : {:04x},         : {}
PSR   = {:016b}, : {:04x},         : {}
SR    = {:08b},         : {:02x},           : {}
AR    = {:016b}, : {:04x},         : {}
DR    = {:08b},         : {:02x},           : {}
SP    = {:016b}, : {:04x},         : {}
IR    = {:08b},         : {:02x},           : {}
",
            self.pc.output(),
            self.pc.output(),
            self.pc.output(),
            self.acc.output(),
            self.acc.output(),
            self.acc.output(),
            self.temp_a.output(),
            self.temp_a.output(),
            self.temp_a.output(),
            self.temp_b.output(),
            self.temp_b.output(),
            self.temp_b.output(),
            self.psr.output(),
            self.psr.output(),
            self.psr.output(),
            self.sr.output(),
            self.sr.output(),
            self.sr.output(),
            self.ar.output(),
            self.ar.output(),
            self.ar.output(),
            self.dr.output(),
            self.dr.output(),
            self.dr.output(),
            self.sp.output(),
            self.sp.output(),
            self.sp.output(),
            self.ir.output(),
            self.ir.output(),
            self.ir.output(),
        );
    }

    pub fn execute_micro_op(
        &mut self,
        operation: MicroOp,
        rising_edge: bool,
    ) {
        match operation {
            // ================================================
            // 8-BIT TRANSFERS
            // ================================================

            MicroOp::Move8 { source, destination } => {
                let value = match source {
                    R8::ACC => self.acc.output(),
                    R8::DR  => self.dr.output(),
                    R8::SR  => self.sr.output(),
                };

                match destination {
                    R8::ACC => {
                        self.acc.update(value, true, rising_edge);
                    }
                    R8::DR => {
                        self.dr.update(value, true, rising_edge);
                    }
                    R8::SR => {
                        self.sr.update(value, true, rising_edge);
                    }
                }
            }

            // ================================================
            // 16-BIT TRANSFERS
            // ================================================

            MicroOp::Move16 { source, destination } => {
                let value = match source {
                    R16::PC    => self.pc.output(),
                    R16::AR    => self.ar.output(),
                    R16::TempA => self.temp_a.output(),
                    R16::TempB => self.temp_b.output(),
                    R16::PSR   => self.psr.output(),
                    R16::SP    => self.sp.output(),
                };

                match destination {
                    R16::PC => {
                        self.pc.update(
                            value, true, false, false, rising_edge
                        );
                    }
                    R16::AR => {
                        self.ar.update(value, true, rising_edge);
                    }
                    R16::TempA => {
                        self.temp_a.update(value, true, rising_edge);
                    }
                    R16::TempB => {
                        self.temp_b.update(value, true, rising_edge);
                    }
                    R16::PSR => {
                        self.psr.update(value, true, rising_edge);
                    }
                    R16::SP => {
                        self.sp.update(value, true, rising_edge);
                    }
                }
            }

            // ================================================
            // 8-BIT → 16-BIT
            // ================================================

            MicroOp::Extend8To16 { source, destination } => {
                let value = match source {
                    R8::ACC => self.acc.output(),
                    R8::DR  => self.dr.output(),
                    R8::SR  => self.sr.output(),
                };

                let value = value as u16;

                match destination {
                    R16::PC => {
                        self.pc.update(
                            value, true, false, false, rising_edge
                        );
                    }
                    R16::AR => {
                        self.ar.update(value, true, rising_edge);
                    }
                    R16::TempA => {
                        self.temp_a.update(value, true, rising_edge);
                    }
                    R16::TempB => {
                        self.temp_b.update(value, true, rising_edge);
                    }
                    R16::PSR => {
                        self.psr.update(value, true, rising_edge);
                    }
                    R16::SP => {
                        self.sp.update(value, true, rising_edge);
                    }
                }
            }

            // ================================================
            // 16-BIT → 8-BIT
            // ================================================

            MicroOp::Truncate16To8 { source, destination } => {
                let value = match source {
                    R16::PC    => self.pc.output(),
                    R16::AR    => self.ar.output(),
                    R16::TempA => self.temp_a.output(),
                    R16::TempB => self.temp_b.output(),
                    R16::PSR   => self.psr.output(),
                    R16::SP    => self.sp.output(),
                };
                
                let value = value as u8;
                
                match destination {
                    R8::ACC => {
                        self.acc.update(value, true, rising_edge);
                    }
                    R8::DR => {
                        self.dr.update(value, true, rising_edge);
                    }
                    R8::SR => {
                        self.sr.update(value, true, rising_edge);
                    }
                }
            }
            

        
            MicroOp::ALU {
    a,
    b,
    update_flags,
    compare,
    destination,
    zx,
    nx,
    zy,
    ny,
    f,
    no,
} => {
    let x = match a {
        ALUInput::ACC => self.acc.output() as u16,
        ALUInput::DR => self.dr.output() as u16,
        ALUInput::SR => self.sr.output() as u16,
        ALUInput::PC => self.pc.output(),
        ALUInput::AR => self.ar.output(),
        ALUInput::TempA => self.temp_a.output(),
        ALUInput::TempB => self.temp_b.output(),
        ALUInput::PSR => self.psr.output(),
        ALUInput::SP => self.sp.output(),
        ALUInput::Immediate(value) => value,
    };

    let y = match b {
        ALUInput::ACC => self.acc.output() as u16,
        ALUInput::DR => self.dr.output() as u16,
        ALUInput::SR => self.sr.output() as u16,
        ALUInput::PC => self.pc.output(),
        ALUInput::AR => self.ar.output(),
        ALUInput::TempA => self.temp_a.output(),
        ALUInput::TempB => self.temp_b.output(),
        ALUInput::PSR => self.psr.output(),
        ALUInput::SP => self.sp.output(),
        ALUInput::Immediate(value) => value,
    };

    let result = alu(x, y, zx, nx, zy, ny, f, no, );

    if update_flags {
        let mut flags = self.sr.output();

        // Update result flags.
        flags &= !(FLAG_ZR | FLAG_NG | FLAG_CY | FLAG_OV);

        if result.zr {
            flags |= FLAG_ZR;
        }

        if result.ng {
            flags |= FLAG_NG;
        }

        if result.cy {
            flags |= FLAG_CY;
        }

        if result.ov {
            flags |= FLAG_OV;
        }

        // Update comparison flags only for comparison operations.
        if compare {
            flags &= !(FLAG_EQ | FLAG_GT | FLAG_LT);

            if result.eq {
                flags |= FLAG_EQ;
            }

            if result.gt {
                flags |= FLAG_GT;
            }

            if result.lt {
                flags |= FLAG_LT;
            }
        }

        self.sr.update(flags, true, rising_edge);
    }

}
        }
    }
}
