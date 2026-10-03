// SPDX-License-Identifier: Apache-2.0

//! The icon theme page.

use crate::app::Message;
use crate::fl;
use crate::icons::IconTheme;
use crate::preview_size::PreviewSize;
use cosmic::iced::{Alignment, Length};
use cosmic::prelude::*;
use cosmic::widget::{self, button, tooltip};

/// Card width at a scale of 1.0, matching theme cards.
const CARD_WIDTH: f32 = 220.0;
/// Preview icon size, gap, and padding at a scale of 1.0.
const ICON_SIZE: f32 = 36.0;
const ICON_GAP: f32 = 10.0;
const PREVIEW_PADDING: f32 = 12.0;
const COLUMNS: usize = 4;
const ROWS: usize = 3;

pub fn view<'a>(
    themes: &'a [IconTheme],
    active: Option<&str>,
    size: PreviewSize,
) -> Element<'a, Message> {
    let spacing = cosmic::theme::spacing();

    if themes.is_empty() {
        return empty_state();
    }

    let cards = themes
        .iter()
        .map(|theme| card(theme, active == Some(theme.id.as_str()), size.scale()))
        .collect();

    // Same column wrapper as the theme grid, so cards keep their natural
    // height inside the scrollable.
    let grid = widget::flex_row(cards)
        .spacing(spacing.space_s)
        .width(Length::Fill);

    widget::scrollable(widget::column::with_capacity(1).push(grid))
        .height(Length::Fill)
        .into()
}

fn empty_state<'a>() -> Element<'a, Message> {
    let spacing = cosmic::theme::spacing();

    let content = widget::column::with_capacity(3)
        .push(widget::icon::from_name("folder-pictures-symbolic").size(48))
        .push(widget::text::body(fl!("no-icon-themes")))
        .push(widget::button::suggested(fl!("import-icon-themes")).on_press(Message::ImportFiles))
        .spacing(spacing.space_s)
        .align_x(Alignment::Center);

    widget::container(content).center(Length::Fill).into()
}

fn card(theme: &IconTheme, active: bool, scale: f32) -> Element<'_, Message> {
    let spacing = cosmic::theme::spacing();
    let icon_size = (ICON_SIZE * scale).round() as u16;
    let gap = ICON_GAP * scale;
    let padding = PREVIEW_PADDING * scale;

    let mut grid = widget::column::with_capacity(ROWS).spacing(gap);
    for row in theme.previews.chunks(COLUMNS).take(ROWS) {
        grid = grid.push(
            widget::row::with_children(row.iter().map(|path| {
                widget::icon(widget::icon::from_path(path.clone()))
                    .size(icon_size)
                    .into()
            }))
            .spacing(gap),
        );
    }

    // Fixed height, so a theme missing a few icons still lines up with the rest.
    let grid_height = ROWS as f32 * f32::from(icon_size) + (ROWS - 1) as f32 * gap;
    let preview = widget::container(grid)
        .center_x(Length::Fill)
        .height(grid_height + 2.0 * padding)
        .padding(padding)
        .class(cosmic::theme::Container::Card);

    let mut title = widget::row::with_capacity(2)
        .push(widget::text::body(theme.name.as_str()).width(Length::Fill))
        .align_y(Alignment::Center);
    if active {
        title = title.push(widget::icon::from_name("object-select-symbolic").size(16));
    }

    let mut content = widget::column::with_capacity(3)
        .push(preview)
        .push(title)
        .spacing(spacing.space_xs)
        .padding(spacing.space_xs)
        .width(CARD_WIDTH * scale);
    // COSMIC Settings lists themes by folder name, so show it when it differs.
    if theme.name != theme.id {
        content = content.push(widget::text::caption(theme.id.as_str()));
    }

    let card = button::custom_image_button(content, None)
        .class(button::ButtonClass::Image)
        .selected(active)
        .padding(0)
        .on_press(Message::ApplyIcons(theme.id.clone()));

    let hint = if active {
        fl!("active-icon-theme")
    } else {
        fl!("apply-theme", name = theme.name.as_str())
    };

    tooltip(card, widget::text::caption(hint), tooltip::Position::Bottom).into()
}
