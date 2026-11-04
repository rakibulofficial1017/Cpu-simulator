#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level{
    Low,
    High,
}

#[derive(Debug)]
pub struct Clock{
    pub level: Level,
    pub cycles: u64,
}

impl Clock {
    pub fn new() -> Self {
        Self{
            level: Level::Low,
            cycles: 0,
        }
    }
    // Returns true if rising
    pub fn tick(&mut self) -> bool {
        let rising = self.level == Level::Low;

        self.level = match self.level {
            Level::High => Level::Low,
            Level::Low => Level::High,
        };

        if rising {
            self.cycles += 1;
        }

        rising
    }
}

