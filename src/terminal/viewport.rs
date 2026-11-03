//! Per-terminal reading width. Split slots stay unchanged; spare width becomes gutters.

use gpui::{Bounds, Pixels, Point, Size, point, px, size};

pub(super) struct Viewport {
    pub bounds: Bounds<Pixels>,
    pub dimensions: (u16, u16),
}

impl Viewport {
    pub fn new(bounds: Bounds<Pixels>, cell: Size<Pixels>, padding: (f32, f32), max_columns: Option<u16>) -> Self {
        let (pad_x, pad_y) = padding;
        let available = ((f32::from(bounds.size.width) - 2.0 * pad_x) / f32::from(cell.width)).floor().max(2.0) as u16;
        let cols = max_columns.map_or(available, |max| available.min(max.max(2)));
        let rows = ((f32::from(bounds.size.height) - 2.0 * pad_y) / f32::from(cell.height)).floor().max(1.0) as u16;
        let width = max_columns.map_or(bounds.size.width, |max| {
            bounds.size.width.min(cell.width * f32::from(max.max(2)) + px(2.0 * pad_x))
        });
        let origin = bounds.origin + point((bounds.size.width - width) / 2.0, px(0.0));
        Self { bounds: Bounds::new(origin, size(width, bounds.size.height)), dimensions: (cols, rows) }
    }

    /// Only cells have links. Padding and gutters must not resolve to column zero.
    pub fn contains_cell(origin: Point<Pixels>, cell: Size<Pixels>, dimensions: (u16, u16), pos: Point<Pixels>) -> bool {
        pos.x >= origin.x && pos.y >= origin.y
            && pos.x < origin.x + cell.width * f32::from(dimensions.0)
            && pos.y < origin.y + cell.height * f32::from(dimensions.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cap_centers_each_viewport_without_changing_its_split_slot() {
        let cell = size(px(8.4), px(18.0));
        let left = Bounds::new(point(px(30.0), px(20.0)), size(px(1000.0), px(500.0)));
        let right = Bounds::new(point(px(1040.0), px(20.0)), size(px(450.0), px(500.0)));
        let capped = Viewport::new(left, cell, (12.0, 10.0), Some(80));
        assert_eq!(capped.dimensions, (80, 26));
        assert_eq!(capped.bounds.size.width, px(696.0));
        assert_eq!(capped.bounds.origin.x, px(182.0));
        let narrow = Viewport::new(right, cell, (12.0, 10.0), Some(80));
        assert_eq!(narrow.bounds, right);
        assert_eq!(narrow.dimensions, (50, 26));
        assert_eq!(Viewport::new(left, cell, (12.0, 10.0), None).bounds, left);
        assert_eq!(Viewport::new(left, cell, (12.0, 10.0), None).dimensions, (116, 26));
    }

    #[test]
    fn columns_follow_font_and_zoom_and_never_exceed_the_cap() {
        for zoom in [0.5, 1.0, 1.25, 2.0, 3.0] {
            for cell_width in [7.3, 8.4, 11.7] {
                let cell = size(px(cell_width * zoom), px(18.0 * zoom));
                let padding = (12.0 * zoom, 10.0 * zoom);
                for width in [0.0, 100.0, 700.0, 2000.0] {
                    let bounds = Bounds::new(Point::default(), size(px(width), px(500.0)));
                    let v = Viewport::new(bounds, cell, padding, Some(80));
                    assert!((2..=80).contains(&v.dimensions.0));
                    assert!(v.bounds.size.width <= bounds.size.width);
                    assert_eq!(v.dimensions.1, Viewport::new(bounds, cell, padding, None).dimensions.1);
                    if width >= 80.0 * cell_width * zoom + padding.0 * 2.0 {
                        assert_eq!(v.dimensions.0, 80);
                    }
                }
            }
        }
    }

    #[test]
    fn gutter_and_padding_are_not_first_or_last_cell_links() {
        let origin = point(px(200.0), px(30.0));
        let cell = size(px(8.0), px(18.0));
        let hit = |x, y| Viewport::contains_cell(origin, cell, (80, 20), point(px(x), px(y)));
        assert!(hit(200.0, 30.0));
        assert!(hit(839.0, 389.0));
        assert!(!hit(199.0, 30.0));
        assert!(!hit(840.0, 30.0));
        assert!(!hit(200.0, 29.0));
        assert!(!hit(200.0, 390.0));
    }

    #[gpui::test]
    fn live_cap_resizes_each_terminal_and_keeps_gutter_clicks_out_of_the_tui(cx: &mut gpui::TestAppContext) {
        use gpui::*;
        use super::super::{TerminalView, grid_size};

        struct Root {
            left: Entity<TerminalView>,
            right: Entity<TerminalView>,
        }
        impl Render for Root {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                div().flex().w(px(1000.0)).h(px(300.0))
                    .child(div().w(px(800.0)).h_full().flex_none().child(self.left.clone()))
                    .child(div().w(px(200.0)).h_full().flex_none().child(self.right.clone()))
            }
        }

        let original = crate::settings::get();
        let mut settings = (*original).clone();
        settings.pane_max_columns = Some(40);
        settings.terminal_padding = [10.0, 10.0];
        settings.zoom = 1.0;
        crate::settings::set(settings.clone());
        let (root, cx) = cx.add_window_view(|_, cx| Root {
            left: cx.new(|cx| TerminalView::attach(crate::hc::offline(), "t1".into(), (80, 24), cx)),
            right: cx.new(|cx| TerminalView::attach(crate::hc::offline(), "t2".into(), (80, 24), cx)),
        });
        cx.run_until_parked();
        let (left, right) = root.read_with(cx, |root, _| (root.left.clone(), root.right.clone()));
        let (gutter, first_cell, right_origin) = left.read_with(cx, |left, _| {
            let grid = left.grid.borrow();
            assert_eq!(grid.vt.cols, 40);
            assert_eq!(grid_size(size(px(800.0), px(300.0)), grid.cell), (grid.vt.cols, grid.vt.rows));
            assert!(grid.viewport.origin.x > px(0.0));
            (point(px(1.0), px(30.0)), grid.origin + point(px(2.0), px(2.0)), px(800.0))
        });
        right.read_with(cx, |right, _| {
            let grid = right.grid.borrow();
            assert!(grid.vt.cols < 40);
            assert_eq!(grid.viewport.origin.x, right_origin);
            assert_eq!(grid.viewport.size.width, px(200.0));
        });
        for (position, accepted) in [(gutter, false), (first_cell, true)] {
            cx.simulate_event(ScrollWheelEvent {
                position, delta: ScrollDelta::Pixels(point(px(0.0), px(5.0))),
                modifiers: Modifiers::default(), touch_phase: TouchPhase::Moved,
            });
            assert_eq!(left.read_with(cx, |left, _| left.scrolled.y > 0.0), accepted);
        }
        cx.simulate_mouse_down(gutter, MouseButton::Left, Modifiers::default());
        assert!(!left.read_with(cx, |left, _| left.selecting));
        cx.simulate_mouse_up(gutter, MouseButton::Left, Modifiers::default());
        cx.simulate_mouse_down(first_cell, MouseButton::Left, Modifiers::default());
        assert!(left.read_with(cx, |left, _| left.selecting));
        cx.simulate_mouse_move(gutter, Some(MouseButton::Left), Modifiers::default());
        cx.simulate_mouse_up(gutter, MouseButton::Left, Modifiers::default());
        assert!(!left.read_with(cx, |left, _| left.selecting));

        // The live preference must restore the full terminal, not just its paint bounds.
        settings.pane_max_columns = None;
        crate::settings::set(settings);
        root.update(cx, |_, cx| cx.notify());
        cx.run_until_parked();
        left.read_with(cx, |left, _| {
            let grid = left.grid.borrow();
            assert!(grid.vt.cols > 40);
            assert_eq!(grid.viewport.origin.x, px(0.0));
            assert_eq!(grid.viewport.size.width, px(800.0));
        });
        crate::settings::set((*original).clone());
    }
}
