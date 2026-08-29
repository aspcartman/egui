use egui::{Pos2, Rect, Scene, Transform3D, Vec2};

use super::widget_gallery;

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SceneDemo {
    widget_gallery: widget_gallery::WidgetGallery,
    scene_rect: Rect,
    tilted_enabled: bool,
    tilted_value: f32,
}

impl Default for SceneDemo {
    fn default() -> Self {
        Self {
            widget_gallery: widget_gallery::WidgetGallery::default().with_date_button(false), // disable date button so that we don't fail the snapshot test
            scene_rect: Rect::ZERO, // `egui::Scene` will initialize this to something valid
            tilted_enabled: true,
            tilted_value: 0.0,
        }
    }
}

impl crate::Demo for SceneDemo {
    fn name(&self) -> &'static str {
        "🔍 Scene"
    }

    fn show(&mut self, ui: &mut egui::Ui, open: &mut bool) {
        use crate::View as _;
        egui::Window::new("Scene")
            .default_width(300.0)
            .default_height(300.0)
            .scroll(false)
            .open(open)
            .constrain_to(ui.available_rect_before_wrap())
            .show(ui, |ui| self.ui(ui));
    }
}

impl crate::View for SceneDemo {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label(
            "You can pan by scrolling, and zoom using cmd-scroll. \
            Double click on the background to reset view.",
        );
        ui.vertical_centered(|ui| {
            ui.add(crate::egui_github_link_file!());
        });
        ui.separator();

        ui.label(format!("Scene rect: {:#?}", self.scene_rect));

        ui.separator();

        ui.label(
            "Transform3D keeps ordinary egui widgets interactive while projecting their paint.",
        );
        let pivot = ui.cursor().left_top() + Vec2::new(150.0, 80.0);
        let angle = if self.tilted_enabled {
            self.tilted_value
        } else {
            0.0
        };
        let transform = Transform3D::from_rotation_y(angle)
            .with_perspective(-1.0 / 600.0)
            .around(pivot);
        ui.with_transform(transform, |ui| {
            // This uses the normal widget APIs; only the scope's final presentation is transformed.
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.strong("Y-rotated controls");
                ui.checkbox(&mut self.tilted_enabled, "Enabled");
                ui.add(
                    // The public Transform3D rotation constructors take radians, with zero as identity.
                    egui::Slider::new(&mut self.tilted_value, -0.7..=0.7)
                        .step_by(0.01)
                        .text("Y rotation (rad)"),
                );
            });
        });

        ui.separator();

        egui::Frame::group(ui.style())
            .inner_margin(0.0)
            .show(ui, |ui| {
                let scene = Scene::new()
                    .max_inner_size([350.0, 1000.0])
                    .zoom_range(0.1..=2.0);

                let mut reset_view = false;
                let mut inner_rect = Rect::NAN;
                let response = scene
                    .show(ui, &mut self.scene_rect, |ui| {
                        reset_view = ui.button("Reset view").clicked();

                        ui.add_space(16.0);

                        self.widget_gallery.ui(ui);

                        ui.put(
                            Rect::from_min_size(Pos2::new(0.0, -64.0), Vec2::new(200.0, 16.0)),
                            egui::Label::new("You can put a widget anywhere").selectable(false),
                        );

                        inner_rect = ui.min_rect();
                    })
                    .response;

                if reset_view || response.double_clicked() {
                    self.scene_rect = inner_rect;
                }
            });
    }
}
