use bitflags::bitflags;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Vietnamese,
    English,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct OutputOptions: u32 {
        const NONE = 0;
        const TONE_LESS = 1 << 0;
        const MARK_LESS = 1 << 1;
        const LOWER_CASE = 1 << 2;
        const FULL_TEXT = 1 << 3;
        const PUNCTUATION_MODE = 1 << 4;
        const IN_REVERSE_ORDER = 1 << 5;
        const RAW = 1 << 6;
    }
}
