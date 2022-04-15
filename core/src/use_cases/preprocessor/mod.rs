use crate::cfg::Config;
use crate::entities::extension::Ext;
use crate::result::Result;
use crate::use_cases::preprocessor::image::Image;
use crate::use_cases::preprocessor::pdf::Pdf;

use std::path::PathBuf;

mod image;
mod pdf;

#[allow(clippy::module_name_repetitions)]
pub trait FilePreprocessor {
    fn preprocess(&self, paths: &[PathBuf]) -> Result<()>;
}

pub trait PreprocessorFactory {
    fn from_ext(ext: &Ext, config: &Config) -> Preprocessor;
}

pub type Preprocessor = Box<dyn FilePreprocessor>;

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct PreprocessorFactoryImpl;

impl PreprocessorFactory for PreprocessorFactoryImpl {
    fn from_ext(ext: &Ext, config: &Config) -> Preprocessor {
        match ext {
            Ext::Png | Ext::Jpg | Ext::Webp => Box::new(Image::new(config.thumbnails_dir.clone())),
            Ext::Pdf => Box::new(Pdf::new(config.thumbnails_dir.clone())),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::helpers::PathRefExt;
    use crate::result::Result;

    use std::path::PathBuf;
    use std::time::Duration;
    use tempfile::tempdir;

    #[test]
    fn test_processor_factory_with_corrent_file() -> Result<()> {
        // given
        let test_cases = vec![
            (Ext::Png, "res/doc1.png", "doc1.png"),
            (Ext::Jpg, "res/doc3.jpg", "doc3.jpg"),
            (Ext::Webp, "res/doc4.webp", "doc4.webp"),
            (Ext::Pdf, "res/doc1.pdf", "doc1.png"),
        ];

        for test_case in test_cases {
            let ext = test_case.0;

            let thumbnails_dir = tempdir()?;
            let config = Config {
                watched_dir: PathBuf::from("not-important"),
                index_dir: PathBuf::from("not-important"),
                thumbnails_dir: thumbnails_dir.path().to_path_buf(),
                cooldown_time: Duration::from_secs(1),
                notifications_addr: "0.0.0.0:8001".parse()?,
            };
            // when
            let extractor = PreprocessorFactoryImpl::from_ext(&ext, &config);
            extractor.preprocess(&[PathBuf::from(test_case.1)])?;

            // then
            let filename = config.thumbnails_dir.first_filename()?;
            assert_eq!(filename, test_case.2);
        }

        Ok(())
    }
}
