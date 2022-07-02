pub struct LowerCaseString<S>(S);

impl<S: AsRef<str>> LowerCaseString<S> {
    pub fn new(string: S) -> Result<Self, ()> {
        if string.as_ref().chars().all(|char| !char.is_uppercase()) {
            Ok(Self(string))
        } else {
            Err(())
        }
    }
}

impl<T: ?Sized, S: AsRef<T>> AsRef<T> for LowerCaseString<S> {
    fn as_ref(&self) -> &T {
        self.0.as_ref()
    }
}

impl From<&str> for LowerCaseString<String> {
    fn from(s: &str) -> Self {
        Self(s.to_lowercase())
    }
}
