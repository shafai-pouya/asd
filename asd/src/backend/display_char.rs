use crate::assets::colors::{
    C_BG_SPECIAL_BYTES, apply_base_style, apply_deprecated_state, apply_diagnostic_style,
};
use crate::backend::display_string::DisplayString;
use crate::backend::little_string::LittleStringUni;
use once_cell::sync::Lazy;
use ratatui::buffer::{Cell, CellDiffOption};
use ratatui::prelude::Modifier;
use ratatui::style::Style;
use std::cmp::Ordering;
use std::io::{Error, Write};
use std::sync::Mutex;

pub static LONG_LIST: Lazy<Mutex<Vec<LittleStringUni>>> = Lazy::new(|| Mutex::new(Vec::new()));
pub static EMOJI_LIST: Lazy<Mutex<Vec<LittleStringUni>>> = Lazy::new(|| Mutex::new(Vec::new()));

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
                {
                    const {
                        assert!($s.len() <= 4);
                    };
                    pad4($s)[0]
                },
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
            (self.0 == 0x0013_FFFF) as usize
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
        if idx < 0x000B_FFFF {
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

        // So, the raw buffer can cast into an utf8 buffer without any problems
        unsafe {
            if self.0 & 0xFFFF_F800 == 0x0000_D800 {
                file.push(CharValue::LOSSY.build(style));
            } else {
                file.push(self.build(style));
            }
        } // Safety: The caller
    }

    fn char_to_raw(ch: char, string: &mut DisplayString, style: CharStyle) {
        for &b in ch.encode_utf8(&mut [0; 4]).as_bytes() {
            CharValue::from_u8(b as u32, string, style)
        }
    }

    /// Safety: It should be one cell utf8 char
    #[inline]
    pub(crate) const unsafe fn from_utf8_char(c: char) -> Self {
        Self(c as u32)
    }
    pub(crate) fn get_idx_diff_to_reach_start(self) -> usize {
        // char:          0
        // 0xAD:          0
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

/// Ranges:
///  If it's between 0x0000_0000 to 0x0000_D777 => char
///  If 0x0000_00AD => replace with " "
///  If it's between 0x0000_D800 to 0x0000_DFFF => special:
///     0b11011xxi_iiiiiiII
///         i: index
///         I: index + nth_of_char
///  If it's between 0x0000_E000 to 0x0010_FFFF => char
///  If it's between 0x8000_0000 to 0xFFFF_FFFF => peek from list
///  If it's between 0x4000_0000 to 0x7FFF_FFFE => emoji from emoji list
///  If it's between 0x7FFF_FFFF to 0x7FFF_FFFF => emoji second
///
/// Emoji:
///     First char is the index
///     Second char is 0b11011111_11111111
/// Emoji strategies:
///     for index emojis:
///         if last:
///             return three dots
///         else:
///             return random data
///     for 0b11011111_11111111:
///         if first:
///             return three dots
///         else:
///             return random data
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

    //
    // pub(crate) fn from_lsu(lsu: LittleStringUni) -> Self {
    //     let mut list_lock = NORMAL_LIST.lock().unwrap();
    //     let idx = list_lock.len() as u32;
    //     list_lock.push(lsu); // todo: I know it has memory leak. I may fix it later...
    //     let value = idx | 0x8000_0000u32;
    //     Self(value)
    // }
    //
    /// This function returns a zero value, which does nothing even on drop
    #[inline]
    pub(crate) unsafe fn zeroed() -> DisplayChar {
        Self(0)
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

        // pub(crate) fn to_string_to_show(
        //     &self,
        //     start_x: u16,
        //     start_y: u16,
        //     buf: &mut Buffer,
        //     emojis_to_render: &mut Vec<(u16, u16, u32)>,
        // ) -> String {
        //     let mut second_color = true;
        //     let mut string = String::new();
        //     for (&i, x) in self.gms.iter().zip(start_x..) {
        //         if i.self_to_string_to_show(
        //             x == start_x,
        //             self.len() as u16 - (x - start_x) == 1,
        //             &mut string,
        //         ) {
        //             emojis_to_render.push((x, start_y, i.into()))
        //         }
        //         let wide_idx = i.get_coloring_state();
        //         if wide_idx == ColoringState::NewColor {
        //             second_color = !second_color;
        //         }
        //         if wide_idx != ColoringState::NoColor {
        //             buf.set_style(
        //                 Rect {
        //                     x,
        //                     y: start_y,
        //                     width: 1,
        //                     height: 1,
        //                 },
        //                 Style::new().bg(if second_color {
        //                     C_BG_SPECIAL_BYTE2
        //                 } else {
        //                     C_BG_SPECIAL_BYTE1
        //                 }),
        //             )
        //         }
        //     }
        //     string
        // }
    }

    // pub(crate) fn get_coloring_state(&self) -> ColoringState {
    //     // char low: No
    //     // special if self.0 & 3 == 0: New
    //     // special if self.0 & 3 != 0: Old
    //     // char high: No
    //     // emoji index: New
    //     // emoji second: Old
    //     // peek: No
    //
    //     if self.0 & 0x8000_0000 != 0 {
    //         ColoringState::NoColor
    //     } else if self.0 & 0x4000_0000 != 0 {
    //         if self.0 == 0x7FFF_FFFF {
    //             ColoringState::PrevColor
    //         } else {
    //             ColoringState::NewColor
    //         }
    //     // Same as [`std::char::convert::char_try_from_u32`]
    //     } else if (self.0 ^ 0xD800).wrapping_sub(0x800) < 0x110000 - 0x800 {
    //         ColoringState::NoColor
    //     } else if self.0 & 3 == 0 {
    //         ColoringState::NewColor
    //     } else {
    //         ColoringState::PrevColor
    //     }
    // }
    //
    // pub(crate) fn self_to_string_to_show(self, first: bool, last: bool, s: &mut String) -> bool {
    //     // char:            put
    //     // 0xAD:            put space
    //     // emoji index:     last: three dots, else: random
    //     // emoji second     first: three dots, else: random
    //     // peek from list   push_str
    //     // special          LOOKUP[self.0 & 0x1FF]
    //     if self.0 == 0xAD {
    //         s.push(' ');
    //         false
    //     } else if let Some(c) = char::from_u32(self.0) {
    //         s.push(c);
    //         false
    //     } else if self.0 & 0x8000_0000u32 != 0 {
    //         let lock = NORMAL_LIST.lock().unwrap();
    //         s.push_str(lock[self.0 as usize & 0x7FFF_FFFF].as_ref());
    //         false
    //     } else if self.0 == 0x7FFF_FFFF {
    //         if first {
    //             s.push('…');
    //             false
    //         } else {
    //             s.push(char::from_u32(rng().random_range(0x20..=0x7E)).unwrap());
    //             false
    //         }
    //     } else if self.0 & 0x4000_0000u32 != 0 {
    //         if last {
    //             s.push('…');
    //             false
    //         } else {
    //             s.push(char::from_u32(rng().random_range(0x20..=0x7E)).unwrap());
    //             true
    //         }
    //     } else {
    //         s.push_str(
    //             str::from_utf8(&[LOOKUP_SPECIAL[self.0 as usize & 0b00000001_11111111]]).unwrap(),
    //         );
    //         false
    //     }
    // }
}

// impl From<DisplayChar> for u32 {
//     fn from(value: DisplayChar) -> Self {
//         value.0
//     }
// }
//
// impl PartialEq<char> for DisplayChar {
//     #[inline]
//     fn eq(&self, other: &char) -> bool {
//         self.0 == *other as u32
//     }
// }
