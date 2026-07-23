use gpui::*;
use gpui_component::{
    ActiveTheme, Icon, IconName, Root, Sizable, TITLE_BAR_HEIGHT,
    h_flex, v_flex,
};
use rust_i18n::t;

use crate::app::{AppEntityHolder, CONTENT_PADDING, WINDOW_WIDTH};
use crate::clipboard::ClipboardItem;
use crate::ui::clipboard_item_card::{ITEM_HEIGHT, render_card_content};

const TITLE_BAR_LEFT_PADDING: Pixels = px(12.);
/// Match main window width; height = title bar + padded card body.
pub const PINNED_WINDOW_WIDTH: f32 = WINDOW_WIDTH;

fn pinned_window_height() -> f32 {
    f32::from(TITLE_BAR_HEIGHT) + ITEM_HEIGHT + CONTENT_PADDING * 2.
}

struct PinnedTitleBarDragState {
    should_move: bool,
}

#[derive(Clone, Copy)]
struct TitleBarActionColors {
    foreground: Hsla,
    hover_fg: Hsla,
    hover_bg: Hsla,
    active_bg: Hsla,
}

impl TitleBarActionColors {
    fn from_theme(cx: &App, danger: bool) -> Self {
        let theme = cx.theme();
        let foreground = theme.foreground;
        if danger {
            Self {
                foreground,
                hover_fg: theme.danger_foreground,
                hover_bg: theme.danger,
                active_bg: theme.danger_active,
            }
        } else {
            Self {
                foreground,
                hover_fg: theme.secondary_foreground,
                hover_bg: theme.secondary_hover,
                active_bg: theme.secondary_active,
            }
        }
    }
}

pub struct PinnedCardWindow {
    item: ClipboardItem,
}

impl PinnedCardWindow {
    pub fn new(item: ClipboardItem) -> Self {
        Self { item }
    }

    fn paste(&self, cx: &mut Context<Self>) {
        let item_id = self.item.id;
        let app = cx.global::<AppEntityHolder>().0.clone();
        app.update(cx, |app, cx| app.paste_clipboard_item(item_id, cx));
    }

    fn close(&self, window: &mut Window, cx: &mut Context<Self>) {
        let item_id = self.item.id;
        let app = cx.global::<AppEntityHolder>().0.clone();
        window.remove_window();
        app.update(cx, |app, cx| {
            app.pinned_card_handles.remove(&item_id);
            cx.notify();
        });
    }
}

impl Render for PinnedCardWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let bg = cx.theme().background;
        let item_id = self.item.id;

        div()
            .relative()
            .w_full()
            .h_full()
            .child(
                v_flex()
                    .w_full()
                    .h_full()
                    .overflow_hidden()
                    .bg(bg)
                    .child(render_pinned_title_bar(window, cx))
                    .child(
                        div()
                            .id(("pinned-card", item_id as u32))
                            .w_full()
                            .flex_1()
                            .min_h_0()
                            .px(px(CONTENT_PADDING))
                            .pb(px(CONTENT_PADDING))
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| this.paste(cx)))
                            .child(
                                div()
                                    .w_full()
                                    .h(px(ITEM_HEIGHT))
                                    .overflow_hidden()
                                    .child(render_card_content(&self.item, cx)),
                            ),
                    ),
            )
            .children(Root::render_dialog_layer(window, cx))
    }
}

fn render_pinned_title_bar(
    window: &mut Window,
    cx: &mut Context<PinnedCardWindow>,
) -> impl IntoElement {    let state = window.use_state(cx, |_, _| PinnedTitleBarDragState { should_move: false });
    let theme = cx.theme();
    let title_bar_border = theme.title_bar_border;
    let title_bar_bg = theme.title_bar;
    let action_colors = TitleBarActionColors::from_theme(cx, false);
    let close_colors = TitleBarActionColors::from_theme(cx, true);

    div()
        .flex_shrink_0()
        .child(
            div()
                .id("pinned-title-bar")
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .h(TITLE_BAR_HEIGHT)
                .pl(TITLE_BAR_LEFT_PADDING)
                .border_b_1()
                .border_color(title_bar_border)
                .bg(title_bar_bg)
                .child(title_bar_drag_area(window, &state))
                .child(
                    h_flex()
                        .items_center()
                        .flex_shrink_0()
                        .h_full()
                        .child(title_bar_action(
                            "pinned-paste",
                            IconName::Copy,
                            action_colors,
                            cx,
                            |this, _, cx| this.paste(cx),
                        ))
                        .child(title_bar_close(cx, close_colors)),
                ),
        )
}

fn title_bar_drag_area(
    window: &mut Window,
    state: &Entity<PinnedTitleBarDragState>,
) -> impl IntoElement {
    div()
        .id("pinned-title-drag")
        .h_full()
        .flex_shrink_0()
        .flex_1()
        .min_w_0()
        .window_control_area(WindowControlArea::Drag)
        .on_mouse_down_out(window.listener_for(state, |state, _, _, _| {
            state.should_move = false;
        }))
        .on_mouse_down(
            MouseButton::Left,
            window.listener_for(state, |state, _, _, _| {
                state.should_move = true;
            }),
        )
        .on_mouse_up(
            MouseButton::Left,
            window.listener_for(state, |state, _, _, _| {
                state.should_move = false;
            }),
        )
        .on_mouse_move(window.listener_for(state, |state, _, window, _| {
            if state.should_move {
                state.should_move = false;
                window.start_window_move();
            }
        }))
}
fn title_bar_action(
    id: &'static str,
    icon: IconName,
    colors: TitleBarActionColors,
    cx: &mut Context<PinnedCardWindow>,
    on_click: impl Fn(&PinnedCardWindow, &mut Window, &mut Context<PinnedCardWindow>) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .flex()
        .w(TITLE_BAR_HEIGHT)
        .h_full()
        .flex_shrink_0()
        .justify_center()
        .content_center()
        .items_center()
        .text_color(colors.foreground)
        .hover(|style| style.bg(colors.hover_bg).text_color(colors.hover_fg))
        .active(|style| style.bg(colors.active_bg).text_color(colors.hover_fg))
        .on_click(cx.listener(move |this, _, window, cx| {
            cx.stop_propagation();
            on_click(this, window, cx);
        }))
        .child(Icon::new(icon).small())
}

fn title_bar_close(
    cx: &mut Context<PinnedCardWindow>,
    colors: TitleBarActionColors,
) -> impl IntoElement {
    div()
        .id("pinned-close")
        .flex()
        .w(TITLE_BAR_HEIGHT)
        .h_full()
        .flex_shrink_0()
        .justify_center()
        .content_center()
        .items_center()
        .text_color(colors.foreground)
        .hover(|style| style.bg(colors.hover_bg).text_color(colors.hover_fg))
        .active(|style| style.bg(colors.active_bg).text_color(colors.hover_fg))
        .on_click(cx.listener(|this, _, window, cx| {
            cx.stop_propagation();
            this.close(window, cx);
        }))
        .child(Icon::new(IconName::WindowClose).small())
}

pub fn pinned_window_options(origin: Point<Pixels>) -> WindowOptions {
    WindowOptions {
        titlebar: Some(gpui_component::TitleBar::title_bar_options()),
        window_bounds: Some(WindowBounds::Windowed(Bounds::new(
            origin,
            size(px(PINNED_WINDOW_WIDTH), px(pinned_window_height())),
        ))),
        is_resizable: false,
        focus: false,
        ..Default::default()
    }
}

pub fn pinned_window_origin(screen: Point<Pixels>) -> Point<Pixels> {
    point(
        screen.x - px(PINNED_WINDOW_WIDTH / 2.),
        screen.y - px(pinned_window_height() / 2.),
    )
}

pub fn window_title_for_item(item: &ClipboardItem) -> SharedString {
    let title = item
        .preview
        .lines()
        .next()
        .unwrap_or(item.preview.as_str())
        .trim();
    let mut title: String = title.chars().take(48).collect();
    if item.preview.chars().count() > 48 {
        title.push('…');
    }
    if title.is_empty() {
        title = t!("clipboard.pinned_title").to_string();
    }
    title.into()
}
