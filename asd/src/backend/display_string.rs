use crate::backend::display_char::{CharStyle, CharValue, DisplayChar};
use crate::backend::encoding::Encoding;
use crate::backend::little_string::{LSIntoIter, LittleString};
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

    pub(crate) fn from_str_utf8(value: &str, style: CharStyle) -> Self {
        let mut self_ = Self::empty();
        for x in value.graphemes(true) {
            CharValue::from_utf8_grapheme_to_dstring(x, &mut self_, style);
        }
        self_
    }

    #[inline]
    pub fn clear(&mut self) {
        self.gms.clear();
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
                i.char().utf8__write_to(file)?;
            } // Safety: the caller
        }
        Ok(())
    }

    /// Safety: Make sure it is raw
    #[allow(nonstandard_style)]
    pub(crate) unsafe fn raw__write_to<F: Write>(&self, file: &mut F) -> Result<(), Error> {
        for i in self {
            unsafe {
                i.char().raw__write_to(file)?;
            } // Safety: the caller
        }
        Ok(())
    }

    /// Safety: Make sure it is raw
    #[allow(nonstandard_style)]
    #[allow(dead_code)]
    pub(crate) unsafe fn utf8_to_raw(&self, style: CharStyle) -> DisplayString {
        let mut output = DisplayString::empty();
        for i in self.gms.iter() {
            unsafe {
                i.char().utf8_to_raw(&mut output, style);
            } // Safety: the caller
        }
        output
    }

    /// Safety: Make sure it is raw
    #[allow(nonstandard_style)]
    #[allow(dead_code)]
    pub(crate) unsafe fn raw_to_utf8(&self, style: CharStyle) -> DisplayString {
        let mut output = DisplayString::empty();
        for i in self.gms.iter() {
            unsafe {
                i.char().raw_to_utf8(&mut output, style);
            } // Safety: the caller
        }
        output
    }

    /// For copying and using outside the buffer.
    /// # Note
    /// The returned string can contain anything allowed in a string, containing "\0" or lossy char mark.
    pub(crate) fn to_string_utf8(&self, encoding: Encoding) -> String {
        struct A(Vec<u8>);
        impl Write for A {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.0.extend(buf);
                Ok(buf.len())
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let mut string = A(Vec::new());
        for &i in self {
            unsafe {
                match encoding {
                    Encoding::UTF8(_) => i.char().utf8__write_to(&mut string).unwrap(),
                    Encoding::Raw => i.char().raw__write_to(&mut string).unwrap(),
                }
            }
        }
        String::from_utf8_lossy(&string.0).to_string()
    }
}

impl<'a> IntoIterator for &'a DisplaySlice {
    type Item = &'a DisplayChar;
    type IntoIter = slice::Iter<'a, DisplayChar>;

    fn into_iter(self) -> Self::IntoIter {
        self.gms.iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::backend::display_char::{CharStyle, CharValue};
    use crate::backend::display_string::DisplayString;
    use crate::backend::encoding::Encoding;

    #[test]
    fn to_string_utf8() {
        let utf8_slice = "abcdefgdkgs;g sdffkadf سخبشکمبسکگ fefafkla!!#$@$%$&4567436325\0\x11\x01";
        let (e, s) = Encoding::from_str_utf8(utf8_slice);

        assert!(matches!(e, Encoding::UTF8(_)));
        let s = s[0].to_string_utf8(e);
        assert_eq!(s, utf8_slice);

        let mut s = DisplayString::empty();
        for i in 0..0x100 {
            CharValue::from_u8(i, &mut s, CharStyle::NONE);
        }

        let s = s.to_string_utf8(Encoding::Raw);
        assert_eq!(
            s,
            "\0\u{1}\u{2}\u{3}\u{4}\u{5}\u{6}\u{7}\u{8}\t\n\u{b}\u{c}\r\u{e}\u{f}\u{10}\u{11}\u{12}\u{13}\u{14}\u{15}\u{16}\u{17}\u{18}\u{19}\u{1a}\u{1b}\u{1c}\u{1d}\u{1e}\u{1f} !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~\u{7f}��������������������������������������������������������������������������������������������������������������������������������"
        );
    }
}
