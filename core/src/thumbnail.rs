use crate::helpers::PathExt;
use crate::result::Result;

use cairo::{Context, Format, ImageSurface};
use log::debug;
use poppler::{PopplerDocument, PopplerPage};
use std::fs::File;
use std::path::Path;

const FIRST: usize = 0;

pub fn generate<P: AsRef<Path>>(pdf_path: P, out_path: P) -> Result<()> {
    debug!("generating thumbnail for '{}'", pdf_path.as_ref().display());
    let page = first_page(&pdf_path)?;
    let surface = paint_background_and_scale(&page)?;
    debug!("writing thumbnail to: '{}'", out_path.as_ref().display());
    let mut f: File = File::create(out_path)?;
    surface.write_to_png(&mut f)?;
    Ok(())
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
    #[allow(clippy::cast_possible_truncation)] // TODO: make sure that's necessary
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
