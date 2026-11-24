use std::fs;
use std::path::Path;
use std::time::Duration;

use ::image as img;
use iced::widget::image::Handle as ImageHandle;
use iced::widget::text_editor;
use iced::{Subscription, Task, time};

use crate::codec::{self, AnimationFrame, OutputFormat};
use crate::sample;

pub(crate) struct Decoded {
    pub(crate) image: img::DynamicImage,
    pub(crate) preview: ImageHandle,
    pub(crate) kind: &'static str,
    pub(crate) mime: &'static str,
    pub(crate) size: usize,
    source: Vec<u8>,
    animation: Option<Animation>,
    rotation: u8,
}

struct Animation {
    frames: Vec<AnimationFrame>,
    index: usize,
}

impl Animation {
    fn current_delay(&self) -> Duration {
        let delay = self.frames[self.index].delay_ms;
        // GIF 中 0 或极小延时会导致过高的刷新频率，按常见浏览器行为当作 100ms
        let delay = if delay < 20 { 100 } else { delay };
        Duration::from_millis(u64::from(delay))
    }
}

pub struct App {
    pub(crate) input: text_editor::Content,
    pub(crate) decoded: Option<Decoded>,
    pub(crate) filename: String,
    pub(crate) format: OutputFormat,
    pub(crate) chars: usize,
    pub(crate) status: String,
    pub(crate) is_error: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    InputAction(text_editor::Action),
    LoadSample,
    OpenFile,
    Clear,
    RotateLeft,
    RotateRight,
    NextFrame,
    FilenameChanged(String),
    FormatChanged(OutputFormat),
    Download,
}

impl App {
    pub fn new() -> Self {
        Self {
            input: text_editor::Content::new(),
            decoded: None,
            filename: format!("file.{}", OutputFormat::default().extension()),
            format: OutputFormat::default(),
            chars: 0,
            status: "等待输入 Base64……".to_owned(),
            is_error: false,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::InputAction(action) => {
                let is_edit = action.is_edit();
                self.input.perform(action);

                if is_edit {
                    self.decode();
                }
            }
            Message::LoadSample => {
                let sample = sample::build_base64();
                self.input = text_editor::Content::with_text(&sample);
                self.decode();
            }
            Message::OpenFile => self.open_file(),
            Message::RotateLeft => self.rotate(false),
            Message::RotateRight => self.rotate(true),
            Message::NextFrame => self.advance_frame(),
            Message::Clear => {
                self.input = text_editor::Content::new();
                self.decoded = None;
                self.chars = 0;
                self.format = OutputFormat::Png;
                self.filename = "file.png".to_owned();
                self.set_status("已清空，等待输入 Base64……".to_owned(), false);
            }
            Message::FilenameChanged(filename) => self.filename = filename,
            Message::FormatChanged(format) => {
                self.format = format;

                let stem = Path::new(self.filename.trim())
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .filter(|stem| !stem.is_empty())
                    .unwrap_or("file");

                self.filename = format!("{stem}.{}", format.extension());
            }
            Message::Download => self.download(),
        }

        Task::none()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let delay = self
            .decoded
            .as_ref()
            .and_then(|decoded| decoded.animation.as_ref())
            .filter(|animation| animation.frames.len() > 1)
            .map(Animation::current_delay);

        match delay {
            Some(delay) => time::every(delay).map(|_| Message::NextFrame),
            None => Subscription::none(),
        }
    }

    fn decode(&mut self) {
        let content = self.input.text();
        self.chars = content.chars().count();

        let trimmed = content.trim();

        if trimmed.is_empty() {
            self.decoded = None;
            self.set_status("等待输入 Base64……".to_owned(), false);
            return;
        }

        match codec::decode_image(trimmed) {
            Ok(file) => {
                let (kind, mime) = codec::describe_format(file.format);
                let preview = preview_handle(&file.image);
                let frame_count = file.frames.len();
                let animation = (frame_count > 1).then_some(Animation {
                    frames: file.frames,
                    index: 0,
                });

                let first_image = self.decoded.is_none();
                self.decoded = Some(Decoded {
                    image: file.image,
                    preview,
                    kind,
                    mime,
                    size: file.bytes.len(),
                    source: file.bytes,
                    animation,
                    rotation: 0,
                });

                if first_image || self.filename.trim().is_empty() {
                    let (format, extension) = match file.format {
                        img::ImageFormat::Png => (OutputFormat::Png, "png"),
                        img::ImageFormat::Jpeg => (OutputFormat::Jpg, "jpg"),
                        img::ImageFormat::Gif => (OutputFormat::Gif, "gif"),
                        _ => (OutputFormat::Png, "png"),
                    };
                    self.format = format;
                    self.filename = format!("file.{extension}");
                }

                let status = if frame_count > 1 {
                    format!("识别类型：{kind} ({mime})，共 {frame_count} 帧，循环播放中")
                } else {
                    format!("识别类型：{kind} ({mime})")
                };
                self.set_status(status, false);
            }
            Err(error) => {
                self.decoded = None;
                self.set_status(error, true);
            }
        }
    }

    fn rotate(&mut self, clockwise: bool) {
        let Some(decoded) = self.decoded.as_mut() else {
            return;
        };

        decoded.image = if clockwise {
            decoded.image.rotate90()
        } else {
            decoded.image.rotate270()
        };
        decoded.rotation = if clockwise {
            (decoded.rotation + 1) % 4
        } else {
            (decoded.rotation + 3) % 4
        };

        if let Some(animation) = decoded.animation.as_mut() {
            for frame in &mut animation.frames {
                frame.image = if clockwise {
                    img::imageops::rotate90(&frame.image)
                } else {
                    img::imageops::rotate270(&frame.image)
                };
            }

            let preview = preview_handle_rgba(&animation.frames[animation.index].image);
            decoded.preview = preview;
        } else {
            decoded.preview = preview_handle(&decoded.image);
        }

        self.set_status("已旋转 90°，下载时保存旋转后的图片".to_owned(), false);
    }

    fn advance_frame(&mut self) {
        let Some(decoded) = self.decoded.as_mut() else {
            return;
        };
        let Some(animation) = decoded.animation.as_mut() else {
            return;
        };

        if animation.frames.len() < 2 {
            return;
        }

        animation.index = (animation.index + 1) % animation.frames.len();
        decoded.preview = preview_handle_rgba(&animation.frames[animation.index].image);
    }

    fn open_file(&mut self) {
        let picked = rfd::FileDialog::new()
            .add_filter("Base64 文本", &["txt", "b64", "base64", "text"])
            .add_filter("所有文件", &["*"])
            .pick_file();

        let Some(path) = picked else {
            return;
        };

        match fs::read_to_string(&path) {
            Ok(content) => {
                self.input = text_editor::Content::with_text(&content);
                self.decode();
            }
            Err(error) => {
                self.set_status(format!("读取文件失败：{error}"), true);
            }
        }
    }

    fn download(&mut self) {
        let Some(decoded) = self.decoded.as_ref() else {
            self.set_status("还没有可下载的图片".to_owned(), true);
            return;
        };

        let extension = self.format.extension();
        let mut filename = self.filename.trim().to_owned();

        if filename.is_empty() {
            filename = format!("file.{extension}");
        } else if Path::new(&filename).extension().is_none() {
            filename.push('.');
            filename.push_str(extension);
        }

        let path = rfd::FileDialog::new()
            .add_filter(self.format.label(), &[extension])
            .set_file_name(filename)
            .save_file();

        let Some(path) = path else {
            return;
        };

        let encoded = match (self.format, decoded.animation.as_ref()) {
            // 未旋转时直接保存原始 GIF 字节，完整保留动画与元数据
            (OutputFormat::Gif, Some(_)) if decoded.rotation == 0 => Ok(decoded.source.clone()),
            (OutputFormat::Gif, Some(animation)) => codec::encode_gif_frames(&animation.frames),
            (format, _) => codec::encode_image(&decoded.image, format),
        };

        match encoded {
            Ok(bytes) => match fs::write(&path, bytes) {
                Ok(()) => self.set_status(format!("已保存：{}", path.display()), false),
                Err(error) => {
                    self.set_status(format!("保存失败：{error}"), true);
                }
            },
            Err(error) => self.set_status(error, true),
        }
    }

    fn set_status(&mut self, status: String, is_error: bool) {
        self.status = status;
        self.is_error = is_error;
    }
}

fn preview_handle(image: &img::DynamicImage) -> ImageHandle {
    let (width, height) = (image.width(), image.height());
    let (preview_width, preview_height) = preview_size(width, height);

    let source = if (width, height) == (preview_width, preview_height) {
        image.clone()
    } else {
        image.thumbnail(preview_width, preview_height)
    };

    let rgba = source.to_rgba8();
    let (width, height) = rgba.dimensions();
    ImageHandle::from_rgba(width, height, rgba.into_raw())
}

fn preview_handle_rgba(image: &img::RgbaImage) -> ImageHandle {
    let (width, height) = image.dimensions();
    let (preview_width, preview_height) = preview_size(width, height);

    let source = if (width, height) == (preview_width, preview_height) {
        image.clone()
    } else {
        img::imageops::thumbnail(image, preview_width, preview_height)
    };

    let (width, height) = source.dimensions();
    ImageHandle::from_rgba(width, height, source.into_raw())
}

fn preview_size(width: u32, height: u32) -> (u32, u32) {
    const MAX: u32 = 1600;

    if width <= MAX && height <= MAX {
        return (width, height);
    }

    let scale = MAX as f32 / width.max(height) as f32;
    (
        (width as f32 * scale).round().max(1.0) as u32,
        (height as f32 * scale).round().max(1.0) as u32,
    )
}
