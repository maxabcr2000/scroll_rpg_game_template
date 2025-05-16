use bevy::prelude::*;
use bevy_sprite3d::prelude::*;
use avian3d::prelude::*;

use bevy_tnua::{
    builtins::TnuaBuiltinJumpState, prelude::*, TnuaAnimatingState, TnuaAnimatingStateDirective,
};
use bevy_tnua_avian3d::*;

#[derive(States, Hash, Clone, PartialEq, Eq, Debug, Default)]
enum GameState { #[default] Loading, Ready }

// #[derive(Component, PartialEq, Eq)]
// enum PlayerState {
//     Idle,
//     Moving,
// }

#[derive(Component)]
struct Player;

#[derive(Resource, Default)]
struct ImageAssets {
    image: Handle<Image>,               // the `image` field here is only used to query the load state, lots of the
    layout: Handle<TextureAtlasLayout>, // code in this file disappears if something like bevy_asset_loader is used.
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(Sprite3dPlugin)
        .init_state::<GameState>()

        // initially load assets
        .add_systems(Startup, |asset_server:         Res<AssetServer>,
                               mut assets:           ResMut<ImageAssets>,
                               mut texture_atlases:  ResMut<Assets<TextureAtlasLayout>>| {

            assets.image = asset_server.load("gabe-idle-run.png");
            assets.layout = texture_atlases.add(
                TextureAtlasLayout::from_grid(UVec2::new(24, 24), 7, 1, None, None)
            );
        })

        // run `setup` every frame while loading. Once it detects the right
        // conditions it'll switch to the next state.
        // #NOTE: this will be replaced by Tnua systems
        // .add_systems(Update, setup.run_if(in_state(GameState::Loading)))

        // every frame, animate the sprite
        // .add_systems(Update, animate_sprite.run_if(in_state(GameState::Ready)))
        
        // #NOTE: this will be replaced by a Tnua system
        // .add_systems(Update, player_movement_system.run_if(in_state(GameState::Ready)))

        .insert_resource(ImageAssets::default());


    app        
        .add_plugins((
            PhysicsPlugins::default(),
            TnuaControllerPlugin::new(FixedUpdate),
            TnuaAvian3dPlugin::new(FixedUpdate),
        ))
        .add_systems(
            Startup,
            (setup_camera_and_lights, setup_level),
        )
        //#NOTE: We need to run setup_player under Update schedule, or it will spawn more than one player entity
        .add_systems(Update, (setup_player).run_if(in_state(GameState::Loading)))
        .add_systems(
            FixedUpdate,
            (
                apply_controls.in_set(TnuaUserControlsSystemSet),
                animate_sprite,
                face_player_to_camera,
                // prepare_animations,
                // handle_animating,
            ).run_if(in_state(GameState::Ready))
        );

    app.run();

}

// fn setup(
//     asset_server      : Res<AssetServer>,
//     assets            : Res<ImageAssets>,
//     mut commands      : Commands,
//     mut next_state    : ResMut<NextState<GameState>>,
//     mut sprite_params : Sprite3dParams
// ) {

//     // poll every frame to check if assets are loaded. Once they are, we can proceed with setup.
//     if !asset_server.get_load_state(assets.image.id()).is_some_and(|s| s.is_loaded()) { return; }
//     next_state.set(GameState::Ready);

//     // -----------------------------------------------------------------------

//     commands.spawn(Camera3d::default()).insert(Transform::from_xyz(0., 0., 5.));

//     // -------------------- Spawn a 3D atlas sprite --------------------------

//     let texture_atlas = TextureAtlas {
//         layout: assets.layout.clone(),
//         index: 3,
//     };

//     commands.spawn(Sprite3dBuilder {
//             image: assets.image.clone(),
//             pixels_per_metre: 32.,
//             alpha_mode: AlphaMode::Blend,
//             unlit: true,
//             // pivot: Some(Vec2::new(0.5, 0.5)),
//             ..default()
//     }.bundle_with_atlas(&mut sprite_params, texture_atlas))
//     .insert(AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))
//     .insert(Player)
//     .insert(PlayerState::Idle);

//     // -----------------------------------------------------------------------
// }


fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut Sprite3d, &TnuaAnimatingState<AnimationState>)>,
) {

    for (mut timer, mut sprite_3d, state) in query.iter_mut() {
        //use match patter to implement different logic for different animation states
        match state.get() {
            Some(AnimationState::Running(_)) => {
                println!("Running");
                timer.tick(time.delta());
                if timer.just_finished() {
                    let length = sprite_3d.texture_atlas_keys.as_ref().unwrap().len();
                    let atlas = sprite_3d.texture_atlas.as_mut().unwrap();
                    atlas.index = (atlas.index + 1) % length;
                }
            }
            Some(AnimationState::Standing) => {
                println!("Standing");
                // 重設為第0幀
                let atlas = sprite_3d.texture_atlas.as_mut().unwrap();
                atlas.index = 0;
            }
            Some(AnimationState::Jumping) => {
                println!("Jumping");
                // 重設為第0幀
                let atlas = sprite_3d.texture_atlas.as_mut().unwrap();
                atlas.index = 0;
            }
            Some(AnimationState::Falling) => {
                println!("Falling");
                // 重設為第0幀
                let atlas = sprite_3d.texture_atlas.as_mut().unwrap();
                atlas.index = 0;
            }
            None => ()
        }
    }
}

// fn player_movement_system(
//     keyboard_input: Res<ButtonInput<KeyCode>>,
//     mut query: Query<(&mut Transform, &mut PlayerState), With<Player>>,
//     time: Res<Time>,
// ) {
//     for (mut transform, mut state) in query.iter_mut() {
//         let mut direction = 0.0;
//         if keyboard_input.pressed(KeyCode::ArrowLeft) {
//             direction -= 1.0;
//         }
//         if keyboard_input.pressed(KeyCode::ArrowRight) {
//             direction += 1.0;
//         }

//         if direction != 0.0 {
//             transform.translation.x += direction * 0.5 * time.delta_secs();
//             transform.scale.x = direction.signum();
//             *state = PlayerState::Moving;
//         } else {
//             *state = PlayerState::Idle;
//         }
//     }
// }

// No Tnua-related setup here - this is just normal Bevy stuff.
fn setup_camera_and_lights(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 16.0, 40.0).looking_at(Vec3::new(0.0, 10.0, 0.0), Vec3::Y),
    ));

    commands.spawn((PointLight::default(), Transform::from_xyz(5.0, 5.0, 5.0)));

    // A directly-down light to tell where the player is going to land.
    commands.spawn((
        DirectionalLight {
            illuminance: 4000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::default().looking_at(-Vec3::Y, Vec3::Z),
    ));
}

// No Tnua-related setup here - this is just normal Bevy (and Avian) stuff.
fn setup_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    println!("setup_level");

    // Spawn the ground.
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(128.0, 128.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        RigidBody::Static,
        Collider::half_space(Vec3::Y),
    ));
}

fn setup_player(mut commands: Commands, 
    asset_server: Res<AssetServer>,
    assets            : Res<ImageAssets>,
    mut sprite_params : Sprite3dParams,
    mut next_state    : ResMut<NextState<GameState>>,
) {
    // poll every frame to check if assets are loaded. Once they are, we can proceed with setup.
    if !asset_server.get_load_state(assets.image.id()).is_some_and(|s| s.is_loaded()) { return; }
    next_state.set(GameState::Ready);

    // We'll need this in `prepare_animations` to build the animation graph.
    // commands.insert_resource(PlayerGltfHandle(asset_server.load("player.glb")));

    let texture_atlas = TextureAtlas {
        layout: assets.layout.clone(),
        index: 3,
    };

    // -------------------- Spawn a 3D atlas sprite --------------------------
    println!("spawn_player");
    commands.spawn((
        Transform::from_xyz(0.0, 2.0, 0.0),
        // We'll need this in the `handle_animating` system to keep track of the players animating
        // state.
        TnuaAnimatingState::<AnimationState>::default(),
        // The player character needs to be configured as a dynamic rigid body of the physics
        // engine.
        RigidBody::Dynamic,
        Collider::capsule(0.5, 1.0),
        // This is Tnua's interface component.
        TnuaController::default(),
        // A sensor shape is not strictly necessary, but without it we'll get weird results.
        TnuaAvian3dSensorShape(Collider::cylinder(0.49, 0.0)),
        // Tnua can fix the rotation, but the character will still get rotated before it can do so.
        // By locking the rotation we can prevent this.
        LockedAxes::ROTATION_LOCKED.unlock_rotation_y(),
    ))
    .insert(        Sprite3dBuilder {
            image: assets.image.clone(),
            pixels_per_metre: 16.,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            // pivot: Some(Vec2::new(0.5, 0.5)),
            ..default()
        }.bundle_with_atlas(&mut sprite_params, texture_atlas))
    .insert(AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))
    .insert(Player);

}

// This enum projects the player's state into something we can use to decide which animation to
// play. Each variant of this enum corresponds to an animation, and the variant data can affect the
// animation's parameters.
//
// By itself this does not do much, but we can attach a `TnuaAnimatingState<AnimationState>`
// component to the player entity and use it to track the animating state.
pub enum AnimationState {
    Standing,
    Running(f32),
    Jumping,
    Falling,
}

// Bevy's animation handling is a bit manual. We'll use this struct to register the animation clips
// as nodes in the animation graph.
// #[derive(Resource)]
// struct AnimationNodes {
//     standing: AnimationNodeIndex,
//     running: AnimationNodeIndex,
//     jumping: AnimationNodeIndex,
//     falling: AnimationNodeIndex,
// }

// This is the important system for this example
// fn handle_animating(
//     mut player_query: Query<(&TnuaController, &mut TnuaAnimatingState<AnimationState>)>,
//     mut animation_player_query: Query<&mut AnimationPlayer>,
//     animation_nodes: Option<Res<AnimationNodes>>,
// ) {
//     // An actual game should match the animation player and the controller. Here we cheat for
//     // simplicity and use the only controller and only player.
//     let Ok((controller, mut animating_state)) = player_query.single_mut() else {
//         return;
//     };
//     let Ok(mut animation_player) = animation_player_query.single_mut() else {
//         return;
//     };
//     let Some(animation_nodes) = animation_nodes else {
//         return;
//     };

//     // Here we use the data from TnuaController to determine what the character is currently doing,
//     // so that we can later use that information to decide which animation to play.

//     // First we look at the `action_name` to determine which action (if at all) the character is
//     // currently performing:
//     let current_status_for_animating = match controller.action_name() {
//         // Unless you provide the action names yourself, prefer matching against the `NAME` const
//         // of the `TnuaAction` trait. Once `type_name` is stabilized as `const` Tnua will use it to
//         // generate these names automatically, which may result in a change to the name.
//         Some(TnuaBuiltinJump::NAME) => {
//             // In case of jump, we want to cast it so that we can get the concrete jump state.
//             let (_, jump_state) = controller
//                 .concrete_action::<TnuaBuiltinJump>()
//                 .expect("action name mismatch");
//             // Depending on the state of the jump, we need to decide if we want to play the jump
//             // animation or the fall animation.
//             match jump_state {
//                 TnuaBuiltinJumpState::NoJump => return,
//                 TnuaBuiltinJumpState::StartingJump { .. } => AnimationState::Jumping,
//                 TnuaBuiltinJumpState::SlowDownTooFastSlopeJump { .. } => AnimationState::Jumping,
//                 TnuaBuiltinJumpState::MaintainingJump { .. } => AnimationState::Jumping,
//                 TnuaBuiltinJumpState::StoppedMaintainingJump => AnimationState::Jumping,
//                 TnuaBuiltinJumpState::FallSection => AnimationState::Falling,
//             }
//         }
//         // Tnua should only have the `action_name` of the actions you feed to it. If it has
//         // anything else - consider it a bug.
//         Some(other) => panic!("Unknown action {other}"),
//         // No action name means that no action is currently being performed - which means the
//         // animation should be decided by the basis.
//         None => {
//             // If there is no action going on, we'll base the animation on the state of the
//             // basis.
//             let Some((_, basis_state)) = controller.concrete_basis::<TnuaBuiltinWalk>() else {
//                 // Since we only use the walk basis in this example, if we can't get get this
//                 // basis' state it probably means the system ran before any basis was set, so we
//                 // just stkip this frame.
//                 return;
//             };
//             if basis_state.standing_on_entity().is_none() {
//                 // The walk basis keeps track of what the character is standing on. If it doesn't
//                 // stand on anything, `standing_on_entity` will be empty - which means the
//                 // character has walked off a cliff and needs to fall.
//                 AnimationState::Falling
//             } else {
//                 let speed = basis_state.running_velocity.length();
//                 if 0.01 < speed {
//                     AnimationState::Running(0.1 * speed)
//                 } else {
//                     AnimationState::Standing
//                 }
//             }
//         }
//     };

//     let animating_directive = animating_state.update_by_discriminant(current_status_for_animating);

//     match animating_directive {
//         TnuaAnimatingStateDirective::Maintain { state } => {
//             // `Maintain` means that we did not switch to a different variant, so there is no need
//             // to change animations.

//             // Specifically for the running animation, even when the state remains the speed can
//             // still change. When it does, we simply need to update the speed in the animation
//             // player.
//             if let AnimationState::Running(speed) = state {
//                 if let Some(animation) = animation_player.animation_mut(animation_nodes.running) {
//                     animation.set_speed(*speed);
//                 }
//             }
//         }
//         TnuaAnimatingStateDirective::Alter {
//             old_state: _,
//             state,
//         } => {
//             // `Alter` means that we have switched to a different variant and need to play a
//             // different animation.

//             // First - stop the currently running animation. We don't check which one is running
//             // here because we just assume it belongs to the old state, but more sophisticated code
//             // can try to phase from the old animation to the new one.
//             animation_player.stop_all();

//             // Depending on the new state, we choose the animation to run and its parameters (here
//             // they are the speed and whether or not to repeat)
//             match state {
//                 AnimationState::Standing => {
//                     animation_player
//                         .start(animation_nodes.standing)
//                         .set_speed(1.0)
//                         .repeat();
//                 }
//                 AnimationState::Running(speed) => {
//                     animation_player
//                         .start(animation_nodes.running)
//                         // The running animation, in particular, has a speed that depends on how
//                         // fast the character is running. Note that if the speed changes while the
//                         // character is still running we won't get `Alter` again - so it's
//                         // important to also update the speed in `Maintain { State: Running }`.
//                         .set_speed(*speed)
//                         .repeat();
//                 }
//                 AnimationState::Jumping => {
//                     animation_player
//                         .start(animation_nodes.jumping)
//                         .set_speed(2.0);
//                 }
//                 AnimationState::Falling => {
//                     animation_player
//                         .start(animation_nodes.falling)
//                         .set_speed(1.0);
//                 }
//             }
//         }
//     }
// }

fn apply_controls(keyboard: Res<ButtonInput<KeyCode>>, mut query: Query<&mut TnuaController>) {
    let Ok(mut controller) = query.single_mut() else {
        return;
    };

    let mut direction = Vec3::ZERO;

    // if keyboard.pressed(KeyCode::ArrowUp) {
    //     direction -= Vec3::Z;
    // }
    // if keyboard.pressed(KeyCode::ArrowDown) {
    //     direction += Vec3::Z;
    // }
    if keyboard.pressed(KeyCode::ArrowLeft) {
        direction -= Vec3::X;
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        direction += Vec3::X;
    }

    // Feed the basis every frame. Even if the player doesn't move - just use `desired_velocity:
    // Vec3::ZERO`. `TnuaController` starts without a basis, which will make the character collider
    // just fall.
    controller.basis(TnuaBuiltinWalk {
        // The `desired_velocity` determines how the character will move.
        desired_velocity: direction.normalize_or_zero() * 10.0,
        desired_forward: Dir3::new(direction).ok(),
        // The `float_height` must be greater (even if by little) from the distance between the
        // character's center and the lowest point of its collider.
        float_height: 2.0,
        // `TnuaBuiltinWalk` has many other fields for customizing the movement - but they have
        // sensible defaults. Refer to the `TnuaBuiltinWalk`'s documentation to learn what they do.
        ..Default::default()
    });

    // Feed the jump action every frame as long as the player holds the jump button. If the player
    // stops holding the jump button, simply stop feeding the action.
    if keyboard.pressed(KeyCode::Space) {
        controller.action(TnuaBuiltinJump {
            // The height is the only mandatory field of the jump button.
            height: 4.0,
            // `TnuaBuiltinJump` also has customization fields with sensible defaults.
            ..Default::default()
        });
    }
}

// No Tnua-related setup here - this is just for dealing with Bevy's animation graph.
// fn prepare_animations(
//     handle: Option<Res<PlayerGltfHandle>>,
//     gltf_assets: Res<Assets<Gltf>>,
//     mut commands: Commands,
//     animation_player_query: Query<Entity, With<AnimationPlayer>>,
//     mut animation_graphs_assets: ResMut<Assets<AnimationGraph>>,
// ) {
//     let Some(handle) = handle else { return };
//     let Some(gltf) = gltf_assets.get(&handle.0) else {
//         return;
//     };
//     let Ok(animation_player_entity) = animation_player_query.single() else {
//         return;
//     };

//     let mut graph = AnimationGraph::new();
//     let root_node = graph.root;

//     commands.insert_resource(AnimationNodes {
//         standing: graph.add_clip(gltf.named_animations["Standing"].clone(), 1.0, root_node),
//         running: graph.add_clip(gltf.named_animations["Running"].clone(), 1.0, root_node),
//         jumping: graph.add_clip(gltf.named_animations["Jumping"].clone(), 1.0, root_node),
//         falling: graph.add_clip(gltf.named_animations["Falling"].clone(), 1.0, root_node),
//     });

//     commands
//         .entity(animation_player_entity)
//         .insert(AnimationGraphHandle(animation_graphs_assets.add(graph)));

//     // So that we won't run this again
//     commands.remove_resource::<PlayerGltfHandle>();
// }

fn face_player_to_camera(
    camera_query: Query<&Transform, (With<Camera3d>, Without<Player>)>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    let Ok(camera_transform) = camera_query.get_single() else { return; };
    let Ok(mut player_transform) = player_query.get_single_mut() else { return; };

    // 只考慮 XZ 平面上的朝向
    let player_pos = player_transform.translation;
    let camera_pos = camera_transform.translation;

    let mut look_dir = camera_pos - player_pos;
    look_dir.y = 0.0;
    if look_dir.length_squared() > 0.0001 {
        player_transform.look_at(camera_pos, Vec3::Y);
    }
}