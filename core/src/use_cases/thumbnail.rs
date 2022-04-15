use crate::helpers::PathRefExt;
use crate::result::Result;

use cairo::{Context, Format, ImageSurface};
use poppler::{PopplerDocument, PopplerPage};
use std::path::{Path, PathBuf};
use std::{fmt::Debug, fs::File};
use tracing::{debug, instrument};

const FIRST: usize = 0;

pub trait ThumbnailGenerator: Debug {
    fn generate(&self, pdf_path: &PathBuf, out_path: &PathBuf) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct ThumbnailGeneratorImpl;

impl ThumbnailGenerator for ThumbnailGeneratorImpl {
    #[instrument]
    fn generate(&self, pdf_path: &PathBuf, out_path: &PathBuf) -> Result<()> {
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

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_generate() -> Result<()> {
        // given
        let pdf_path = PathBuf::from("res/doc1.pdf");
        let tmp_dir = tempdir()?;
        let out_path = tmp_dir.path().join("output.png");
        assert!(!out_path.exists());
        let generator = ThumbnailGeneratorImpl;

        // when
        generator.generate(&pdf_path, &out_path)?;

        // then
        // TODO: check also the thumbnail itself
        assert!(out_path.exists());

        Ok(())
    }
}
