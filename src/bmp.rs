use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use crate::framebuffer::Framebuffer;

pub fn write_bmp_file(path: &str, buffer: &[u32], width: usize, height: usize) -> io::Result<()> {
    let mut file = BufWriter::new(File::create(path)?);
    write_bmp(&mut file, buffer, width, height)?;
    file.flush()
}

fn write_bmp(file: &mut impl Write, buffer: &[u32], width: usize, height: usize) -> io::Result<()> {
    let stride = (width * 3 + 3) & !3;
    let pixel_size = stride * height;
    file.write_all(b"BM")?;
    file.write_all(&((54 + pixel_size) as u32).to_le_bytes())?;
    file.write_all(&0u32.to_le_bytes())?;
    file.write_all(&54u32.to_le_bytes())?;
    file.write_all(&40u32.to_le_bytes())?;
    file.write_all(&(width as u32).to_le_bytes())?;
    file.write_all(&(height as u32).to_le_bytes())?;
    file.write_all(&1u16.to_le_bytes())?;
    file.write_all(&24u16.to_le_bytes())?;
    file.write_all(&0u32.to_le_bytes())?;
    file.write_all(&(pixel_size as u32).to_le_bytes())?;
    for _ in 0..4 { file.write_all(&0u32.to_le_bytes())?; }
    let padding = [0u8; 3];
    for y in (0..height).rev() {
        for x in 0..width {
            let pixel = buffer[y * width + x];
            // BMP stores BGR, bottom row first. Framebuffer stores 0xRRGGBB.
            file.write_all(&[pixel as u8, (pixel >> 8) as u8, (pixel >> 16) as u8])?;
        }
        file.write_all(&padding[..stride - width * 3])?;
    }
    Ok(())
}

fn write_png(writer: impl Write, fb: &Framebuffer) -> io::Result<()> {
    let mut encoder = png::Encoder::new(writer, fb.width as u32, fb.height as u32);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(io::Error::other)?;
    writer.write_image_data(&fb.to_rgb_bytes()).map_err(io::Error::other)?;
    writer.finish().map_err(io::Error::other)?;
    Ok(())
}

// Retains the original public API, now dispatching to the real file format.
pub trait WriteBmp {
    fn render_buffer(&self, file_path: &str) -> io::Result<()>;
}

impl WriteBmp for Framebuffer {
    fn render_buffer(&self, path: &str) -> io::Result<()> {
        match Path::new(path).extension().and_then(|s| s.to_str()) {
            Some(ext) if ext.eq_ignore_ascii_case("bmp") => {
                write_bmp_file(path, &self.buffer, self.width, self.height)
            }
            Some(ext) if ext.eq_ignore_ascii_case("png") => {
                let mut file = BufWriter::new(File::create(path)?);
                write_png(&mut file, self)?;
                file.flush()
            }
            _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "Usa .bmp o .png")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bmp_padding_header_channel_order_and_orientation() {
        for width in 1..=4 {
            let mut pixels = vec![0xFF0000; width];
            pixels.extend(vec![0x0000FF; width]);
            let mut bytes = Vec::new();
            write_bmp(&mut bytes, &pixels, width, 2).unwrap();
            let stride = (width * 3 + 3) & !3;
            assert_eq!(&bytes[..2], b"BM");
            assert_eq!(bytes.len(), 54 + stride * 2);
            assert_eq!(u32::from_le_bytes(bytes[2..6].try_into().unwrap()) as usize, bytes.len());
            assert_eq!(&bytes[54..57], &[255, 0, 0]);
            assert_eq!(&bytes[54 + stride..57 + stride], &[0, 0, 255]);
        }
    }

    #[test]
    fn png_has_real_signature_and_round_trips_rgb() {
        let mut fb = Framebuffer::new(3, 2);
        fb.buffer = vec![0xFF0000, 0x00FF00, 0x0000FF, 0x123456, 0, 0xFFFFFF];
        let mut bytes = Vec::new();
        write_png(&mut bytes, &fb).unwrap();
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
        let mut reader = png::Decoder::new(std::io::Cursor::new(bytes)).read_info().unwrap();
        let mut decoded = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut decoded).unwrap();
        assert_eq!((info.width, info.height), (3, 2));
        assert_eq!(&decoded[..info.buffer_size()], fb.to_rgb_bytes());
    }
}
