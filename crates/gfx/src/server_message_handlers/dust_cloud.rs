use bevy::prelude::*;
use bevy_hanabi::prelude::*;

use bevy_hanabi::Gradient;

/// Plugin that provides dust explosion effects
pub struct DustExplosionPlugin;

impl Plugin for DustExplosionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HanabiPlugin)
            .init_resource::<DustExplosionAssets>()
            .add_systems(Startup, setup_dust_explosion_assets);
    }
}

/// Resource that holds the dust explosion effect asset handle
#[derive(Resource, Default)]
pub struct DustExplosionAssets {
    pub effect: Handle<EffectAsset>,
}

fn setup_dust_explosion_assets(
    mut assets: ResMut<DustExplosionAssets>,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    // Create the dust explosion effect
    let gradient = Gradient::linear(Vec4::new(0.6, 0.5, 0.3, 1.0), Vec4::ZERO);
    let color_over_lifetime = ColorOverLifetimeModifier {
        gradient,
        blend: ColorBlendMode::Overwrite,
        mask: ColorBlendMask::RGBA,
    };

    let mut module = Module::default();

    // Initialize particles in a small sphere
    let init_pos = SetPositionSphereModifier {
        center: module.lit(Vec3::ZERO),
        radius: module.lit(0.2),
        dimension: ShapeDimension::Volume,
    };

    // Particles explode outward radially
    let init_vel = SetVelocitySphereModifier {
        center: module.lit(Vec3::new(0.0, 0.0, 0.0)),
        speed: module.lit(5.0),
    };

    // Particle lifetime
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, module.lit(0.3));

    // Gravity
    let accel = module.lit(Vec3::new(0.0, -1.5, 0.0));
    let update_accel = AccelModifier::new(accel);

    let size_modifier = SetSizeModifier {
        size: Vec3::splat(0.05).into(),
    };

    let orient_modifier = OrientModifier {
        mode: OrientMode::FaceCameraPosition,
        rotation: default(),
    };

    let effect = EffectAsset::new(4096, SpawnerSettings::once(2048.0.into()), module)
        .with_name("DustExplosion")
        .init(init_pos)
        .init(init_vel)
        .init(init_lifetime)
        .update(update_accel)
        .render(color_over_lifetime)
        .render(size_modifier)
        .render(orient_modifier)
        .update(update_accel);

    assets.effect = effects.add(effect);
}

/// Spawn a dust explosion at the given position
pub fn spawn_dust_explosion(
    commands: &mut Commands,
    assets: &DustExplosionAssets,
    transform: Transform,
) {
    commands.spawn((ParticleEffect::new(assets.effect.clone()), transform));
}
