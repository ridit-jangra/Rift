use std::path::{Path, PathBuf};

use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, SharedString, Styled, Window,
    div, prelude::FluentBuilder as _, px,
};
use gpui_component::{
    ActiveTheme, IconName, Sizable,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputState},
    list::ListItem,
    resizable::{h_resizable, resizable_panel},
    status_bar::StatusBar,
    tab::{Tab, TabBar},
    tree::{TreeState, tree},
    v_flex,
};

use crate::explorer_panel;

pub struct EditorView {
    editor: Entity<InputState>,
    tree_state: Entity<TreeState>,
    open_files: Vec<PathBuf>,
    active_tab: Option<usize>,
}

impl EditorView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let editor = cx.new(|cx| {
            InputState::new(window, cx)
                .code_editor("rust".to_string())
                .line_number(true)
                .placeholder("Start typing...")
        });

        let tree_state = cx.new(|cx| TreeState::new(cx));

        let tree_state_clone = tree_state.clone();
        cx.spawn(async move |_this, cx| {
            let items = explorer_panel::build_file_items(Path::new("."));
            _ = tree_state_clone.update(cx, |state, cx| {
                state.set_items(items, cx);
            });
        })
        .detach();

        Self {
            editor,
            tree_state,
            open_files: Vec::new(),
            active_tab: None,
        }
    }

    fn open_file(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(ix) = self.open_files.iter().position(|p| p == &path) {
            self.switch_tab(ix, window, cx);
            return;
        }

        let Ok(content) = std::fs::read_to_string(&path) else {
            return;
        };

        self.editor.update(cx, |state, cx| {
            state.set_value(content, window, cx);
        });

        self.open_files.push(path);
        self.active_tab = Some(self.open_files.len() - 1);
        cx.notify();
    }

    fn switch_tab(&mut self, ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(path) = self.open_files.get(ix).cloned() else {
            return;
        };
        let Ok(content) = std::fs::read_to_string(&path) else {
            return;
        };

        self.editor.update(cx, |state, cx| {
            state.set_value(content, window, cx);
        });
        self.active_tab = Some(ix);
        cx.notify();
    }

    fn close_tab(&mut self, ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        if ix >= self.open_files.len() {
            return;
        }
        self.open_files.remove(ix);

        if self.open_files.is_empty() {
            self.active_tab = None;
            self.editor.update(cx, |state, cx| {
                state.set_value(String::new(), window, cx);
            });
            cx.notify();
        } else {
            let new_active = ix.min(self.open_files.len() - 1);
            self.switch_tab(new_active, window, cx);
        }
    }

    fn render_tab_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        TabBar::new("open-tabs")
            .selected_index(self.active_tab.unwrap_or(0))
            .on_click(cx.listener(|this, ix: &usize, window, cx| {
                this.switch_tab(*ix, window, cx);
            }))
            .children(self.open_files.iter().enumerate().map(|(ix, path)| {
                let label = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("untitled")
                    .to_string();

                Tab::new().label(label).suffix(
                    Button::new(("close-tab", ix))
                        .icon(IconName::Close)
                        .ghost()
                        .xsmall()
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.close_tab(ix, window, cx);
                        })),
                )
            }))
    }

    fn render_file_tree(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();

        tree(
            &self.tree_state,
            move |ix, entry, _selected, _window, cx| {
                view.update(cx, |_, cx| {
                    let item = entry.item();
                    let icon = if !entry.is_folder() {
                        IconName::File
                    } else if entry.is_expanded() {
                        IconName::FolderOpen
                    } else {
                        IconName::Folder
                    };

                    ListItem::new(ix)
                        .w_full()
                        .rounded(cx.theme().radius)
                        .py_0p5()
                        .px_2()
                        .pl(px(16.) * entry.depth() + px(8.))
                        .child(h_flex().gap_2().child(icon).child(item.label.clone()))
                        .on_click(cx.listener({
                            let item = item.clone();
                            move |this, _, window, cx| {
                                if item.is_folder() {
                                    return;
                                }
                                this.open_file(PathBuf::from(item.id.as_str()), window, cx);
                            }
                        }))
                })
            },
        )
        .text_sm()
        .p_1()
        .bg(cx.theme().sidebar)
        .text_color(cx.theme().sidebar_foreground)
        .h_full()
    }
}

impl Render for EditorView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let file_label = match self.active_tab.and_then(|ix| self.open_files.get(ix)) {
            Some(path) => path.display().to_string(),
            None => "No file open".to_string(),
        };

        let cursor = self.editor.read(cx).cursor_position();
        let position_label = format!("Ln {}, Col {}", cursor.line + 1, cursor.character + 1);

        let editor_area = if self.active_tab.is_some() {
            Input::new(&self.editor)
                .h_full()
                .w_full()
                .into_any_element()
        } else {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(cx.theme().muted_foreground)
                .child("Open a file to start editing")
                .into_any_element()
        };

        let editor_column = v_flex()
            .size_full()
            .when(!self.open_files.is_empty(), |this| {
                this.child(self.render_tab_bar(cx))
            })
            .child(div().flex_1().w_full().child(editor_area));

        v_flex()
            .size_full()
            .child(
                v_flex().w_full().flex_1().child(
                    h_resizable("editor-container")
                        .child(
                            resizable_panel()
                                .size(px(240.))
                                .child(self.render_file_tree(window, cx)),
                        )
                        .child(editor_column.into_any_element()),
                ),
            )
            .child(
                StatusBar::new()
                    .left(SharedString::from(file_label))
                    .right(SharedString::from(position_label)),
            )
    }
}
