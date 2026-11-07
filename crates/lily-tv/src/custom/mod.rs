//! Custom sources: hand-made channels and groups, and sharing them as files.

mod channel_editor;
mod group_editor;

use std::rc::Rc;

use gpui_kit::component::{WindowExt as _, button::ButtonVariant, notification::Notification};
use gpui_kit::*;
use lily_core::{
    anyhow::Result,
    media_type, share, sql,
    types::{Channel, Source},
};

pub use channel_editor::ChannelEditor;
pub use group_editor::GroupEditor;

use crate::{
    backend::Backend,
    file_prompts,
    naming::{CHANNEL_EXTENSION, GROUP_EXTENSION, SOURCE_EXTENSION, sanitize_file_name},
};

pub enum EditorEvent {
    Saved,
    Cancelled,
}

/// Called on the UI thread once a change has been saved.
pub type Done = Rc<dyn Fn(&mut Window, &mut App)>;

/// Shows `editor` in a dialog that closes when it saves or cancels; `on_saved` runs after a save.
fn open_editor<E>(title: &'static str, editor: Entity<E>, on_saved: Done, window: &mut Window, cx: &mut App)
where
    E: Render + EventEmitter<EditorEvent>,
{
    window
        .subscribe(&editor, cx, move |_, event: &EditorEvent, window, cx| {
            window.close_dialog(cx);
            if matches!(event, EditorEvent::Saved) {
                window.push_notification(Notification::success("Saved."), cx);
                on_saved(window, cx);
            }
        })
        .detach();
    window.open_dialog(cx, move |dialog, _, _| dialog.title(title).w(px(640.)).child(editor.clone()));
}

pub fn add_channel(source_id: i64, on_saved: Done, window: &mut Window, cx: &mut App) {
    let editor = cx.new(|cx| ChannelEditor::add(source_id, window, cx));
    open_editor("Add channel", editor, on_saved, window, cx);
}

pub fn add_group(source_id: i64, on_saved: Done, window: &mut Window, cx: &mut App) {
    let editor = cx.new(|cx| GroupEditor::add(source_id, window, cx));
    open_editor("Add group", editor, on_saved, window, cx);
}

/// Edits a custom channel or group from the results list.
pub fn edit(row: Channel, on_saved: Done, window: &mut Window, cx: &mut App) {
    if row.media_type == media_type::GROUP {
        let editor = cx.new(|cx| GroupEditor::edit(&row, window, cx));
        open_editor("Edit group", editor, on_saved, window, cx);
    } else {
        let editor = cx.new(|cx| ChannelEditor::edit(row, window, cx));
        open_editor("Edit channel", editor, on_saved, window, cx);
    }
}

/// Asks before deleting a custom channel or group. A group's channels stay, without a group.
pub fn confirm_delete(row: Channel, on_deleted: Done, window: &mut Window, cx: &mut App) {
    let Some(id) = row.id else {
        return;
    };
    let is_group = row.media_type == media_type::GROUP;
    let description = if is_group {
        "Channels in this group are kept and moved out of it."
    } else {
        "This removes the channel from its custom source."
    };
    let name = row.name.clone();
    window.open_alert_dialog(cx, move |alert, _, _| {
        let on_deleted = on_deleted.clone();
        alert
            .title(format!("Delete “{name}”?"))
            .description(description)
            .confirm()
            .ok_text("Delete")
            .ok_variant(ButtonVariant::Danger)
            .on_ok(move |_, window, cx| {
                let on_deleted = on_deleted.clone();
                run(
                    move || {
                        if is_group {
                            sql::delete_custom_group(id, None, true)
                        } else {
                            sql::delete_custom_channel(id)
                        }
                    },
                    "Deleted.",
                    "Couldn't delete",
                    Some(on_deleted),
                    window,
                    cx,
                );
                true
            })
    });
}

/// Saves a custom channel or group to a file others can import.
pub fn share_row(row: Channel, window: &mut Window, cx: &mut App) {
    let is_group = row.media_type == media_type::GROUP;
    let ext = if is_group { GROUP_EXTENSION } else { CHANNEL_EXTENSION };
    let name = format!("{}.{ext}", sanitize_file_name(&row.name));
    export(name, move |path| {
        if is_group {
            share::share_custom_group(row, path)
        } else {
            share::share_custom_channel(row, path)
        }
    }, window, cx);
}

/// Saves a whole custom source to a file others can import.
pub fn share_source(source: Source, window: &mut Window, cx: &mut App) {
    let name = format!("{}.{SOURCE_EXTENSION}", sanitize_file_name(&source.name));
    export(name, move |path| share::share_custom_source(source, path), window, cx);
}

/// Imports a shared channel, group or source file. `into` None creates a new source.
pub fn import(into: Option<i64>, name_override: Option<String>, on_done: Done, window: &mut Window, cx: &mut App) {
    let picked = file_prompts::open_file(cx, "Choose a Lily TV export (.otv, .otvg, .otvp)");
    window
        .spawn(cx, async move |cx| {
            let Some(path) = picked.await else {
                return;
            };
            _ = cx.update(|window, cx| {
                run(
                    move || share::import(path, into, name_override),
                    "Imported.",
                    "Couldn't import",
                    Some(on_done),
                    window,
                    cx,
                )
            });
        })
        .detach();
}

fn export<F>(suggested_name: String, write: F, window: &mut Window, cx: &mut App)
where
    F: FnOnce(String) -> Result<()> + Send + 'static,
{
    let picked = file_prompts::save_file(cx, None, &suggested_name);
    window
        .spawn(cx, async move |cx| {
            let Some(path) = picked.await else {
                return;
            };
            let message = format!("Saved to {path}");
            _ = cx.update(|window, cx| run(move || write(path), message, "Couldn't export", None, window, cx));
        })
        .detach();
}

/// Runs a blocking backend change and reports the outcome as a notification.
fn run<F>(
    op: F,
    success: impl Into<SharedString>,
    failure: &'static str,
    then: Option<Done>,
    window: &mut Window,
    cx: &mut App,
) where
    F: FnOnce() -> Result<()> + Send + 'static,
{
    let success = success.into();
    let request = Backend::blocking(cx, op);
    window
        .spawn(cx, async move |cx| {
            let result = request.await;
            _ = cx.update(|window, cx| {
                match result {
                    Ok(()) => {
                        window.push_notification(Notification::success(success), cx);
                        if let Some(then) = then {
                            then(window, cx);
                        }
                    }
                    Err(e) => window.push_notification(Notification::error(format!("{e:#}")).title(failure), cx),
                }
            });
        })
        .detach();
}

/// Creates an empty custom source.
pub fn create_source(name: String) -> Result<()> {
    sql::do_tx(|tx| sql::create_or_find_source_by_name(tx, &sql::get_custom_source(name)).map(|_| ()))
}
