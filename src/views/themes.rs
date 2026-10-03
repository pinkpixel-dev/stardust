// SPDX-License-Identifier: Apache-2.0

//! The theme library page.

use super::preview;
use crate::app::Message;
use crate::fl;
use crate::library::{Library, SavedTheme};
use cosmic::iced::{Alignment, Length};
use cosmic::prelude::*;
use cosmic::widget::{self, button, tooltip};

const CARD_WIDTH: f32 = 220.0;

pub fn view<'a>(library: &'a Library, active: Option<&str>) -> Element<'a, Message> {
    let spacing = cosmic::theme::spacing();

    if library.themes().is_empty() {
        return empty_state();
    }

    let cards = library
        .themes()
        .iter()
        .map(|theme| card(theme, active == Some(theme.name.as_str())))
        .collect();

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

fn card(theme: &SavedTheme, active: bool) -> Element<'_, Message> {
    let spacing = cosmic::theme::spacing();

    let (mode_icon, mode_label) = if theme.is_dark() {
        ("weather-clear-night-symbolic", fl!("dark"))
    } else {
        ("weather-clear-symbolic", fl!("light"))
    };

    let mut title = widget::row::with_capacity(2)
        .push(widget::text::body(theme.name.as_str()).width(Length::Fill))
        .align_y(Alignment::Center);
    if active {
        title = title.push(widget::icon::from_name("object-select-symbolic").size(16));
    }

    let mode = widget::row::with_capacity(2)
        .push(widget::icon::from_name(mode_icon).size(14))
        .push(widget::text::caption(mode_label))
        .spacing(spacing.space_xxs)
        .align_y(Alignment::Center);

    let content = widget::column::with_capacity(3)
        .push(preview::view(&theme.theme))
        .push(title)
        .push(mode)
        .spacing(spacing.space_xs)
        .padding(spacing.space_xs)
        .width(CARD_WIDTH);

    let card = button::custom_image_button(content, None)
        .class(button::ButtonClass::Image)
        .selected(active)
        .padding(0)
        .on_press(Message::Apply(theme.name.clone()));

    let hint = if active {
        fl!("active-theme")
    } else {
        fl!("apply-theme", name = theme.name.as_str())
    };

    tooltip(card, widget::text::caption(hint), tooltip::Position::Bottom).into()
}
