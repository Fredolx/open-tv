use std::collections::HashSet;

use chrono::{Local, NaiveDate};
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, IconName, Sizable as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    spinner::Spinner,
    v_flex,
};
use gpui_kit::{prelude::FluentBuilder as _, *};
use lily_core::{
    epg, media_type, sql,
    types::{Channel, EPG, EPGNotify},
    utils, xtream,
};

use crate::backend::Backend;

/// Requests the panel makes of its owner.
pub enum GuideEvent {
    /// Play a past programme from the provider's archive.
    PlayCatchUp(Channel),
    /// Save a past programme from the archive to disk.
    DownloadCatchUp(Channel),
}

enum Load {
    Loading,
    Failed(SharedString),
    Loaded(Vec<EPG>),
}

/// Programme guide for one live channel, shown a day at a time.
pub struct GuidePanel {
    channel: Channel,
    load: Load,
    day: NaiveDate,
    reminders: HashSet<String>,
    /// Programme ids with a reminder change in flight
    pending: HashSet<String>,
    error: Option<SharedString>,
    scroll: ScrollHandle,
    /// Programme to bring to the top once the list has been laid out
    scroll_target: Option<usize>,
}

impl EventEmitter<GuideEvent> for GuidePanel {}

/// "November 07, 18:30" (the backend's format) -> "18:30"
fn clock(formatted: &str) -> &str {
    formatted.rsplit(", ").next().unwrap_or(formatted)
}

fn local_date(timestamp: i64) -> Option<NaiveDate> {
    utils::get_local_time(timestamp).ok().map(|t| t.date_naive())
}

/// Index of the programme airing at `now`: the last one that has already started.
/// Providers' own "now playing" flags are often stale, so this ignores them.
fn current_index(day_programmes: &[&EPG], now: i64) -> Option<usize> {
    let started = day_programmes.iter().filter(|p| p.start_timestamp <= now).count();
    let later_exists = day_programmes.iter().any(|p| p.start_timestamp > now);
    // after the day's last start we can't tell when it ends, so only claim it if more follow
    (started > 0 && later_exists).then(|| started - 1)
}

/// Days that have at least one programme, in order.
fn days_with_programmes(programmes: &[EPG]) -> Vec<NaiveDate> {
    let mut days: Vec<NaiveDate> = programmes
        .iter()
        .filter_map(|p| local_date(p.start_timestamp))
        .collect();
    days.sort_unstable();
    days.dedup();
    days
}

impl GuidePanel {
    pub fn new(channel: Channel, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut this = Self {
            channel,
            load: Load::Loading,
            day: Local::now().date_naive(),
            reminders: HashSet::new(),
            pending: HashSet::new(),
            error: None,
            scroll: ScrollHandle::new(),
            scroll_target: None,
        };
        this.load(window, cx);
        this
    }

    fn load(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let channel = self.channel.clone();
        let programmes = Backend::run(cx, move |_| xtream::get_epg(channel));
        let reminders = Backend::blocking(cx, sql::get_epg_ids);
        cx.spawn_in(window, async move |this, cx| {
            let (programmes, reminders) = (programmes.await, reminders.await);
            _ = this.update(cx, |this, cx| {
                this.reminders = reminders.unwrap_or_default().into_iter().collect();
                this.load = match programmes {
                    Ok(list) if list.is_empty() => Load::Failed("No guide data for this channel.".into()),
                    Ok(list) => {
                        // open on today, or the first day with data if today has none
                        let days = days_with_programmes(&list);
                        if !days.contains(&this.day) {
                            if let Some(first) = days.first() {
                                this.day = *first;
                            }
                        }
                        Load::Loaded(list)
                    }
                    Err(e) => Load::Failed(
                        format!("Couldn't load the guide. If this keeps happening, refresh the source. ({e:#})")
                            .into(),
                    ),
                };
                this.scroll_to_now();
                cx.notify();
            });
        })
        .detach();
    }

    fn step_day(&mut self, forward: bool, cx: &mut Context<Self>) {
        let Load::Loaded(list) = &self.load else {
            return;
        };
        let days = days_with_programmes(list);
        let next = if forward {
            days.iter().find(|d| **d > self.day)
        } else {
            days.iter().rev().find(|d| **d < self.day)
        };
        if let Some(day) = next {
            self.day = *day;
            self.scroll_to_now();
            cx.notify();
        }
    }

    /// Today opens at what's on now; other days at their first programme.
    fn scroll_to_now(&mut self) {
        let Load::Loaded(list) = &self.load else {
            return;
        };
        let day: Vec<&EPG> = list
            .iter()
            .filter(|p| local_date(p.start_timestamp) == Some(self.day))
            .collect();
        self.scroll_target = Some(current_index(&day, Local::now().timestamp()).unwrap_or(0));
    }

    fn catch_up_channel(&self, programme: &EPG) -> Option<Channel> {
        let url = programme.timeshift_url.clone()?;
        Some(Channel {
            id: Some(-1),
            name: programme.title.clone(),
            url: Some(url),
            group: None,
            image: self.channel.image.clone(),
            media_type: media_type::MOVIE,
            source_id: self.channel.source_id,
            series_id: None,
            group_id: None,
            favorite: false,
            stream_id: None,
            tv_archive: None,
            season_id: None,
            episode_num: None,
            hidden: Some(false),
        })
    }

    fn toggle_reminder(&mut self, programme: &EPG, window: &mut Window, cx: &mut Context<Self>) {
        let id = programme.epg_id.clone();
        if !self.pending.insert(id.clone()) {
            return;
        }
        let on = self.reminders.contains(&id);
        let notify = EPGNotify {
            epg_id: id.clone(),
            title: programme.title.clone(),
            start_timestamp: programme.start_timestamp,
            channel_name: self.channel.name.clone(),
        };
        let remove_id = id.clone();
        let request = Backend::run_with_events(cx, move |state, events| async move {
            if on {
                epg::remove_epg(&state, events, remove_id).await
            } else {
                epg::add_epg(&state, events, notify).await
            }
        });
        cx.notify();
        cx.spawn_in(window, async move |this, cx| {
            let result = request.await;
            _ = this.update(cx, |this, cx| {
                this.pending.remove(&id);
                match result {
                    Ok(()) if on => {
                        this.reminders.remove(&id);
                    }
                    Ok(()) => {
                        this.reminders.insert(id);
                    }
                    Err(e) => this.error = Some(format!("Couldn't change the reminder: {e:#}").into()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn render_programme(&self, programme: &EPG, now: i64, on_now: bool, cx: &mut Context<Self>) -> impl IntoElement {
        let id = programme.epg_id.clone();
        let upcoming = programme.start_timestamp > now;
        let catch_up = !upcoming && !on_now && programme.has_archive && programme.timeshift_url.is_some();
        let reminder_on = self.reminders.contains(&id);
        let pending = self.pending.contains(&id);
        let for_reminder = programme.clone();
        let catch_up_channel = self.catch_up_channel(programme);
        let download_channel = catch_up_channel.clone();
        h_flex()
            .id(ElementId::Name(format!("epg-{id}").into()))
            .gap_3()
            .px_3()
            .py_2()
            .items_start()
            .rounded(cx.theme().radius)
            .when(on_now, |el| el.bg(cx.theme().secondary))
            .child(
                div()
                    .flex_none()
                    .w_24()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("{}–{}", clock(&programme.start_time), clock(&programme.end_time))),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .child(
                        h_flex()
                            .gap_2()
                            .child(div().truncate().child(programme.title.clone()))
                            .when(on_now, |el| {
                                el.child(div().text_xs().text_color(cx.theme().primary).child("ON NOW"))
                            }),
                    )
                    .when(!programme.description.is_empty(), |el| {
                        el.child(
                            div()
                                .text_sm()
                                .line_clamp(2)
                                .text_color(cx.theme().muted_foreground)
                                .child(programme.description.clone()),
                        )
                    }),
            )
            .when(upcoming, |el| {
                el.child(
                    Button::new(ElementId::Name(format!("epg-{id}-remind").into()))
                        .small()
                        .ghost()
                        .icon(IconName::Bell)
                        .label(if reminder_on { "Reminder set" } else { "Remind me" })
                        .when(reminder_on, |b| b.text_color(cx.theme().primary))
                        .loading(pending)
                        .disabled(pending)
                        .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
                            this.toggle_reminder(&for_reminder, window, cx)
                        })),
                )
            })
            .when_some(catch_up_channel.filter(|_| catch_up), |el, channel| {
                el.child(
                    Button::new(ElementId::Name(format!("epg-{id}-play").into()))
                        .small()
                        .ghost()
                        .icon(IconName::Play)
                        .tooltip("Watch from the archive")
                        .on_click(cx.listener(move |_, _: &ClickEvent, _, cx| {
                            cx.emit(GuideEvent::PlayCatchUp(channel.clone()))
                        })),
                )
            })
            .when_some(download_channel.filter(|_| catch_up), |el, channel| {
                el.child(
                    Button::new(ElementId::Name(format!("epg-{id}-download").into()))
                        .small()
                        .ghost()
                        .icon(IconName::ArrowDown)
                        .tooltip("Download from the archive")
                        .on_click(cx.listener(move |_, _: &ClickEvent, _, cx| {
                            cx.emit(GuideEvent::DownloadCatchUp(channel.clone()))
                        })),
                )
            })
    }
}

impl Render for GuidePanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(index) = self.scroll_target.take() {
            // row positions are only known after this frame's layout
            let scroll = self.scroll.clone();
            window.on_next_frame(move |window, _| {
                scroll.scroll_to_top_of_item(index);
                window.refresh();
            });
        }
        let body = match &self.load {
            Load::Loading => h_flex()
                .h_40()
                .justify_center()
                .child(Spinner::new().large())
                .into_any_element(),
            Load::Failed(message) => div()
                .py_8()
                .text_color(cx.theme().muted_foreground)
                .child(message.clone())
                .into_any_element(),
            Load::Loaded(list) => {
                let days = days_with_programmes(list);
                let has_prev = days.iter().any(|d| *d < self.day);
                let has_next = days.iter().any(|d| *d > self.day);
                let now = Local::now().timestamp();
                let day: Vec<&EPG> = list
                    .iter()
                    .filter(|p| local_date(p.start_timestamp) == Some(self.day))
                    .collect();
                let current = current_index(&day, now);
                let programmes: Vec<AnyElement> = day
                    .iter()
                    .enumerate()
                    .map(|(i, p)| self.render_programme(p, now, current == Some(i), cx).into_any_element())
                    .collect();
                v_flex()
                    .gap_2()
                    .child(
                        h_flex()
                            .gap_2()
                            .justify_between()
                            .child(
                                Button::new("epg-prev-day")
                                    .ghost()
                                    .icon(IconName::ChevronLeft)
                                    .disabled(!has_prev)
                                    .tooltip("Previous day")
                                    .on_click(cx.listener(|this, _: &ClickEvent, _, cx| this.step_day(false, cx))),
                            )
                            .child(div().child(self.day.format("%A %-d %B").to_string()))
                            .child(
                                Button::new("epg-next-day")
                                    .ghost()
                                    .icon(IconName::ChevronRight)
                                    .disabled(!has_next)
                                    .tooltip("Next day")
                                    .on_click(cx.listener(|this, _: &ClickEvent, _, cx| this.step_day(true, cx))),
                            ),
                    )
                    .child(
                        v_flex()
                            .id("epg-programmes")
                            .track_scroll(&self.scroll)
                            .max_h(px(480.))
                            .overflow_y_scroll()
                            .gap_1()
                            .children(programmes),
                    )
                    .into_any_element()
            }
        };
        v_flex()
            .gap_2()
            .when_some(self.error.clone(), |el, error| {
                el.child(div().text_color(cx.theme().danger).child(error))
            })
            .child(body)
    }
}

#[cfg(test)]
mod tests {
    // not a glob: gpui_kit's own test attribute would shadow #[test]
    use super::{EPG, clock, current_index, days_with_programmes};

    fn programme(start: i64) -> EPG {
        EPG {
            epg_id: start.to_string(),
            title: "t".into(),
            description: String::new(),
            start_time: String::new(),
            start_timestamp: start,
            end_time: String::new(),
            timeshift_url: None,
            has_archive: false,
            now_playing: false,
        }
    }

    #[test]
    fn current_programme_is_the_last_started_one() {
        let list = [programme(100), programme(200), programme(300)];
        let refs: Vec<&EPG> = list.iter().collect();
        assert_eq!(current_index(&refs, 50), None);
        assert_eq!(current_index(&refs, 100), Some(0));
        assert_eq!(current_index(&refs, 250), Some(1));
        // past the last start nobody knows when it ends
        assert_eq!(current_index(&refs, 350), None);
    }

    #[test]
    fn clock_keeps_only_the_time() {
        assert_eq!(clock("November 07, 18:30"), "18:30");
        assert_eq!(clock("18:30"), "18:30");
    }

    #[test]
    fn days_are_unique_and_ordered() {
        let day = 86_400;
        let base = 1_794_000_000;
        let list = [programme(base + day), programme(base), programme(base + 60), programme(base + 2 * day)];
        let days = days_with_programmes(&list);
        assert_eq!(days.len(), 3);
        assert!(days.windows(2).all(|w| w[0] < w[1]));
    }
}
