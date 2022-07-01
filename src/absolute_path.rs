use std::path::Path;

pub struct AbsolutePath<P>(P);

impl<P: AsRef<Path>> AbsolutePath<P> {
    pub fn new(path: P) -> Result<Self, P> {
        if path.as_ref().is_absolute() {
            Ok(Self(path))
        } else {
            Err(path)
        }
    }
}

impl<P: AsRef<Path>> AsRef<Path> for AbsolutePath<P> {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}
