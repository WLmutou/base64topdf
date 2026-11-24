use iced::widget::text::Wrapping;
use iced::widget::{
    Space, button, column, container, image, pick_list, row, scrollable, text, text_editor,
    text_input,
};
use iced::{
    Alignment, Background, Border, Color, ContentFit, Element, Font, Length, Padding, Shadow,
    Theme, Vector,
};

use crate::app::{App, Message};
use crate::codec::{OutputFormat, format_size};
use crate::platform::UI_FONT;

impl App {
    pub fn view(&self) -> Element<'_, Message> {
        let header = row![
            text("输入").size(20).font(semibold()).color(text_color()),
            Space::new().width(Length::Fill),
            button(text("载入示例").size(13))
                .style(toolbar_button)
                .padding(Padding::from([7, 14]))
                .on_press(Message::LoadSample),
            button(text("打开文件").size(13))
                .style(toolbar_button)
                .padding(Padding::from([7, 14]))
                .on_press(Message::OpenFile),
            button(text("清空").size(13))
                .style(toolbar_button)
                .padding(Padding::from([7, 14]))
                .on_press(Message::Clear),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let editor = text_editor(&self.input)
            .placeholder("在此粘贴 Base64 字符串，或使用右上角的“载入示例”/“打开文件”")
            .size(13)
            .padding(Padding {
                right: 12.0,
                ..Padding::ZERO
            })
            .min_height(230.0)
            .wrapping(Wrapping::WordOrGlyph)
            .style(editor_style)
            .on_action(Message::InputAction);

        let editor_scroll = scrollable(editor)
            .width(Length::Fill)
            .height(Length::Fixed(230.0))
            .style(scrollable_style);

        let editor_card = container(editor_scroll)
            .width(Length::Fill)
            .padding(Padding::from([14, 16]))
            .style(card_style);

        let input_meta = row![
            text(format!("{} 个字符", format_number(self.chars)))
                .size(12)
                .color(muted()),
            Space::new().width(Length::Fill),
            text("支持 data:image/…;base64 前缀，自动识别 PNG / JPEG / GIF")
                .size(12)
                .color(muted()),
        ]
        .align_y(Alignment::Center);

        let input_section = column![header, editor_card, input_meta].spacing(10);

        let has_image = self.decoded.is_some();

        let preview_content: Element<'_, Message> = match &self.decoded {
            Some(decoded) => container(
                image(decoded.preview.clone())
                    .content_fit(ContentFit::Contain)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
            None => container(text("解码成功后在此显示").size(12).color(faint()))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .into(),
        };

        let preview_header = row![
            text("图片预览")
                .size(15)
                .font(semibold())
                .color(text_color()),
            Space::new().width(Length::Fill),
            button(text("左转 90°").size(12))
                .style(toolbar_button)
                .padding(Padding::from([6, 12]))
                .on_press_maybe(has_image.then_some(Message::RotateLeft)),
            button(text("右转 90°").size(12))
                .style(toolbar_button)
                .padding(Padding::from([6, 12]))
                .on_press_maybe(has_image.then_some(Message::RotateRight)),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let preview = container(
            column![
                preview_header,
                Space::new().height(Length::Fixed(10.0)),
                preview_content,
            ]
            .height(Length::Fill),
        )
        .width(Length::FillPortion(3))
        .height(Length::Fill)
        .padding(18)
        .style(card_style);

        let details = column![
            meta_row(
                "识别类型",
                self.decoded.as_ref().map(|decoded| decoded.kind.to_owned())
            ),
            meta_row(
                "MIME",
                self.decoded.as_ref().map(|decoded| decoded.mime.to_owned())
            ),
            meta_row(
                "大小",
                self.decoded
                    .as_ref()
                    .map(|decoded| format_size(decoded.size))
            ),
        ]
        .spacing(10);

        let filename_field = column![
            text("文件名").size(12).color(muted()),
            text_input("file.png", &self.filename)
                .size(14)
                .padding(Padding::from([9, 12]))
                .style(input_style)
                .on_input(Message::FilenameChanged),
        ]
        .spacing(6);

        let format_field = column![
            text("输出格式").size(12).color(muted()),
            pick_list(OutputFormat::ALL, Some(self.format), Message::FormatChanged)
                .text_size(14)
                .padding(Padding::from([9, 12]))
                .width(Length::Fill)
                .style(pick_list_style),
        ]
        .spacing(6);

        let download_button = button(text("下载").size(14))
            .style(button::primary)
            .padding(Padding::from([11, 20]))
            .width(Length::Fill)
            .on_press_maybe(self.decoded.is_some().then_some(Message::Download));

        let panel = container(
            column![
                text("图片信息")
                    .size(16)
                    .font(semibold())
                    .color(text_color()),
                details,
                divider(),
                filename_field,
                format_field,
                Space::new().height(Length::Fill),
                download_button,
            ]
            .spacing(14)
            .height(Length::Fill),
        )
        .width(Length::Fixed(440.0))
        .height(Length::Fill)
        .padding(Padding::from(18))
        .style(card_style);

        let preview_row = row![preview, panel].spacing(16).height(Length::Fill);

        let status = row![
            text("●")
                .size(9)
                .color(if self.is_error { danger() } else { primary() }),
            text(&self.status)
                .size(13)
                .width(Length::Fill)
                .wrapping(Wrapping::WordOrGlyph)
                .color(if self.is_error { danger() } else { muted() }),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        container(
            column![input_section, preview_row, status]
                .spacing(14)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(22)
        .style(window_style)
        .into()
    }
}

fn meta_row<'a>(label: &'a str, value: Option<String>) -> Element<'a, Message> {
    row![
        text(label).size(13).color(muted()).width(72),
        text(value.unwrap_or_else(|| "—".to_owned()))
            .size(14)
            .color(text_color()),
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .into()
}

fn divider<'a>() -> Element<'a, Message> {
    container(Space::new().height(Length::Fixed(1.0)))
        .width(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(Background::Color(border_color())),
            ..container::Style::default()
        })
        .into()
}

fn text_color() -> Color {
    Color::from_rgb(0.13, 0.17, 0.23)
}

fn muted() -> Color {
    Color::from_rgb(0.44, 0.49, 0.57)
}

fn faint() -> Color {
    Color::from_rgb(0.66, 0.70, 0.76)
}

fn border_color() -> Color {
    Color::from_rgb(0.88, 0.91, 0.95)
}

fn primary() -> Color {
    Color::from_rgb(0.15, 0.39, 0.92)
}

fn danger() -> Color {
    Color::from_rgb(0.83, 0.21, 0.24)
}

fn semibold() -> Font {
    Font {
        weight: iced::font::Weight::Semibold,
        ..Font::with_name(UI_FONT)
    }
}

fn card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::WHITE)),
        border: Border {
            color: border_color(),
            width: 1.0,
            radius: 12.0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.06, 0.09, 0.16, 0.05),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 10.0,
        },
        ..container::Style::default()
    }
}

fn window_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.955, 0.965, 0.98))),
        ..container::Style::default()
    }
}

fn scrollable_style(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let mut style = scrollable::default(theme, status);
    style.vertical_rail.background = None;
    style.vertical_rail.border = Border::default();
    style
}

fn editor_style(_theme: &Theme, _status: text_editor::Status) -> text_editor::Style {
    text_editor::Style {
        background: Background::Color(Color::TRANSPARENT),
        border: Border::default(),
        placeholder: faint(),
        value: text_color(),
        selection: Color::from_rgba(0.15, 0.39, 0.92, 0.22),
    }
}

fn input_style(_theme: &Theme, _status: text_input::Status) -> text_input::Style {
    text_input::Style {
        background: Background::Color(Color::WHITE),
        border: Border {
            color: border_color(),
            width: 1.0,
            radius: 8.0.into(),
        },
        icon: muted(),
        placeholder: faint(),
        value: text_color(),
        selection: Color::from_rgba(0.15, 0.39, 0.92, 0.22),
    }
}

fn pick_list_style(_theme: &Theme, _status: pick_list::Status) -> pick_list::Style {
    pick_list::Style {
        text_color: text_color(),
        placeholder_color: faint(),
        handle_color: muted(),
        background: Background::Color(Color::WHITE),
        border: Border {
            color: border_color(),
            width: 1.0,
            radius: 8.0.into(),
        },
    }
}

fn toolbar_button(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => Color::from_rgb(0.93, 0.95, 0.99),
        button::Status::Pressed => Color::from_rgb(0.88, 0.91, 0.97),
        _ => Color::WHITE,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: match status {
            button::Status::Disabled => faint(),
            _ => text_color(),
        },
        border: Border {
            color: border_color(),
            width: 1.0,
            radius: 8.0.into(),
        },
        shadow: Shadow::default(),
        snap: true,
    }
}

fn format_number(value: usize) -> String {
    let digits = value.to_string();
    let mut formatted = String::with_capacity(digits.len() + digits.len() / 3);

    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            formatted.push(',');
        }
        formatted.push(digit);
    }

    formatted
}
