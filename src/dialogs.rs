// SPDX-License-Identifier: Apache-2.0

//! File picker dialogs and running slow work off the UI thread.

use crate::app::Message;
use cosmic::dialog::file_chooser::{self, FileFilter};
use cosmic::iced::futures::channel::oneshot;
use cosmic::prelude::*;
use std::path::{Path, PathBuf};

/// Picks one or more files. Cancelling sends an empty list.
pub fn pick_files(
    title: String,
    filter: FileFilter,
    on_chosen: fn(Vec<PathBuf>) -> Message,
) -> Task<cosmic::Action<Message>> {
    cosmic::task::future(async move {
        let dialog = file_chooser::open::Dialog::new()
            .title(title)
            .filter(filter);

        match dialog.open_files().await {
            Ok(response) => on_chosen(
                response
                    .urls()
                    .iter()
                    .filter_map(|url| url.to_file_path().ok())
                    .collect(),
            ),
            Err(file_chooser::Error::Cancelled) => on_chosen(Vec::new()),
            Err(why) => Message::DialogFailed(why.to_string()),
        }
    })
}

/// Picks a folder. Cancelling does nothing.
pub fn pick_folder(
    title: String,
    on_chosen: fn(PathBuf) -> Message,
) -> Task<cosmic::Action<Message>> {
    cosmic::task::future(async move {
        let dialog = file_chooser::open::Dialog::new().title(title);

        match dialog.open_folder().await {
            Ok(response) => match response.url().to_file_path() {
                Ok(path) => on_chosen(path),
                Err(()) => Message::DialogFailed("not a local folder".to_string()),
            },
            Err(file_chooser::Error::Cancelled) => Message::FilesChosen(Vec::new()),
            Err(why) => Message::DialogFailed(why.to_string()),
        }
    })
}

/// Runs blocking work (scanning folders, copying files) on its own thread.
pub fn background<T: Send + 'static>(
    work: impl FnOnce() -> T + Send + 'static,
    done: fn(T) -> Message,
) -> Task<cosmic::Action<Message>> {
    let (sender, receiver) = oneshot::channel();
    std::thread::spawn(move || {
        let _ = sender.send(work());
    });
    cosmic::task::future(async move {
        match receiver.await {
            Ok(result) => done(result),
            // The worker panicked, which already printed why.
            Err(_) => Message::TaskFailed,
        }
    })
}

pub fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}
