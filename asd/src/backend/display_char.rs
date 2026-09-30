use crate::assets::colors::{
    C_BG_SPECIAL_BYTES, C_LOG_ERROR, apply_base_style, apply_deprecated_state,
    apply_diagnostic_style,
};
use crate::backend::display_string::DisplayString;
use crate::backend::little_string::LittleStringUni;
use crate::ui::log::{LOGS, Log};
use once_cell::sync::Lazy;
use ratatui::buffer::{Cell, CellDiffOption};
use ratatui::prelude::Modifier;
use ratatui::style::Style;
use std::cmp::Ordering;
use std::io::{Error, Write};
use std::sync::Mutex;

pub static LONG_LIST: Lazy<Mutex<Vec<LittleStringUni>>> = Lazy::new(|| Mutex::new(Vec::new()));
pub static EMOJI_LIST: Lazy<Mutex<Vec<LittleStringUni>>> = Lazy::new(|| Mutex::new(Vec::new()));

#[cfg(test)]
pub const PAD4: fn(&str) -> [u8; 4] = pad4;

/// This function is used to expand macro `strings_to_bytes`, to make easier generation of variable `LOOKUP_SPECIAL`
const fn pad4(s: &str) -> [u8; 4] {
    let bytes = s.as_bytes();
    let mut out = [0; 4];

    let mut i = 0;
    while i < bytes.len() {
        assert!(i < 4);
        out[i] = bytes[i];
        i += 1;
    }

    out
}

macro_rules! strings_to_bytes {
    ($($s:literal),* $(,)?) => {
        [
            $(
                pad4($s)[0],
                pad4($s)[1],
                pad4($s)[2],
                pad4($s)[3],
            )*
        ]
    };
}
pub const LOOKUP_SPECIAL: [u8; 260] = strings_to_bytes!(
    "NUL",  // 0x00 - NUL - 00000000
    "SOH",  // 0x01 - SOH - 00000001
    "STX",  // 0x02 - STX - 00000010
    "ETX",  // 0x03 - ETX - 00000011
    "EOT",  // 0x04 - EOT - 00000100
    "ENQ",  // 0x05 - ENQ - 00000101
    "ACK",  // 0x06 - ACK - 00000110
    "BEL",  // 0x07 - BEL - 00000111
    "BS",   // 0x08 - BS - 00001000
    "HT",   // 0x09 - HT - 00001001
    "LF",   // 0x0A - LF - 00001010
    "VT",   // 0x0B - VT - 00001011
    "FF",   // 0x0C - FF - 00001100
    "CR",   // 0x0D - CR - 00001101
    "SO",   // 0x0E - SO - 00001110
    "SI",   // 0x0F - SI - 00001111
    "DLE",  // 0x10 - DLE - 00010000
    "DC1",  // 0x11 - DC1 - 00010001
    "DC2",  // 0x12 - DC2 - 00010010
    "DC3",  // 0x13 - DC3 - 00010011
    "DC4",  // 0x14 - DC4 - 00010100
    "NAK",  // 0x15 - NAK - 00010101
    "SYN",  // 0x16 - SYN - 00010110
    "ETB",  // 0x17 - ETB - 00010111
    "CAN",  // 0x18 - CAN - 00011000
    "EM",   // 0x19 - EM - 00011001
    "SUB",  // 0x1A - SUB - 00011010
    "ESC",  // 0x1B - ESC - 00011011
    "FS",   // 0x1C - FS - 00011100
    "GS",   // 0x1D - GS - 00011101
    "RS",   // 0x1E - RS - 00011110
    "US",   // 0x1F - US - 00011111
    "DEL",  // 0x7F - DEL - 00100000
    "PAD",  // 0x80 - PAD - 00100001
    "HOP",  // 0x81 - HOP - 00100010
    "BPH",  // 0x82 - BPH - 00100011
    "NBH",  // 0x83 - NBH - 00100100
    "IND",  // 0x84 - IND - 00100101
    "NEL",  // 0x85 - NEL - 00100110
    "SSA",  // 0x86 - SSA - 00100111
    "ESA",  // 0x87 - ESA - 00101000
    "HTS",  // 0x88 - HTS - 00101001
    "HTJ",  // 0x89 - HTJ - 00101010
    "VTS",  // 0x8A - VTS - 00101011
    "PLD",  // 0x8B - PLD - 00101100
    "PLU",  // 0x8C - PLU - 00101101
    "RI",   // 0x8D - RI - 00101110
    "SS2",  // 0x8E - SS2 - 00101111
    "SS3",  // 0x8F - SS3 - 00110000
    "DCS",  // 0x90 - DCS - 00110001
    "PU1",  // 0x91 - PU1 - 00110010
    "PU2",  // 0x92 - PU2 - 00110011
    "STS",  // 0x93 - STS - 00110100
    "CCH",  // 0x94 - CCH - 00110101
    "MW",   // 0x95 - MW - 00110110
    "SPA",  // 0x96 - SPA - 00110111
    "EPA",  // 0x97 - EPA - 00111000
    "SOS",  // 0x98 - SOS - 00111001
    "SGCI", // 0x99 - SGCI - 00111010
    "SCI",  // 0x9A - SCI - 00111011
    "CSI",  // 0x9B - CSI - 00111100
    "ST",   // 0x9C - ST - 00111101
    "OSC",  // 0x9D - OSC - 00111110
    "PM",   // 0x9E - PM - 00111111
    "APC",  // 0x9F - APC - 01000000
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BaseStyle {
    NonStyled = 0,
    Comment,
    DocComment,
    ConstantOrField,
    FunctionOrOverloadOperators,
    Number,
    String,
    KeywordOrStringEscape,
    Attribute,
    Macro,
    LabelOrLifeTime,
    Self_,
    TypeParameter,
    Type,
    EnumVariant,
    Trait,
    UnsafeCall,
    QuestionMark,
    Todo,
    Unused,
    ConditionallyDisabled,
    LspHint,
    LspLens,
    Autocompletion,
}

impl From<CharStyle> for BaseStyle {
    fn from(value: CharStyle) -> Self {
        unsafe { std::mem::transmute((value.0 >> 26) as u8) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DiagnosticStyle {
    None = 0,
    Error,
    UnknownSymbol,
    RuntimeProblem,
    Warning,
    Typo,
}

impl From<CharStyle> for DiagnosticStyle {
    fn from(value: CharStyle) -> Self {
        unsafe { std::mem::transmute(((value.0 >> 22) & 0b111) as u8) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeprecatedState(pub bool);

impl From<CharStyle> for DeprecatedState {
    fn from(d: CharStyle) -> Self {
        Self((d.0 & 0x0020_0000) != 0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct CharStyle(u32);

impl CharStyle {
    pub const NONE: CharStyle = CharStyle(0);
    pub fn new(
        base_style: BaseStyle,
        diagnostic_style: DiagnosticStyle,
        deprecated_state: DeprecatedState,
    ) -> Self {
        let base_style = base_style as u32;
        let diagnostic_style = diagnostic_style as u32;
        let deprecated_state = deprecated_state.0;

        debug_assert!(base_style < 64);
        debug_assert!(diagnostic_style < 16);

        let base_mask = base_style << 26;
        let diagnostic_mask = diagnostic_style << 22;
        let deprecated_mask = if deprecated_state { 0x0020_0000 } else { 0 };

        Self(base_mask | diagnostic_mask | deprecated_mask)
    }

    pub fn render_style(self, is_special: bool, second_color: &mut bool, cell: &mut Cell) {
        if is_special {
            let color = C_BG_SPECIAL_BYTES[*second_color as usize];
            *second_color = !*second_color;
            cell.set_style(
                Style::new()
                    .add_modifier(Modifier::DOUBLE_UNDERLINED)
                    .bg(color),
            );
        }
        cell.set_style(apply_base_style(self.into()));
        cell.set_style(apply_diagnostic_style(self.into()));
        cell.set_style(apply_deprecated_state(self.into()));
    }
}

impl From<DisplayChar> for CharStyle {
    fn from(value: DisplayChar) -> Self {
        Self(value.0 & 0xFFE0_0000)
    }
}

/// [0x0000_0000, 0x0000_D777] => char
/// [0x0000_D800, 0x0000_DFFF] => special value
/// [0x0000_E000, 0x0010_FFFF] => char
/// [0x0011_0000, 0x0013_FFFF) => emojis
/// 0x0013_FFFF                => emoji last
/// [0x0014_0000, 0x001F_FFFF] => long
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct CharValue(u32);

impl CharValue {
    pub const EMOJI_LAST: Self = Self(0x0013_FFFF);
    pub const LOSSY: Self = Self('�' as u32);

    pub(crate) fn is_whitespace(&self) -> bool {
        if let Some(c) = char::from_u32(self.0) {
            c.is_whitespace()
        } else {
            // long:
            // emoji:
            // emoji last:
            // special:
            false
        }
    }

    pub(crate) fn char_start_offset(self) -> usize {
        // char:          0
        // emoji          0
        // emoji last     1
        // long           0
        // special        self.0 & 3

        if (self.0 & 0xFFFF_F800) == 0x0000_D800 {
            self.0 as usize & 3
        } else {
            (self.0 == Self::EMOJI_LAST.0) as usize
        }
    }

    pub(crate) fn is_variable_name(self) -> bool {
        // char:         alphanum or '_'
        // emoji:        false
        // emoji last:   false
        // long:         is_var_name
        // special:      false
        if let Ok(c) = char::try_from(self.0) {
            c.is_alphanumeric() || c == '_'
        } else if let index = self.0.wrapping_sub(0x0014_0000)
            && index <= 0x000B_FFFF
        {
            let list_lock = LONG_LIST.lock().unwrap();
            list_lock[index as usize].is_variable_name()
        } else {
            false
        }
    }

    pub(crate) fn from_utf8_grapheme_to_dstring(
        grapheme: &str,
        string: &mut DisplayString,
        style: CharStyle,
    ) {
        // char:         convert
        // emoji:        emoji + emoji last
        // long:         long
        // special:      lookup table
        if emojis::get(grapheme).is_some() {
            Self::from_emoji(grapheme, string);
        } else if grapheme.len() == 1 {
            Self::from_u8(grapheme.as_bytes()[0] as u32, string, style);
        } else if grapheme.chars().count() == 1 {
            unsafe {
                string.push(Self(grapheme.chars().next().unwrap() as u32).build(style));
            }
        } else {
            unsafe { string.push(Self::from_lsu(LittleStringUni::new(grapheme)).build(style)) }
        }
    }

    fn from_emoji(grapheme: &str, string: &mut DisplayString) {
        let mut lock = EMOJI_LIST.lock().unwrap();
        if lock.len() >= 0x2F_FFFF {
            emoji_list_is_full()
        }
        let idx = lock.len() as u32 + 0x11_0000;
        lock.push(LittleStringUni::new(grapheme)); // todo: I know it leaks memory. I may fix it later...
        unsafe {
            string.push(Self(idx).emoji_to_dchar());
            string.push(Self::EMOJI_LAST.emoji_to_dchar());
        }
    }

    const fn emoji_to_dchar(self) -> DisplayChar {
        debug_assert!(0x0011_0000 <= self.0);
        debug_assert!(0x0013_FFFF >= self.0);
        DisplayChar(self.0)
    }

    pub const fn build(self, style: CharStyle) -> DisplayChar {
        debug_assert!(self.0 & 0xFFE0_0000 == 0);
        debug_assert!(style.0 & 0x001F_FFFF == 0);
        DisplayChar(self.0 | style.0)
    }

    pub(crate) fn from_u8(c: u32, string: &mut DisplayString, style: CharStyle) {
        unsafe {
            if c < 0x20 {
                Self::from_lookup_idx(c, string, style);
            } else if (0x7F..0xA0).contains(&c) {
                let idx = c - const { 0x7f - 0b00100000 };
                Self::from_lookup_idx(idx, string, style);
            } else {
                string.push(Self(c).build(style))
            }
        }
    }

    pub fn from_lsu(lsu: LittleStringUni) -> Self {
        let mut list_lock = LONG_LIST.lock().unwrap();
        let idx = list_lock.len() as u32;
        if idx > 0x000B_FFFF {
            long_list_is_full()
        }
        list_lock.push(lsu); // todo: I know it leaks memory. I may fix it later...
        let idx = idx + 0x0014_0000;
        Self(idx)
    }

    /// Safety: Make sure the encoding is correct, and you give it the correct [idx/4]
    pub(crate) unsafe fn from_lookup_idx(
        idx_div_4: u32,
        string: &mut DisplayString,
        style: CharStyle,
    ) {
        let in_list_idx = idx_div_4 * 4;
        let mut in_list_sym_idx = in_list_idx;
        while in_list_sym_idx < in_list_idx + 4 {
            if LOOKUP_SPECIAL[in_list_sym_idx as usize] == 0 {
                break;
            }
            unsafe {
                string.push(Self(0x0000_D800 + in_list_sym_idx).build(style));
            } // Safety: The caller
            in_list_sym_idx += 1;
        }
    }

    /// Safety: Make sure it is utf8
    #[allow(nonstandard_style)]
    pub(crate) unsafe fn utf8__write_to<F: Write>(self, file: &mut F) -> Result<(), Error> {
        // char:             c.encode_utf8
        // special:          write
        // peek from list:   just push_str as bytes
        // emoji from list:  just push_str as bytes
        // emoji second:     DO NOTHING

        if self.0 < 0x11_0000 {
            if let Some(c) = char::from_u32(self.0) {
                file.write_all(c.encode_utf8(&mut [0; 4]).as_bytes())
            } else if self.0 & 3 != 0 {
                // Nothing to do
                Ok(())
            } else if self.0 < const { 0xD800 + (0x20 * 4) } {
                let idx = self.0 - const { 0xD800 };
                let ch = idx / 4;
                file.write_all(&[ch as u8])
            } else {
                // if self.0 < const { 0xD800 + (65 * 4) } {
                let idx = self.0 - const { 0xD800 + 0x20 * 4 - 0x7f * 4 };
                let ch = idx / 4;
                file.write_all(&[ch as u8])
            }
        } else {
            match self.0.cmp(&0x13_FFFF) {
                Ordering::Less => {
                    let emoji_lock = EMOJI_LIST.lock().unwrap();
                    file.write_all(emoji_lock[self.0 as usize - 0x11_0000].as_bytes())
                }
                Ordering::Equal => {
                    // Emoji last: Do nothing
                    Ok(())
                }
                Ordering::Greater => {
                    let long_lock = LONG_LIST.lock().unwrap();
                    file.write_all(long_lock[self.0 as usize - 0x14_0000].as_bytes())
                }
            }
        }
    }

    /// Safety: Make sure it is raw
    #[allow(nonstandard_style)]
    pub(crate) unsafe fn raw__write_to<F: Write>(self, file: &mut F) -> Result<(), Error> {
        // byte:             byte
        // special:          peek from the special list
        // peek from list:   PANIC
        // emoji from list:  PANIC
        // emoji second:     PANIC

        if self.0 < 0x100 {
            file.write_all(&[self.0 as u8])?;
        } else if self.0 & 3 != 0 {
            // Nothing to do
        } else if self.0 < 0xD800 {
            unreachable!(
                "raw buffer contains non-raw chars. It should not happen. You called \
            raw__write_to function on a non-raw buffer, or a bug happened"
            )
        } else if self.0 < const { 0xD800 + (0x20 * 4) } {
            let idx = self.0 - const { 0xD800 };
            let ch = idx / 4;
            file.write_all(&[ch as u8])?;
        } else if self.0 < const { 0xD800 + (65 * 4) } {
            let idx = self.0 - const { 0xD800 + 0x20 * 4 - 0x7f * 4 };
            let ch = idx / 4;
            file.write_all(&[ch as u8])?;
        } else {
            unreachable!(
                "raw buffer contains non-raw chars. It should not happen. You called \
            raw__write_to function on a non-raw buffer, or a bug happened"
            )
        }

        Ok(())
    }

    /// Safety: Make sure the encoding is utf8
    pub(crate) unsafe fn utf8_to_raw(self, string: &mut DisplayString, style: CharStyle) {
        // char:         Self::from_char...(c.encode_chars as bytes)
        // special:      self
        // long:         for each char, do step 1
        // emoji:        for each char, do step 1
        // emoji last:   DO NOTHING

        if let Ok(c) = char::try_from(self.0) {
            Self::char_to_raw(c, string, style);
        } else if self.0 < 0x11_0000 {
            unsafe { string.push(self.build(style)) }
        } else if self.0 < 0x13_FFFF {
            let index = self.0 as usize - 0x11_0000;
            let list_lock = EMOJI_LIST.lock().unwrap();
            for c in list_lock[index].chars() {
                Self::char_to_raw(c, string, style);
            }
        } else if self.0 == 0x13_FFFF {
            // Do nothing
        } else {
            let index = self.0 as usize - 0x14_0000;
            let list_lock = LONG_LIST.lock().unwrap();
            for c in list_lock[index].chars() {
                Self::char_to_raw(c, string, style);
            }
        }
    }

    /// Safety: Make sure the encoding is raw
    pub(crate) unsafe fn raw_to_utf8(self, file: &mut DisplayString, style: CharStyle) {
        // char:             char
        // special:          peek from the special list
        // peek from list:   PANIC
        // emoji from list:  PANIC
        // emoji second:     PANIC

        let ch = if self.0 & 0xFFFF_F800 == 0x0000_D800 {
            if self.0 & 3 != 0 {
                return;
            }
            let idx = if self.0 < const { 0xD800 + (0x20 * 4) } {
                self.0 - const { 0xD800 }
            } else {
                self.0 - const { 0xD800 + 0x20 * 4 - 0x7f * 4 }
            };
            idx / 4
        } else {
            self.0
        };

        unsafe {
            if ch < 0x80 {
                Self::from_u8(ch, file, style);
            } else {
                file.push(Self::LOSSY.build(style));
            }
        } // Safety: The caller
    }

    fn char_to_raw(ch: char, string: &mut DisplayString, style: CharStyle) {
        for &b in ch.encode_utf8(&mut [0; 4]).as_bytes() {
            CharValue::from_u8(b as u32, string, style)
        }
    }

    /// The utf8 char should fit into one cell
    #[inline]
    pub(crate) const fn from_utf8_char_one_cell(c: char) -> Self {
        Self(c as u32)
    }

    pub fn render_cell_content(
        self,
        cell: &mut Cell,
        last: bool,
        emojis_to_render: &mut Vec<(u16, u16, u32)>,
        x: u16,
        y: u16,
    ) {
        // char:        encode bytes
        // 0xAD:        " "
        // emoji:       emoji queue (if not last) + "..."
        // emoji last:  "..."
        // long:        render
        // special:     from lookup table
        if self.0 == 0xAD {
            cell.set_symbol(" ");
        } else if let Some(ch) = char::from_u32(self.0) {
            cell.set_symbol(ch.encode_utf8(&mut [0; 4]));
        } else if self.0 < 0x11_0000 {
            cell.set_symbol(
                char::from_u32(LOOKUP_SPECIAL[self.0 as usize & 0x1FF] as u32)
                    .unwrap()
                    .encode_utf8(&mut [0; 4]),
            );
        } else if self.0 >= 0x14_0000 {
            let list_lock = LONG_LIST.lock().unwrap();
            cell.set_symbol(&list_lock[self.0 as usize - 0x14_0000]);
        } else {
            cell.set_symbol("…");
            cell.set_diff_option(CellDiffOption::AlwaysUpdate);
            if !last && self.0 != Self::EMOJI_LAST.0 {
                emojis_to_render.push((x, y, self.0))
            }
        }
    }

    pub fn is_special(self) -> bool {
        self.0 & 0xFFFF_F800 == 0x0000_D800
    }

    pub fn u32(self) -> u32 {
        self.0
    }
}

#[cold]
fn emoji_list_is_full() -> ! {
    panic!("emoji list is full");
}

#[cold]
fn long_list_is_full() -> ! {
    panic!("long list is full");
}

impl From<DisplayChar> for CharValue {
    fn from(value: DisplayChar) -> Self {
        Self(value.0 & 0x001F_FFFF)
    }
}

impl PartialEq<char> for CharValue {
    fn eq(&self, other: &char) -> bool {
        (*other as u32) == self.0
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct DisplayChar(u32);

impl DisplayChar {
    pub fn char(self) -> CharValue {
        self.into()
    }

    pub fn style(self) -> CharStyle {
        self.into()
    }

    /// This function returns a zero value, which does nothing even on drop
    #[inline]
    pub(crate) unsafe fn zeroed() -> DisplayChar {
        Self(0)
    }

    pub fn u32(self) -> u32 {
        self.0
    }

    pub fn render(
        self,
        emojis_to_render: &mut Vec<(u16, u16, u32)>,
        second_color: &mut bool,
        x: u16,
        y: u16,
        last: bool,
        buf: &mut ratatui::buffer::Buffer,
    ) {
        #[cfg(debug_assertions)]
        if buf.cell((x, y)).is_none() {
            LOGS.push(Log {
                message: format!("BUG!! Please contact us. This is a bug about rendering the ui. The position does not exist. file {}, line {}", file!(), line!()),
                color: C_LOG_ERROR,
                handler: None,
            });
            return;
        }
        self.char().render_cell_content(
            buf.cell_mut((x, y)).unwrap(),
            last,
            emojis_to_render,
            x,
            y,
        );
        self.style().render_style(
            self.char().is_special(),
            second_color,
            buf.cell_mut((x, y)).unwrap(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::random;
    use rstest::rstest;

    #[test]
    fn zeroed() {
        unsafe { assert_eq!(DisplayChar::zeroed().u32(), 0) }
    }

    #[test]
    fn split_dchar() {
        let d = DisplayChar(0b10001011_10110111_10101011_00000100);
        assert_eq!(
            d.char().u32(),
            CharValue(0b00000000_00010111_10101011_00000100).u32()
        );
        assert_eq!(d.style(), CharStyle(0b10001011_10100000_00000000_00000000))
    }

    #[test]
    fn pad4_test() {
        assert_eq!(PAD4(""), [0, 0, 0, 0]);
        assert_eq!(PAD4("a"), [b'a', 0, 0, 0]);
        assert_eq!(PAD4("ab"), [b'a', b'b', 0, 0]);
        assert_eq!(PAD4("abc"), [b'a', b'b', b'c', 0]);
        assert_eq!(PAD4("abcd"), [b'a', b'b', b'c', b'd']);
    }

    #[test]
    #[should_panic]
    fn pad4_test_long() {
        PAD4("abcde");
    }

    #[rstest]
    // Base styles:
    #[case(
        BaseStyle::NonStyled,
        DiagnosticStyle::None,
        DeprecatedState(false),
        0x0000_0000
    )]
    #[case(
        BaseStyle::Comment,
        DiagnosticStyle::None,
        DeprecatedState(false),
        0x0400_0000
    )]
    #[case(
        BaseStyle::DocComment,
        DiagnosticStyle::None,
        DeprecatedState(false),
        0x0800_0000
    )]
    #[case(
        BaseStyle::ConstantOrField,
        DiagnosticStyle::None,
        DeprecatedState(false),
        0x0C00_0000
    )]
    #[case(
        BaseStyle::FunctionOrOverloadOperators,
        DiagnosticStyle::None,
        DeprecatedState(false),
        0x1000_0000
    )]
    // ...
    // (Middle variants should be correct if others are)
    // ...
    #[case(
        BaseStyle::LspHint,
        DiagnosticStyle::None,
        DeprecatedState(false),
        0x5400_0000
    )]
    #[case(
        BaseStyle::LspLens,
        DiagnosticStyle::None,
        DeprecatedState(false),
        0x5800_0000
    )]
    #[case(
        BaseStyle::Autocompletion,
        DiagnosticStyle::None,
        DeprecatedState(false),
        0x5C00_0000
    )]
    // Diagnostic styles:
    #[case(
        BaseStyle::NonStyled,
        DiagnosticStyle::Error,
        DeprecatedState(false),
        0x0040_0000
    )]
    #[case(
        BaseStyle::NonStyled,
        DiagnosticStyle::UnknownSymbol,
        DeprecatedState(false),
        0x0080_0000
    )]
    #[case(
        BaseStyle::NonStyled,
        DiagnosticStyle::RuntimeProblem,
        DeprecatedState(false),
        0x00C0_0000
    )]
    #[case(
        BaseStyle::NonStyled,
        DiagnosticStyle::Warning,
        DeprecatedState(false),
        0x0100_0000
    )]
    #[case(
        BaseStyle::NonStyled,
        DiagnosticStyle::Typo,
        DeprecatedState(false),
        0x0140_0000
    )]
    // deprecated = true:
    #[case(
        BaseStyle::NonStyled,
        DiagnosticStyle::Error,
        DeprecatedState(true),
        0x0060_0000
    )]
    // All together:
    #[case(
        BaseStyle::Comment,
        DiagnosticStyle::Error,
        DeprecatedState(false),
        0x0440_0000
    )]
    #[case(
        BaseStyle::DocComment,
        DiagnosticStyle::UnknownSymbol,
        DeprecatedState(true),
        0x08A0_0000
    )]
    #[case(
        BaseStyle::Number,
        DiagnosticStyle::RuntimeProblem,
        DeprecatedState(false),
        0x14C0_0000
    )]
    #[case(
        BaseStyle::String,
        DiagnosticStyle::Warning,
        DeprecatedState(true),
        0x1920_0000
    )]
    #[case(
        BaseStyle::Trait,
        DiagnosticStyle::Typo,
        DeprecatedState(false),
        0x3D40_0000
    )]
    #[case(
        BaseStyle::UnsafeCall,
        DiagnosticStyle::Error,
        DeprecatedState(true),
        0x4060_0000
    )]
    #[case(
        BaseStyle::LspHint,
        DiagnosticStyle::UnknownSymbol,
        DeprecatedState(false),
        0x5480_0000
    )]
    #[case(
        BaseStyle::Autocompletion,
        DiagnosticStyle::Typo,
        DeprecatedState(true),
        0x5D60_0000
    )]
    fn test_base_style_encoding(
        #[case] base_style: BaseStyle,
        #[case] diagnostic_style: DiagnosticStyle,
        #[case] deprecated_state: DeprecatedState,
        #[case] expected: u32,
    ) {
        let result = CharStyle::new(base_style, diagnostic_style, deprecated_state);
        assert_eq!(result.0, expected);
    }

    #[test]
    fn test_render_style() {
        let mut cell = Cell::new("a");

        let prev_cell = cell.clone();
        CharStyle::new(
            BaseStyle::NonStyled,
            DiagnosticStyle::None,
            DeprecatedState(false),
        )
        .render_style(false, &mut false, &mut cell);
        assert_eq!(cell.modifier, prev_cell.modifier);

        let prev_cell = cell.clone();
        CharStyle::new(
            BaseStyle::NonStyled,
            DiagnosticStyle::Error,
            DeprecatedState(false),
        )
        .render_style(false, &mut false, &mut cell);
        assert_eq!(cell.modifier, prev_cell.modifier | Modifier::UNDER_CURLED);

        let prev_cell = cell.clone();
        CharStyle::new(
            BaseStyle::Todo,
            DiagnosticStyle::None,
            DeprecatedState(true),
        )
        .render_style(false, &mut false, &mut cell);
        assert_eq!(cell.modifier, prev_cell.modifier | Modifier::CROSSED_OUT);

        // special:
        let mut second_color = false;

        let prev_cell = cell.clone();
        CharStyle::new(
            BaseStyle::Todo,
            DiagnosticStyle::None,
            DeprecatedState(false),
        )
        .render_style(true, &mut second_color, &mut cell);
        assert_ne!(cell.bg, prev_cell.bg);
        assert!(second_color);

        let prev_cell = cell.clone();
        CharStyle::new(
            BaseStyle::Todo,
            DiagnosticStyle::None,
            DeprecatedState(false),
        )
        .render_style(true, &mut second_color, &mut cell);
        assert_ne!(cell.bg, prev_cell.bg);
        assert!(!second_color);
    }

    #[rstest]
    #[case('a', "a")]
    #[case('b', "b")]
    #[case('ا', "ا")]
    #[case('?', "?")]
    #[case('\u{AD}', " ")]
    fn render_cell_content_char(#[case] c: char, #[case] expected: &str) {
        let mut emojis_to_render = vec![];

        let mut cell1 = Cell::new(" ");
        CharValue::from_utf8_char_one_cell(c).render_cell_content(
            &mut cell1,
            false,
            &mut emojis_to_render,
            random(),
            random(),
        );
        assert!(emojis_to_render.is_empty());

        let mut cell2 = Cell::new(" ");
        CharValue::from_utf8_char_one_cell(c).render_cell_content(
            &mut cell2,
            true,
            &mut emojis_to_render,
            random(),
            random(),
        );
        assert!(emojis_to_render.is_empty());

        assert_eq!(cell1, cell2);
        assert_eq!(cell1.symbol(), expected);
    }

    #[test]
    fn render_cell_content_raw() {
        let mut emojis_to_render = vec![];
        for i in (0..0x20).chain(0x7F..0xA0) {
            let mut string = DisplayString::empty();
            CharValue::from_u8(i, &mut string, CharStyle::NONE);

            assert!(string.len() > 1);
            for dc in string {
                assert!(dc.char().is_special());

                let mut cell1 = Cell::new(" ");
                dc.char().render_cell_content(
                    &mut cell1,
                    false,
                    &mut emojis_to_render,
                    random(),
                    random(),
                );
                assert!(emojis_to_render.is_empty());

                let mut cell2 = Cell::new(" ");
                dc.char().render_cell_content(
                    &mut cell2,
                    true,
                    &mut emojis_to_render,
                    random(),
                    random(),
                );
                assert!(emojis_to_render.is_empty());

                assert_eq!(cell1, cell2);
            }
        }
        for i in (0x20..0x7F).chain(0xA0..=0xFF) {
            let mut string = DisplayString::empty();
            CharValue::from_u8(i, &mut string, CharStyle::NONE);

            assert_eq!(string.len(), 1);
            let dc = string.into_iter().next().unwrap();

            assert!(!dc.char().is_special());
            assert_eq!(
                dc.char().u32(),
                CharValue::from_utf8_char_one_cell(char::from_u32(i).unwrap()).u32()
            )
        }
    }

    #[test]
    fn render_cell_content_long() {
        let mut emojis_to_render = vec![];

        let a = "آََََََََََ";
        assert_ne!(a.chars().count(), 1, "This string is not a long sequence");

        let cv = CharValue::from_lsu(LittleStringUni::new(a));

        let mut cell1 = Cell::new(" ");
        cv.render_cell_content(&mut cell1, false, &mut emojis_to_render, random(), random());
        assert!(emojis_to_render.is_empty());

        let mut cell2 = Cell::new(" ");
        cv.render_cell_content(&mut cell2, true, &mut emojis_to_render, random(), random());
        assert!(emojis_to_render.is_empty());

        assert_eq!(cell1, cell2);
        assert_eq!(cell1.symbol(), a);
    }

    #[test]
    fn render_cell_content_emoji() {
        let mut emojis_to_render = vec![];
        let mut string = DisplayString::empty();
        for emoji in ["😂", "👍", "😒", "🌷", "❤", "️✅"] {
            CharValue::from_emoji(emoji, &mut string);

            assert_eq!(string.len(), 2);
            assert_eq!(string[1].char().u32(), CharValue::EMOJI_LAST.u32());

            // emoji part:
            let c = string[0].char();
            let (x, y) = (random(), random());
            let mut cell1 = Cell::new(" ");
            c.render_cell_content(&mut cell1, false, &mut emojis_to_render, x, y);
            assert_eq!(emojis_to_render, [(x, y, c.0)]);

            let mut cell2 = Cell::new(" ");
            c.render_cell_content(&mut cell2, true, &mut emojis_to_render, x, y);
            assert_eq!(emojis_to_render, [(x, y, c.0)]); // Did not change, because last=true

            assert_eq!(cell1, cell2);

            // emoji last part:
            emojis_to_render.clear();
            let c = string[1].char();

            let mut cell1 = Cell::new(" ");
            c.render_cell_content(&mut cell1, false, &mut emojis_to_render, x, y);

            let mut cell2 = Cell::new(" ");
            c.render_cell_content(&mut cell2, true, &mut emojis_to_render, x, y);

            assert_eq!(cell1, cell2);
            assert!(emojis_to_render.is_empty());

            // Cleanup for next loop cycle
            string.clear();
        }
    }

    #[test]
    fn test_whitespace() {
        assert!(CharValue::from_utf8_char_one_cell(' ').is_whitespace());
        assert!(CharValue::from_utf8_char_one_cell('\t').is_whitespace());
        assert!(CharValue::from_utf8_char_one_cell('\n').is_whitespace());
        assert!(CharValue::from_utf8_char_one_cell('\r').is_whitespace());
        assert!(!CharValue::from_utf8_char_one_cell('a').is_whitespace());

        let mut string = DisplayString::empty();
        CharValue::from_u8(0, &mut string, CharStyle::NONE);
        for c in string {
            assert!(!c.char().is_whitespace())
        }
    }

    #[test]
    fn test_char_start_offset() {
        // chars:
        for c in ['a', 'b', 'ا', '1'] {
            assert_eq!(CharValue::from_utf8_char_one_cell(c).char_start_offset(), 0);
        }

        // longs:
        let a = "آََََََََََ";
        assert_ne!(a.chars().count(), 1, "This string is not a long sequence");
        assert_eq!(
            CharValue::from_lsu(LittleStringUni::new(a)).char_start_offset(),
            0
        );

        // special:
        for i in (0..0x20).chain(0x7F..0xA0) {
            let mut string = DisplayString::empty();
            CharValue::from_u8(i, &mut string, CharStyle::NONE);

            assert!(string.len() > 1);
            for (idx, dc) in string.into_iter().enumerate() {
                assert_eq!(dc.char().char_start_offset(), idx);
            }
        }

        // emoji:
        let mut string = DisplayString::empty();
        for emoji in ["😂", "👍", "😒", "🌷", "❤", "️✅"] {
            CharValue::from_emoji(emoji, &mut string);

            assert_eq!(string.len(), 2);
            assert_eq!(string[1].char().u32(), CharValue::EMOJI_LAST.u32());

            assert_eq!(string[0].char().char_start_offset(), 0);
            assert_eq!(string[1].char().char_start_offset(), 1);

            // Cleanup for next loop cycle
            string.clear();
        }
    }

    #[test]
    fn test_is_variable_name() {
        // ========= True cases =========
        // char:
        for c in ['a', 'b', 'ا', '1', '_', 'A'] {
            assert!(CharValue::from_utf8_char_one_cell(c).is_variable_name())
        }
        // long:
        for c in ["a", "b", "آََََََََََ", "1", "_", "a"] {
            assert!(CharValue::from_lsu(LittleStringUni::new(c)).is_variable_name())
        }
        // ========= False cases ==========
        // char:
        for c in ['@', '*', '&'] {
            assert!(!CharValue::from_utf8_char_one_cell(c).is_variable_name())
        }
        // long:
        for c in ["@ََََََ", "*"] {
            assert!(!CharValue::from_lsu(LittleStringUni::new(c)).is_variable_name())
        }
        // emoji:
        let mut string = DisplayString::empty();
        for emoji in ["😂", "👍", "😒", "🌷", "❤", "️✅"] {
            CharValue::from_emoji(emoji, &mut string);

            assert_eq!(string.len(), 2);

            assert!(!string[0].char().is_variable_name());
            assert!(!string[1].char().is_variable_name());

            // Cleanup for next loop cycle
            string.clear();
        }
    }

    #[test]
    fn from_utf8_grapheme_to_dstring() {
        let mut string = DisplayString::empty();

        // char:
        for c in ["a", "b", "%", "ا"] {
            CharValue::from_utf8_grapheme_to_dstring(c, &mut string, CharStyle::NONE);
            assert_eq!(string.len(), 1);
            assert_eq!(string[0].char().u32(), c.chars().next().unwrap() as u32);
            string.clear();
        }

        // special: Impossible

        // long:
        let a = "آََََََََََ";
        assert_ne!(a.chars().count(), 1, "This string is not a long sequence");

        CharValue::from_utf8_grapheme_to_dstring(a, &mut string, CharStyle::NONE);
        assert_eq!(string.len(), 1);
        assert!((0x0014_0000..=0x001F_FFFF).contains(&string[0].char().u32()));
        string.clear();

        // emoji:
        for emoji in ["😂", "👍", "😒", "🌷", "❤"] {
            // Todo: the ️✅ emoji is not supported by `emojis` crate. Should change the emoji detection platform
            CharValue::from_utf8_grapheme_to_dstring(emoji, &mut string, CharStyle::NONE);

            assert_eq!(string.len(), 2, "{emoji}");
            assert_eq!(
                string[1].char().u32(),
                CharValue::EMOJI_LAST.u32(),
                "{emoji}"
            );

            string.clear();
        }
    }

    #[test]
    fn test_char_eq() {
        // compare char with char:
        for c in ['a', 'b', 'ا'] {
            assert_eq!(CharValue::from_utf8_char_one_cell(c), c);
            assert_ne!(CharValue::from_utf8_char_one_cell(c), '#');
        }

        // compare special with char:
        for i in (0..0x20).chain(0x7F..0xA0) {
            let mut string = DisplayString::empty();
            CharValue::from_u8(i, &mut string, CharStyle::NONE);

            assert!(string.len() > 1);
            for dc in string {
                assert_ne!(dc.char(), 'a');
            }
        }

        // compare with long:
        for c in ["a", "b", "ا"] {
            assert_ne!(CharValue::from_lsu(LittleStringUni::new(c)), '#');
            assert_ne!(
                CharValue::from_lsu(LittleStringUni::new(c)),
                c.chars().next().unwrap()
            ); // This is true. a long dchar SHOULD contain more than one character.
        }

        // compare with emojis:
        let mut string = DisplayString::empty();
        CharValue::from_emoji("😂", &mut string);
        assert_eq!(string.len(), 2);

        assert_ne!(string[0].char(), '1');
        assert_ne!(string[1].char(), '1');
    }

    #[test]
    fn utf8_write() {
        let mut dstring = DisplayString::empty();
        unsafe {
            dstring.push(CharValue::from_utf8_char_one_cell('c').build(CharStyle::NONE));
            dstring.push(CharValue::from_utf8_char_one_cell('h').build(CharStyle::NONE));
            dstring.push(CharValue::from_utf8_char_one_cell('a').build(CharStyle::NONE));
            dstring.push(CharValue::from_utf8_char_one_cell('r').build(CharStyle::NONE));
            dstring.push(CharValue::from_utf8_char_one_cell(' ').build(CharStyle::NONE));

            let a = "آََََََََََ";
            assert_ne!(a.chars().count(), 1, "This string is not a long sequence");
            dstring.push(CharValue::from_lsu(LittleStringUni::new(a)).build(CharStyle::NONE));

            dstring.push(CharValue::from_utf8_char_one_cell(' ').build(CharStyle::NONE));

            CharValue::from_u8(0, &mut dstring, CharStyle::NONE);

            dstring.push(CharValue::from_utf8_char_one_cell(' ').build(CharStyle::NONE));

            CharValue::from_emoji("😂", &mut dstring);

            let mut buffer = Vec::new();
            dstring.utf8__write_to(&mut buffer).unwrap();

            assert_eq!(buffer, "char آََََََََََ \0 😂".bytes().collect::<Vec<_>>());
        }
    }

    #[test]
    fn raw_write() {
        let mut dstring = DisplayString::empty();
        for i in 0..0x100 {
            CharValue::from_u8(i, &mut dstring, CharStyle::NONE)
        }

        let mut buffer = Vec::new();
        unsafe {
            dstring.raw__write_to(&mut buffer).unwrap();
        }

        assert_eq!(buffer, b"\x00\x01\x02\x03\x04\x05\x06\x07\x08\t\n\x0b\x0c\r\x0e\x0f\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1a\x1b\x1c\x1d\x1e\x1f !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~\x7f\x80\x81\x82\x83\x84\x85\x86\x87\x88\x89\x8a\x8b\x8c\x8d\x8e\x8f\x90\x91\x92\x93\x94\x95\x96\x97\x98\x99\x9a\x9b\x9c\x9d\x9e\x9f\xa0\xa1\xa2\xa3\xa4\xa5\xa6\xa7\xa8\xa9\xaa\xab\xac\xad\xae\xaf\xb0\xb1\xb2\xb3\xb4\xb5\xb6\xb7\xb8\xb9\xba\xbb\xbc\xbd\xbe\xbf\xc0\xc1\xc2\xc3\xc4\xc5\xc6\xc7\xc8\xc9\xca\xcb\xcc\xcd\xce\xcf\xd0\xd1\xd2\xd3\xd4\xd5\xd6\xd7\xd8\xd9\xda\xdb\xdc\xdd\xde\xdf\xe0\xe1\xe2\xe3\xe4\xe5\xe6\xe7\xe8\xe9\xea\xeb\xec\xed\xee\xef\xf0\xf1\xf2\xf3\xf4\xf5\xf6\xf7\xf8\xf9\xfa\xfb\xfc\xfd\xfe\xff");
    }

    #[test]
    fn utf8_to_raw() {
        let mut dstring = DisplayString::empty();
        unsafe {
            dstring.push(CharValue::from_utf8_char_one_cell('c').build(CharStyle::NONE));
            dstring.push(CharValue::from_utf8_char_one_cell('h').build(CharStyle::NONE));
            dstring.push(CharValue::from_utf8_char_one_cell('a').build(CharStyle::NONE));
            dstring.push(CharValue::from_utf8_char_one_cell('r').build(CharStyle::NONE));
            dstring.push(CharValue::from_utf8_char_one_cell(' ').build(CharStyle::NONE));

            let a = "آََََََََََ";
            assert_ne!(a.chars().count(), 1, "This string is not a long sequence");
            dstring.push(CharValue::from_lsu(LittleStringUni::new(a)).build(CharStyle::NONE));

            dstring.push(CharValue::from_utf8_char_one_cell(' ').build(CharStyle::NONE));

            CharValue::from_u8(0, &mut dstring, CharStyle::NONE);

            dstring.push(CharValue::from_utf8_char_one_cell(' ').build(CharStyle::NONE));

            CharValue::from_emoji("😂", &mut dstring);

            let raw = dstring.utf8_to_raw(CharStyle::NONE);

            let mut iter = "char آََََََََََ \0 😂".bytes();
            let mut string = Vec::with_capacity(1);
            for a in raw {
                a.char().raw__write_to(&mut string).unwrap();
                if !string.is_empty() {
                    assert_eq!(string[0], iter.next().unwrap());
                }
                string.clear();
            }
        }
    }

    #[test]
    fn raw_to_utf8() {
        let mut dstring = DisplayString::empty();
        for i in 0..0x100 {
            CharValue::from_u8(i, &mut dstring, CharStyle::NONE)
        }

        unsafe {
            let utf8 = dstring.raw_to_utf8(CharStyle::NONE);

            let mut string = Vec::with_capacity(0x100);
            for a in utf8 {
                a.char().utf8__write_to(&mut string).unwrap();
            }
            assert_eq!(string, "\x00\x01\x02\x03\x04\x05\x06\x07\x08\t\n\x0b\x0c\r\x0e\x0f\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1a\x1b\x1c\x1d\x1e\x1f !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~\x7f��������������������������������������������������������������������������������������������������������������������������������".as_bytes())
        }
    }
}
