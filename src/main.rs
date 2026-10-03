// SPDX-License-Identifier: Apache-2.0

mod app;
mod compat;
mod desktop;
mod i18n;
mod library;
mod preview_size;
mod views;

fn main() -> cosmic::iced::Result {
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();
    i18n::init(&requested_languages);

    let settings = cosmic::app::Settings::default()
        .size(cosmic::iced::Size::new(1024.0, 720.0))
        .size_limits(
            cosmic::iced::Limits::NONE
                .min_width(360.0)
                .min_height(320.0),
        );

    cosmic::app::run::<app::AppModel>(settings, ())
}
