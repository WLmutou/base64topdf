use std::io::Cursor;

use ::image as img;
use base64::Engine;

pub(crate) fn build_base64() -> String {
    let (width, height) = (640u32, 400u32);
    let mut canvas = img::RgbaImage::from_pixel(width, height, img::Rgba([236, 240, 246, 255]));

    fill_rect(
        &mut canvas,
        24,
        24,
        i64::from(width - 24),
        i64::from(height - 24),
        img::Rgba([255, 255, 255, 255]),
    );
    fill_rect(
        &mut canvas,
        24,
        24,
        i64::from(width - 24),
        88,
        img::Rgba([45, 99, 214, 255]),
    );
    fill_rect(
        &mut canvas,
        52,
        46,
        i64::from(width - 52),
        66,
        img::Rgba([255, 255, 255, 255]),
    );

    let origin = (64i64, 128i64);
    let cell = 8i64;

    for row in 0..25i64 {
        for column in 0..25i64 {
            let outer_column = !(7..18).contains(&column);
            let finder = (outer_column && row < 7) || (column < 7 && row >= 18);

            let filled = if finder {
                let x = column % 7 - 3;
                let y = row % 7 - 3;
                x.abs().max(y.abs()) != 2
            } else {
                (column * 73 + row * 149 + column * row * 17) % 11 < 5
            };

            if filled {
                let x = origin.0 + column * cell;
                let y = origin.1 + row * cell;
                fill_rect(
                    &mut canvas,
                    x,
                    y,
                    x + cell,
                    y + cell,
                    img::Rgba([24, 28, 36, 255]),
                );
            }
        }
    }

    for line in 0..7i64 {
        let y = 140 + line * 30;
        let length = 240 - (line % 3) * 36;
        fill_rect(
            &mut canvas,
            320,
            y,
            320 + length,
            y + 12,
            img::Rgba([203, 210, 222, 255]),
        );
    }

    fill_rect(
        &mut canvas,
        320,
        336,
        320 + 110,
        336 + 26,
        img::Rgba([45, 99, 214, 255]),
    );

    let mut png = Cursor::new(Vec::new());
    img::DynamicImage::ImageRgba8(canvas)
        .write_to(&mut png, img::ImageFormat::Png)
        .expect("示例图片编码失败");

    base64::engine::general_purpose::STANDARD.encode(png.into_inner())
}

fn fill_rect(
    image: &mut img::RgbaImage,
    left: i64,
    top: i64,
    right: i64,
    bottom: i64,
    color: img::Rgba<u8>,
) {
    let (image_width, image_height) = image.dimensions();
    let left = left.clamp(0, image_width as i64);
    let top = top.clamp(0, image_height as i64);
    let right = right.clamp(0, image_width as i64);
    let bottom = bottom.clamp(0, image_height as i64);

    for y in top..bottom {
        for x in left..right {
            image.put_pixel(x as u32, y as u32, color);
        }
    }
}
