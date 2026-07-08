mod editor_view;
mod explorer_panel;

use gpui::{App, AppContext, Bounds, Entity, WindowBounds, WindowKind, WindowOptions, px, size};
use gpui_component::{Root, TitleBar};

use editor_view::EditorView;

fn main() {
    let app = gpui_platform::application().with_assets(gpui_component_assets::Assets);

    app.run(move |cx: &mut App| {
        gpui_component::init(cx);
        cx.activate(true);

        let window_size = size(px(1000.0), px(700.0));
        let window_bounds = Bounds::centered(None, window_size, cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(window_bounds)),
                titlebar: Some(TitleBar::title_bar_options()),
                kind: WindowKind::Normal,
                #[cfg(target_os = "linux")]
                window_background: gpui::WindowBackgroundAppearance::Transparent,
                #[cfg(target_os = "linux")]
                window_decorations: Some(gpui::WindowDecorations::Client),
                ..Default::default()
            },
            |window, cx| {
                let editor_view: Entity<EditorView> = cx.new(|cx| EditorView::new(window, cx));
                cx.new(|cx| Root::new(editor_view, window, cx))
            },
        )
        .expect("failed to open window");
    })
}
