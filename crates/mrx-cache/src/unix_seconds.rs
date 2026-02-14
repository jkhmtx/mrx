use std::ops::Deref;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct UnixSeconds(i64);

impl From<i64> for UnixSeconds {
    fn from(value: i64) -> Self {
        Self(value.max(0))
    }
}

impl Deref for UnixSeconds {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl UnixSeconds {
    pub(crate) fn to_sql(self) -> i64 {
        self.0
    }
}
