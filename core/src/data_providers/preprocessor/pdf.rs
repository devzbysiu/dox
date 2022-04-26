use crate::entities::location::Location;
use crate::helpers::PathRefExt;
use crate::result::Result;
use crate::use_cases::preprocessor::FilePreprocessor;

use cairo::{Context, Format, ImageSurface};
use poppler::{PopplerDocument, PopplerPage};
use std::path::{Path, PathBuf};
use std::{fmt::Debug, fs::File};
use tracing::{debug, instrument};

const FIRST: usize = 0;

/// Generates thumbnail of the PDF file.
///
/// The thumbnail is used by the client application to display the document. Always the first page
/// of the PDF document is used to generate the thumbnail.
#[derive(Debug)]
pub struct Pdf {
    thumbnails_dir: PathBuf,
}

impl Pdf {
    pub fn new<P: AsRef<Path>>(thumbnails_dir: P) -> Self {
        let thumbnails_dir = thumbnails_dir.as_ref().to_path_buf();
        Self { thumbnails_dir }
    }

    fn thumbnail_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.thumbnails_dir.join(format!("{}.png", path.filestem()))
    }

    #[instrument(skip(self))]
    fn generate(&self, pdf_path: &Path, out_path: &Path) -> Result<()> {
        let page = first_page(&pdf_path)?;
        let surface = paint_background_and_scale(&page)?;
        debug!("writing thumbnail to: '{}'", out_path.display());
        let mut f: File = File::create(out_path)?;
        surface.write_to_png(&mut f)?;
        Ok(())
    }
}

fn first_page<P: AsRef<Path>>(pdf_path: P) -> Result<PopplerPage> {
    debug!("getting first page of PDF '{}'", pdf_path.as_ref().string());
    let doc: PopplerDocument = PopplerDocument::new_from_file(pdf_path, "")?;
    Ok(doc
        .get_page(FIRST)
        .unwrap_or_else(|| panic!("failed to get page")))
}

fn paint_background_and_scale(page: &PopplerPage) -> Result<ImageSurface> {
    debug!("painting while backgroud and scaling");
    let (width, height) = page.get_size();
    #[allow(clippy::cast_possible_truncation)]
    let surface = ImageSurface::create(Format::Rgb24, width as i32, height as i32)?;
    // Draw a white background to start with.  If you don't, any transparent
    // regions in the PDF will be rendered as black in the final image.
    let ctxt = Context::new(&surface)?;
    ctxt.set_source_rgb(1.0, 1.0, 1.0);
    ctxt.scale(1.0, 1.0);
    ctxt.paint()?;
    page.render(&ctxt);
    Ok(surface)
}

impl FilePreprocessor for Pdf {
    #[instrument]
    fn preprocess(&self, paths: &[PathBuf]) -> Result<()> {
        for pdf_path in paths {
            self.generate(pdf_path, &self.thumbnail_path(pdf_path))?;
        }
        Ok(())
    }

    #[instrument]
    fn preprocess_location(&self, location: &Location) -> Result<()> {
        let Location::FileSystem(paths) = location;
        for pdf_path in paths {
            self.generate(pdf_path, &self.thumbnail_path(pdf_path))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use crate::helpers::DirEntryExt;

    use super::*;

    #[test]
    fn test_preprocess_with_correct_files() -> Result<()> {
        // given
        let tmp_dir = tempdir()?;
        let preprocessor = Pdf::new(&tmp_dir);
        let paths = &[PathBuf::from("res/doc1.pdf")];
        let is_empty = tmp_dir.path().read_dir()?.next().is_none();
        assert!(is_empty);

        // when
        preprocessor.preprocess(paths)?;
        let file = tmp_dir.path().read_dir()?.next().unwrap()?.filename();

        // then
        assert_eq!(file, "doc1.png");

        Ok(())
    }

    #[test]
    #[should_panic(expected = "PDF document is damaged")]
    fn test_preprocess_with_wrong_files() {
        // given
        let tmp_dir = tempdir().unwrap();
        let preprocessor = Pdf::new(tmp_dir);
        let paths = &[PathBuf::from("res/doc8.jpg")];

        // then
        preprocessor.preprocess(paths).unwrap(); // should panic
    }
}
