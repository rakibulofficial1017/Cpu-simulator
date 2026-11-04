use crate::hardware::{logic::mux, register::Register16, alu::inc16};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PC {
    register: Register16,
}

impl PC {
    pub fn new() -> Self {
        Self {
            register: Register16::new(),
        }
    }

    pub fn output(&self) -> u16 {
        self.register.output()
    }

    pub fn update(
        &mut self,
        input: u16,
        load: bool,
        inc: bool,
        reset: bool,
        rising: bool,
    ) -> u16 {
        let reg_out = self.register.output();

        // 1. Increment the current program counter.
        let inc_out = inc16(reg_out);

        // 2. If inc, select the incremented value; otherwise hold.
        let out_inc_selection = mux(reg_out, inc_out, inc);

        // 3. If load, select the input address.
        let out_load_selection =
            mux(out_inc_selection, input, load);

        // 4. If reset, select zero.
        let to_reg_in = mux(out_load_selection, 0u16, reset);

        // 5. Update the register if any control signal is active.
        let reg_load_signal = inc || load || reset;

        // 6. Store the selected value on the rising edge.
        self.register
            .update(to_reg_in, reg_load_signal, rising);

        self.output()
    }
}
