//! Which actions a result row offers, depending on its type and the current view.

use std::collections::HashSet;

use lily_core::{media_type, types::Channel, view_type};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowAction {
    RemoveFromHistory,
    ToggleFavorite,
    ToggleHidden,
    Record,
    Download,
    CancelDownload,
    Guide,
    Restream,
    CopyUrl,
    Edit,
    Share,
    Delete,
}

/// What the menu needs to know beyond the row itself.
#[derive(Clone, Debug, Default)]
pub struct RowContext {
    pub view: u8,
    pub xtream_sources: HashSet<i64>,
    pub custom_sources: HashSet<i64>,
    /// Channel ids with a download in progress
    pub downloading: HashSet<i64>,
}

impl RowContext {
    pub fn is_custom(&self, channel: &Channel) -> bool {
        channel.source_id.is_some_and(|s| self.custom_sources.contains(&s))
    }

    fn is_xtream(&self, channel: &Channel) -> bool {
        channel.source_id.is_some_and(|s| self.xtream_sources.contains(&s))
    }

    fn is_downloading(&self, channel: &Channel) -> bool {
        channel.id.is_some_and(|id| self.downloading.contains(&id))
    }
}

pub fn actions_for(channel: &Channel, context: &RowContext) -> Vec<RowAction> {
    let media = channel.media_type;
    let container = media == media_type::GROUP || media == media_type::SEASON;
    let live = media == media_type::LIVESTREAM;
    let movie = media == media_type::MOVIE;
    let history = context.view == view_type::HISTORY;
    let custom = context.is_custom(channel);

    let mut actions = Vec::new();
    let mut add = |condition: bool, action| {
        if condition {
            actions.push(action);
        }
    };
    add(history, RowAction::RemoveFromHistory);
    add(!container && !history, RowAction::ToggleFavorite);
    add(!custom && !history && media != media_type::SEASON, RowAction::ToggleHidden);
    add(live, RowAction::Record);
    add(movie && !context.is_downloading(channel), RowAction::Download);
    add(movie && context.is_downloading(channel), RowAction::CancelDownload);
    add(live && !custom && context.is_xtream(channel), RowAction::Guide);
    add(live, RowAction::Restream);
    add(!container, RowAction::CopyUrl);
    add(custom, RowAction::Edit);
    add(custom, RowAction::Share);
    add(custom, RowAction::Delete);
    actions
}

pub fn label(action: RowAction, channel: &Channel) -> &'static str {
    match action {
        RowAction::RemoveFromHistory => "Remove from history",
        RowAction::ToggleFavorite if channel.favorite => "Remove from favourites",
        RowAction::ToggleFavorite => "Add to favourites",
        RowAction::ToggleHidden if channel.hidden == Some(true) => "Unhide",
        RowAction::ToggleHidden => "Hide",
        RowAction::Record => "Record",
        RowAction::Download => "Download",
        RowAction::CancelDownload => "Cancel download",
        RowAction::Guide => "TV guide",
        RowAction::Restream => "Re-stream…",
        RowAction::CopyUrl => "Copy URL",
        RowAction::Edit => "Edit…",
        RowAction::Share => "Share…",
        RowAction::Delete => "Delete…",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(media: u8, source: i64) -> Channel {
        Channel {
            id: Some(1),
            name: "x".into(),
            url: Some("http://x".into()),
            group: None,
            image: None,
            media_type: media,
            source_id: Some(source),
            series_id: None,
            group_id: None,
            favorite: false,
            stream_id: None,
            tv_archive: None,
            season_id: None,
            episode_num: None,
            hidden: Some(false),
        }
    }

    fn context(view: u8) -> RowContext {
        RowContext {
            view,
            xtream_sources: HashSet::from([1]),
            custom_sources: HashSet::from([9]),
            downloading: HashSet::new(),
        }
    }

    #[test]
    fn xtream_live_channel_has_guide_record_and_restream() {
        let actions = actions_for(&row(media_type::LIVESTREAM, 1), &context(view_type::ALL));
        assert_eq!(
            actions,
            vec![
                RowAction::ToggleFavorite,
                RowAction::ToggleHidden,
                RowAction::Record,
                RowAction::Guide,
                RowAction::Restream,
                RowAction::CopyUrl,
            ]
        );
    }

    #[test]
    fn m3u_live_channel_has_no_guide() {
        let actions = actions_for(&row(media_type::LIVESTREAM, 2), &context(view_type::ALL));
        assert!(!actions.contains(&RowAction::Guide));
    }

    #[test]
    fn history_view_offers_removal_but_not_favourite_or_hide() {
        let actions = actions_for(&row(media_type::MOVIE, 1), &context(view_type::HISTORY));
        assert!(actions.contains(&RowAction::RemoveFromHistory));
        assert!(!actions.contains(&RowAction::ToggleFavorite));
        assert!(!actions.contains(&RowAction::ToggleHidden));
    }

    #[test]
    fn movie_download_switches_to_cancel_while_running() {
        let mut ctx = context(view_type::ALL);
        assert!(actions_for(&row(media_type::MOVIE, 1), &ctx).contains(&RowAction::Download));
        ctx.downloading.insert(1);
        let actions = actions_for(&row(media_type::MOVIE, 1), &ctx);
        assert!(actions.contains(&RowAction::CancelDownload));
        assert!(!actions.contains(&RowAction::Download));
    }

    #[test]
    fn custom_items_can_be_edited_but_not_hidden() {
        let actions = actions_for(&row(media_type::GROUP, 9), &context(view_type::CATEGORIES));
        assert!(actions.contains(&RowAction::Edit));
        assert!(actions.contains(&RowAction::Delete));
        assert!(!actions.contains(&RowAction::ToggleHidden));
        assert!(!actions.contains(&RowAction::CopyUrl));
    }
}
