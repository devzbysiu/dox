use std::path::Path;

pub trait PathRefExt {
    fn str(&self) -> &str;
    #[cfg(test)] // TODO: should this really be here?
    fn first_filename(&self) -> String;
}

impl<T: AsRef<Path>> PathRefExt for T {
    fn str(&self) -> &str {
        self.as_ref().to_str().expect("path is not utf8")
    }

    #[cfg(test)]
    fn first_filename(&self) -> String {
        use crate::data_providers::thumbnailer::DirEntryExt;

        self.as_ref()
            .read_dir()
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .name()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::path::PathBuf;

    #[test]
    fn test_str_in_path_ref_ext() {
        // given
        let path = PathBuf::from("/some-path/here");

        // when
        let result = path.str();

        // then
        assert_eq!(path.to_str().unwrap(), result);
    }
}
