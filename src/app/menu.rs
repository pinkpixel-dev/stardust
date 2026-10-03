// SPDX-License-Identifier: Apache-2.0

//! The header menu bar and keyboard shortcuts.

use super::{AppModel, ContextPage, Message, Page};
use crate::fl;
use crate::preview_size::PreviewSize;
use cosmic::iced::keyboard::Key;
use cosmic::prelude::*;
use cosmic::widget::menu::{self, KeyBind, key_bind::Modifier};
use std::collections::HashMap;

pub fn bar(app: &AppModel) -> Element<'_, Message> {
    // File > Import works on whichever page is open.
    let (import_files, import_folder) = match app.page() {
        Page::Themes => (fl!("import-themes"), fl!("import-folder")),
        Page::Icons => (fl!("import-icon-themes"), fl!("import-icon-folder")),
    };

    let menu_bar = menu::bar(vec![
        menu::Tree::with_children(
            menu::root(fl!("file")).apply(Element::from),
            menu::items(
                &app.key_binds,
                vec![
                    menu::Item::Button(import_files, None, MenuAction::ImportFiles),
                    menu::Item::Button(import_folder, None, MenuAction::ImportFolder),
                ],
            ),
        ),
        menu::Tree::with_children(
            menu::root(fl!("view")).apply(Element::from),
            menu::items(
                &app.key_binds,
                vec![
                    size_item(fl!("size-small"), PreviewSize::Small, app.preview_size),
                    size_item(fl!("size-medium"), PreviewSize::Medium, app.preview_size),
                    size_item(fl!("size-large"), PreviewSize::Large, app.preview_size),
                    menu::Item::Divider,
                    menu::Item::Button(fl!("about"), None, MenuAction::About),
                ],
            ),
        ),
    ])
    // The default 150px clips labels plus shortcuts, especially in wider fonts.
    .item_width(menu::ItemWidth::Uniform(260));

    menu_bar.into()
}

pub fn key_binds() -> HashMap<KeyBind, MenuAction> {
    HashMap::from([
        (
            KeyBind {
                modifiers: vec![Modifier::Ctrl],
                key: Key::Character("o".into()),
            },
            MenuAction::ImportFiles,
        ),
        (
            KeyBind {
                modifiers: vec![Modifier::Ctrl, Modifier::Shift],
                key: Key::Character("o".into()),
            },
            MenuAction::ImportFolder,
        ),
        (
            KeyBind {
                modifiers: vec![Modifier::Ctrl],
                key: Key::Character("=".into()),
            },
            MenuAction::GrowPreviews,
        ),
        (
            KeyBind {
                modifiers: vec![Modifier::Ctrl],
                key: Key::Character("-".into()),
            },
            MenuAction::ShrinkPreviews,
        ),
    ])
}

fn size_item(
    label: String,
    size: PreviewSize,
    current: PreviewSize,
) -> menu::Item<MenuAction, String> {
    menu::Item::CheckBox(label, None, size == current, MenuAction::PreviewSize(size))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
    ImportFiles,
    ImportFolder,
    PreviewSize(PreviewSize),
    GrowPreviews,
    ShrinkPreviews,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
            MenuAction::ImportFiles => Message::ImportFiles,
            MenuAction::ImportFolder => Message::ImportFolder,
            MenuAction::PreviewSize(size) => Message::SetPreviewSize(*size),
            MenuAction::GrowPreviews => Message::GrowPreviews,
            MenuAction::ShrinkPreviews => Message::ShrinkPreviews,
        }
    }
}
