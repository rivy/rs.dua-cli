use crate::interactive::{widgets::COLOR_MARKED_LIGHT, CursorDirection, EntryMark, EntryMarkMap};
use dua::path_of;
use dua::traverse::{Tree, TreeIndex};
use std::borrow::Borrow;
use termion::{event::Key, event::Key::*};
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::Block,
    widgets::Borders,
    widgets::Text,
};
use tui_react::{List, ListProps};

#[derive(Default)]
pub struct MarkPane {
    selected: Option<usize>,
    marked: EntryMarkMap,
    list: List,
    has_focus: bool,
}

pub struct MarkPaneProps {
    pub border_style: Style,
}

impl MarkPane {
    pub fn has_focus(&self) -> bool {
        self.has_focus
    }
    pub fn set_focus(&mut self, has_focus: bool) {
        self.has_focus = has_focus;
        if has_focus {
            self.selected = Some(self.marked.len().saturating_sub(1));
        } else {
            self.selected = None
        }
    }
    pub fn toggle_index(mut self, index: TreeIndex, tree: &Tree) -> Option<Self> {
        // TODO: use HashMapEntry (Vacant/Occupied)
        if self.marked.get(&index).is_some() {
            self.marked.remove(&index);
        } else {
            if let Some(e) = tree.node_weight(index) {
                self.marked.insert(
                    index,
                    EntryMark {
                        size: e.size,
                        path: path_of(tree, index),
                    },
                );
            }
        }
        if self.marked.is_empty() {
            None
        } else {
            Some(self)
        }
    }
    pub fn marked(&self) -> &EntryMarkMap {
        &self.marked
    }
    pub fn key(&mut self, key: Key) {
        match key {
            Ctrl('u') | PageUp => self.change_selection(CursorDirection::PageUp),
            Char('k') | Up => self.change_selection(CursorDirection::Up),
            Char('j') | Down => self.change_selection(CursorDirection::Down),
            Ctrl('d') | PageDown => self.change_selection(CursorDirection::PageDown),
            _ => {}
        };
    }

    fn change_selection(&mut self, direction: CursorDirection) {
        self.selected = self.selected.map(|selected| {
            direction
                .move_cursor(selected)
                .min(self.marked.len().saturating_sub(1))
        });
    }

    pub fn render(&mut self, props: impl Borrow<MarkPaneProps>, area: Rect, buf: &mut Buffer) {
        let MarkPaneProps { border_style } = props.borrow();

        let marked: &_ = &self.marked;
        let block = Block::default()
            .title("Marked Entries")
            .border_style(*border_style)
            .borders(Borders::ALL);
        let entry_in_view = self
            .selected
            .map(|selected| selected)
            .or(Some(marked.len().saturating_sub(1)));
        let selected = self.selected;
        let entries = marked.values().enumerate().map(|(idx, v)| {
            let modifier = match selected {
                Some(selected) if idx == selected => Modifier::BOLD,
                _ => Modifier::empty(),
            };
            let name = Text::Styled(
                v.path.to_string_lossy(),
                Style {
                    fg: COLOR_MARKED_LIGHT,
                    modifier,
                    ..Style::default()
                },
            );
            vec![name]
        });

        let props = ListProps {
            block: Some(block),
            entry_in_view,
        };
        self.list.render(props, entries, area, buf)
    }
}