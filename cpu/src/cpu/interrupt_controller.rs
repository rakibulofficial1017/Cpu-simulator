#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Interrupt {
    Keyboard = 0,
    Timer = 1,
    Display = 2,
    IO = 3,
}

#[derive(Debug, Default)]
pub struct InterruptController {
    pub enabled: bool,
    pub pending: u16,
}

impl InterruptController {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn request(&mut self, interrupt: Interrupt) {
        self.pending |= 1 << (interrupt as u8);
    }

    pub fn clear(&mut self, interrupt: Interrupt) {
        self.pending &= !(1 << (interrupt as u8));
    }

    pub fn has_pending(&self) -> bool {
        self.pending != 0
    }

    pub fn next_pending(&self) -> Option<Interrupt> {
        if !self.enabled {
            return None;
        }

        [
            Interrupt::Keyboard,
            Interrupt::Timer,
            Interrupt::Display,
            Interrupt::IO,
        ]
        .iter()
        .copied()
        .find(|interrupt| self.pending & (1 << (*interrupt as u8)) != 0)
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }
    pub fn acknowledge(&mut self, interrupt: Interrupt) {
        self.clear(interrupt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_interrupts_are_masked_until_enabled() {
        let mut controller = InterruptController::new();
        controller.request(Interrupt::Keyboard);

        assert!(controller.has_pending());
        assert_eq!(controller.next_pending(), None);

        controller.enable();
        assert_eq!(controller.next_pending(), Some(Interrupt::Keyboard));
    }

    #[test]
    fn pending_interrupts_use_priority_order_and_can_be_acknowledged() {
        let mut controller = InterruptController::new();
        controller.request(Interrupt::IO);
        controller.request(Interrupt::Timer);
        controller.request(Interrupt::Keyboard);
        controller.enable();

        assert_eq!(controller.next_pending(), Some(Interrupt::Keyboard));
        controller.acknowledge(Interrupt::Keyboard);
        assert_eq!(controller.next_pending(), Some(Interrupt::Timer));
        controller.acknowledge(Interrupt::Timer);
        assert_eq!(controller.next_pending(), Some(Interrupt::IO));
        controller.acknowledge(Interrupt::IO);
        assert!(!controller.has_pending());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterruptState {
    Idle,

    // Interrupt entry
    SavePCHigh,
    SavePCLow,
    SaveSR,
    ReadVectorHigh,
    ReadVectorLow,
    SetHandlerPC,

    // Interrupt return
    RestoreSR,
    RestorePCLow,
    RestorePCHigh,
    FinishReturn,
}
