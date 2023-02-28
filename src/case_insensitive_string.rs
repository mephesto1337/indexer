use std::{
    borrow::Cow,
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CaseInsensitiveString<'a>(Cow<'a, str>);

impl Hash for CaseInsensitiveString<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for b in self.as_bytes() {
            state.write_u8(b.to_ascii_lowercase());
        }
        state.write_u8(0xff);
    }
}

impl fmt::Debug for CaseInsensitiveString<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.0.as_ref(), f)
    }
}

impl<'a> CaseInsensitiveString<'a> {
    pub fn new(s: Cow<'a, str>) -> Self {
        Self(s)
    }

    pub fn into_owned(self) -> CaseInsensitiveString<'static> {
        CaseInsensitiveString(self.0.into_owned().into())
    }
}

impl<'a> From<&'a str> for CaseInsensitiveString<'a> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl From<String> for CaseInsensitiveString<'_> {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl Deref for CaseInsensitiveString<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CaseInsensitiveString<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.to_mut()
    }
}

impl PartialEq for CaseInsensitiveString<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(other)
    }
}

impl Eq for CaseInsensitiveString<'_> {}

impl PartialOrd for CaseInsensitiveString<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        for (c1, c2) in self.0.chars().zip(other.0.chars()) {
            match c1
                .to_ascii_uppercase()
                .partial_cmp(&c2.to_ascii_uppercase())?
            {
                Ordering::Equal => continue,
                Ordering::Greater => return Some(Ordering::Greater),
                Ordering::Less => return Some(Ordering::Equal),
            }
        }
        Some(self.len().cmp(&other.len()))
    }
}

impl Ord for CaseInsensitiveString<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works_str() {
        let a: CaseInsensitiveString<'_> = "this".into();
        let b: CaseInsensitiveString<'_> = "This".into();
        assert_eq!(a, b);
        assert!(a <= b);
    }

    #[test]
    fn it_works_string() {
        let a: CaseInsensitiveString<'_> = "this".to_owned().into();
        let b: CaseInsensitiveString<'_> = "THIS".to_owned().into();
        assert_eq!(a, b);
        assert!(a <= b);
    }

    #[test]
    fn it_works_str_string() {
        let a: CaseInsensitiveString<'_> = "this".to_owned().into();
        let b: CaseInsensitiveString<'_> = "THIS".into();
        assert_eq!(a, b);
        assert!(a <= b);
    }

    #[test]
    fn differs() {
        let a: CaseInsensitiveString<'_> = "this1".into();
        let b: CaseInsensitiveString<'_> = "this".into();
        assert_ne!(a, b);
        assert!(a > b);
    }
}
