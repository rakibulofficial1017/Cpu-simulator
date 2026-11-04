use crate::hardware::logic::mux;

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub struct DFF {
    pub q: bool,
}

impl DFF {
    pub fn new() -> Self {
        Self { q: false }
    }

    pub fn update(&mut self, d: bool, rising_edge: bool) {
        if rising_edge {
            self.q = d;
        }
    }
}


#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub struct Bit {
    dff: DFF,
}

impl Bit {
    pub fn new() -> Self {
        Self {
            dff: DFF::new(),
        }
    }

    pub fn output(&self) -> bool {
        self.dff.q
    }

    pub fn update(&mut self, input: bool, load: bool, rising_edge: bool) {
        let d = mux(self.dff.q, input, load);
        self.dff.update(d, rising_edge);
    }
}

