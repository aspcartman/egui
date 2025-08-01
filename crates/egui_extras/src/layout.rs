use egui::{Id, Pos2, Rect, Response, Sense, Ui, UiBuilder, emath::GuiRounding as _};

#[derive(Clone, Copy)]
pub(crate) enum CellSize {
    /// Absolute size in points
    Absolute(f32),

    /// Take all available space
    Remainder,
}

/// Cells are positioned in two dimensions, cells go in one direction and form lines.
///
/// In a strip there's only one line which goes in the direction of the strip:
///
/// In a horizontal strip, a [`StripLayout`] with horizontal [`CellDirection`] is used.
/// Its cells go from left to right inside this [`StripLayout`].
///
/// In a table there's a [`StripLayout`] for each table row with a horizontal [`CellDirection`].
/// Its cells go from left to right. And the lines go from top to bottom.
pub(crate) enum CellDirection {
    /// Cells go from left to right.
    Horizontal,

    /// Cells go from top to bottom.
    Vertical,
}

/// Flags used by [`StripLayout::add`].
#[derive(Clone, Copy, Default)]
pub(crate) struct StripLayoutFlags {
    pub(crate) clip: bool,
    pub(crate) striped: bool,
    pub(crate) hovered: bool,
    pub(crate) selected: bool,
    pub(crate) overline: bool,

    /// Used when we want to accruately measure the size of this cell.
    pub(crate) sizing_pass: bool,
}

/// Positions cells in [`CellDirection`] and starts a new line on [`StripLayout::end_line`]
pub struct StripLayout<'l> {
    pub(crate) ui: &'l mut Ui,
    direction: CellDirection,
    pub(crate) rect: Rect,
    pub(crate) cursor: Pos2,

    /// Keeps track of the max used position,
    /// so we know how much space we used.
    max: Pos2,

    cell_layout: egui::Layout,
    sense: Sense,
}

impl<'l> StripLayout<'l> {
    pub(crate) fn new(
        ui: &'l mut Ui,
        direction: CellDirection,
        cell_layout: egui::Layout,
        sense: Sense,
    ) -> Self {
        let rect = ui.available_rect_before_wrap();
        let pos = rect.left_top();

        Self {
            ui,
            direction,
            rect,
            cursor: pos,
            max: pos,
            cell_layout,
            sense,
        }
    }

    fn cell_rect(&self, width: &CellSize, height: &CellSize) -> Rect {
        Rect {
            min: self.cursor,
            max: Pos2 {
                x: match width {
                    CellSize::Absolute(width) => self.cursor.x + width,
                    CellSize::Remainder => self.rect.right(),
                },
                y: match height {
                    CellSize::Absolute(height) => self.cursor.y + height,
                    CellSize::Remainder => self.rect.bottom(),
                },
            },
        }
    }

    fn set_pos(&mut self, rect: Rect) {
        self.max.x = self.max.x.max(rect.right());
        self.max.y = self.max.y.max(rect.bottom());

        match self.direction {
            CellDirection::Horizontal => {
                self.cursor.x = rect.right() + self.ui.spacing().item_spacing.x;
            }
            CellDirection::Vertical => {
                self.cursor.y = rect.bottom() + self.ui.spacing().item_spacing.y;
            }
        }
    }

    pub(crate) fn empty(&mut self, width: CellSize, height: CellSize) {
        self.set_pos(self.cell_rect(&width, &height));
    }

    /// This is the innermost part of [`crate::Table`] and [`crate::Strip`].
    ///
    /// Return the used space (`min_rect`) plus the [`Response`] of the whole cell.
    pub(crate) fn add(
        &mut self,
        flags: StripLayoutFlags,
        width: CellSize,
        height: CellSize,
        child_ui_id_salt: Id,
        add_cell_contents: impl FnOnce(&mut Ui),
    ) -> (Rect, Response) {
        let max_rect = self.cell_rect(&width, &height);

        // Make sure we don't have a gap in the stripe/frame/selection background:
        let item_spacing = self.ui.spacing().item_spacing;
        let gapless_rect = max_rect.expand2(0.5 * item_spacing).round_ui();

        if flags.striped {
            self.ui.painter().rect_filled(
                gapless_rect,
                egui::CornerRadius::ZERO,
                self.ui.visuals().faint_bg_color,
            );
        }

        if flags.selected {
            self.ui.painter().rect_filled(
                gapless_rect,
                egui::CornerRadius::ZERO,
                self.ui.visuals().selection.bg_fill,
            );
        }

        if flags.hovered && !flags.selected && self.sense.interactive() {
            self.ui.painter().rect_filled(
                gapless_rect,
                egui::CornerRadius::ZERO,
                self.ui.visuals().widgets.hovered.bg_fill,
            );
        }

        let mut child_ui = self.cell(flags, max_rect, child_ui_id_salt, add_cell_contents);

        let used_rect = child_ui.min_rect();

        // Make sure we catch clicks etc on the _whole_ cell:
        child_ui.set_min_size(max_rect.size());

        let allocation_rect = if self.ui.is_sizing_pass() {
            used_rect
        } else if flags.clip {
            max_rect
        } else {
            max_rect | used_rect
        };

        self.set_pos(allocation_rect);

        self.ui.advance_cursor_after_rect(allocation_rect);

        let response = child_ui.response();

        (used_rect, response)
    }

    /// only needed for layouts with multiple lines, like [`Table`](crate::Table).
    pub fn end_line(&mut self) {
        match self.direction {
            CellDirection::Horizontal => {
                self.cursor.y = self.max.y + self.ui.spacing().item_spacing.y;
                self.cursor.x = self.rect.left();
            }
            CellDirection::Vertical => {
                self.cursor.x = self.max.x + self.ui.spacing().item_spacing.x;
                self.cursor.y = self.rect.top();
            }
        }
    }

    /// Skip a lot of space.
    pub(crate) fn skip_space(&mut self, delta: egui::Vec2) {
        let before = self.cursor;
        self.cursor += delta;
        let rect = Rect::from_two_pos(before, self.cursor);
        self.ui.allocate_rect(rect, Sense::hover());
    }

    /// Return the Ui to which the contents where added
    fn cell(
        &mut self,
        flags: StripLayoutFlags,
        max_rect: Rect,
        child_ui_id_salt: egui::Id,
        add_cell_contents: impl FnOnce(&mut Ui),
    ) -> Ui {
        let mut ui_builder = UiBuilder::new()
            .id_salt(child_ui_id_salt)
            .ui_stack_info(egui::UiStackInfo::new(egui::UiKind::TableCell))
            .max_rect(max_rect)
            .layout(self.cell_layout)
            .sense(self.sense);
        if flags.sizing_pass {
            ui_builder = ui_builder.sizing_pass();
        }

        let mut child_ui = self.ui.new_child(ui_builder);

        if flags.clip {
            let margin = egui::Vec2::splat(self.ui.visuals().clip_rect_margin);
            let margin = margin.min(0.5 * self.ui.spacing().item_spacing);
            let clip_rect = max_rect.expand2(margin);
            child_ui.shrink_clip_rect(clip_rect);

            if !child_ui.is_sizing_pass() {
                // Better to truncate (if we can), rather than hard clipping:
                child_ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
            }
        }

        if flags.selected {
            let stroke_color = child_ui.style().visuals.selection.stroke.color;
            child_ui.style_mut().visuals.override_text_color = Some(stroke_color);
        }

        if flags.overline {
            child_ui.painter().hline(
                max_rect.x_range(),
                max_rect.top(),
                child_ui.visuals().widgets.noninteractive.bg_stroke,
            );
        }

        add_cell_contents(&mut child_ui);

        child_ui
    }

    /// Allocate the rect in [`Self::ui`] so that the scrollview knows about our size
    pub fn allocate_rect(&mut self) -> Response {
        let mut rect = self.rect;
        rect.set_right(self.max.x);
        rect.set_bottom(self.max.y);

        self.ui.allocate_rect(rect, Sense::hover())
    }
}
