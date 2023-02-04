use crate::entities::location::{Location, SafePathBuf};
use crate::helpers::PathRefExt;
use crate::result::PreprocessorErr;
use crate::use_cases::services::preprocessor::FilePreprocessor;

use cairo::{Context, Format, ImageSurface};
use poppler::{PopplerDocument, PopplerPage};
use std::fs::create_dir_all;
use std::path::Path;
use std::{fmt::Debug, fs::File};
use tracing::{debug, instrument};

const FIRST: usize = 0;

/// Generates thumbnail of the PDF file.
///
/// The thumbnail is used by the client application to display the document. Always the first page
/// of the PDF document is used to generate the thumbnail.
#[derive(Debug)]
pub struct Pdf;

impl Pdf {
    #[instrument(skip(self))]
    fn generate(&self, pdf_path: &SafePathBuf, out_path: &Path) -> Result<(), PreprocessorErr> {
        let parent_path = out_path.parent().expect("failed to get parent dir");
        create_dir_all(parent_path)?;
        let page = first_page(pdf_path)?;
        let surface = paint_background_and_scale(&page)?;
        debug!("writing thumbnail to: '{}'", out_path.display());
        let mut f: File = File::create(out_path)?;
        surface.write_to_png(&mut f)?;
        Ok(())
    }
}

fn first_page<P: AsRef<Path>>(pdf_path: P) -> Result<PopplerPage, PreprocessorErr> {
    debug!("getting first page of PDF '{}'", pdf_path.str());
    let doc: PopplerDocument = PopplerDocument::new_from_file(pdf_path, "")?;
    Ok(doc
        .get_page(FIRST)
        .unwrap_or_else(|| panic!("failed to get page")))
}

fn paint_background_and_scale(page: &PopplerPage) -> Result<ImageSurface, PreprocessorErr> {
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
    fn preprocess(&self, loc: &Location, target_dir: &Path) -> Result<Location, PreprocessorErr> {
        let Location::FS(paths) = loc;
        let mut result_paths = Vec::new();
        for pdf_path in paths {
            let thumbnail_path = target_dir.join(format!("{}.png", pdf_path.rel_stem()));
            self.generate(pdf_path, &thumbnail_path)?;
            result_paths.push(thumbnail_path.into());
        }
        Ok(Location::FS(result_paths))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::data_providers::preprocessor::DirEntryExt;

    use anyhow::Result;
    use claim::assert_err;
    use tempfile::tempdir;

    #[test]
    fn pdf_preprocessor_returns_generated_thumbnail_location() -> Result<()> {
        // given
        let tmp_dir = tempdir()?;
        let preprocessor = Pdf;
        let paths = vec![SafePathBuf::from("res/doc1.pdf")];
        let target_path = tmp_dir.path().join(format!("{}.png", paths[0].rel_stem()));

        // when
        let res = preprocessor.preprocess(&Location::FS(paths), tmp_dir.path())?;
        let target_loc = Location::FS(vec![SafePathBuf::from(target_path)]);

        // then
        assert_eq!(res, target_loc);

        Ok(())
    }

    #[test]
    fn pdf_preprocessor_puts_pdf_files_under_user_dir() -> Result<()> {
        // given
        let tmp_dir = tempdir()?;
        let preprocessor = Pdf;
        let paths = vec![SafePathBuf::from("res/doc1.pdf")];
        let is_empty = tmp_dir.path().read_dir()?.next().is_none();
        assert!(is_empty);

        // when
        preprocessor.preprocess(&Location::FS(paths), tmp_dir.path())?;
        let user_dir = tmp_dir.path().read_dir()?.next().unwrap()?;

        // then
        assert_eq!(user_dir.name(), "res");
        assert_eq!(user_dir.path().first_filename(), "doc1.png");

        Ok(())
    }

    #[test]
    fn pdf_preprocessor_fails_with_non_pdf_files() {
        // given
        let tmp_dir = tempdir().unwrap();
        let preprocessor = Pdf;
        let paths = vec![SafePathBuf::from("res/doc8.jpg")];

        // when
        let res = preprocessor.preprocess(&Location::FS(paths), tmp_dir.path());

        // then
        assert_err!(res);
    }
}
