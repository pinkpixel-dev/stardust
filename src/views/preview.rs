// SPDX-License-Identifier: Apache-2.0

//! A tiny COSMIC window drawn from a theme's real colors and corner radii.

use cosmic::cosmic_theme::Theme;
use cosmic::cosmic_theme::palette::{Srgb, Srgba};
use cosmic::iced::{Background, Border, Color, Length};
use cosmic::prelude::*;
use cosmic::widget;

/// Height at the small size. Everything else scales from the `scale` argument.
pub const HEIGHT: f32 = 116.0;

/// Previews are roughly a third of real size, so radii shrink with them.
const SCALE: f32 = 0.4;

pub fn view<'a, Message: 'a>(theme: &Theme, scale: f32) -> Element<'a, Message> {
    let s = scale;
    let radius_window = theme.corner_radii.radius_m[0] * SCALE * s;
    let radius_container = theme.corner_radii.radius_s[0] * SCALE * s;
    let radius_button = theme.corner_radii.radius_xl[0] * SCALE * s;

    let bg_on = theme.on_bg_color();
    let primary_on = theme.on_primary_container_color();
    let secondary_on = theme.on_secondary_container_color();
    let accent = theme.accent.base;

    // Header: window title.
    let header = widget::row::with_capacity(1)
        .push(bar(bg_on, 0.55, Length::Fixed(44.0 * s), 5.0 * s))
        .height(10.0 * s);

    // Sidebar: one selected item using the accent, two plain items.
    let nav_item = |selected: bool| {
        let label = bar(secondary_on, 0.5, Length::Fill, 4.0 * s);
        let item = widget::container(label)
            .padding([4.0 * s, 5.0 * s])
            .width(Length::Fill);
        if selected {
            filled(item, with_alpha(accent, 0.25), radius_container)
        } else {
            item.into()
        }
    };
    let nav = filled(
        widget::column::with_capacity(3)
            .push(nav_item(true))
            .push(nav_item(false))
            .push(nav_item(false))
            .spacing(2.0 * s)
            .padding(4.0 * s)
            .width(Length::FillPortion(2))
            .height(Length::Fill),
        theme.secondary_container_color(),
        radius_container,
    );

    // Content: two lines of text and an accent button.
    let accent_button = filled(
        widget::container(bar(
            theme.accent_button.on,
            0.9,
            Length::Fixed(18.0 * s),
            3.0 * s,
        ))
        .padding([4.0 * s, 7.0 * s]),
        accent,
        radius_button,
    );
    let content = filled(
        widget::column::with_capacity(4)
            .push(bar(primary_on, 0.85, Length::Fixed(52.0 * s), 5.0 * s))
            .push(bar(primary_on, 0.45, Length::Fill, 4.0 * s))
            .push(bar(primary_on, 0.45, Length::Fixed(40.0 * s), 4.0 * s))
            .push(widget::Space::new().height(Length::Fill))
            .push(accent_button)
            .spacing(5.0 * s)
            .padding(7.0 * s)
            .width(Length::FillPortion(5))
            .height(Length::Fill),
        theme.primary_container_color(),
        radius_container,
    );

    let body = widget::row::with_capacity(2)
        .push(nav)
        .push(content)
        .spacing(4.0 * s)
        .height(Length::Fill);

    let window = widget::column::with_capacity(2)
        .push(header)
        .push(body)
        .spacing(5.0 * s)
        .padding(6.0 * s);

    // The border is the active window hint, same as on the desktop.
    let hint = theme.window_hint.map_or(accent, srgb_to_srgba);
    let background = to_color(theme.bg_color());
    widget::container(window)
        .width(Length::Fill)
        .height(HEIGHT * s)
        .style(move |_| widget::container::Style {
            background: Some(Background::Color(background)),
            border: Border {
                radius: radius_window.into(),
                width: 2.0,
                color: to_color(hint),
            },
            ..Default::default()
        })
        .into()
}

/// A rounded bar standing in for a line of text.
fn bar<'a, Message: 'a>(color: Srgba, alpha: f32, width: Length, height: f32) -> Element<'a, Message> {
    let color = to_color(with_alpha(color, alpha));
    widget::container(widget::Space::new())
        .width(width)
        .height(height)
        .style(move |_| widget::container::Style {
            background: Some(Background::Color(color)),
            border: Border {
                radius: (height / 2.0).into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

fn filled<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    color: Srgba,
    radius: f32,
) -> Element<'a, Message> {
    let color = to_color(color);
    widget::container(content)
        .style(move |_| widget::container::Style {
            background: Some(Background::Color(color)),
            border: Border {
                radius: radius.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

fn with_alpha(color: Srgba, alpha: f32) -> Srgba {
    Srgba::new(color.red, color.green, color.blue, color.alpha * alpha)
}

fn srgb_to_srgba(color: Srgb) -> Srgba {
    Srgba::new(color.red, color.green, color.blue, 1.0)
}

pub fn to_color(color: Srgba) -> Color {
    Color::from_rgba(color.red, color.green, color.blue, color.alpha)
}
