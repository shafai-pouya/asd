use crate::assets::constants::{N_MAX_LITTLE, N_MAX_LITTLE_UNI};
use crate::backend::display_char::DisplayChar;
use crate::backend::display_string::{DisplaySlice, DisplayString};
use crate::backend::encoding::Encoding;
use std::ops::Deref;
use std::vec;

pub enum LSIntoIter {
    Little([DisplayChar; N_MAX_LITTLE], u8, u8),
    Big(vec::IntoIter<DisplayChar>),
}

impl Iterator for LSIntoIter {
    type Item = DisplayChar;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            LSIntoIter::Little(data, idx, len) => {
                if idx >= len {
                    None
                } else {
                    let to_return = Some(data[*idx as usize]);
                    *idx += 1;
                    to_return
                }
            },
            LSIntoIter::Big(b) => b.next()
        }
    }
}

#[derive(Clone)]
pub enum LittleString {
    Little((u8, [DisplayChar; N_MAX_LITTLE])),
    Big(DisplayString)
}

impl IntoIterator for LittleString {
    type Item = DisplayChar;
    type IntoIter = LSIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            LittleString::Little((len, data)) => {
                LSIntoIter::Little(data, 0, len)
            }
            LittleString::Big(s) => {
                LSIntoIter::Big(s.into_iter())
            }
        }
    }
}

impl AsRef<DisplaySlice> for LittleString {
    fn as_ref(&self) -> &DisplaySlice {
        match self {
            LittleString::Little((len, data)) => {
                DisplaySlice::from_slice(&data[..*len as usize])
            }
            LittleString::Big(s) => {
                s
            }
        }
    }
}

impl LittleString {
    pub(crate) fn iter(&'_ self) -> core::slice::Iter<'_, DisplayChar> {
        match self {
            LittleString::Little((len, data)) => {
                data[..*len as usize].iter()
            }
            LittleString::Big(s) => {
                s.gms.iter()
            }
        }
    }

    pub(crate) fn len(&self) -> usize {
        match self {
            LittleString::Little((len, _data)) => *len as usize,
            LittleString::Big(s) => s.len()
        }
    }

    pub(crate) fn from_raw(bytes: &[u8]) -> Self {
        let mut s = DisplayString::empty();
        for &b in bytes {
            DisplayChar::from_u8_checked(b as u32, &mut s)
        }
        Self::Big(s)
    }

    pub(crate) fn encoding_change(&self, src: Encoding, dst: Encoding) -> Self {
        match (src, dst) {
            (Encoding::UTF8(_), Encoding::UTF8(_)) => self.clone(),
            (Encoding::UTF8(_), Encoding::Raw    ) => {
                let mut new = DisplayString::with_capacity(self.len());
                for i in self.iter() {
                    unsafe { i.utf8_to_raw(&mut new); } // Safety: We checked the encoding and it was utf8
                }
                Self::Big(new)
            },
            (Encoding::Raw    , Encoding::UTF8(_)) => {
                let mut new = DisplayString::with_capacity(self.len());
                for i in self.iter() {
                    unsafe { i.raw_to_utf8(&mut new); } // Safety: We checked the encoding and it was utf8
                }
                Self::Big(new)
            },
            (Encoding::Raw    , Encoding::Raw    ) => self.clone(),
        }
    }

    pub(crate) fn from_spaces_repeated(len: usize) -> Self {
        if len > N_MAX_LITTLE {
            let mut s = DisplayString::with_capacity(len);
            for _ in 0..len {
                unsafe {
                    s.push(const { DisplayChar::from_one_cell_utf8_char_unchecked(' ') })
                } // Safety: spaces work for all encodings
            }
            Self::Big(s)
        } else {
            Self::Little((len as u8, [ unsafe { 
                const {
                    DisplayChar::from_one_cell_utf8_char_unchecked(' ')
                }
            }; N_MAX_LITTLE]))
        }
    }

    pub(crate) fn into_dstring(self) -> DisplayString {
        match self {
            LittleString::Little((len, data)) => {
                DisplaySlice::from_slice(&data[..len as usize]).to_dstring()
            }
            LittleString::Big(s) => {
                s
            }
        }
    }

    /// Safety: Make sure about the encodings
    pub(crate) unsafe fn insert(&mut self, index: usize, ch: DisplayChar) {
        let self_ = std::mem::replace(self, Self::empty());
        match self_ {
            LittleString::Little((len, mut data)) => {
                if len == N_MAX_LITTLE as u8 {
                    let mut s = DisplayString::with_capacity(len as usize + 1);
                    unsafe {
                        s.push_slice(DisplaySlice::from_slice(&data[..index]));
                        s.push(ch);
                        s.push_slice(DisplaySlice::from_slice(&data[index..]));
                    } // Safety: the caller
                    *self = LittleString::Big(s);
                } else {
                    for i in (index..len as usize).rev() {
                        data[i + 1] = data[i];
                    }
                    data[index] = ch;
                    *self = LittleString::Little((len + 1, data))
                }
            }
            LittleString::Big(mut s) => {
                s.insert(index, ch);
                *self = LittleString::Big(s);
            }
        }
    }

    pub(crate) fn empty() -> LittleString {
        LittleString::Little((0, [unsafe { DisplayChar::zeroed() }; N_MAX_LITTLE]))
    }

    /// Safety: It should be one cell utf8 char
    pub(crate) unsafe fn from_one_cell_utf8_char_unchecked(c: char) -> LittleString {
        let mut data = [unsafe { DisplayChar::zeroed() }; N_MAX_LITTLE];
        unsafe {
            data[0] = DisplayChar::from_one_cell_utf8_char_unchecked(c);
        } // Safety: The caller
        LittleString::Little((1, data))
    }


    pub(crate) fn from_slice(slice: &DisplaySlice) -> LittleString {
        if slice.len() > N_MAX_LITTLE {
            LittleString::Big(slice.to_dstring())
        } else {
            let mut data = [unsafe { DisplayChar::zeroed() }; N_MAX_LITTLE];
            data[..slice.len()].copy_from_slice(slice.as_ref());
            LittleString::Little((slice.len() as u8, data))
        }
    }


    /// Safety: Make sure the encoding is correct
    pub(crate) unsafe fn push(&mut self, ch: DisplayChar) {
        match self {
            LittleString::Little((len, data)) => {
                if *len == N_MAX_LITTLE as u8 {
                    let mut s = DisplayString::with_capacity(N_MAX_LITTLE + 1);
                    for i in data {
                        unsafe { s.push(*i); } // Safety: The caller
                    }
                    unsafe { s.push(ch); } // Safety: The caller
                    *self = LittleString::Big(s);
                } else {
                    data[*len as usize] = ch;
                    *len += 1;
                }
            }
            LittleString::Big(s) => {
                unsafe { s.push(ch); } // Safety: The caller
            }
        }
    }
}

#[derive(Clone)]
pub enum LittleStringUni {
    /// This SHOULD be a valid utf-8 slice:
    Little((u8, [u8; N_MAX_LITTLE_UNI])),
    Big(String)
}

impl LittleStringUni {

    pub(crate) fn new(value: &str) -> Self {
        if value.len() > N_MAX_LITTLE_UNI {
            Self::Big(value.to_string())
        } else {
            let mut slice = [0; N_MAX_LITTLE_UNI];
            slice[..value.len()].copy_from_slice(value.as_bytes());
            Self::Little((value.len() as u8, slice))
        }
    }

    pub(crate) fn is_variable_name(&self) -> bool {
        self.deref().chars().next().map(|ch| ch.is_alphanumeric() || ch == '_').unwrap_or(false)
    }
}

impl Deref for LittleStringUni {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            LittleStringUni::Little((len, data)) => {
                str::from_utf8(&data[..*len as usize]).unwrap()
            }
            LittleStringUni::Big(s) => {
                s
            }
        }
    }
}