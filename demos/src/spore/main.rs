extern crate bincode;
extern crate cgmath;
extern crate nalgebra;
extern crate ncollide2d;
extern crate nphysics2d;
extern crate piston_window;
extern crate psyche;
extern crate rand;

mod managers;
mod world;

use crate::managers::items_manager::ItemsManager;
use crate::managers::physics_manager::body::Vec2;
use crate::managers::renderables_manager::renderable::Graphics;
use crate::world::world_builder::WorldBuilder;
use clap::{App, Arg};
use piston_window::*;
use psyche::core::brain_builder::BrainBuilder;
use psyche::core::config::Config;
use psyche::core::Scalar;
use std::ops::Range;

const WORLD_SIZE: [u32; 2] = [800, 600];
const RANDOMIZED_FLUID: Scalar = 10.0;
const FLUID_DIFFUSE: Scalar = 0.1;
const FLUID_DRAG: Scalar = 0.1;
const FLUID_RESOLUTION: usize = 20;
const SPORES_COUNT: usize = 2;
const SPORES_RADIUS: Range<Scalar> = 15.0..30.0;
const FOOD_COUNT: usize = 40;
const FOOD_CALORIES: Range<Scalar> = 50.0..100.0;

fn main() {
    let matches = App::new("Spores")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Spores Evolution Simulator")
        .arg(
            Arg::with_name("headless")
                .short("h")
                .long("headless")
                .help("Headless mode"),
        )
        // .arg(
        //     Arg::with_name("snapshot")
        //         .short("s")
        //         .long("snapshot")
        //         .value_name("FILE")
        //         .help("World snapshot file path")
        //         .takes_value(true)
        //         .required(false),
        // )
        .get_matches();

    let mut config = Config::default();
    config.propagation_speed = 50.0;
    config.synapse_reconnection_range = Some(15.0);
    config.neuron_potential_decay = 0.1;
    config.synapse_propagation_decay = 0.01;
    config.synapse_new_connection_receptors = Some(2.0);
    let builder = BrainBuilder::new()
        .config(config)
        .neurons(100)
        .connections(200)
        .min_neurogenesis_range(5.0)
        .max_neurogenesis_range(15.0)
        .radius(30.0)
        .sensors(10)
        .effectors(10);

    if matches.is_present("headless") {
        main_headless(builder);
    } else {
        main_visual(builder);
    }
}

fn main_headless(builder: BrainBuilder) {
    let size = (Scalar::from(WORLD_SIZE[0]), Scalar::from(WORLD_SIZE[1]));
    let dt = 1.0 / 20.0;

    let mut world = WorldBuilder::new()
        .size(size)
        .grid_cols_rows((
            WORLD_SIZE[0] as usize / FLUID_RESOLUTION,
            WORLD_SIZE[1] as usize / FLUID_RESOLUTION,
        ))
        .randomized_fluid(RANDOMIZED_FLUID)
        .fluid_diffuse(FLUID_DIFFUSE)
        .fluid_drag(FLUID_DRAG)
        .spores_count(SPORES_COUNT)
        .spores_radius(SPORES_RADIUS)
        .spores_brain_builder(builder)
        .food_count(FOOD_COUNT)
        .food_calories(FOOD_CALORIES)
        .build();

    loop {
        world.process(dt);
    }
}

fn main_visual(builder: BrainBuilder) {
    let mut window: PistonWindow = WindowSettings::new("Spores Evolution Simulator", WORLD_SIZE)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let size = (window.size().width, window.size().height);
    window.set_max_fps(60);
    window.set_ups(20);
    let mut world = WorldBuilder::new()
        .size(size)
        .grid_cols_rows((
            size.0 as usize / FLUID_RESOLUTION,
            size.1 as usize / FLUID_RESOLUTION,
        ))
        .randomized_fluid(RANDOMIZED_FLUID)
        .fluid_diffuse(FLUID_DIFFUSE)
        .fluid_drag(FLUID_DRAG)
        .spores_count(SPORES_COUNT)
        .spores_radius(SPORES_RADIUS)
        .spores_brain_builder(builder)
        .food_count(FOOD_COUNT)
        .food_calories(FOOD_CALORIES)
        .build_and_setup(|world| {
            let water = world.renderables_mut().create_with(|renderable, _| {
                renderable.depth = -1.0;
                renderable.transform.position = (size.0 * 0.5, size.1 * 0.5).into();
                renderable.graphics = Graphics::Rectangle([0.0, 0.0, 0.5, 1.0], size.into());
            });
            let water = ("water", water).into();
            let food = "food".into();
            let spores = "spores".into();
            world
                .renderables_mut()
                .set_root(Some(vec![water, food, spores].into()));
        });

    let mut dragging = false;
    let mut mouse_pos = (0.0, 0.0);
    let mut last_mouse_pos = mouse_pos;
    while let Some(e) = window.next() {
        if let Event::Input(input) = &e {
            match input {
                Input::Button(button) => {
                    if let Button::Mouse(mouse) = button.button {
                        if let mouse::MouseButton::Left = mouse {
                            match button.state {
                                ButtonState::Press => {
                                    dragging = true;
                                }
                                ButtonState::Release => {
                                    dragging = false;
                                }
                            }
                        }
                    }
                }
                Input::Move(motion) => {
                    if let Motion::MouseCursor(x, y) = motion {
                        mouse_pos = (*x, *y);
                    }
                }
                _ => {}
            }
        }

        if let Some(args) = e.update_args() {
            let dt = args.dt;
            world.process(dt);

            if dragging {
                world.physics_mut().apply_fluid_force(
                    Vec2::new(mouse_pos.0, mouse_pos.1),
                    Vec2::new(
                        mouse_pos.0 - last_mouse_pos.0,
                        mouse_pos.1 - last_mouse_pos.1,
                    ) * 5.0,
                );
            }
            last_mouse_pos = mouse_pos;
        }

        if e.render_args().is_some() {
            window.draw_2d(&e, |context, gfx| {
                clear([0.0, 0.0, 0.0, 1.0], gfx);
                world.renderables_mut().refresh();
                world.renderables().render(context, gfx);
            });
        }
    }
}
