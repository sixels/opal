#![allow(dead_code)]

use std::path::PathBuf;

use image::{io::Reader, GenericImageView, ImageBuffer, Pixel};
use rayon::prelude::*;

macro_rules! rgba {
    ($color:expr) => {
        Rgba::from_u32($color)
    };
}

const NORD: Palette = &[
    rgba!(0x2E3440AF),
    rgba!(0x3B4252AF),
    rgba!(0x434C5EAF),
    rgba!(0x4C566AAF),
    rgba!(0xD8DEE9AF),
    rgba!(0xE5E9F0AF),
    rgba!(0xECEFF4AF),
    rgba!(0x8FBCBBAF),
    rgba!(0x88C0D0AF),
    rgba!(0x81A1C1AF),
    rgba!(0x5E81ACAF),
    rgba!(0xBF616AAF),
    rgba!(0xD08770AF),
    rgba!(0xEBCB8BAF),
    rgba!(0xA3BE8CAF),
    rgba!(0xB48EADAF),
];

const GRUVBOX: Palette = &[
    rgba!(0x1D2021FF),
    rgba!(0x282828FF),
    rgba!(0x32302FFF),
    rgba!(0x3c3836FF),
    rgba!(0x504945FF),
    rgba!(0x665c54FF),
    rgba!(0x7C6F64FF),
    rgba!(0x928374FF),
    rgba!(0x928374FF),
    rgba!(0xFB4934FF),
    rgba!(0xFBF1C7FF),
    rgba!(0xF2E5BCFF),
    rgba!(0xEBDBB2FF),
    rgba!(0xD5C4A1FF),
    rgba!(0xBDAE93FF),
    rgba!(0xA89984FF),
    rgba!(0xFB4934FF),
    rgba!(0xB8BB26FF),
    rgba!(0xFABD2FFF),
    rgba!(0x83A598FF),
    rgba!(0xD3869BFF),
    rgba!(0x8EC07CFF),
    rgba!(0xFE8019FF),
    rgba!(0xCC241DFF),
    rgba!(0x98971AFF),
    rgba!(0xD79921FF),
    rgba!(0x458588FF),
    rgba!(0xB16286FF),
    rgba!(0x689D6AFF),
    rgba!(0xD65D0EFF),
    rgba!(0x9D0006FF),
    rgba!(0x79740EFF),
    rgba!(0xB57614FF),
    rgba!(0x076678FF),
    rgba!(0x8F3F71FF),
    rgba!(0x427B58FF),
    rgba!(0xAF3A03FF),
];

const OCEANIC: Palette = &[
    rgba!(0x1B2B34FF),
    rgba!(0x343D46FF),
    rgba!(0x4F5B66FF),
    rgba!(0x65737EFF),
    rgba!(0xA7ADBAFF),
    rgba!(0xC0C5CEFF),
    rgba!(0xCDD3DEFF),
    rgba!(0xD8DEE9FF),
    rgba!(0xEC5f67FF),
    rgba!(0xF99157FF),
    rgba!(0xFAC863FF),
    rgba!(0x99C794FF),
    rgba!(0x5FB3B3FF),
    rgba!(0x6699CCFF),
    rgba!(0xC594C5FF),
    rgba!(0xAB7967FF),
];

const DRACULA: Palette = &[
    rgba!(0x282a36FF),
    rgba!(0x44475aFF),
    rgba!(0xf8f8f2FF),
    rgba!(0x6272a4FF),
    rgba!(0x8be9fdFF),
    rgba!(0x50fa7bFF),
    rgba!(0xffb86cFF),
    rgba!(0xff79c6FF),
    rgba!(0xbd93f9FF),
    rgba!(0xff5555FF),
    rgba!(0xf1fa8cFF),
];

type Palette<'a> = &'a [Rgba];

struct Rgba {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
    col: u32,
}

impl Rgba {
    const fn from_u32(color: u32) -> Self {
        Self {
            r: ((color >> 3 * 8) & 0xFF) as u8,
            g: ((color >> 2 * 8) & 0xFF) as u8,
            b: ((color >> 1 * 8) & 0xFF) as u8,
            a: ((color >> 0 * 8) & 0xFF) as u8,
            col: color,
        }
    }
}

// find the nearest color from the palette
fn nearest_rgb(palette: Palette, color: image::Rgba<u8>) -> u32 {
    let mut nearest: (f32, u32) = (f32::MAX, 0); // (distance, color)

    let rgba = color.channels();
    let (r, g, b) = (rgba[0] as f32, rgba[1] as f32, rgba[2] as f32);

    for pal_color in palette {
        let pr = pal_color.r as f32;
        let pg = pal_color.g as f32;
        let pb = pal_color.b as f32;

        let redmean = (pr + r) / 2.0;

        let d = ((2.0 + redmean / 256.0) * (pr - r) * (pr - r)
            + 4.0 * (pg - g) * (pg - g)
            + (2.0 + (255.0 - redmean) / 256.0) * (pb - b) * (pb - b))
            .sqrt();

        if d < nearest.0 {
            nearest = (d, pal_color.col);
        }
    }

    nearest.1
}

fn main() {
    let file_name = PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("You should pass an image as argument"),
    );
    let palette = std::env::args().nth(2).unwrap_or(String::from("nord"));
    let file_extension = file_name
        .extension()
        .expect(&format!("{:?} Does not have an extension", &file_name));

    let col_pal = if &palette == "nord" {
        &NORD
    } else if &palette == "gruvbox" {
        &GRUVBOX
    } else if &palette == "oceanic" {
        &OCEANIC
    } else if &palette == "dracula" {
        &DRACULA
    } else {
        panic!("Invalid color palette");
    };

    let data = Reader::open(&file_name)
        .expect(&format!("Could not open {:?}", &file_name))
        .decode()
        .expect("Invalid image");

    let dim = data.dimensions();

    let mut img = ImageBuffer::new(dim.0, dim.1);
    img.enumerate_pixels_mut()
        .par_bridge()
        .for_each(|(x, y, p)| {
            let pixel = data.get_pixel(x, y);
            let color = pixel.to_rgba();

            let nearest = nearest_rgb(col_pal, color);

            *p = image::Rgba::from(nearest.to_be_bytes());
        });

    let _save_file =
        file_name.with_extension(format!("{}.{}", &palette, file_extension.to_str().unwrap()));
    let save_file = _save_file.to_str().unwrap();

    println!("Saving output to {}", save_file);
    img.save(&save_file)
        .expect(&format!("Could not write to {:?}", &save_file));
}
