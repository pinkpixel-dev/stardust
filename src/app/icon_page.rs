// SPDX-License-Identifier: Apache-2.0

//! The Icons page: loading, applying, and importing icon themes.

use super::{AppModel, Message, Page};
use crate::dialogs::{self, file_name};
use crate::fl;
use crate::icons::{self, InstallReport};
use cosmic::dialog::file_chooser::FileFilter;
use cosmic::prelude::*;
use cosmic::widget::{self, nav_bar};
use std::path::Path;

impl AppModel {
    pub(super) fn update_icons(&mut self, message: Message) -> Task<cosmic::Action<Message>> {
        match message {
            Message::IconThemesLoaded(themes) => self.icon_themes = Some(themes),

            Message::ApplyIcons(id) => return self.apply_icons(&id),

            Message::IconThemeChanged => self.active_icons = icons::current().ok(),

            Message::IconArchivesChosen(paths) => {
                if paths.is_empty() {
                    return Task::none();
                }
                return install_icons(move |dest| {
                    let mut report = InstallReport::default();
                    for path in &paths {
                        match icons::install_archive(path, dest) {
                            Ok(one) => {
                                report.installed.extend(one.installed);
                                report.failed.extend(one.failed);
                            }
                            Err(err) => report.failed.push((file_name(path), err.to_string())),
                        }
                    }
                    report
                });
            }

            Message::IconFolderChosen(dir) => {
                return install_icons(move |dest| {
                    icons::install_dir(&dir, dest).unwrap_or_else(|err| InstallReport {
                        failed: vec![(file_name(&dir), err.to_string())],
                        ..Default::default()
                    })
                });
            }

            Message::IconsInstalled(report) => {
                self.imported_icons.extend(report.installed.iter().cloned());
                let toast = match report.failed.as_slice() {
                    [(name, error)] if report.installed.is_empty() => {
                        fl!(
                            "import-failed",
                            name = name.as_str(),
                            error = error.as_str()
                        )
                    }
                    _ => fl!(
                        "icons-installed",
                        installed = report.installed.len(),
                        failed = report.failed.len()
                    ),
                };
                return Task::batch([self.toast(toast), scan_icon_themes()]);
            }
            _ => {}
        }
        Task::none()
    }

    fn apply_icons(&mut self, id: &str) -> Task<cosmic::Action<Message>> {
        let name = self
            .icon_themes
            .iter()
            .flatten()
            .find(|theme| theme.id == id)
            .map_or(id, |theme| theme.name.as_str())
            .to_string();

        match icons::apply(id) {
            Ok(()) => {
                self.active_icons = Some(id.to_string());
                let message = if self.imported_icons.contains(id) {
                    fl!("icons-applied-restart", name = name)
                } else {
                    fl!("applied", name = name)
                };
                self.toast(message)
            }
            Err(err) => self.toast(fl!("apply-failed", name = name, error = format!("{err:?}"))),
        }
    }
}

pub(super) fn nav_model() -> nav_bar::Model {
    let mut nav = nav_bar::Model::default();
    nav.insert()
        .text(fl!("themes"))
        .icon(widget::icon::from_name("applications-graphics-symbolic"))
        .data(Page::Themes)
        .activate();
    nav.insert()
        .text(fl!("icons"))
        .icon(widget::icon::from_name("folder-pictures-symbolic"))
        .data(Page::Icons);
    nav
}

pub(super) fn scan_icon_themes() -> Task<cosmic::Action<Message>> {
    dialogs::background(
        || icons::scan(&icons::base_dirs()),
        Message::IconThemesLoaded,
    )
}

fn install_icons(
    work: impl FnOnce(&Path) -> InstallReport + Send + 'static,
) -> Task<cosmic::Action<Message>> {
    dialogs::background(
        move || match icons::user_dir() {
            Some(dest) => work(&dest),
            None => InstallReport {
                failed: vec![("icons".to_string(), "no data folder".to_string())],
                ..Default::default()
            },
        },
        Message::IconsInstalled,
    )
}

/// Icon themes are usually shared as tarballs. `tar` handles all of these.
pub(super) fn icon_archive_filter() -> FileFilter {
    [
        "*.tar",
        "*.tar.gz",
        "*.tgz",
        "*.tar.xz",
        "*.txz",
        "*.tar.bz2",
        "*.tar.zst",
    ]
    .into_iter()
    .fold(FileFilter::new("Icon theme archives"), FileFilter::glob)
}
