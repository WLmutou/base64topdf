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

pub(crate) fn decode_image(
    input: &str,
) -> Result<(Vec<u8>, img::DynamicImage, img::ImageFormat), String> {
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
        .map_err(|_| "无法识别图片格式（支持 PNG / JPG / JPEG）".to_owned())?;

    let decoded_image =
        img::load_from_memory(&bytes).map_err(|error| format!("图片解析失败：{error}"))?;

    Ok((bytes, decoded_image, detected))
}

pub(crate) fn encode_image(
    image: &img::DynamicImage,
    format: OutputFormat,
) -> Result<Vec<u8>, String> {
    let mut bytes = Cursor::new(Vec::new());

    let result = match format {
        OutputFormat::Png => image.write_to(&mut bytes, img::ImageFormat::Png),
        OutputFormat::Jpg | OutputFormat::Jpeg => {
            image.to_rgb8().write_to(&mut bytes, img::ImageFormat::Jpeg)
        }
    };

    result.map_err(|error| format!("图片编码失败：{error}"))?;

    Ok(bytes.into_inner())
}

pub(crate) fn describe_format(format: img::ImageFormat) -> (&'static str, &'static str) {
    match format {
        img::ImageFormat::Png => ("PNG", "image/png"),
        img::ImageFormat::Jpeg => ("JPEG", "image/jpeg"),
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
