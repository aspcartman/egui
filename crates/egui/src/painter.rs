use std::sync::Arc;

use emath::GuiRounding as _;
use epaint::{
    CircleShape, ClippedShape, CornerRadius, PathStroke, RectShape, Shape, Stroke, StrokeKind,
    text::{Fonts, Galley, LayoutJob},
};

use crate::{
    Color32, Context, FontId,
    emath::{Align2, Pos2, Rangef, Rect, Vec2},
    layers::{LayerId, PaintList, ShapeIdx},
};

/// Helper to paint shapes and text to a specific region on a specific layer.
///
/// All coordinates are screen coordinates in the unit points (one point can consist of many physical pixels).
///
/// A [`Painter`] never outlive a single frame/pass.
#[derive(Clone)]
pub struct Painter {
    /// Source of fonts and destination of shapes
    ctx: Context,

    /// For quick access, without having to go via [`Context`].
    pixels_per_point: f32,

    /// Where we paint
    layer_id: LayerId,

    /// Everything painted in this [`Painter`] will be clipped against this.
    /// This means nothing outside of this rectangle will be visible on screen.
    clip_rect: Rect,

    /// If set, all shapes will have their colors modified to be closer to this.
    /// This is used to implement grayed out interfaces.
    fade_to_color: Option<Color32>,

    /// If set, all shapes will have their colors modified with [`Color32::gamma_multiply`] with
    /// this value as the factor.
    /// This is used to make interfaces semi-transparent.
    opacity_factor: f32,
}

impl Painter {
    /// Create a painter to a specific layer within a certain clip rectangle.
    pub fn new(ctx: Context, layer_id: LayerId, clip_rect: Rect) -> Self {
        let pixels_per_point = ctx.pixels_per_point();
        Self {
            ctx,
            pixels_per_point,
            layer_id,
            clip_rect,
            fade_to_color: None,
            opacity_factor: 1.0,
        }
    }

    /// Redirect where you are painting.
    #[must_use]
    #[inline]
    pub fn with_layer_id(mut self, layer_id: LayerId) -> Self {
        self.layer_id = layer_id;
        self
    }

    /// Create a painter for a sub-region of this [`Painter`].
    ///
    /// The clip-rect of the returned [`Painter`] will be the intersection
    /// of the given rectangle and the `clip_rect()` of the parent [`Painter`].
    pub fn with_clip_rect(&self, rect: Rect) -> Self {
        let mut new_self = self.clone();
        new_self.clip_rect = rect.intersect(self.clip_rect);
        new_self
    }

    /// Redirect where you are painting.
    ///
    /// It is undefined behavior to change the [`LayerId`]
    /// of [`crate::Ui::painter`].
    pub fn set_layer_id(&mut self, layer_id: LayerId) {
        self.layer_id = layer_id;
    }

    /// If set, colors will be modified to look like this
    #[deprecated = "Use `multiply_opacity` instead"]
    pub fn set_fade_to_color(&mut self, fade_to_color: Option<Color32>) {
        self.fade_to_color = fade_to_color;
    }

    /// Set the opacity (alpha multiplier) of everything painted by this painter from this point forward.
    ///
    /// `opacity` must be between 0.0 and 1.0, where 0.0 means fully transparent (i.e., invisible)
    /// and 1.0 means fully opaque.
    ///
    /// See also: [`Self::opacity`] and [`Self::multiply_opacity`].
    pub fn set_opacity(&mut self, opacity: f32) {
        if opacity.is_finite() {
            self.opacity_factor = opacity.clamp(0.0, 1.0);
        }
    }

    /// Like [`Self::set_opacity`], but multiplies the given value with the current opacity.
    ///
    /// See also: [`Self::set_opacity`] and [`Self::opacity`].
    pub fn multiply_opacity(&mut self, opacity: f32) {
        if opacity.is_finite() {
            self.opacity_factor *= opacity.clamp(0.0, 1.0);
        }
    }

    /// Read the current opacity of the underlying painter.
    ///
    /// See also: [`Self::set_opacity`] and [`Self::multiply_opacity`].
    #[inline]
    pub fn opacity(&self) -> f32 {
        self.opacity_factor
    }

    /// If `false`, nothing you paint will show up.
    ///
    /// Also checks [`Context::will_discard`].
    pub fn is_visible(&self) -> bool {
        self.fade_to_color != Some(Color32::TRANSPARENT) && !self.ctx.will_discard()
    }

    /// If `false`, nothing added to the painter will be visible
    pub fn set_invisible(&mut self) {
        self.fade_to_color = Some(Color32::TRANSPARENT);
    }

    /// Get a reference to the parent [`Context`].
    #[inline]
    pub fn ctx(&self) -> &Context {
        &self.ctx
    }

    /// Number of physical pixels for each logical UI point.
    #[inline]
    pub fn pixels_per_point(&self) -> f32 {
        self.pixels_per_point
    }

    /// Read-only access to the shared [`Fonts`].
    ///
    /// See [`Context`] documentation for how locks work.
    #[inline]
    pub fn fonts<R>(&self, reader: impl FnOnce(&Fonts) -> R) -> R {
        self.ctx.fonts(reader)
    }

    /// Where we paint
    #[inline]
    pub fn layer_id(&self) -> LayerId {
        self.layer_id
    }

    /// Everything painted in this [`Painter`] will be clipped against this.
    /// This means nothing outside of this rectangle will be visible on screen.
    #[inline]
    pub fn clip_rect(&self) -> Rect {
        self.clip_rect
    }

    /// Constrain the rectangle in which we can paint.
    ///
    /// Short for `painter.set_clip_rect(painter.clip_rect().intersect(new_clip_rect))`.
    ///
    /// See also: [`Self::clip_rect`] and [`Self::set_clip_rect`].
    #[inline]
    pub fn shrink_clip_rect(&mut self, new_clip_rect: Rect) {
        self.clip_rect = self.clip_rect.intersect(new_clip_rect);
    }

    /// Everything painted in this [`Painter`] will be clipped against this.
    /// This means nothing outside of this rectangle will be visible on screen.
    ///
    /// Warning: growing the clip rect might cause unexpected results!
    /// When in doubt, use [`Self::shrink_clip_rect`] instead.
    #[inline]
    pub fn set_clip_rect(&mut self, clip_rect: Rect) {
        self.clip_rect = clip_rect;
    }

    /// Useful for pixel-perfect rendering of lines that are one pixel wide (or any odd number of pixels).
    #[inline]
    pub fn round_to_pixel_center(&self, point: f32) -> f32 {
        point.round_to_pixel_center(self.pixels_per_point())
    }

    /// Useful for pixel-perfect rendering of lines that are one pixel wide (or any odd number of pixels).
    #[deprecated = "Use `emath::GuiRounding` with `painter.pixels_per_point()` instead"]
    #[inline]
    pub fn round_pos_to_pixel_center(&self, pos: Pos2) -> Pos2 {
        pos.round_to_pixel_center(self.pixels_per_point())
    }

    /// Useful for pixel-perfect rendering of filled shapes.
    #[deprecated = "Use `emath::GuiRounding` with `painter.pixels_per_point()` instead"]
    #[inline]
    pub fn round_to_pixel(&self, point: f32) -> f32 {
        point.round_to_pixels(self.pixels_per_point())
    }

    /// Useful for pixel-perfect rendering.
    #[deprecated = "Use `emath::GuiRounding` with `painter.pixels_per_point()` instead"]
    #[inline]
    pub fn round_vec_to_pixels(&self, vec: Vec2) -> Vec2 {
        vec.round_to_pixels(self.pixels_per_point())
    }

    /// Useful for pixel-perfect rendering.
    #[deprecated = "Use `emath::GuiRounding` with `painter.pixels_per_point()` instead"]
    #[inline]
    pub fn round_pos_to_pixels(&self, pos: Pos2) -> Pos2 {
        pos.round_to_pixels(self.pixels_per_point())
    }

    /// Useful for pixel-perfect rendering.
    #[deprecated = "Use `emath::GuiRounding` with `painter.pixels_per_point()` instead"]
    #[inline]
    pub fn round_rect_to_pixels(&self, rect: Rect) -> Rect {
        rect.round_to_pixels(self.pixels_per_point())
    }
}

/// ## Low level
impl Painter {
    #[inline]
    fn paint_list<R>(&self, writer: impl FnOnce(&mut PaintList) -> R) -> R {
        self.ctx.graphics_mut(|g| writer(g.entry(self.layer_id)))
    }

    fn transform_shape(&self, shape: &mut Shape) {
        if let Some(fade_to_color) = self.fade_to_color {
            tint_shape_towards(shape, fade_to_color);
        }
        if self.opacity_factor < 1.0 {
            multiply_opacity(shape, self.opacity_factor);
        }
    }

    /// It is up to the caller to make sure there is room for this.
    /// Can be used for free painting.
    /// NOTE: all coordinates are screen coordinates!
    pub fn add(&self, shape: impl Into<Shape>) -> ShapeIdx {
        if self.fade_to_color == Some(Color32::TRANSPARENT) || self.opacity_factor == 0.0 {
            self.paint_list(|l| l.add(self.clip_rect, Shape::Noop))
        } else {
            let mut shape = shape.into();
            self.transform_shape(&mut shape);
            self.paint_list(|l| l.add(self.clip_rect, shape))
        }
    }

    /// Add many shapes at once.
    ///
    /// Calling this once is generally faster than calling [`Self::add`] multiple times.
    pub fn extend<I: IntoIterator<Item = Shape>>(&self, shapes: I) {
        if self.fade_to_color == Some(Color32::TRANSPARENT) || self.opacity_factor == 0.0 {
            return;
        }
        if self.fade_to_color.is_some() || self.opacity_factor < 1.0 {
            let shapes = shapes.into_iter().map(|mut shape| {
                self.transform_shape(&mut shape);
                shape
            });
            self.paint_list(|l| l.extend(self.clip_rect, shapes));
        } else {
            self.paint_list(|l| l.extend(self.clip_rect, shapes));
        }
    }

    /// Modify an existing [`Shape`].
    pub fn set(&self, idx: ShapeIdx, shape: impl Into<Shape>) {
        if self.fade_to_color == Some(Color32::TRANSPARENT) {
            return;
        }
        let mut shape = shape.into();
        self.transform_shape(&mut shape);
        self.paint_list(|l| l.set(idx, self.clip_rect, shape));
    }

    /// Access all shapes added this frame.
    pub fn for_each_shape(&self, mut reader: impl FnMut(&ClippedShape)) {
        self.ctx.graphics(|g| {
            if let Some(list) = g.get(self.layer_id) {
                for c in list.all_entries() {
                    reader(c);
                }
            }
        });
    }
}

/// ## Debug painting
impl Painter {
    #[expect(clippy::needless_pass_by_value)]
    pub fn debug_rect(&self, rect: Rect, color: Color32, text: impl ToString) {
        self.rect(
            rect,
            0.0,
            color.additive().linear_multiply(0.015),
            (1.0, color),
            StrokeKind::Outside,
        );
        self.text(
            rect.min,
            Align2::LEFT_TOP,
            text.to_string(),
            FontId::monospace(12.0),
            color,
        );
    }

    pub fn error(&self, pos: Pos2, text: impl std::fmt::Display) -> Rect {
        let color = self.ctx.style().visuals.error_fg_color;
        self.debug_text(pos, Align2::LEFT_TOP, color, format!("🔥 {text}"))
    }

    /// Text with a background.
    ///
    /// See also [`Context::debug_text`].
    #[expect(clippy::needless_pass_by_value)]
    pub fn debug_text(
        &self,
        pos: Pos2,
        anchor: Align2,
        color: Color32,
        text: impl ToString,
    ) -> Rect {
        let galley = self.layout_no_wrap(text.to_string(), FontId::monospace(12.0), color);
        let rect = anchor.anchor_size(pos, galley.size());
        let frame_rect = rect.expand(2.0);

        let is_text_bright = color.is_additive() || epaint::Rgba::from(color).intensity() > 0.5;
        let bg_color = if is_text_bright {
            Color32::from_black_alpha(150)
        } else {
            Color32::from_white_alpha(150)
        };
        self.add(Shape::rect_filled(frame_rect, 0.0, bg_color));
        self.galley(rect.min, galley, color);
        frame_rect
    }
}

/// # Paint different primitives
impl Painter {
    /// Paints a line from the first point to the second.
    pub fn line_segment(&self, points: [Pos2; 2], stroke: impl Into<Stroke>) -> ShapeIdx {
        self.add(Shape::LineSegment {
            points,
            stroke: stroke.into(),
        })
    }

    /// Paints a line connecting the points.
    /// NOTE: all coordinates are screen coordinates!
    pub fn line(&self, points: Vec<Pos2>, stroke: impl Into<PathStroke>) -> ShapeIdx {
        self.add(Shape::line(points, stroke))
    }

    /// Paints a horizontal line.
    pub fn hline(&self, x: impl Into<Rangef>, y: f32, stroke: impl Into<Stroke>) -> ShapeIdx {
        self.add(Shape::hline(x, y, stroke))
    }

    /// Paints a vertical line.
    pub fn vline(&self, x: f32, y: impl Into<Rangef>, stroke: impl Into<Stroke>) -> ShapeIdx {
        self.add(Shape::vline(x, y, stroke))
    }

    pub fn circle(
        &self,
        center: Pos2,
        radius: f32,
        fill_color: impl Into<Color32>,
        stroke: impl Into<Stroke>,
    ) -> ShapeIdx {
        self.add(CircleShape {
            center,
            radius,
            fill: fill_color.into(),
            stroke: stroke.into(),
        })
    }

    pub fn circle_filled(
        &self,
        center: Pos2,
        radius: f32,
        fill_color: impl Into<Color32>,
    ) -> ShapeIdx {
        self.add(CircleShape {
            center,
            radius,
            fill: fill_color.into(),
            stroke: Default::default(),
        })
    }

    pub fn circle_stroke(&self, center: Pos2, radius: f32, stroke: impl Into<Stroke>) -> ShapeIdx {
        self.add(CircleShape {
            center,
            radius,
            fill: Default::default(),
            stroke: stroke.into(),
        })
    }

    /// See also [`Self::rect_filled`] and [`Self::rect_stroke`].
    pub fn rect(
        &self,
        rect: Rect,
        corner_radius: impl Into<CornerRadius>,
        fill_color: impl Into<Color32>,
        stroke: impl Into<Stroke>,
        stroke_kind: StrokeKind,
    ) -> ShapeIdx {
        self.add(RectShape::new(
            rect,
            corner_radius,
            fill_color,
            stroke,
            stroke_kind,
        ))
    }

    pub fn rect_filled(
        &self,
        rect: Rect,
        corner_radius: impl Into<CornerRadius>,
        fill_color: impl Into<Color32>,
    ) -> ShapeIdx {
        self.add(RectShape::filled(rect, corner_radius, fill_color))
    }

    pub fn rect_stroke(
        &self,
        rect: Rect,
        corner_radius: impl Into<CornerRadius>,
        stroke: impl Into<Stroke>,
        stroke_kind: StrokeKind,
    ) -> ShapeIdx {
        self.add(RectShape::stroke(rect, corner_radius, stroke, stroke_kind))
    }

    /// Show an arrow starting at `origin` and going in the direction of `vec`, with the length `vec.length()`.
    pub fn arrow(&self, origin: Pos2, vec: Vec2, stroke: impl Into<Stroke>) {
        use crate::emath::Rot2;
        let rot = Rot2::from_angle(std::f32::consts::TAU / 10.0);
        let tip_length = vec.length() / 4.0;
        let tip = origin + vec;
        let dir = vec.normalized();
        let stroke = stroke.into();
        self.line_segment([origin, tip], stroke);
        self.line_segment([tip, tip - tip_length * (rot * dir)], stroke);
        self.line_segment([tip, tip - tip_length * (rot.inverse() * dir)], stroke);
    }

    /// An image at the given position.
    ///
    /// `uv` should normally be `Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0))`
    /// unless you want to crop or flip the image.
    ///
    /// `tint` is a color multiplier. Use [`Color32::WHITE`] if you don't want to tint the image.
    ///
    /// Usually it is easier to use [`crate::Image::paint_at`] instead:
    ///
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// # let rect = egui::Rect::from_min_size(Default::default(), egui::Vec2::splat(100.0));
    /// egui::Image::new(egui::include_image!("../assets/ferris.png"))
    ///     .corner_radius(5)
    ///     .tint(egui::Color32::LIGHT_BLUE)
    ///     .paint_at(ui, rect);
    /// # });
    /// ```
    pub fn image(
        &self,
        texture_id: epaint::TextureId,
        rect: Rect,
        uv: Rect,
        tint: Color32,
    ) -> ShapeIdx {
        self.add(Shape::image(texture_id, rect, uv, tint))
    }
}

/// ## Text
impl Painter {
    /// Lay out and paint some text.
    ///
    /// To center the text at the given position, use `Align2::CENTER_CENTER`.
    ///
    /// To find out the size of text before painting it, use
    /// [`Self::layout`] or [`Self::layout_no_wrap`].
    ///
    /// Returns where the text ended up.
    #[expect(clippy::needless_pass_by_value)]
    pub fn text(
        &self,
        pos: Pos2,
        anchor: Align2,
        text: impl ToString,
        font_id: FontId,
        text_color: Color32,
    ) -> Rect {
        let galley = self.layout_no_wrap(text.to_string(), font_id, text_color);
        let rect = anchor.anchor_size(pos, galley.size());
        self.galley(rect.min, galley, text_color);
        rect
    }

    /// Will wrap text at the given width and line break at `\n`.
    ///
    /// Paint the results with [`Self::galley`].
    #[inline]
    #[must_use]
    pub fn layout(
        &self,
        text: String,
        font_id: FontId,
        color: crate::Color32,
        wrap_width: f32,
    ) -> Arc<Galley> {
        self.fonts(|f| f.layout(text, font_id, color, wrap_width))
    }

    /// Will line break at `\n`.
    ///
    /// Paint the results with [`Self::galley`].
    #[inline]
    #[must_use]
    pub fn layout_no_wrap(
        &self,
        text: String,
        font_id: FontId,
        color: crate::Color32,
    ) -> Arc<Galley> {
        self.fonts(|f| f.layout(text, font_id, color, f32::INFINITY))
    }

    /// Lay out this text layut job in a galley.
    ///
    /// Paint the results with [`Self::galley`].
    #[inline]
    #[must_use]
    pub fn layout_job(&self, layout_job: LayoutJob) -> Arc<Galley> {
        self.fonts(|f| f.layout_job(layout_job))
    }

    /// Paint text that has already been laid out in a [`Galley`].
    ///
    /// You can create the [`Galley`] with [`Self::layout`] or [`Self::layout_job`].
    ///
    /// Any uncolored parts of the [`Galley`] (using [`Color32::PLACEHOLDER`]) will be replaced with the given color.
    ///
    /// Any non-placeholder color in the galley takes precedence over this fallback color.
    #[inline]
    pub fn galley(&self, pos: Pos2, galley: Arc<Galley>, fallback_color: Color32) {
        if !galley.is_empty() {
            self.add(Shape::galley(pos, galley, fallback_color));
        }
    }

    /// Paint text that has already been laid out in a [`Galley`].
    ///
    /// You can create the [`Galley`] with [`Self::layout`].
    ///
    /// All text color in the [`Galley`] will be replaced with the given color.
    #[inline]
    pub fn galley_with_override_text_color(
        &self,
        pos: Pos2,
        galley: Arc<Galley>,
        text_color: Color32,
    ) {
        if !galley.is_empty() {
            self.add(Shape::galley_with_override_text_color(
                pos, galley, text_color,
            ));
        }
    }
}

fn tint_shape_towards(shape: &mut Shape, target: Color32) {
    epaint::shape_transform::adjust_colors(shape, move |color| {
        if *color != Color32::PLACEHOLDER {
            *color = crate::ecolor::tint_color_towards(*color, target);
        }
    });
}

fn multiply_opacity(shape: &mut Shape, opacity: f32) {
    epaint::shape_transform::adjust_colors(shape, move |color| {
        if *color != Color32::PLACEHOLDER {
            *color = color.gamma_multiply(opacity);
        }
    });
}
