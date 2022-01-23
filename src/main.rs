use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use wrd::prelude::*;

fn main() {
    let mut app = App::new();

    app.insert_resource(WindowDescriptor {
        cursor_visible: true,
        cursor_locked: false,
        width: 720f32,
        height: 720f32,
        ..Default::default()
    })
    .insert_resource(Msaa { samples: 4 })
    .insert_resource(ClearColor(Color::rgb(0.0, 0.02, 0.05)))
    .add_plugins(DefaultPlugins)
    .add_plugin(EguiPlugin)
    .add_plugin(GamePlugin)
    .run()
}
