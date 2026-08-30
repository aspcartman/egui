#![cfg(feature = "wgpu")]

use egui::{Pos2, Transform3D, Vec2};
use egui_kittest::{Harness, kittest::Queryable as _};

#[test]
fn projective_scope_renders_and_receives_clicks() {
    let mut harness = Harness::builder()
        .with_size(Vec2::new(400.0, 300.0))
        .wgpu()
        .build_ui_state(
            |ui, enabled| {
                let transform = Transform3D::from_rotation_y(-0.35)
                    .with_perspective(-1.0 / 600.0)
                    .around(Pos2::new(150.0, 80.0));
                // Keep the widget normal; the scope supplies the projection and input mapping.
                ui.with_transform(transform, |ui| {
                    ui.checkbox(enabled, "Tilted checkbox");
                });
            },
            false,
        );

    harness
        .render()
        .expect("the transformed WGPU pipeline should render");
    harness.get_by_label("Tilted checkbox").click();
    harness.run();
    assert!(*harness.state());
}
