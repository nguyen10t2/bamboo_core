/// Configuration options for the Bamboo engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    /// If true, allows typing tone marks at any position in the word (Free Tone Marking).
    /// Default: true.
    pub free_tone_marking: bool,
    /// If true, uses the standard (new) tone placement (e.g., "hòa", "khỏe").
    /// If false, uses the old style (e.g., "hoà", "khoẻ").
    /// Default: true.
    pub std_tone_style: bool,
    /// If true, enables automatic spelling correction to ensure valid Vietnamese syllables.
    /// Default: true.
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
    /// Creates a new configuration with default values.
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

    /// Creates a configuration from a bitmask of flags.
    pub fn from_flags(flags: u32) -> Self {
        Self {
            free_tone_marking: (flags & (1 << 0)) != 0,
            std_tone_style: (flags & (1 << 1)) != 0,
            auto_correct: (flags & (1 << 2)) != 0,
        }
    }
}
