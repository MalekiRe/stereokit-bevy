#[cfg(test)]
mod tests;

use bevy_app::{App, Plugin};
use bevy_ecs::prelude::Bundle;
use bevy_ecs::prelude::{NonSend, Query};
use bevy_transform::prelude::Transform;
use stereokit::{Color128, Model, RenderLayer, Settings, SkDraw, StereoKitDraw};

pub struct StereoKitBevy;

impl Plugin for StereoKitBevy {
    fn build(&self, app: &mut App) {
        fn stereokit_loop(mut app: App) {
            Settings::default()
                .init()
                .unwrap()
                .run(|_| app.update(), |_| ());
        }
        app.set_runner(stereokit_loop);
        app.insert_resource(unsafe { stereokit::Sk::create_unsafe() });
        app.insert_non_send_resource(unsafe { stereokit::SkDraw::create_unsafe() });
        #[cfg(feature = "model-draw-system")]
        app.add_system(model_draw);
    }
}

#[cfg(feature = "model-draw-system")]
#[derive(Bundle)]
pub struct ModelBundle {
    model: Model,
    transform: Transform,
    color: Color128,
    render_layer: RenderLayer,
}

impl ModelBundle {
    pub fn new(
        model: Model,
        transform: Transform,
        color: Color128,
        render_layer: RenderLayer,
    ) -> Self {
        Self {
            model,
            transform,
            color,
            render_layer,
        }
    }
}

#[cfg(feature = "model-draw-system")]
fn model_draw(query: Query<(&Model, &Transform, &Color128, &RenderLayer)>, sk: NonSend<SkDraw>) {
    query.iter().for_each(|(model, transform, color, layer)| {
        sk.model_draw(model, transform.compute_matrix(), *color, *layer)
    });
}
