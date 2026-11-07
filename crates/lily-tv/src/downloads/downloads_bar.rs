use gpui_kit::component::{
    ActiveTheme as _, IconName, Sizable as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    progress::Progress,
    v_flex,
};
use gpui_kit::{prelude::FluentBuilder as _, *};

use super::model::{DownloadStatus, Downloads};

/// Strip along the bottom of the window listing this session's downloads.
pub struct DownloadsBar {
    downloads: Entity<Downloads>,
    _observe: Subscription,
}

impl DownloadsBar {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let downloads = Downloads::global(cx);
        Self {
            _observe: cx.observe(&downloads, |_, _, cx| cx.notify()),
            downloads,
        }
    }
}

impl Render for DownloadsBar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let items = self.downloads.read(cx).items().to_vec();
        if items.is_empty() {
            return div().into_any_element();
        }
        let any_finished = items.iter().any(|d| d.status != DownloadStatus::Running);
        v_flex()
            .px_3()
            .py_2()
            .gap_1()
            .border_t_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().secondary)
            .child(
                h_flex()
                    .justify_between()
                    .child(div().text_sm().text_color(cx.theme().muted_foreground).child("Downloads"))
                    .when(any_finished, |el| {
                        el.child(
                            Button::new("downloads-clear")
                                .xsmall()
                                .ghost()
                                .label("Clear finished")
                                .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                                    this.downloads.update(cx, |d, cx| d.clear_finished(cx))
                                })),
                        )
                    }),
            )
            .children(items.into_iter().map(|item| {
                let id = item.id.clone();
                let (status, color): (SharedString, _) = match &item.status {
                    DownloadStatus::Running if item.progress > 0. => {
                        (format!("{:.0}%", item.progress).into(), cx.theme().muted_foreground)
                    }
                    DownloadStatus::Running => ("Downloading…".into(), cx.theme().muted_foreground),
                    DownloadStatus::Done => ("Saved".into(), cx.theme().success),
                    DownloadStatus::Cancelled => ("Cancelled".into(), cx.theme().muted_foreground),
                    DownloadStatus::Failed(e) => (e.clone(), cx.theme().danger),
                };
                let running = item.status == DownloadStatus::Running;
                h_flex()
                    .gap_3()
                    .child(div().w_64().flex_none().truncate().child(item.name.clone()))
                    .child(
                        div()
                            .flex_1()
                            .child(Progress::new(ElementId::Name(format!("dl-{id}-bar").into())).value(item.progress as f32)),
                    )
                    .child(div().w_40().flex_none().truncate().text_sm().text_color(color).child(status))
                    .when(running, |el| {
                        el.child(
                            Button::new(ElementId::Name(format!("dl-{id}-cancel").into()))
                                .xsmall()
                                .ghost()
                                .icon(IconName::Close)
                                .tooltip("Cancel download")
                                .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                                    let id = id.clone();
                                    this.downloads.update(cx, |d, cx| d.cancel(&id, cx))
                                })),
                        )
                    })
            }))
            .into_any_element()
    }
}
