use crate::backend::display_string::DisplayString;
use crate::backend::little_string::LittleStringUni;
use once_cell::sync::Lazy;
use rand::{rng, RngExt};
use std::io::{Error, Write};
use std::sync::Mutex;

pub static NORMAL_LIST: Lazy<Mutex<Vec<LittleStringUni>>> = Lazy::new(|| Mutex::new(Vec::new()));
pub static EMOJI_LIST: Lazy<Mutex<Vec<LittleStringUni>>> = Lazy::new(|| Mutex::new(Vec::new()));

macro_rules! my_helper {
    ([$($e:expr),* $(,)?]) => {
        [
            $($e as u8),*
        ]
    };
}
pub const LOOKUP_SPECIAL: [u8; 260] = my_helper!([
'N', 'U', 'L', 0, // 0x00 - NUL - 00000000
'S', 'O', 'H', 0, // 0x01 - SOH - 00000001
'S', 'T', 'X', 0, // 0x02 - STX - 00000010
'E', 'T', 'X', 0, // 0x03 - ETX - 00000011
'E', 'O', 'T', 0, // 0x04 - EOT - 00000100
'E', 'N', 'Q', 0, // 0x05 - ENQ - 00000101
'A', 'C', 'K', 0, // 0x06 - ACK - 00000110
'B', 'E', 'L', 0, // 0x07 - BEL - 00000111
'B', 'S', 0, 0, // 0x08 - BS - 00001000
'H', 'T', 0, 0, // 0x09 - HT - 00001001
'L', 'F', 0, 0, // 0x0A - LF - 00001010
'V', 'T', 0, 0, // 0x0B - VT - 00001011
'F', 'F', 0, 0, // 0x0C - FF - 00001100
'C', 'R', 0, 0, // 0x0D - CR - 00001101
'S', 'O', 0, 0, // 0x0E - SO - 00001110
'S', 'I', 0, 0, // 0x0F - SI - 00001111
'D', 'L', 'E', 0, // 0x10 - DLE - 00010000
'D', 'C', '1', 0, // 0x11 - DC1 - 00010001
'D', 'C', '2', 0, // 0x12 - DC2 - 00010010
'D', 'C', '3', 0, // 0x13 - DC3 - 00010011
'D', 'C', '4', 0, // 0x14 - DC4 - 00010100
'N', 'A', 'K', 0, // 0x15 - NAK - 00010101
'S', 'Y', 'N', 0, // 0x16 - SYN - 00010110
'E', 'T', 'B', 0, // 0x17 - ETB - 00010111
'C', 'A', 'N', 0, // 0x18 - CAN - 00011000
'E', 'M', 0, 0, // 0x19 - EM - 00011001
'S', 'U', 'B', 0, // 0x1A - SUB - 00011010
'E', 'S', 'C', 0, // 0x1B - ESC - 00011011
'F', 'S', 0, 0, // 0x1C - FS - 00011100
'G', 'S', 0, 0, // 0x1D - GS - 00011101
'R', 'S', 0, 0, // 0x1E - RS - 00011110
'U', 'S', 0, 0, // 0x1F - US - 00011111
'D', 'E', 'L', 0, // 0x7F - DEL - 00100000
'P', 'A', 'D', 0, // 0x80 - PAD - 00100001
'H', 'O', 'P', 0, // 0x81 - HOP - 00100010
'B', 'P', 'H', 0, // 0x82 - BPH - 00100011
'N', 'B', 'H', 0, // 0x83 - NBH - 00100100
'I', 'N', 'D', 0, // 0x84 - IND - 00100101
'N', 'E', 'L', 0, // 0x85 - NEL - 00100110
'S', 'S', 'A', 0, // 0x86 - SSA - 00100111
'E', 'S', 'A', 0, // 0x87 - ESA - 00101000
'H', 'T', 'S', 0, // 0x88 - HTS - 00101001
'H', 'T', 'J', 0, // 0x89 - HTJ - 00101010
'V', 'T', 'S', 0, // 0x8A - VTS - 00101011
'P', 'L', 'D', 0, // 0x8B - PLD - 00101100
'P', 'L', 'U', 0, // 0x8C - PLU - 00101101
'R', 'I', 0, 0, // 0x8D - RI - 00101110
'S', 'S', '2', 0, // 0x8E - SS2 - 00101111
'S', 'S', '3', 0, // 0x8F - SS3 - 00110000
'D', 'C', 'S', 0, // 0x90 - DCS - 00110001
'P', 'U', '1', 0, // 0x91 - PU1 - 00110010
'P', 'U', '2', 0, // 0x92 - PU2 - 00110011
'S', 'T', 'S', 0, // 0x93 - STS - 00110100
'C', 'C', 'H', 0, // 0x94 - CCH - 00110101
'M', 'W', 0, 0, // 0x95 - MW - 00110110
'S', 'P', 'A', 0, // 0x96 - SPA - 00110111
'E', 'P', 'A', 0, // 0x97 - EPA - 00111000
'S', 'O', 'S', 0, // 0x98 - SOS - 00111001
'S', 'G', 'C', 'I', // 0x99 - SGCI - 00111010
'S', 'C', 'I', 0, // 0x9A - SCI - 00111011
'C', 'S', 'I', 0, // 0x9B - CSI - 00111100
'S', 'T', 0, 0, // 0x9C - ST - 00111101
'O', 'S', 'C', 0, // 0x9D - OSC - 00111110
'P', 'M', 0, 0, // 0x9E - PM - 00111111
'A', 'P', 'C', 0, // 0x9F - APC - 01000000
]);

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColoringState {
    NoColor,
    NewColor,
    PrevColor,
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

// impl DisplayChar {
//     pub(crate) fn null1() -> DisplayChar { // todo: temporary function
//         Self(0x0000_D800)
//     }
//     pub(crate) fn null2() -> DisplayChar { // todo: temporary function
//         Self(0x0000_D801)
//     }
//     pub(crate) fn null3() -> DisplayChar { // todo: temporary function
//         Self(0x0000_D802)
//     }
// }

impl DisplayChar {
    pub(crate) fn from_utf8_grapheme_to_dstring(grapheme: &str, string: &mut DisplayString) {
        if let Some(_) = emojis::get(grapheme) {
            let mut lock = EMOJI_LIST.lock().unwrap();
            let idx = lock.len() as u32;
            let idx = idx | 0x4000_0000;
            lock.push(LittleStringUni::new(grapheme));
            unsafe {
                string.push(Self(idx));
                string.push(Self(0x7FFF_FFFF));
            }
        } else {
            if grapheme.chars().count() == 1 {
                let c = grapheme.chars().next().unwrap() as u32;
                Self::from_u8_checked(c, string)
            } else {
                unsafe {
                    string.push(Self::from_lsu(LittleStringUni::new(grapheme)))
                }
            }
        }
    }
    
    pub(crate) fn from_lsu(lsu: LittleStringUni) -> Self {
        let mut list_lock = NORMAL_LIST.lock().unwrap();
        let idx = list_lock.len() as u32;
        list_lock.push(lsu); // todo: I know it has memory leak. I may fix it later...
        let value = idx | 0x8000_0000u32;
        Self(value)
    }

    /// Safety: Make sure the encoding is correct, and you give it the correct [idx/4]
    pub(crate) unsafe fn from_lookup_idx(idx_div_4: u32, string: &mut DisplayString) {
        let in_list_idx = idx_div_4 * 4;
        let mut in_list_sym_idx = in_list_idx;
        while in_list_sym_idx < in_list_idx + 4 {
            if LOOKUP_SPECIAL[in_list_sym_idx as usize] == 0 {
                break;
            }
            unsafe {
                string.push(Self(0x0000_D800 + in_list_sym_idx));
            } // Safety: The caller
            in_list_sym_idx += 1;
        }
    }


    pub(crate) fn is_whitespace(&self) -> bool {
        if let Some(c) = char::from_u32(self.0) {
            c.is_whitespace()
        } else {
            // peek from list:
            // emojis:
            // emoji second char:
            // special:
            false
        }
    }

    /// This function returns a zero value, which does nothing even on drop
    #[inline]
    pub(crate) unsafe fn zeroed() -> DisplayChar {
        Self(0)
    }

    /// Safety: It should be one cell utf8 char
    #[inline]
    pub(crate) const unsafe fn from_one_cell_utf8_char_unchecked(c: char) -> DisplayChar {
        Self(c as u32)
    }

    pub(crate) fn from_u8_checked(c: u32, string: &mut DisplayString) {
        unsafe {
            if c < 0x20 {
                Self::from_lookup_idx(c, string);
            } else if c >= 0x7F && c < 0xA0 {
                let idx = c - const { 0x7f - 0b00100000 };
                Self::from_lookup_idx(idx, string);
            } else {
                string.push(Self(c))
            }
        }
    }

    pub(crate) fn get_idx_diff_to_reach_start(self) -> usize {
        // char:            0
        // 0xAD:            0
        // emoji first      0
        // emoji second     1
        // peek from list   0
        // special          self.0 & 3

        if (self.0 & 0xFFFF_F800) == 0x0000_D800 {
            self.0 as usize & 3
        } else {
            (self.0 == 0x7FFF_FFFF) as usize
        }
    }

    pub(crate) fn get_coloring_state(&self) -> ColoringState {
        // char low: No
        // special if self.0 & 3 == 0: New
        // special if self.0 & 3 != 0: Old
        // char high: No
        // emoji index: New
        // emoji second: Old
        // peek: No

        if self.0 & 0x8000_0000 != 0 {
            ColoringState::NoColor
        } else if self.0 & 0x4000_0000 != 0 {
            if self.0 == 0x7FFF_FFFF {
                ColoringState::PrevColor
            } else {
                ColoringState::NewColor
            }
        // Same as [`std::char::convert::char_try_from_u32`]
        } else if (self.0 ^ 0xD800).wrapping_sub(0x800) < 0x110000 - 0x800 {
            ColoringState::NoColor
        } else {
            if self.0 & 3 == 0 {
                ColoringState::NewColor
            } else {
                ColoringState::PrevColor
            }
        }
    }


    pub(crate) fn self_to_string_to_show(self, first: bool, last: bool, s: &mut String) -> bool {
        // char:            put
        // 0xAD:            put space
        // emoji index:     last: three dots, else: random
        // emoji second     first: three dots, else: random
        // peek from list   push_str
        // special          LOOKUP[self.0 & 0x1FF]
        if self.0 == 0xAD {
            s.push(' ');
            false
        } else if let Some(c) = char::from_u32(self.0) {
            s.push(c);
            false
        } else if self.0 & 0x8000_0000u32 != 0 {
            let lock = NORMAL_LIST.lock().unwrap();
            s.push_str(lock[self.0 as usize & 0x7FFF_FFFF].as_ref());
            false
        } else if self.0 == 0x7FFF_FFFF {
            if first {
                s.push_str("…");
                false
            } else {
                s.push(char::from_u32(rng().random_range(0x20..=0x7E)).unwrap());
                false
            }
        } else if self.0 & 0x4000_0000u32 != 0 {
            if last {
                s.push_str("…");
                false
            } else {
                s.push(char::from_u32(rng().random_range(0x20..=0x7E)).unwrap());
                true
            }
        } else {
            s.push_str(str::from_utf8(&[LOOKUP_SPECIAL[self.0 as usize & 0b00000001_11111111]]).unwrap());
            false
        }
    }

    pub(crate) fn is_variable_name(self) -> bool {
        if let Ok(c) = char::try_from(self.0) {
            c.is_alphanumeric() || c == '_'
        } else if self.0 & 0x8000_0000u32 != 0 {
            let index = self.0 & 0x7fff_ffffu32;
            let index = index as usize;
            let list_lock = NORMAL_LIST.lock().unwrap();
            list_lock[index].is_variable_name()
        // I could write these two following lines, but I won't because of the optimizations:
        // } else if self.0 & 0x4000_0000u32 != 0 {
        //     false
        } else {
            false
        }
    }

    /// Safety: Make sure it is utf8
    #[allow(nonstandard_style)]
    pub(crate) unsafe fn utf8__write_to<F: Write>(self, file: &mut F) -> Result<(), Error> {
        // char:             c.encode_utf8
        // special:          PANIC
        // peek from list:   just push_str as bytes
        // emoji from list:  just push_str as bytes
        // emoji second:     DO NOTHING

        if let Ok(c) = char::try_from(self.0) {
            file.write(c.encode_utf8(&mut [0; 4]).as_bytes())?;
        } else if self.0 & 0x8000_0000u32 != 0 {
            let index = self.0 & 0x7fff_ffffu32;
            let index = index as usize;
            let list_lock = NORMAL_LIST.lock().unwrap();
            file.write(list_lock[index].as_bytes())?;
        } else if self.0 == 0x7FFF_FFFF {
            // Do nothing
        } else if self.0 & 0x4000_0000u32 != 0 {
            let index = self.0 & 0x3fff_ffffu32;
            let index = index as usize;
            let list_lock = EMOJI_LIST.lock().unwrap();
            file.write(list_lock[index].as_bytes())?;
        } else if self.0 & 3 != 0 {
            // Do nothing
        } else if self.0 < const { 0xD800 + (0x20 * 4) } {
            let idx = self.0 - const { 0xD800 };
            let ch = idx / 4;
            file.write(&[ch as u8])?;
        } else {
            let idx = self.0 - const { 0xD800 + 0x20 * 4 - 0x7f * 4 };
            let ch = idx / 4;
            file.write(&[ch as u8])?;
        }
        Ok(())
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
            file.write(&[self.0 as u8])?;
        } else if self.0 & 3 != 0 {
            // Nothing to do
        } else if self.0 < 0xD800 {
            panic!("raw buffer contains non-raw chars. It should not happen. You called \
            raw__write_to function on a non-raw buffer, or a bug happened")
        } else if self.0 < const { 0xD800 + (0x20 * 4) } {
            let idx = self.0 - const { 0xD800 };
            let ch = idx / 4;
            file.write(&[ch as u8])?;
        } else if self.0 < const { 0xD800 + (65 * 4) } {
            let idx = self.0 - const { 0xD800 + 0x20 * 4 - 0x7f * 4 };
            let ch = idx / 4;
            file.write(&[ch as u8])?;
        } else {
            panic!("raw buffer contains non-raw chars. It should not happen. You called \
            raw__write_to function on a non-raw buffer, or a bug happened")
        }

        Ok(())
    }

    pub(crate) fn char_to_raw(ch: char, string: &mut DisplayString) {
        for &b in ch.encode_utf8(&mut [0; 4]).as_bytes() {
            Self::from_u8_checked(b as u32, string)
        }
    }


    /// Safety: Make sure the encoding is utf8
    pub(crate) unsafe fn utf8_to_raw(self, string: &mut DisplayString) {
        // char:             Self::from_char...(c.encode_chars as bytes)
        // special:          PANIC
        // peek from list:   for each char, do step 1
        // emoji from list:  for each char, do step 1
        // emoji second:     DO NOTHING

        if let Ok(c) = char::try_from(self.0) {
            Self::char_to_raw(c, string);
        } else if self.0 & 0x8000_0000u32 != 0 {
            let index = self.0 & 0x7fff_ffffu32;
            let index = index as usize;
            let list_lock = NORMAL_LIST.lock().unwrap();
            for c in list_lock[index].chars() {
                Self::char_to_raw(c, string);
            }
        } else if self.0 == 0x7FFF_FFFF {
            // Do nothing
        } else if self.0 & 0x4000_0000u32 != 0 {
            let index = self.0 & 0x3fff_ffffu32;
            let index = index as usize;
            let list_lock = EMOJI_LIST.lock().unwrap();
            for c in list_lock[index].chars() {
                Self::char_to_raw(c, string);
            }
        } else if self.0 & 3 != 0 {
            // Do nothing
        } else {
            panic!("utf8 buffer contains non-utf8 special chars. It should not happen. You called \
            utf8__write_to function on a non-utf8 buffer, or a bug happened")
        }
    }

    /// Safety: Make sure the encoding is raw
    pub(crate) unsafe fn raw_to_utf8(self, file: &mut DisplayString) {
        // char:             char
        // special:          peek from the special list
        // peek from list:   PANIC
        // emoji from list:  PANIC
        // emoji second:     PANIC

        // So, the raw buffer can cast into an utf8 buffer without any problems
        unsafe {
            file.push(self);
        } // Safety: The caller
    }
}

impl Into<u32> for DisplayChar {
    fn into(self) -> u32 {
        self.0
    }
}

impl PartialEq<char> for DisplayChar {
    #[inline]
    fn eq(&self, other: &char) -> bool {
        self.0 == *other as u32
    }
}