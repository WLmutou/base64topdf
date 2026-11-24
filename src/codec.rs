use std::io::Cursor;

use ::image as img;
use base64::Engine;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum OutputFormat {
    #[default]
    Png,
    Jpg,
    Jpeg,
    Gif,
}

impl OutputFormat {
    pub(crate) const ALL: [Self; 4] = [Self::Png, Self::Jpg, Self::Jpeg, Self::Gif];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Png => "PNG",
            Self::Jpg => "JPG",
            Self::Jpeg => "JPEG",
            Self::Gif => "GIF",
        }
    }

    pub(crate) fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpg => "jpg",
            Self::Jpeg => "jpeg",
            Self::Gif => "gif",
        }
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

pub(crate) struct DecodedFile {
    pub(crate) bytes: Vec<u8>,
    pub(crate) image: img::DynamicImage,
    pub(crate) format: img::ImageFormat,
    pub(crate) frames: Vec<AnimationFrame>,
}

pub(crate) struct AnimationFrame {
    pub(crate) delay_ms: u32,
    pub(crate) image: img::RgbaImage,
}

pub(crate) fn decode_image(input: &str) -> Result<DecodedFile, String> {
    let payload = match input.find("base64,") {
        Some(index) => &input[index + "base64,".len()..],
        None => input,
    };

    let cleaned: String = payload
        .chars()
        // 去掉空白以及 Windows 文本文件常见的 UTF-8 BOM
        .filter(|character| !character.is_whitespace() && *character != '\u{feff}')
        .collect();

    if cleaned.is_empty() {
        return Err("Base64 内容为空".to_owned());
    }

    let bytes = decode_base64(&cleaned).map_err(|error| format!("Base64 解码失败：{error}"))?;

    let detected = img::guess_format(&bytes)
        .map_err(|_| "无法识别图片格式（支持 PNG / JPG / JPEG / GIF）".to_owned())?;

    if detected == img::ImageFormat::Gif {
        return decode_gif(bytes);
    }

    let image = img::load_from_memory(&bytes).map_err(|error| format!("图片解析失败：{error}"))?;

    Ok(DecodedFile {
        bytes,
        image,
        format: detected,
        frames: Vec::new(),
    })
}

fn decode_gif(bytes: Vec<u8>) -> Result<DecodedFile, String> {
    use img::AnimationDecoder;

    let decoder = img::codecs::gif::GifDecoder::new(Cursor::new(&bytes))
        .map_err(|error| format!("GIF 解码失败：{error}"))?;

    let frames = decoder
        .into_frames()
        .collect_frames()
        .map_err(|error| format!("GIF 解码失败：{error}"))?;

    let first = frames
        .first()
        .ok_or_else(|| "GIF 不包含任何图像帧".to_owned())?;

    let image = img::DynamicImage::ImageRgba8(first.buffer().clone());

    let frames = frames
        .into_iter()
        .map(|frame| {
            let (numerator, denominator) = frame.delay().numer_denom_ms();
            AnimationFrame {
                delay_ms: numerator / denominator.max(1),
                image: frame.into_buffer(),
            }
        })
        .collect();

    Ok(DecodedFile {
        bytes,
        image,
        format: img::ImageFormat::Gif,
        frames,
    })
}

pub(crate) fn encode_image(
    image: &img::DynamicImage,
    format: OutputFormat,
) -> Result<Vec<u8>, String> {
    if format == OutputFormat::Gif {
        return encode_static_gif(image);
    }

    let mut bytes = Cursor::new(Vec::new());

    let result = match format {
        OutputFormat::Png => image.write_to(&mut bytes, img::ImageFormat::Png),
        OutputFormat::Jpg | OutputFormat::Jpeg => {
            image.to_rgb8().write_to(&mut bytes, img::ImageFormat::Jpeg)
        }
        OutputFormat::Gif => unreachable!(),
    };

    result.map_err(|error| format!("图片编码失败：{error}"))?;

    Ok(bytes.into_inner())
}

fn encode_static_gif(image: &img::DynamicImage) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();

    {
        let mut encoder = img::codecs::gif::GifEncoder::new_with_speed(&mut bytes, 10);
        encoder
            .encode_frame(img::Frame::new(image.to_rgba8()))
            .map_err(|error| format!("图片编码失败：{error}"))?;
    }

    Ok(bytes)
}

pub(crate) fn encode_gif_frames(frames: &[AnimationFrame]) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();

    {
        let mut encoder = img::codecs::gif::GifEncoder::new_with_speed(&mut bytes, 10);
        encoder
            .set_repeat(img::codecs::gif::Repeat::Infinite)
            .map_err(|error| format!("GIF 编码失败：{error}"))?;

        for frame in frames {
            let delay = img::Delay::from_numer_denom_ms(frame.delay_ms.max(10), 1);
            encoder
                .encode_frame(img::Frame::from_parts(frame.image.clone(), 0, 0, delay))
                .map_err(|error| format!("GIF 编码失败：{error}"))?;
        }
    }

    Ok(bytes)
}

pub(crate) fn describe_format(format: img::ImageFormat) -> (&'static str, &'static str) {
    match format {
        img::ImageFormat::Png => ("PNG", "image/png"),
        img::ImageFormat::Jpeg => ("JPEG", "image/jpeg"),
        img::ImageFormat::Gif => ("GIF", "image/gif"),
        _ => ("图片", "application/octet-stream"),
    }
}

pub(crate) fn format_size(bytes: usize) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;

    let bytes = bytes as f64;

    if bytes >= MB {
        format!("{:.1} MB", bytes / MB)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes / KB)
    } else {
        format!("{bytes:.0} B")
    }
}

fn decode_base64(input: &str) -> Result<Vec<u8>, base64::DecodeError> {
    use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD};

    STANDARD
        .decode(input)
        .or_else(|_| STANDARD_NO_PAD.decode(input))
        .or_else(|_| URL_SAFE.decode(input))
        .or_else(|_| URL_SAFE_NO_PAD.decode(input))
}
