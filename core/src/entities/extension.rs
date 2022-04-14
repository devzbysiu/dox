#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ext {
    Png,
    Jpg,
    Webp,
    Pdf,
}

impl<S: Into<String>> From<S> for Ext {
    fn from(ext: S) -> Self {
        let ext = ext.into();
        match ext.as_ref() {
            "png" => Self::Png,
            "jpg" | "jpeg" => Self::Jpg,
            "webp" => Self::Webp,
            "pdf" => Self::Pdf,
            _ => panic!("failed to create extension from '{}'", ext),
        }
    }
}
