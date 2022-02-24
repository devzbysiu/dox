use cairo::{Context, Format, ImageSurface};
use poppler::{PopplerDocument, PopplerPage};
use std::fs::File;
use std::path::Path;

#[allow(unused)]
pub fn generate<P: AsRef<Path>>(
    pdf_path: P,
    out_path: P,
) -> Result<(), Box<dyn std::error::Error>> {
    let doc: PopplerDocument = PopplerDocument::new_from_file(pdf_path, "")?;
    let page: PopplerPage = match doc.get_page(0) {
        Some(p) => p,
        None => panic!("failed to get page"),
    };
    let (width, height) = page.get_size();
    #[allow(clippy::cast_possible_truncation)]
    let surface = ImageSurface::create(Format::Rgb24, width as i32, height as i32)?;
    // Draw a white background to start with.  If you don't, any transparent
    // regions in the PDF will be rendered as black in the final image.
    let ctxt = Context::new(&surface)?;
    ctxt.set_source_rgb(1.0, 1.0, 1.0);
    ctxt.scale(1.0, 1.0);
    ctxt.paint()?;
    // Draw the contents of the PDF onto the page.
    page.render(&ctxt);
    let filename = format!("{}_{}.png", out_path.as_ref().display(), 0);
    println!("Exporting {} ...", filename);
    let mut f: File = File::create(filename)?;
    surface.write_to_png(&mut f)?;
    Ok(())
}
