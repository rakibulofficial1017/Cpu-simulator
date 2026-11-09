
use crate::hardware::{
    clock::Clock,
    ram::RAM64K,
};

use super::cpu::{CPU, MemoryResponse};

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
    let request = self.cpu.memory_request;

    debug_assert!(
        !(request.read && request.write),
        "CPU cannot read and write simultaneously"
    );

    if request.write {
        self.ram.update(
            request.data,
            true,
            request.address,
            rising_edge,
        );

        self.cpu.memory_ready = true;
    }

    if request.read {
        let data = self.ram.output(request.address);

        self.cpu.receive_memory_response(
            MemoryResponse { data },
            rising_edge,
        );

        self.cpu.memory_ready = true;
    }

    self.cpu.clear_memory_request();


        true
    }
}
