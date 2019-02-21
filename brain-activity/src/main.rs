extern crate cgmath;
extern crate piston_window;
extern crate psyche_core;
extern crate psyche_serde;

use cgmath::*;
use piston_window::*;
use psyche_core::brain::Brain;
use psyche_core::brain_builder::BrainBuilder;
use psyche_core::config::Config;
use psyche_core::neuron::Position;
use psyche_core::Scalar;
use std::time::Instant;

fn point(point: Position, rot: &Quaternion<Scalar>) -> (Scalar, Scalar) {
    let p = Point3::new(point.x, point.y, point.z);
    let p = rot.rotate_point(p);
    (p.x, p.y)
}

fn connection_into_line(pair: &(Position, Position), rot: &Quaternion<Scalar>) -> [f64; 4] {
    let p0 = Point3::new(pair.0.x, pair.0.y, pair.0.z);
    let p1 = Point3::new(pair.1.x, pair.1.y, pair.1.z);
    let p0 = rot.rotate_point(p0);
    let p1 = rot.rotate_point(p1);
    [p0.x, p0.y, p1.x, p1.y]
}

fn impulse_into_point(
    impulse: &(Position, Position, Scalar),
    rot: &Quaternion<Scalar>,
) -> (Scalar, Scalar) {
    point(
        Position {
            x: (impulse.1.x - impulse.0.x) * impulse.2 + impulse.0.x,
            y: (impulse.1.y - impulse.0.y) * impulse.2 + impulse.0.y,
            z: (impulse.1.z - impulse.0.z) * impulse.2 + impulse.0.z,
        },
        rot,
    )
}

fn make_brain() -> Brain {
    let mut config = Config::default();
    config.propagation_speed = 50.0;
    config.synapse_inactivity_time = 0.25;
    config.synapse_reconnection_range = 20.0;

    BrainBuilder::new()
        .config(config)
        .neurons(750)
        .connections(750)
        .min_neurogenesis_range(5.0)
        .max_neurogenesis_range(20.0)
        .radius(50.0)
        .sensors(50)
        .effectors(25)
        .build()
}

fn main() {
    let mut window: PistonWindow =
        WindowSettings::new("Psyche - Brain Activity Visualizer", [600, 600])
            .exit_on_esc(true)
            .build()
            .unwrap();

    // let mut brain =
    //     psyche_serde::bytes::brain_from_bytes(&::std::fs::read("./brain.bin").unwrap()).unwrap();
    let mut brain = make_brain();
    // ::std::fs::write(
    //     "./brain.bin",
    //     &psyche_serde::bytes::brain_to_bytes(&brain).unwrap(),
    // )
    // .unwrap_or(());
    brain.ignite_random_synapses(brain.synapses_count());

    let vx = 300.0;
    let vy = 300.0;
    let zoom = 5.0;
    let thickness = 0.5 / zoom;
    let mut sensor_impulse_accum = 0.0;
    let mut processing = false;
    let mut rendering = true;
    let mut hold_rot_x = 0.0;
    let mut hold_rot_y = 0.0;
    let mut rot_x = 0.0;
    let mut rot_y = 0.0;
    let rot_speed = 30.0;
    let mut rot = Quaternion::zero();
    let trigger_sensors = false;
    let trigger_sensors_delay = 0.25;

    window.set_max_fps(30);
    window.set_ups(20);
    while let Some(e) = window.next() {
        if let Event::Input(input) = &e {
            if let Input::Button(button) = input {
                if let Button::Keyboard(key) = button.button {
                    match key {
                        keyboard::Key::Space => {
                            if let ButtonState::Press = button.state {
                                processing = !processing;
                            }
                        }
                        keyboard::Key::R => {
                            if let ButtonState::Press = button.state {
                                rendering = !rendering;
                            }
                        }
                        keyboard::Key::W => match button.state {
                            ButtonState::Press => hold_rot_y = -1.0,
                            ButtonState::Release => hold_rot_y = 0.0,
                        },
                        keyboard::Key::S => match button.state {
                            ButtonState::Press => hold_rot_y = 1.0,
                            ButtonState::Release => hold_rot_y = 0.0,
                        },
                        keyboard::Key::A => match button.state {
                            ButtonState::Press => hold_rot_x = -1.0,
                            ButtonState::Release => hold_rot_x = 0.0,
                        },
                        keyboard::Key::D => match button.state {
                            ButtonState::Press => hold_rot_x = 1.0,
                            ButtonState::Release => hold_rot_x = 0.0,
                        },
                        keyboard::Key::Return => {
                            if let ButtonState::Press = button.state {
                                for sensor in brain.get_sensors() {
                                    brain
                                        .sensor_trigger_impulse(sensor, 10.0, 0.0)
                                        .unwrap_or(());
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if let Some(args) = e.update_args() {
            let dt = args.dt;
            rot_x += dt * hold_rot_x * rot_speed;
            rot_y += dt * hold_rot_y * rot_speed;
            rot_y = rot_y.max(-90.0).min(90.0);
            if processing {
                if trigger_sensors {
                    sensor_impulse_accum += dt;
                    if sensor_impulse_accum > trigger_sensors_delay {
                        sensor_impulse_accum = 0.0;
                        for sensor in brain.get_sensors() {
                            brain
                                .sensor_trigger_impulse(sensor, 10.0, 0.0)
                                .unwrap_or(());
                        }
                    }
                }
                let now = Instant::now();
                brain.process_parallel(dt).unwrap_or(());
                println!("processing: {:?}", now.elapsed());
            }
        }

        if e.render_args().is_some() {
            window.draw_2d(&e, |c, g| {
                clear([0.0, 0.0, 0.0, 1.0], g);
                if !rendering {
                    return;
                }
                rot = Quaternion::from(Euler {
                    x: Deg(rot_y),
                    y: Deg(rot_x),
                    z: Deg(0.0),
                });
                let now = Instant::now();
                let activity = brain.build_activity_map_parallel();
                println!("building activity map: {:?}", now.elapsed());
                let transform = c.transform.trans(vx, vy).zoom(zoom);
                for connection in &activity.connections {
                    line(
                        [0.0, 0.0, 1.0, 0.2],
                        thickness,
                        connection_into_line(connection, &rot),
                        transform,
                        g,
                    );
                }
                for impulse in &activity.impulses {
                    let (x, y) = impulse_into_point(impulse, &rot);
                    rectangle(
                        [1.0, 1.0, 1.0, 0.5],
                        rectangle::square(x, y, thickness * 2.0),
                        transform,
                        g,
                    );
                }
                for sensor in &activity.sensors {
                    let (x, y) = point(*sensor, &rot);
                    rectangle(
                        [1.0, 1.0, 0.0, 1.0],
                        rectangle::square(x, y, thickness * 4.0),
                        transform,
                        g,
                    );
                }
                for effector in &activity.effectors {
                    let (x, y) = point(*effector, &rot);
                    rectangle(
                        [1.0, 0.0, 0.0, 1.0],
                        rectangle::square(x, y, thickness * 4.0),
                        transform,
                        g,
                    );
                }
            });
        }
    }
}
