use gpui_kit::component::{
    ActiveTheme as _, Disableable as _,
    button::{Button, ButtonVariants as _},
    form::{Field, Form},
    h_flex,
    input::{Input, InputState},
    v_flex,
};
use gpui_kit::{prelude::FluentBuilder as _, *};
use lily_core::{anyhow::bail, sql, types::Group};

use super::EditorEvent;
use crate::backend::Backend;

/// Adds a group to a custom source, or renames one.
pub struct GroupEditor {
    /// (id, original name) when editing
    existing: Option<(i64, String)>,
    source_id: i64,
    name: Entity<InputState>,
    image: Entity<InputState>,
    saving: bool,
    error: Option<SharedString>,
}

impl EventEmitter<EditorEvent> for GroupEditor {}

impl GroupEditor {
    pub fn add(source_id: i64, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self::build(None, source_id, String::new(), String::new(), window, cx)
    }

    /// `group` is the group's row from the results list.
    pub fn edit(group: &lily_core::types::Channel, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self::build(
            group.id.map(|id| (id, group.name.clone())),
            group.source_id.unwrap_or_default(),
            group.name.clone(),
            group.image.clone().unwrap_or_default(),
            window,
            cx,
        )
    }

    fn build(
        existing: Option<(i64, String)>,
        source_id: i64,
        name: String,
        image: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            existing,
            source_id,
            name: cx.new(|cx| InputState::new(window, cx).default_value(name)),
            image: cx.new(|cx| InputState::new(window, cx).placeholder("Image URL (optional)").default_value(image)),
            saving: false,
            error: None,
        }
    }

    fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.saving {
            return;
        }
        let name = self.name.read(cx).value().trim().to_string();
        if name.is_empty() {
            self.error = Some("Give the group a name.".into());
            cx.notify();
            return;
        }
        let image = Some(self.image.read(cx).value().trim().to_string()).filter(|s| !s.is_empty());
        let group = Group {
            id: self.existing.as_ref().map(|(id, _)| *id),
            name: name.clone(),
            image,
            source_id: Some(self.source_id),
            hidden: None,
        };
        let renamed = self.existing.as_ref().is_none_or(|(_, old)| *old != name);
        let source_id = self.source_id;
        self.saving = true;
        self.error = None;
        cx.notify();
        let request = Backend::blocking(cx, move || {
            if renamed && sql::group_exists(&group.name, source_id)? {
                bail!("This source already has a group called “{}”.", group.name);
            }
            if group.id.is_some() {
                sql::edit_custom_group(group)
            } else {
                sql::do_tx(|tx| sql::add_custom_group(tx, group).map(|_| ()))
            }
        });
        cx.spawn_in(window, async move |this, cx| {
            let result = request.await;
            _ = this.update(cx, |this, cx| {
                this.saving = false;
                match result {
                    Ok(()) => cx.emit(EditorEvent::Saved),
                    Err(e) => this.error = Some(format!("{e:#}").into()),
                }
                cx.notify();
            });
        })
        .detach();
    }
}

impl Render for GroupEditor {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let saving = self.saving;
        v_flex()
            .gap_3()
            .child(
                Form::new()
                    .child(Field::new().label("Name").child(Input::new(&self.name)))
                    .child(Field::new().label("Image").child(Input::new(&self.image))),
            )
            .when_some(self.error.clone(), |el, e| el.child(div().text_color(cx.theme().danger).child(e)))
            .child(
                h_flex()
                    .gap_2()
                    .justify_end()
                    .child(
                        Button::new("group-cancel")
                            .ghost()
                            .label("Cancel")
                            .disabled(saving)
                            .on_click(cx.listener(|_, _: &ClickEvent, _, cx| cx.emit(EditorEvent::Cancelled))),
                    )
                    .child(
                        Button::new("group-save")
                            .primary()
                            .label(if self.existing.is_some() { "Save" } else { "Add group" })
                            .loading(saving)
                            .disabled(saving)
                            .on_click(cx.listener(|this, _: &ClickEvent, window, cx| this.save(window, cx))),
                    ),
            )
    }
}
