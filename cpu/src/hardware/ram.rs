use crate::hardware::register::Register;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RAM8 {
    registers: Vec<Register>,
}

impl RAM8 {
    pub fn new() -> Self {
        Self {
            registers: vec![Register::new(); 8],
        }
    }

    pub fn output(&self, address: u16) -> u8 {
        self.registers[(address & 0b111) as usize].output()
    }

    pub fn update(
        &mut self,
        input: u8,
        load: bool,
        address: u16,
        rising: bool,
    ) -> u8 {
        let selected = (address & 0b111) as usize;

        for i in 0..8 {
            self.registers[i].update(
                input,
                load && i == selected,
                rising,
            );
        }

        self.output(address)
    }
}
macro_rules! define_ram {
    ($name:ident, $child:ty, $shift:expr, $count:expr) => {
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $name {
            banks: Vec<$child>,
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    banks: (0..$count)
                        .map(|_| <$child>::new())
                        .collect(),
                }
            }

            pub fn output(&self, address: u16) -> u8 {
                let bank =
                    ((address >> $shift) as usize) & ($count - 1);

                let local_address =
                    address & ((1 << $shift) - 1);

                self.banks[bank].output(local_address)
            }

            pub fn update(
                &mut self,
                input: u8,
                load: bool,
                address: u16,
                rising: bool,
            ) -> u8 {
                let bank =
                    ((address >> $shift) as usize) & ($count - 1);

                let local_address =
                    address & ((1 << $shift) - 1);

                self.banks[bank].update(
                    input,
                    load,
                    local_address,
                    rising,
                );

                self.output(address)
            }
        }
    };
}

define_ram!(RAM64,   RAM8,   3,  8);
define_ram!(RAM512,  RAM64,  6,  8);
define_ram!(RAM4K,   RAM512, 9,  8);
define_ram!(RAM16K,  RAM4K,  12, 8);
define_ram!(RAM64K,  RAM16K, 14, 4);
