use std::fs;
use std::path::Path;

use ::image as img;
use iced::Task;
use iced::widget::image::Handle as ImageHandle;
use iced::widget::text_editor;

use crate::codec::{self, OutputFormat};
use crate::sample;

pub(crate) struct Decoded {
    pub(crate) image: img::DynamicImage,
    pub(crate) preview: ImageHandle,
    pub(crate) kind: &'static str,
    pub(crate) mime: &'static str,
    pub(crate) size: usize,
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
            Ok((bytes, decoded_image, detected)) => {
                let (kind, mime) = codec::describe_format(detected);
                let preview = preview_handle(&decoded_image);

                let first_image = self.decoded.is_none();
                self.decoded = Some(Decoded {
                    image: decoded_image,
                    preview,
                    kind,
                    mime,
                    size: bytes.len(),
                });

                if first_image || self.filename.trim().is_empty() {
                    let (format, extension) = match detected {
                        img::ImageFormat::Png => (OutputFormat::Png, "png"),
                        img::ImageFormat::Jpeg => (OutputFormat::Jpg, "jpg"),
                        _ => (OutputFormat::Png, "png"),
                    };
                    self.format = format;
                    self.filename = format!("file.{extension}");
                }

                self.set_status(format!("识别类型：{kind} ({mime})"), false);
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
        decoded.preview = preview_handle(&decoded.image);

        self.set_status("已旋转 90°，下载时保存旋转后的图片".to_owned(), false);
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

        match codec::encode_image(&decoded.image, self.format) {
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
    let source = if image.width() > 1600 || image.height() > 1600 {
        image.thumbnail(1600, 1600)
    } else {
        image.clone()
    };

    let rgba = source.to_rgba8();
    let (width, height) = rgba.dimensions();
    ImageHandle::from_rgba(width, height, rgba.into_raw())
}
