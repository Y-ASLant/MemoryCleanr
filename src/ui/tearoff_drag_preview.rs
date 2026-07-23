use gpui::*;
use gpui_component::ActiveTheme;

use crate::clipboard::ClipboardItem;
use crate::ui::clipboard_item_card::{DRAG_CARD_WIDTH, ITEM_HEIGHT, render_card_content};

/// Screen origin for the tear-off drag ghost (cursor centered on card).
pub fn tearoff_preview_origin(screen: Point<Pixels>) -> Point<Pixels> {
    point(
        screen.x - px(DRAG_CARD_WIDTH / 2.),
        screen.y - px(ITEM_HEIGHT / 2.),
    )
}

pub fn tearoff_preview_window_options(origin: Point<Pixels>) -> WindowOptions {
    WindowOptions {
        titlebar: None,
        window_bounds: Some(WindowBounds::Windowed(Bounds::new(
            origin,
            size(px(DRAG_CARD_WIDTH), px(ITEM_HEIGHT)),
        ))),
        kind: WindowKind::PopUp,
        focus: false,
        is_resizable: false,
        is_movable: false,
        ..Default::default()
    }
}

/// Follower card shown while dragging outside the main window (normal card chrome).
pub struct TearoffDragPreview {
    item: ClipboardItem,
}

impl TearoffDragPreview {
    pub fn new(item: ClipboardItem) -> Self {
        Self { item }
    }
}

impl Render for TearoffDragPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .relative()
            .w(px(DRAG_CARD_WIDTH))
            .h(px(ITEM_HEIGHT))
            .overflow_hidden()
            .bg(theme.background)
            .border_1()
            .border_color(theme.primary.opacity(0.55))
            .rounded_md()
            .cursor_grabbing()
            .px_2()
            .py_2()
            .child(render_card_content(&self.item, cx))
            .shadow(vec![BoxShadow {
                color: hsla(0., 0., 0., 0.16),
                offset: point(px(0.), px(6.)),
                blur_radius: px(16.),
                spread_radius: px(0.),
                inset: false,
            }])
            .opacity(0.96)
    }
}
