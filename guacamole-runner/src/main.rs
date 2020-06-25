mod systems;
#[allow(dead_code)]
mod consts;
mod map;
mod components;

use components::{
    Player,
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
            Camera,
            self,
            Canvas,
            DrawParams,
        },
        input::{
            InputContext,
        },
        math::{
            Vec2,
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
        self.world.add_unique(systems::CamScroller::new(0));

        self.world
            .add_rendering_workload(ctx)
            .with_rendering_systems()
            .build();


        let sprite = self.world.borrow::<NonSendSync<UniqueView<Drawables>>>().alias[textures::FLOOR_BRICK];
        let _player = self.world
            .entity_builder()
            .with(Sprite::new(sprite))
            .with(Transform::new(0., 0.))
            .with(Player {})
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

        Ok(Trans::None)
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