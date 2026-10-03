// SPDX-License-Identifier: Apache-2.0

//! The theme library page.

use crate::app::Message;
use crate::fl;
use crate::library::{Library, SavedTheme};
use cosmic::cosmic_theme::palette::Srgba;
use cosmic::iced::{Alignment, Background, Border, Color, Length};
use cosmic::prelude::*;
use cosmic::widget;

const CARD_WIDTH: f32 = 200.0;

pub fn view(library: &Library) -> Element<'_, Message> {
    let spacing = cosmic::theme::spacing();

    if library.themes().is_empty() {
        return empty_state();
    }

    let cards = library.themes().iter().map(card).collect();

    widget::scrollable(
        widget::flex_row(cards)
            .spacing(spacing.space_s)
            .width(Length::Fill),
    )
    .height(Length::Fill)
    .into()
}

fn empty_state<'a>() -> Element<'a, Message> {
    let spacing = cosmic::theme::spacing();

    let content = widget::column::with_capacity(3)
        .push(widget::icon::from_name("applications-graphics-symbolic").size(48))
        .push(widget::text::body(fl!("no-themes")))
        .push(
            widget::button::suggested(fl!("import-themes"))
                .on_press(Message::ImportFiles),
        )
        .spacing(spacing.space_s)
        .align_x(Alignment::Center);

    widget::container(content).center(Length::Fill).into()
}

fn card(theme: &SavedTheme) -> Element<'_, Message> {
    let spacing = cosmic::theme::spacing();
    let cosmic = &theme.theme;

    let swatches = widget::row::with_capacity(2)
        .push(swatch(cosmic.bg_color(), Length::Fill))
        .push(swatch(cosmic.accent_color(), Length::Fixed(40.0)))
        .spacing(spacing.space_xxs)
        .height(48);

    let (mode_icon, mode_label) = if theme.is_dark() {
        ("weather-clear-night-symbolic", fl!("dark"))
    } else {
        ("weather-clear-symbolic", fl!("light"))
    };

    let mode = widget::row::with_capacity(2)
        .push(widget::icon::from_name(mode_icon).size(14))
        .push(widget::text::caption(mode_label))
        .spacing(spacing.space_xxs)
        .align_y(Alignment::Center);

    let content = widget::column::with_capacity(3)
        .push(swatches)
        .push(widget::text::body(theme.name.as_str()))
        .push(mode)
        .spacing(spacing.space_xs);

    widget::container(content)
        .padding(spacing.space_s)
        .width(CARD_WIDTH)
        .class(cosmic::theme::Container::Card)
        .into()
}

fn swatch<'a>(color: Srgba, width: Length) -> Element<'a, Message> {
    let color = Color::from_rgba(color.red, color.green, color.blue, color.alpha);

    widget::container(widget::Space::new().width(width).height(Length::Fill))
        .width(width)
        .height(Length::Fill)
        .style(move |_| widget::container::Style {
            background: Some(Background::Color(color)),
            border: Border {
                radius: 8.0.into(),
                width: 1.0,
                color: Color::from_rgba(0.5, 0.5, 0.5, 0.35),
            },
            ..Default::default()
        })
        .into()
}
