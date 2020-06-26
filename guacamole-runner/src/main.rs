mod systems;
#[allow(dead_code)]
mod consts;
mod map;
mod components;

use components::{
    Player,
    Height,
    Collider,
};

use map::{
    render_hex_map,
};

use consts::*;

use vermarine_lib::{
    rendering::{
        RenderingWorkloadCreator,
        RenderingWorkloadSystems,
        draw_buffer::{
            DrawBuffer,
            DrawCommand,
        },
        Drawables,
        Sprite,
    },
    components::{
        Transform,
    },
    tetra::{
        self,
        ContextBuilder,
        State,
        Context,
        Trans,
        graphics::{
            Color,
            Camera,
            self,
            Canvas,
            DrawParams,
            text::{
                Text,
                Font,
            }
        },
        input::{
            self,
            InputContext,
            Key,
        },
        math::{
            Vec2,
            Vec3,
            Mat4,
        },
    },
    shipyard::{
        self,
        *,
    },
};

type Res = ();

fn main() -> tetra::Result {
    ContextBuilder::new("Guacamole-Runner", 1280, 720)
        .show_mouse(true)
        .build()?
        .run(Game::new, |_| Ok(()))
}

pub struct Game {
    world: World,
    background_canvas: Canvas,
}

impl Game {
    pub fn new(ctx: &mut Context) -> tetra::Result<Self> {
        let world = World::new();
        let mut game = Game {
            world,
            background_canvas: Canvas::new(ctx, 640, 360)
                .expect("Could not make canvas"),
        };

        game.init_world(ctx);

        Ok(game)
    }

    fn init_world(&mut self, ctx: &mut Context) {
        self.world.add_unique(map::HexMap::new(WIDTH, HEIGHT));
        self.world.add_unique((*ctx.input_context()).clone());
        self.world.add_unique(systems::SpawnTimer::new(60));

        self.world
            .add_rendering_workload(ctx)
            .with_rendering_systems()
            .build();

        self.world.run(|mut camera: UniqueViewMut<Camera>| {
            camera.position = Vec2::new(640., 360.);
        });

        let (player_tex, _) = self.world.run(|drawables: NonSendSync<UniqueView<Drawables>>| {
            (
                drawables.alias[textures::PLAYER],
                drawables.alias[textures::AEROPLANE],
            )
        });

        self.world
            .entity_builder()
            .with(Sprite::from_command(
                DrawCommand::new(player_tex)
                .scale(Vec2::new(3., 3.))
                .draw_layer(draw_layers::PLAYER)
            ))
            .with(Transform::new(0., 0.))
            .with(Player {})
            .with(Collider::new(0, 9 * 3, 36 * 3, 20 * 3))
            .with(Height(10.))
            .build();
    }

    fn draw_background(&mut self, ctx: &mut Context) {
        graphics::set_canvas(ctx, &self.background_canvas);
        graphics::clear(ctx, CLEAR_COL);

        self.world.run(render_hex_map);
        self.world.run_with_data(DrawBuffer::flush, ctx);
        graphics::flush(ctx);
        graphics::reset_canvas(ctx);

        graphics::clear(ctx, CLEAR_COL);

        graphics::draw(ctx, &self.background_canvas, 
            DrawParams::new()
            .scale(Vec2::new(2., 2.))
        );
        graphics::flush(ctx);
    }
}

impl State<Res> for Game {
    fn update(&mut self, ctx: &mut Context, _res: &mut Res) -> tetra::Result<Trans<Res>> {
        let input_ctx = ctx.input_context();
        self.world.run(|mut ctx: UniqueViewMut<InputContext>| {
            *ctx = (*input_ctx).clone();
        });

        self.world.run(systems::scroll_map);
        self.world.run(systems::move_player);
        self.world.run(systems::platform_spawner);
        self.world.run(systems::move_planes);
        self.world.run(systems::grow_ground);
        self.world.run(systems::player_platform_check);

        let trans = self.world.run(|player: View<Player>, height: View<Height>| {
            let (_, height) = (&player, &height).iter().next().unwrap();
            if height.0 <= 8.5 {
                Trans::Replace(Box::new(DeadState::new(ctx).unwrap()))
            } else {
                Trans::None
            }
        });

        Ok(trans)
    }

    fn draw(&mut self, ctx: &mut Context, _res: &mut Res) -> tetra::Result {
        self.world.run(|mut draw_buff: UniqueViewMut<DrawBuffer>| {
            draw_buff.transform_mat = Mat4::identity();
        });

        self.draw_background(ctx);

        self.world.run(|mut camera: UniqueViewMut<Camera>, mut draw_buff: UniqueViewMut<DrawBuffer>| {
            camera.update();
            draw_buff.transform_mat = camera.as_matrix();
        });

        self.world.run_workload("Rendering");
        self.world.run_with_data(DrawBuffer::flush, ctx);

        tetra::window::set_title(
            ctx,
            &format!(
                "Guacamole-Runner - {:.0} FPS",
                tetra::time::get_fps(ctx)
            ),
        );

        Ok(())
    }
}


struct DeadState {
    text: Text,
}

impl State<Res> for DeadState {
    fn update(&mut self, ctx: &mut Context, _resources: &mut Res) -> tetra::Result<tetra::Trans<Res>> {
        if input::is_key_down(ctx.input_context(), Key::Space) {
            return Ok(Trans::Switch(Box::new(Game::new(ctx)?)));
        }

        Ok(Trans::None)
    }

    fn draw(&mut self, ctx: &mut Context, _resources: &mut Res) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.45, 0.65, 1.0));
        graphics::draw(ctx, &self.text, Vec2::new(550., 300.));

        Ok(())
    }
}

impl DeadState {
    pub fn new(ctx: &mut Context) -> tetra::Result<Self> {
        Ok(Self {
            text: Text::new(
                
"
         You died :<
Press <SPACEBAR> to restart
",
                Font::vector(ctx, "./assets/DejaVuSansMono.ttf", 16.0)?
            )
        })
    }
}