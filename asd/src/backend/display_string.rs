use crate::assets::colors::{C_BG_SPECIAL_BYTE1, C_BG_SPECIAL_BYTE2};
use crate::backend::display_char::{ColoringState, DisplayChar};
use crate::backend::encoding::Encoding;
use crate::backend::little_string::{LSIntoIter, LittleString};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use std::io::{Error, Write};
use std::ops::{Deref, Index, IndexMut, Range, RangeFrom, RangeTo};
use std::slice::SliceIndex;
use std::vec::Splice;
use std::{slice, vec};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone)]
pub struct DisplayString {
    pub gms: Vec<DisplayChar>,
}

impl Index<usize> for DisplayString {
    type Output = DisplayChar;

    fn index(&self, index: usize) -> &Self::Output {
        &self.gms[index]
    }
}

impl IndexMut<usize> for DisplayString {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.gms.index_mut(index)
    }
}

impl Index<Range<usize>> for DisplayString {
    type Output = DisplaySlice;

    fn index(&self, index: Range<usize>) -> &Self::Output {
        DisplaySlice::from_slice(&self.gms[index])
    }
}

impl Index<RangeFrom<usize>> for DisplayString {
    type Output = DisplaySlice;
    fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
        DisplaySlice::from_slice(&self.gms[index])
    }
}
impl Index<RangeTo<usize>> for DisplayString {
    type Output = DisplaySlice;
    fn index(&self, index: RangeTo<usize>) -> &Self::Output {
        DisplaySlice::from_slice(&self.gms[index])
    }
}

impl Deref for DisplayString {
    type Target = DisplaySlice;

    fn deref(&self) -> &Self::Target {
        &self[0..]
    }
}

impl IntoIterator for DisplayString {
    type Item = DisplayChar;
    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.gms.into_iter()
    }
}

impl DisplayString {
    pub(crate) fn empty() -> Self {
        Self { gms: vec![] }
    }

    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.gms.len()
    }

    #[inline]
    pub(crate) fn reserve(&mut self, cap: usize) {
        self.gms.reserve(cap);
    }

    pub(crate) fn from_str(value: &str) -> Self {
        let mut self_ = Self::empty();
        for x in value.graphemes(true) {
            DisplayChar::from_utf8_grapheme_to_dstring(x, &mut self_);
        }
        self_
    }

    #[inline]
    pub(crate) fn with_capacity(cap: usize) -> Self {
        Self {
            gms: Vec::with_capacity(cap),
        }
    }

    /// Safety: If you sure you are pushing an utf8 DChar into an utf8 DString or a Raw DChar into
    /// a raw DString, it's okay
    #[inline]
    pub(crate) unsafe fn push(&mut self, ch: DisplayChar) {
        self.gms.push(ch);
    }

    /// Safety: If you sure you are pushing an utf8 DSlice into an utf8 DString or a Raw DSlice into
    /// a raw DString, it's okay
    pub(crate) unsafe fn push_slice(&mut self, ch: &DisplaySlice) {
        self.reserve(ch.len());
        for &ch in ch {
            self.gms.push(ch);
        }
    }

    /// Safety: If you sure you are pushing an utf8 DChar into an utf8 DString or a Raw DChar into
    /// a raw DString, it's okay
    #[inline]
    pub(crate) fn insert(&mut self, index: usize, ch: DisplayChar) {
        self.gms.insert(index, ch);
    }

    /// Safety: If you sure you are pushing an utf8 LittleString into an utf8 DString or a
    /// Raw LittleString into a raw DString, it's okay
    #[inline]
    pub(crate) fn replace_range(
        &'_ mut self,
        range: Range<usize>,
        i: LittleString,
    ) -> Splice<'_, LSIntoIter> {
        self.gms.splice(range, i)
    }

    #[inline]
    pub(crate) fn truncate(&mut self, len: usize) {
        self.gms.truncate(len)
    }

    pub(crate) fn get<I>(&self, range: I) -> Option<&I::Output>
    where
        I: SliceIndex<[DisplayChar]>,
    {
        self.gms.get(range)
    }

    // pub(crate) fn null() -> Self { // todo: temporary function
    //     Self {
    //         gms: vec![
    //             DisplayChar::null1(),
    //             DisplayChar::null2(),
    //             DisplayChar::null3()
    //         ]
    //     }
    // }
}

#[repr(transparent)] // Should be because some unsafe types later
pub struct DisplaySlice {
    pub gms: [DisplayChar],
}

impl AsRef<[DisplayChar]> for DisplaySlice {
    fn as_ref(&self) -> &[DisplayChar] {
        &self.gms
    }
}

impl Default for &'static DisplaySlice {
    fn default() -> Self {
        DisplaySlice::from_slice(&[])
    }
}

impl DisplaySlice {
    pub const EMPTY: &'static Self = Self::from_slice(&[]);

    pub(crate) const fn from_slice(slice: &[DisplayChar]) -> &Self {
        unsafe { &*(slice as *const [DisplayChar] as *const DisplaySlice) }
    }

    pub(crate) fn to_string_to_show(
        &self,
        start_x: u16,
        start_y: u16,
        buf: &mut Buffer,
        emojis_to_render: &mut Vec<(u16, u16, u32)>,
    ) -> String {
        let mut second_color = true;
        let mut string = String::new();
        for (&i, x) in self.gms.iter().zip(start_x..) {
            if i.self_to_string_to_show(
                x == start_x,
                self.len() as u16 - (x - start_x) == 1,
                &mut string,
            ) {
                emojis_to_render.push((x, start_y, i.into()))
            }
            let wide_idx = i.get_coloring_state();
            if wide_idx == ColoringState::NewColor {
                second_color = !second_color;
            }
            if wide_idx != ColoringState::NoColor {
                buf.set_style(
                    Rect {
                        x,
                        y: start_y,
                        width: 1,
                        height: 1,
                    },
                    Style::new().bg(if second_color {
                        C_BG_SPECIAL_BYTE2
                    } else {
                        C_BG_SPECIAL_BYTE1
                    }),
                )
            }
        }
        string
    }

    #[inline]
    pub(crate) fn to_dstring(&self) -> DisplayString {
        DisplayString {
            gms: self.gms.to_vec(),
        }
    }

    #[inline]
    pub(crate) fn iter(&'_ self) -> slice::Iter<'_, DisplayChar> {
        self.into_iter()
    }

    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.gms.len()
    }

    /// Safety: Make sure it is utf8
    #[allow(nonstandard_style)]
    pub(crate) unsafe fn utf8__write_to<F: Write>(&self, file: &mut F) -> Result<(), Error> {
        for i in self {
            unsafe {
                i.utf8__write_to(file)?;
            } // Safety: the caller
        }
        Ok(())
    }

    /// Safety: Make sure it is raw
    #[allow(nonstandard_style)]
    pub(crate) unsafe fn raw__write_to<F: Write>(&self, file: &mut F) -> Result<(), Error> {
        for i in self {
            unsafe {
                i.raw__write_to(file)?;
            } // Safety: the caller
        }
        Ok(())
    }

    pub(crate) fn to_string_utf8(&self, encoding: Encoding) -> String {
        struct A(String);
        impl Write for A {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.0.push_str(str::from_utf8(buf).unwrap());
                Ok(buf.len())
            } // todo: remove unwrap

            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let mut string = A(String::new());
        for &i in self {
            unsafe {
                match encoding {
                    Encoding::UTF8(_) => i.utf8__write_to(&mut string).unwrap(),
                    Encoding::Raw => i.raw__write_to(&mut string).unwrap(),
                }
            }
        }
        string.0
    }
}

impl<'a> IntoIterator for &'a DisplaySlice {
    type Item = &'a DisplayChar;
    type IntoIter = slice::Iter<'a, DisplayChar>;

    fn into_iter(self) -> Self::IntoIter {
        self.gms.iter()
    }
}
