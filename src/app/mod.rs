// SPDX-License-Identifier: Apache-2.0

mod icon_page;
mod menu;

use crate::compat::{self, Compat};
use crate::dialogs::{self, file_name};
use crate::icons::{self, IconTheme, InstallReport};
use crate::library::{self, Library};
use crate::preview_size::PreviewSize;
use crate::{desktop, fl, views};
use cosmic::app::context_drawer;
use cosmic::config::CosmicTk;
use cosmic::cosmic_theme::{
    DARK_THEME_BUILDER_ID, LIGHT_THEME_BUILDER_ID, THEME_MODE_ID, ThemeBuilder, ThemeMode,
};
use cosmic::dialog::file_chooser::FileFilter;
use cosmic::iced::keyboard::{self, Key, Modifiers, key::Physical};
use cosmic::iced::{Length, Subscription, event};
use cosmic::prelude::*;
use cosmic::widget::menu::{KeyBind, action::MenuAction as _};
use cosmic::widget::{self, about::About, nav_bar, toaster};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../../resources/icons/hicolor/scalable/apps/icon.svg");

pub struct AppModel {
    core: cosmic::Core,
    context_page: ContextPage,
    about: About,
    key_binds: HashMap<KeyBind, menu::MenuAction>,
    nav: nav_bar::Model,
    library: Option<Library>,
    library_error: Option<String>,
    /// Name of the saved theme that matches what the desktop shows right now.
    active: Option<String>,
    compat: Compat,
    compat_dismissed: bool,
    preview_size: PreviewSize,
    /// `None` until the first scan finishes.
    icon_themes: Option<Vec<IconTheme>>,
    /// Folder name of the icon theme COSMIC is set to.
    active_icons: Option<String>,
    /// Icon themes imported since Stardust started. Apps that were already
    /// running can't see them until they restart.
    imported_icons: HashSet<String>,
    toasts: toaster::Toasts<Message>,
}

#[derive(Debug, Clone)]
pub enum Message {
    LaunchUrl(String),
    ToggleContextPage(ContextPage),
    ImportFiles,
    ImportFolder,
    FilesChosen(Vec<PathBuf>),
    FolderChosen(PathBuf),
    DialogFailed(String),
    CloseToast(toaster::ToastId),
    DismissCompat,
    SetPreviewSize(PreviewSize),
    GrowPreviews,
    ShrinkPreviews,
    Apply(String),
    DesktopThemeChanged,
    IconThemesLoaded(Vec<IconTheme>),
    ApplyIcons(String),
    IconThemeChanged,
    IconArchivesChosen(Vec<PathBuf>),
    IconFolderChosen(PathBuf),
    IconsInstalled(InstallReport),
    TaskFailed,
    Key(Modifiers, Key, Option<Physical>),
}

impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "dev.pinkpixel.Stardust";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let about = About::default()
            .name(fl!("app-title"))
            .icon(widget::icon::from_svg_bytes(APP_ICON))
            .version(env!("CARGO_PKG_VERSION"))
            .author("Pink Pixel")
            .links([(fl!("repository"), REPOSITORY)])
            .license(env!("CARGO_PKG_LICENSE"));

        let (library, library_error) = open_library();

        let mut app = AppModel {
            core,
            context_page: ContextPage::default(),
            about,
            key_binds: menu::key_binds(),
            nav: icon_page::nav_model(),
            library,
            library_error,
            active: None,
            compat: compat::check(),
            compat_dismissed: false,
            preview_size: PreviewSize::load(Self::APP_ID),
            icon_themes: None,
            active_icons: icons::current().ok(),
            imported_icons: HashSet::new(),
            toasts: toaster::Toasts::new(Message::CloseToast),
        };

        app.refresh_active();
        let command = Task::batch([app.update_title(), icon_page::scan_icon_themes()]);
        (app, command)
    }

    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<cosmic::Action<Self::Message>> {
        self.nav.activate(id);
        Task::none()
    }

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        vec![menu::bar(self)]
    }

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => context_drawer::about(
                &self.about,
                |url| Message::LaunchUrl(url.to_string()),
                Message::ToggleContextPage(ContextPage::About),
            ),
        })
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let spacing = cosmic::theme::spacing();
        let mut page = widget::column::with_capacity(4)
            .spacing(spacing.space_m)
            .padding(spacing.space_m)
            .width(Length::Fill)
            .height(Length::Fill);

        if self.page() == Page::Icons {
            page = page.push(widget::text::title3(fl!("icons")));
            if let Some(themes) = &self.icon_themes {
                page = page.push(views::icons::view(
                    themes,
                    self.active_icons.as_deref(),
                    self.preview_size,
                ));
            }
            return toaster::toaster(&self.toasts, page);
        }

        // The version warning is about color themes, so it stays on this page.
        if let Some(warning) = self.compat_warning() {
            page = page.push(
                widget::warning(warning)
                    .on_close(Message::DismissCompat)
                    .into_widget(),
            );
        }

        page = page.push(widget::text::title3(fl!("themes")));

        page = match (&self.library, &self.library_error) {
            (Some(library), _) => page.push(views::themes::view(
                library,
                self.active.as_deref(),
                self.preview_size,
            )),
            (None, Some(error)) => page.push(widget::text::body(error.as_str())),
            (None, None) => page,
        };

        toaster::toaster(&self.toasts, page)
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        // Theme changes made anywhere (Settings, another app, Stardust itself)
        // update the checkmark.
        let watch_dark = self
            .core()
            .watch_config::<ThemeBuilder>(DARK_THEME_BUILDER_ID)
            .map(|_| Message::DesktopThemeChanged);
        let watch_light = self
            .core()
            .watch_config::<ThemeBuilder>(LIGHT_THEME_BUILDER_ID)
            .map(|_| Message::DesktopThemeChanged);
        let watch_mode = self
            .core()
            .watch_config::<ThemeMode>(THEME_MODE_ID)
            .map(|_| Message::DesktopThemeChanged);
        let watch_icons = self
            .core()
            .watch_config::<CosmicTk>(cosmic::config::ID)
            .map(|_| Message::IconThemeChanged);

        let keys = event::listen_with(|event, status, _id| match event {
            cosmic::iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                modifiers,
                physical_key,
                ..
            }) if status == event::Status::Ignored => {
                Some(Message::Key(modifiers, key, Some(physical_key)))
            }
            _ => None,
        });

        Subscription::batch([watch_dark, watch_light, watch_mode, watch_icons, keys])
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
            }

            Message::LaunchUrl(url) => {
                if let Err(err) = open::that_detached(&url) {
                    eprintln!("failed to open {url:?}: {err}");
                }
            }

            Message::ImportFiles => {
                return match self.page() {
                    Page::Themes => dialogs::pick_files(
                        fl!("import-themes"),
                        FileFilter::new("COSMIC themes").glob("*.ron"),
                        Message::FilesChosen,
                    ),
                    Page::Icons => dialogs::pick_files(
                        fl!("import-icon-themes"),
                        icon_page::icon_archive_filter(),
                        Message::IconArchivesChosen,
                    ),
                };
            }
            Message::ImportFolder => {
                return match self.page() {
                    Page::Themes => {
                        dialogs::pick_folder(fl!("import-folder"), Message::FolderChosen)
                    }
                    Page::Icons => {
                        dialogs::pick_folder(fl!("import-icon-folder"), Message::IconFolderChosen)
                    }
                };
            }

            Message::FilesChosen(paths) => {
                let Some(library) = self.library.as_mut() else {
                    return Task::none();
                };
                if paths.is_empty() {
                    return Task::none();
                }
                let mut imported = 0;
                let mut errors = Vec::new();
                for path in &paths {
                    match library.import_file(path) {
                        Ok(_) => imported += 1,
                        Err(err) => errors.push((path, err)),
                    }
                }

                // A single bad file gets the actual reason, not just a count.
                if let [(path, err)] = errors.as_slice()
                    && imported == 0
                {
                    return self.toast(fl!(
                        "import-failed",
                        name = file_name(path),
                        error = err.to_string()
                    ));
                }
                return self.toast(fl!("imported", imported = imported, failed = errors.len()));
            }

            Message::FolderChosen(dir) => {
                let Some(library) = self.library.as_mut() else {
                    return Task::none();
                };
                return match library.import_dir(&dir) {
                    Ok(report) => self.toast(fl!(
                        "imported",
                        imported = report.imported,
                        failed = report.failed
                    )),
                    Err(err) => self.toast(fl!(
                        "import-failed",
                        name = file_name(&dir),
                        error = err.to_string()
                    )),
                };
            }

            Message::DialogFailed(error) => {
                return self.toast(fl!("dialog-failed", error = error));
            }

            Message::CloseToast(id) => self.toasts.remove(id),

            Message::DismissCompat => self.compat_dismissed = true,

            Message::SetPreviewSize(size) => self.set_preview_size(size),
            Message::GrowPreviews => self.set_preview_size(self.preview_size.larger()),
            Message::ShrinkPreviews => self.set_preview_size(self.preview_size.smaller()),

            Message::Apply(name) => return self.apply(&name),

            Message::DesktopThemeChanged => self.refresh_active(),

            Message::IconThemesLoaded(_)
            | Message::ApplyIcons(_)
            | Message::IconThemeChanged
            | Message::IconArchivesChosen(_)
            | Message::IconFolderChosen(_)
            | Message::IconsInstalled(_) => return self.update_icons(message),

            Message::TaskFailed => return self.toast(fl!("task-failed")),

            Message::Key(modifiers, key, physical_key) => {
                for (key_bind, action) in &self.key_binds {
                    if key_bind.matches(modifiers, &key, physical_key.as_ref()) {
                        return self.update(action.message());
                    }
                }
            }
        }
        Task::none()
    }
}

impl AppModel {
    fn page(&self) -> Page {
        self.nav.active_data::<Page>().copied().unwrap_or_default()
    }

    fn set_preview_size(&mut self, size: PreviewSize) {
        if size != self.preview_size {
            self.preview_size = size;
            size.save(<Self as cosmic::Application>::APP_ID);
        }
    }

    pub fn update_title(&mut self) -> Task<cosmic::Action<Message>> {
        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(fl!("app-title"), id)
        } else {
            Task::none()
        }
    }

    fn apply(&mut self, name: &str) -> Task<cosmic::Action<Message>> {
        // Writing to a config version the desktop doesn't read would do nothing.
        if let Some(warning) = self.version_mismatch() {
            return self.toast(warning);
        }

        let Some(theme) = self
            .library
            .as_ref()
            .and_then(|library| library.themes().iter().find(|theme| theme.name == name))
        else {
            return Task::none();
        };

        match desktop::apply(&theme.builder) {
            Ok(()) => {
                self.active = Some(name.to_string());
                self.toast(fl!("applied", name = name))
            }
            Err(err) => self.toast(fl!("apply-failed", name = name, error = format!("{err:?}"))),
        }
    }

    fn refresh_active(&mut self) {
        let current = desktop::current_theme().ok();
        self.active = self.library.as_ref().and_then(|library| {
            library
                .themes()
                .iter()
                .find(|theme| {
                    current
                        .as_ref()
                        .is_some_and(|current| library::same_theme(&theme.builder, current))
                })
                .map(|theme| theme.name.clone())
        });
    }

    fn toast(&mut self, message: String) -> Task<cosmic::Action<Message>> {
        self.toasts
            .push(toaster::Toast::new(message))
            .map(cosmic::Action::App)
    }

    fn compat_warning(&self) -> Option<String> {
        if self.compat_dismissed {
            return None;
        }
        self.version_mismatch()
    }

    fn version_mismatch(&self) -> Option<String> {
        match self.compat {
            Compat::DesktopNewer { desktop, app } => {
                Some(fl!("compat-newer", desktop = desktop, app = app))
            }
            Compat::DesktopOlder { desktop, app } => {
                Some(fl!("compat-older", desktop = desktop, app = app))
            }
            Compat::Match | Compat::Unknown => None,
        }
    }
}

/// Opens the library. On first run, saves the theme the desktop is using right
/// now so the library isn't empty.
fn open_library() -> (Option<Library>, Option<String>) {
    let Some(dir) = Library::default_dir() else {
        return (None, Some(fl!("library-failed", error = "no data folder")));
    };

    match Library::open(dir) {
        Ok((mut library, created)) => {
            if created && library.themes().is_empty() {
                match desktop::current_theme() {
                    Ok(builder) => {
                        if let Err(err) = library.save(&fl!("my-theme"), builder) {
                            eprintln!("failed to save current theme: {err}");
                        }
                    }
                    Err(err) => eprintln!("failed to read current theme: {err:?}"),
                }
            }
            (Some(library), None)
        }
        Err(err) => (None, Some(fl!("library-failed", error = err.to_string()))),
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum Page {
    #[default]
    Themes,
    Icons,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
}
