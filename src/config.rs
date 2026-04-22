#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    pub free_tone_marking: bool,
    pub std_tone_style: bool,
    pub auto_correct: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            free_tone_marking: true,
            std_tone_style: true,
            auto_correct: true,
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn to_flags(self) -> u32 {
        let mut flags = 0;
        if self.free_tone_marking {
            flags |= 1 << 0;
        }
        if self.std_tone_style {
            flags |= 1 << 1;
        }
        if self.auto_correct {
            flags |= 1 << 2;
        }
        flags
    }

    pub fn from_flags(flags: u32) -> Self {
        Self {
            free_tone_marking: (flags & (1 << 0)) != 0,
            std_tone_style: (flags & (1 << 1)) != 0,
            auto_correct: (flags & (1 << 2)) != 0,
        }
    }
}
