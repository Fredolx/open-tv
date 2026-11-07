//! System open/save dialogs, resolving to a path or None when cancelled.

use std::{future::Future, path::PathBuf};

use gpui_kit::{App, PathPromptOptions};

pub fn open_file(cx: &mut App, prompt: &str) -> impl Future<Output = Option<String>> + use<> {
    let paths = cx.prompt_for_paths(PathPromptOptions {
        files: true,
        directories: false,
        multiple: false,
        prompt: Some(prompt.to_string().into()),
    });
    async move {
        let path = paths.await.ok()?.ok()??.into_iter().next()?;
        Some(path.to_string_lossy().to_string())
    }
}

pub fn open_folder(cx: &mut App, prompt: &str) -> impl Future<Output = Option<String>> + use<> {
    let paths = cx.prompt_for_paths(PathPromptOptions {
        files: false,
        directories: true,
        multiple: false,
        prompt: Some(prompt.to_string().into()),
    });
    async move {
        let path = paths.await.ok()?.ok()??.into_iter().next()?;
        Some(path.to_string_lossy().to_string())
    }
}

/// Asks where to save `suggested_name`, starting in `directory` (or the home folder).
pub fn save_file(
    cx: &mut App,
    directory: Option<String>,
    suggested_name: &str,
) -> impl Future<Output = Option<String>> + use<> {
    let directory = directory
        .map(PathBuf::from)
        .filter(|d| d.is_dir())
        .or_else(|| std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")).map(PathBuf::from))
        .unwrap_or_default();
    let path = cx.prompt_for_new_path(&directory, Some(suggested_name));
    async move {
        let path = path.await.ok()?.ok()??;
        Some(path.to_string_lossy().to_string())
    }
}
