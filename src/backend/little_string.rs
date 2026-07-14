use std::fmt::Debug;

pub const N_MAX_LITTLE: usize = 10;

#[derive(Clone)]
pub enum LittleString {
    Little((u8, [u8; N_MAX_LITTLE])),
    Big(String)
}

impl LittleString {
    pub(crate) fn from_char_repeated(ch: u8, len: usize) -> Self {
        if len < N_MAX_LITTLE {
            let mut data = [0; N_MAX_LITTLE];
            for i in 0..len {
                data[i] = ch;
            }
            LittleString::Little((len as u8, data))
        } else {
            let mut s = String::with_capacity(len);
            for _ in 0..len {
                s.push(ch as char);
            }
            LittleString::Big(s)
        }
    }
}

impl Default for LittleString {
    #[inline]
    fn default() -> Self {
        Self::empty()
    }
}

impl FromIterator<u8> for LittleString {
    fn from_iter<T: IntoIterator<Item=u8>>(iter: T) -> Self {
        let mut iter = iter.into_iter();

        let mut arr = [0; N_MAX_LITTLE];
        for i in 0..N_MAX_LITTLE {
            match iter.next() {
                Some(v) => arr[i] = v,
                None => return LittleString::Little((i as u8, arr)),
            }
        }

        let mut vec= Vec::with_capacity(N_MAX_LITTLE + 1);
        vec.extend(arr);
        vec.extend(iter);
        LittleString::Big(String::from_utf8(vec).unwrap())
    }
}

impl Debug for LittleString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LittleString::Little((len, data)) => {
                str::from_utf8(&data[..*len as usize]).unwrap().fmt(f)
            }
            LittleString::Big(s) => {
                s.fmt(f)
            }
        }
    }
}

impl From<&str> for LittleString {
    fn from(value: &str) -> Self {
        if value.len() > N_MAX_LITTLE {
            Self::Big(value.to_string())
        } else {
            let mut slice = [0; N_MAX_LITTLE];
            slice[..value.len()].copy_from_slice(value.as_bytes());
            Self::Little((value.len() as u8, slice))
        }
    }
}

impl From<char> for LittleString {
    fn from(value: char) -> Self {
        let mut slice = [0; N_MAX_LITTLE];
        slice[..1].copy_from_slice(&[value as u8]);
        Self::Little((1u8, slice))
    }
}

impl From<String> for LittleString {
    fn from(value: String) -> Self {
        if value.len() > N_MAX_LITTLE {
            Self::Big(value)
        } else {
            let mut slice = [0; N_MAX_LITTLE];
            slice[..value.len()].copy_from_slice(value.as_bytes());
            Self::Little((value.len() as u8, slice))
        }
    }
}

#[allow(dead_code)]
impl LittleString {
    pub(crate) fn len(&self) -> usize {
        match self {
            LittleString::Little((len, _data)) => *len as usize,
            LittleString::Big(s) => s.len()
        }
    }

    pub(crate) fn as_str(&self) -> &str {
        match self {
            LittleString::Little((len, data)) =>
                str::from_utf8(&data[..*len as usize]).unwrap(),
            LittleString::Big(s) =>
                s.as_str(),
        }
    }

    pub(crate) fn added(mut self, other: &str) -> Self {
        self.push_str(other);
        self
    }

    pub(crate) fn push_str(&mut self, other: &str) {
        match self {
            LittleString::Little((len, data)) => {
                let total_len = *len as usize + other.len();
                if total_len > N_MAX_LITTLE {
                    let mut s = String::with_capacity(total_len);
                    s.push_str(str::from_utf8(&data[..*len as usize]).unwrap());
                    s.push_str(other);
                    *self = LittleString::Big(s);
                } else {
                    data[*len as usize..total_len].copy_from_slice(other.as_bytes());
                    *len = total_len as u8;
                }
            }
            LittleString::Big(s) => {
                s.push_str(other);
            }
        }
    }

    pub(crate) fn insert(&mut self, index: usize, ch: u8) {
        let self_ = std::mem::take(self);
        match self_ {
            LittleString::Little((len, data)) => {
                if len + 1 > N_MAX_LITTLE as u8 {
                    let mut s = String::with_capacity(len as usize + 1);
                    s.push_str(str::from_utf8(&data[..index]).unwrap());
                    s.push(ch as char);
                    s.push_str(str::from_utf8(&data[index..]).unwrap());
                    *self = LittleString::Big(s);
                } else {
                    let mut v = data.to_vec();
                    v.insert(index, ch);
                    *self = LittleString::Little((len + 1, *v.first_chunk().unwrap()))
                }
            }
            LittleString::Big(mut s) => {
                s.insert(index, ch as char);
                *self = LittleString::Big(s);
            }
        }
    }

    pub(crate) fn to_string_clone(&self) -> String {
        match self {
            LittleString::Little((len, data)) => String::from_utf8(data[..*len as usize].to_vec()).unwrap(),
            LittleString::Big(s) => s.clone(),
        }
    }

    pub(crate) fn to_string_added_clone(&self, added: &str) -> String {
        match self {
            LittleString::Little((len, data)) => {
                let mut s = String::with_capacity(*len as usize + added.len());
                s.push_str(str::from_utf8(&data[..*len as usize]).unwrap());
                s.push_str(added);
                s
            }
            LittleString::Big(s) => {
                let mut s = s.clone();
                s.push_str(added);
                s
            }
        }
    }

    pub(crate) fn empty() -> LittleString {
        LittleString::Little((0, [0; N_MAX_LITTLE]))
    }

    pub(crate) fn push(&mut self, ch: char) {
        if ch.len_utf8() != 1 { todo!("Not implemented"); }
        match self {
            LittleString::Little((len, data)) => {
                if *len == N_MAX_LITTLE as u8 {
                    let mut s = String::with_capacity(N_MAX_LITTLE + 1);
                    s.push_str(str::from_utf8(data).unwrap());
                    s.push(ch);
                    *self = LittleString::Big(s);
                } else {
                    data[*len as usize] = ch as u8;
                    *len += 1;
                }
            }
            LittleString::Big(s) => {
                s.push(ch);
            }
        }
    }
}