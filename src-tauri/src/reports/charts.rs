use docx_rs::*;

/// Create a Pic from raw PNG bytes with a given width in EMUs.
/// Standard DOCX image widths: 5486400 EMU = ~6 inches.
pub fn create_image_from_bytes(png_bytes: &[u8], width_emu: u32, height_emu: u32) -> Pic {
    Pic::new(png_bytes).size(width_emu, height_emu)
}

/// Add an image paragraph to a Docx document.
/// Returns the modified Docx.
pub fn add_chart_image(docx: Docx, png_bytes: &[u8], width_emu: u32, height_emu: u32) -> Docx {
    let pic = create_image_from_bytes(png_bytes, width_emu, height_emu);
    docx.add_paragraph(
        Paragraph::new()
            .add_run(Run::new().add_image(pic))
    )
}
