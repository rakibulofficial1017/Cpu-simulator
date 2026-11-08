
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register8 {
    ACC,
    DR,
    SR,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register16 {
    PC,
    AR,
    TempA,
    TempB,
    PSR,
    SP,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicroOp {
    // 8-bit -> 8-bit
    Move8 {
        source: Register8,
        destination: Register8,
    },

    // 16-bit ->16-bit
    Move16 {
        source: Register16,
        destination: Register16,
    },

    // 8-bit -> 16-bit (zero extension)
    Extend8To16 {
        source: Register8,
        destination: Register16,
    },

    // 16-bit  -> 8-bit (truncate to low byte)
    Truncate16To8 {
        source: Register16,
        destination: Register8,
    },

    Increment16 {
        register: Register16,
    }
}
