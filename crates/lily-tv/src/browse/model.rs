//! Browse state without any UI: the search filters plus the drill-down stack
//! (category -> series -> season), matching how Fred TV's home screen navigates.

use lily_core::{
    media_type,
    sort_type,
    types::{Channel, Filters},
    view_type,
};

/// What a drill-down entry narrows the results to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeKind {
    Category,
    Series,
    Season,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub id: i64,
    pub name: String,
    pub kind: NodeKind,
    /// Search text and view to restore when leaving this node
    query: Option<String>,
    from_view: u8,
}

/// Result of activating a row.
#[derive(Clone, Debug, PartialEq)]
pub enum Activation {
    /// The row is a container; the filters now point inside it.
    Entered(NodeKind),
    /// The row is something mpv can play.
    Play(Channel),
}

pub struct BrowseModel {
    filters: Filters,
    all_source_ids: Vec<i64>,
    stack: Vec<Node>,
}

impl BrowseModel {
    pub fn new(source_ids: Vec<i64>, default_view: Option<u8>, default_sort: Option<u8>) -> Self {
        Self {
            filters: Filters {
                query: None,
                source_ids: source_ids.clone(),
                media_types: Some(vec![media_type::LIVESTREAM, media_type::MOVIE, media_type::SERIE]),
                view_type: default_view.unwrap_or(view_type::ALL),
                page: 1,
                series_id: None,
                group_id: None,
                use_keywords: false,
                sort: default_sort.unwrap_or(sort_type::PROVIDER),
                season: None,
            },
            all_source_ids: source_ids,
            stack: Vec::new(),
        }
    }

    pub fn view(&self) -> u8 {
        self.filters.view_type
    }

    pub fn sort(&self) -> u8 {
        self.filters.sort
    }

    pub fn query(&self) -> &str {
        self.filters.query.as_deref().unwrap_or("")
    }

    pub fn stack(&self) -> &[Node] {
        &self.stack
    }

    pub fn shows_media_type(&self, media: u8) -> bool {
        self.filters.media_types.as_ref().is_some_and(|m| m.contains(&media))
    }

    /// Filters for the first page of the current view.
    pub fn first_page(&self) -> Filters {
        Filters { page: 1, ..self.filters.clone() }
    }

    pub fn set_query(&mut self, query: &str) {
        self.filters.query = Some(query.to_string());
    }

    /// Switching views starts from the top level, like picking a tab in Fred TV.
    pub fn set_view(&mut self, view: u8) {
        self.leave_all_nodes();
        self.filters.view_type = view;
    }

    pub fn use_keywords(&self) -> bool {
        self.filters.use_keywords
    }

    /// Switches between matching the exact phrase and all words in any order.
    pub fn toggle_keywords(&mut self) {
        self.filters.use_keywords = !self.filters.use_keywords;
    }

    /// Bulk actions apply to everything the current filters match; inside a series
    /// (before picking a season) there is nothing they can act on.
    pub fn can_bulk_update(&self) -> bool {
        !(self.filters.series_id.is_some() && self.filters.season.is_none())
    }

    pub fn set_sort(&mut self, sort: u8) {
        self.filters.sort = sort;
    }

    /// Shows or hides one media type; at least one stays enabled.
    pub fn toggle_media_type(&mut self, media: u8) {
        let types = self.filters.media_types.get_or_insert_with(Vec::new);
        if let Some(pos) = types.iter().position(|t| *t == media) {
            if types.len() > 1 {
                types.remove(pos);
            }
        } else {
            types.push(media);
            types.sort_unstable();
        }
    }

    /// Opens a container row, or reports that the row should be played.
    pub fn activate(&mut self, channel: &Channel) -> Activation {
        let (kind, id) = match channel.media_type {
            media_type::GROUP => (NodeKind::Category, channel.id),
            // series rows keep their provider series id in `url`
            media_type::SERIE => (
                NodeKind::Series,
                channel.url.as_deref().and_then(|u| u.parse().ok()),
            ),
            media_type::SEASON => (NodeKind::Season, channel.id),
            _ => return Activation::Play(channel.clone()),
        };
        let Some(id) = id else {
            return Activation::Play(channel.clone());
        };
        self.stack.push(Node {
            id,
            name: channel.name.clone(),
            kind,
            query: self.filters.query.clone(),
            from_view: self.filters.view_type,
        });
        match kind {
            NodeKind::Category => self.filters.group_id = Some(id),
            NodeKind::Series => {
                self.filters.series_id = Some(id);
                if let Some(source) = channel.source_id {
                    self.filters.source_ids = vec![source];
                }
            }
            NodeKind::Season => self.filters.season = Some(id),
        }
        if self.filters.view_type == view_type::HIDDEN {
            self.filters.view_type = view_type::CATEGORIES;
        }
        self.filters.query = None;
        Activation::Entered(kind)
    }

    /// Leaves the innermost node. Returns false when already at the top level.
    pub fn back(&mut self) -> bool {
        let Some(node) = self.stack.pop() else {
            return false;
        };
        self.undo_node(&node);
        self.filters.query = node.query;
        self.filters.view_type = node.from_view;
        true
    }

    fn leave_all_nodes(&mut self) {
        while let Some(node) = self.stack.pop() {
            self.undo_node(&node);
        }
    }

    fn undo_node(&mut self, node: &Node) {
        match node.kind {
            NodeKind::Category => self.filters.group_id = None,
            NodeKind::Series => {
                self.filters.series_id = None;
                self.filters.source_ids = self.all_source_ids.clone();
            }
            NodeKind::Season => self.filters.season = None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: i64, media: u8, url: Option<&str>, source: i64) -> Channel {
        Channel {
            id: Some(id),
            name: format!("row {id}"),
            url: url.map(str::to_string),
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

    #[test]
    fn playable_rows_do_not_change_navigation() {
        let mut model = BrowseModel::new(vec![1], None, None);
        let live = row(5, media_type::LIVESTREAM, Some("http://x/1.ts"), 1);
        assert_eq!(model.activate(&live), Activation::Play(live.clone()));
        assert!(model.stack().is_empty());
    }

    #[test]
    fn series_narrows_to_its_source_and_back_restores_everything() {
        let mut model = BrowseModel::new(vec![1, 2], Some(view_type::ALL), None);
        model.set_query("star");
        let series = row(9, media_type::SERIE, Some("4242"), 2);

        assert_eq!(model.activate(&series), Activation::Entered(NodeKind::Series));
        let inside = model.first_page();
        assert_eq!(inside.series_id, Some(4242));
        assert_eq!(inside.source_ids, vec![2]);
        assert_eq!(inside.query, None);

        let season = row(77, media_type::SEASON, None, 2);
        model.activate(&season);
        assert_eq!(model.first_page().season, Some(77));

        assert!(model.back());
        assert_eq!(model.first_page().season, None);
        assert!(model.back());
        let top = model.first_page();
        assert_eq!(top.series_id, None);
        assert_eq!(top.source_ids, vec![1, 2]);
        assert_eq!(top.query.as_deref(), Some("star"));
        assert!(!model.back());
    }

    #[test]
    fn hidden_view_switches_to_categories_when_entering_a_group() {
        let mut model = BrowseModel::new(vec![1], Some(view_type::HIDDEN), None);
        model.activate(&row(3, media_type::GROUP, None, 1));
        assert_eq!(model.view(), view_type::CATEGORIES);
        assert_eq!(model.first_page().group_id, Some(3));
        model.back();
        assert_eq!(model.view(), view_type::HIDDEN);
    }

    #[test]
    fn changing_view_leaves_nested_nodes() {
        let mut model = BrowseModel::new(vec![1], Some(view_type::CATEGORIES), None);
        model.activate(&row(3, media_type::GROUP, None, 1));
        model.set_view(view_type::FAVORITES);
        assert!(model.stack().is_empty());
        assert_eq!(model.first_page().group_id, None);
    }

    #[test]
    fn last_media_type_cannot_be_disabled() {
        let mut model = BrowseModel::new(vec![1], None, None);
        model.toggle_media_type(media_type::MOVIE);
        model.toggle_media_type(media_type::SERIE);
        model.toggle_media_type(media_type::LIVESTREAM);
        assert!(model.shows_media_type(media_type::LIVESTREAM));
        model.toggle_media_type(media_type::MOVIE);
        assert_eq!(model.first_page().media_types, Some(vec![media_type::LIVESTREAM, media_type::MOVIE]));
    }
}
