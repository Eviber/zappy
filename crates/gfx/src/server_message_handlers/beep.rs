use bevy::prelude::*;

pub struct BeepPlugin;

#[derive(Event)]
pub struct Beep;

#[derive(Resource)]
struct BeepHandle(Handle<AudioSource>);

impl Plugin for BeepPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut beep: ResMut<BeepHandle>) {
    beep.0 = asset_server.load("beep.wav");
    commands.add_observer(on_beep);
}

fn on_beep(_: On<Beep>, beep: Res<BeepHandle>, mut commands: Commands) {
    commands.spawn(AudioPlayer::new(beep.0.clone()));
}
