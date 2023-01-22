use bevy::prelude::*;
use rand::Rng;

#[derive(Component)]
struct CornerMessage;

#[derive(Resource)]
struct MessageFont(Handle<Font>);

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct ImageHandle(Handle<Image>);

#[derive(Component)]
struct Speed(Vec3);

#[derive(Component)]
struct OldPos(Option<Vec3>);

#[derive(Component)]
struct ColorLoop(usize);

impl ColorLoop {
    fn new() -> Self {
        Self(0)
    }

    const ALL_COLORS: [Color; 35] = [
        Color::ALICE_BLUE,
        Color::ANTIQUE_WHITE,
        Color::AQUAMARINE,
        Color::AZURE,
        Color::BEIGE,
        Color::BISQUE,
        Color::BLUE,
        Color::CRIMSON,
        Color::CYAN,
        Color::DARK_GRAY,
        Color::DARK_GREEN,
        Color::FUCHSIA,
        Color::GOLD,
        Color::GRAY,
        Color::GREEN,
        Color::INDIGO,
        Color::LIME_GREEN,
        Color::MAROON,
        Color::MIDNIGHT_BLUE,
        Color::NAVY,
        Color::OLIVE,
        Color::ORANGE,
        Color::ORANGE_RED,
        Color::PINK,
        Color::PURPLE,
        Color::RED,
        Color::SALMON,
        Color::SEA_GREEN,
        Color::SILVER,
        Color::TEAL,
        Color::TOMATO,
        Color::TURQUOISE,
        Color::VIOLET,
        Color::YELLOW,
        Color::YELLOW_GREEN,
    ];
}

impl Iterator for ColorLoop {
    type Item = Color;
    fn next(&mut self) -> Option<Self::Item> {
        let color_num = Self::ALL_COLORS.len();
        self.0 = (self.0 + 1) % color_num;
        Some(Self::ALL_COLORS[self.0])
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let sprite = asset_server.load("bevy.png");
    commands.spawn((Camera2dBundle::default(), MainCamera));
    commands.spawn((
        ColorLoop::new(),
        ImageHandle(sprite.clone()),
        OldPos(None),
        Speed(Vec3::new(150., 150., 0.)),
        SpriteBundle {
            texture: sprite,
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        },
    ));
    commands.insert_resource(MessageFont(asset_server.load("comic-sans-bold.ttf")));
}

fn sprite_movement(
    mut commands: Commands,
    time: Res<Time>,
    windows: Res<Windows>,
    assets: Res<Assets<Image>>,
    font: Res<MessageFont>,
    mut q_sprite: Query<(
        &ImageHandle,
        &OldPos,
        &mut ColorLoop,
        &mut Speed,
        &mut Sprite,
        &mut Transform,
    )>,
) {
    let (handle, old_pos, mut color_loop, mut speed, mut sprite, mut transform) =
        q_sprite.single_mut();
    if old_pos.0.is_some() {
        // That means the sprit is currently being dragged, so we don't animate it
        return;
    }
    if let Some(image) = assets.get(&handle.0) {
        transform.translation += speed.0 * time.delta_seconds();

        let window = windows.primary();

        let world_top = window.height() / 2.;
        let world_bottom = -world_top;
        let world_right = window.width() / 2.;
        let world_left = -world_right;

        let size = image.size();

        let sprite_top = transform.translation.y + size.y / 2.;
        let sprite_bottom = transform.translation.y - size.y / 2.;
        let sprite_right = transform.translation.x + size.x / 2.;
        let sprite_left = transform.translation.x - size.x / 2.;

        let mut hits = 0;

        if sprite_top > world_top {
            speed.0.y = speed.0.y.abs() * -1.;
            sprite.color = color_loop.next().unwrap();
            hits += 1;
        } else if sprite_bottom < world_bottom {
            speed.0.y = speed.0.y.abs();
            sprite.color = color_loop.next().unwrap();
            hits += 1;
        }

        if sprite_right > world_right {
            speed.0.x = speed.0.x.abs() * -1.;
            sprite.color = color_loop.next().unwrap();
            hits += 1;
        } else if sprite_left < world_left {
            speed.0.x = speed.0.x.abs();
            sprite.color = color_loop.next().unwrap();
            hits += 1;
        }

        if hits == 2 {
            let mut rng = rand::thread_rng();
            let rotation = Quat::from_rotation_z(rng.gen_range((-1.)..(1.)));
            let x_pos = rng.gen_range((-15.)..(15.));
            let y_pos = rng.gen_range((-15.)..(15.));
            let text_style = TextStyle {
                font: font.0.clone(),
                font_size: 60.0,
                color: Color::WHITE,
            };
            commands.spawn((
                Text2dBundle {
                    text: Text::from_section("wow", text_style)
                        .with_alignment(TextAlignment::CENTER),
                    transform: Transform::from_xyz(x_pos, y_pos, 0.).with_rotation(rotation),
                    ..default()
                },
                CornerMessage,
            ));
        }
    }
}

fn mouse_dragging(
    time: Res<Time>,
    mouse_button_input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut q_sprite: Query<(&mut Speed, &mut Transform, &mut OldPos)>,
) {
    let (mut speed, mut transform, mut old_pos) = q_sprite.single_mut();

    if mouse_button_input.pressed(MouseButton::Left) {
        let (camera, camera_transform) = q_camera.single();
        let window = windows.primary();

        if let Some(screen_pos) = window.cursor_position() {
            old_pos.0 = Some(transform.translation);
            let window_size = Vec2::new(window.width(), window.height());

            // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
            let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

            // matrix for undoing the projection and camera transform
            let ndc_to_world =
                camera_transform.compute_matrix() * camera.projection_matrix().inverse();

            // use it to convert ndc to world-space coordinates
            let mut world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
            world_pos.z = 0.;
            transform.translation = world_pos;
        }
    } else if let Some(pos) = old_pos.0 {
        let new_speed = transform.translation - pos;
        speed.0 = new_speed / time.delta_seconds();
        old_pos.0 = None;
    }
}

fn message_fade(
    mut commands: Commands,
    time: Res<Time>,
    mut q_message: Query<(Entity, &mut Text), With<CornerMessage>>,
) {
    for (entity, mut text) in q_message.iter_mut() {
        let alpha = text.sections[0].style.color.a() - time.delta_seconds() / 5.;
        if alpha <= 0. {
            commands.entity(entity).despawn();
        } else {
            text.sections[0].style.color.set_a(alpha);
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Bevy DVD idle".into(),
                width: 800.,
                height: 800.,
                ..default()
            },
            ..default()
        }))
        .add_startup_system(setup)
        .add_system(sprite_movement)
        .add_system(mouse_dragging)
        .add_system(message_fade)
        .run();
}
