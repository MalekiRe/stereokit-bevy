#[cfg(test)]
mod tests;

use bevy_app::{App, Plugin, PluginGroup, PluginGroupBuilder};
use bevy_ecs::prelude::Bundle;
use bevy_ecs::prelude::{NonSend, Query};
use bevy_transform::components::GlobalTransform;
use bevy_transform::prelude::Transform;
use bevy_transform::systems::sync_simple_transforms;
use bevy_transform::TransformPlugin;
use stereokit::{Color128, Model, RenderLayer, Settings, SkDraw, StereoKitDraw};

pub struct StereoKitBevyMinimalPlugins;

impl PluginGroup for StereoKitBevyMinimalPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(StereoKitBevy)
            .add(TransformPlugin)
    }
}


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
        app.add_system(sync_simple_transforms);
        #[cfg(feature = "model-draw-system")]
        app.add_system(model_draw);
    }
}

#[cfg(feature = "model-draw-system")]
#[derive(Bundle)]
pub struct ModelBundle {
    model: Model,
    transform: Transform,
    global_transform: GlobalTransform,
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
            global_transform: GlobalTransform::from(transform),
            color,
            render_layer,
        }
    }
}

#[cfg(feature = "model-draw-system")]
fn model_draw(query: Query<(&Model, &GlobalTransform, &Color128, &RenderLayer)>, sk: NonSend<SkDraw>) {
    query.iter().for_each(|(model, transform, color, layer)| {
        sk.model_draw(model, transform.compute_matrix(), *color, *layer)
    });
}
