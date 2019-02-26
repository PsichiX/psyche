extern crate cgmath;
extern crate piston_window;
extern crate psyche;
extern crate rand;

use cgmath::*;
use piston_window::*;
use psyche::core::brain::activity;
use psyche::core::brain::{Brain, BrainActivityStats};
use psyche::core::brain_builder::BrainBuilder;
use psyche::core::config::Config;
use psyche::core::neuron::Position;
use psyche::core::Scalar;
use psyche::graphics::obj::generate;
use rand::{thread_rng, Rng};
use std::time::Instant;

fn make_brain() -> Brain {
    let mut config = Config::default();
    config.propagation_speed = 50.0;
    config.synapse_reconnection_range = Some(15.0);
    // config.synapse_overdose_receptors = Some(10.0);
    config.neuron_potential_decay = 0.1;
    config.synapse_propagation_decay = 0.01;
    config.synapse_new_connection_receptors = Some(2.0);
    // config.action_potential_treshold = 0.1;

    BrainBuilder::new()
        .config(config)
        .neurons(600)
        .connections(1000)
        .min_neurogenesis_range(5.0)
        .max_neurogenesis_range(15.0)
        .radius(50.0)
        .sensors(50)
        .effectors(25)
        .build()
    // let mut brain = psyche::core::brain::Brain::new();
    // brain.set_config(config);
    // let n0 = brain.create_neuron(Position { x: 0.0, y: 0.0, z: 0.0 });
    // let n1 = brain.create_neuron(Position { x: -20.0, y: -20.0, z: 0.0 });
    // let n2 = brain.create_neuron(Position { x: 20.0, y: -20.0, z: 0.0 });
    // let n3 = brain.create_neuron(Position { x: 20.0, y: 20.0, z: 0.0 });
    // let n4 = brain.create_neuron(Position { x: -20.0, y: 20.0, z: 0.0 });
    // drop(brain.create_sensor(n0));
    // drop(brain.bind_neurons(n0, n1));
    // drop(brain.bind_neurons(n1, n2));
    // drop(brain.bind_neurons(n2, n3));
    // drop(brain.bind_neurons(n3, n4));
    // drop(brain.bind_neurons(n4, n1));
    // brain
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
    drop(::std::fs::write(
        "./brain.bin",
        &psyche::serde::bytes::brain_to_bytes(&brain).unwrap(),
    ));
    // brain.ignite_random_synapses(brain.synapses_count(), (1.0, 1.0));

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
    let rot_speed = 45.0;
    let mut rot = Quaternion::zero();
    let mut trigger_sensors = true;
    let trigger_sensors_delay = 0.1;
    let activity_flags = activity::CONNECTIONS | activity::IMPULSES;
    // let activity_flags =
    //     activity::CONNECTIONS | activity::IMPULSES | activity::SENSORS | activity::EFFECTORS;
    let mut activity_map = Default::default();
    let mut activity_dirty = true;
    let mut activity_stats = false;
    let fps = 60;
    let sensor_potential = 5.0;

    window.set_max_fps(fps);
    window.set_ups(fps);
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
                        keyboard::Key::T => {
                            if let ButtonState::Press = button.state {
                                trigger_sensors = !trigger_sensors;
                            }
                        }
                        keyboard::Key::Y => {
                            if let ButtonState::Press = button.state {
                                activity_stats = !activity_stats;
                            }
                        }
                        keyboard::Key::P => {
                            if let ButtonState::Press = button.state {
                                if let Ok(bytes) = generate(
                                    &brain.build_activity_map(activity::ALL),
                                    &Default::default(),
                                ) {
                                    drop(::std::fs::write("./activity.obj", &bytes));
                                }
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
                                    drop(brain.sensor_trigger_impulse(sensor, sensor_potential));
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
                let now = Instant::now();
                if trigger_sensors {
                    sensor_impulse_accum += dt;
                    if sensor_impulse_accum > trigger_sensors_delay {
                        sensor_impulse_accum = 0.0;
                        let mut rng = thread_rng();
                        for sensor in brain.get_sensors() {
                            if rng.gen() {
                                drop(brain.sensor_trigger_impulse(sensor, sensor_potential));
                            }
                        }
                    }
                }
                drop(brain.process(dt));
                activity_dirty = true;
                println!("processing: {:?}", now.elapsed());
                println!(
                    "delta_time: {:?} / {:?} ({:?})",
                    (1.0 / dt) as usize,
                    fps,
                    dt
                );
                if activity_stats {
                    print_stats(brain.build_activity_stats());
                }
            }
            if activity_dirty {
                activity_map = brain.build_activity_map(activity_flags);
                activity_dirty = false;
            }
        }

        if e.render_args().is_some() {
            window.draw_2d(&e, |c, g| {
                if !rendering {
                    return;
                }
                clear([0.0, 0.0, 0.0, 1.0], g);
                let now = Instant::now();
                rot = Quaternion::from(Euler {
                    x: Deg(rot_y),
                    y: Deg(rot_x),
                    z: Deg(0.0),
                });
                let transform = c.transform.trans(vx, vy).zoom(zoom);
                let f = brain.config().default_receptors.1;
                for connection in &activity_map.connections {
                    line(
                        [0.0, 0.0, 1.0, (connection.2 / f) as f32 * 0.1],
                        thickness,
                        connection_into_line(connection, &rot),
                        transform,
                        g,
                    );
                }
                for impulse in &activity_map.impulses {
                    let (x, y) = impulse_into_point(impulse, &rot);
                    rectangle(
                        [0.75, 0.75, 1.0, 0.5],
                        rectangle::square(x, y, thickness * 2.0),
                        transform,
                        g,
                    );
                }
                for neuron in &activity_map.neurons {
                    let (x, y) = point(*neuron, &rot);
                    rectangle(
                        [1.0, 0.0, 1.0, 0.5],
                        rectangle::square(x, y, thickness * 2.0),
                        transform,
                        g,
                    );
                }
                for sensor in &activity_map.sensors {
                    let (x, y) = point(*sensor, &rot);
                    rectangle(
                        [1.0, 1.0, 0.0, 1.0],
                        rectangle::square(x, y, thickness * 4.0),
                        transform,
                        g,
                    );
                }
                for effector in &activity_map.effectors {
                    let (x, y) = point(*effector, &rot);
                    rectangle(
                        [0.5, 0.0, 0.0, 1.0],
                        rectangle::square(x, y, thickness * 4.0),
                        transform,
                        g,
                    );
                }
                println!("rendering: {:?}", now.elapsed());
            });
        }
    }
}

fn point(point: Position, rot: &Quaternion<Scalar>) -> (Scalar, Scalar) {
    let p = Point3::new(point.x, point.y, point.z);
    let p = rot.rotate_point(p);
    (p.x, p.y)
}

fn connection_into_line(pair: &(Position, Position, Scalar), rot: &Quaternion<Scalar>) -> [f64; 4] {
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

fn print_stats(stats: BrainActivityStats) {
    println!("=== BRAIN ACTIVITY STATS ===");
    println!("Count:");
    println!("- neurons: {}", stats.neurons_count);
    println!("- synapses: {}", stats.synapses_count);
    println!("- impulses: {}", stats.impulses_count);
    println!("Potential:");
    println!("- neurons: {}", stats.neurons_potential.0);
    println!("  - min: {}", stats.neurons_potential.1);
    println!("  - max: {}", stats.neurons_potential.2);
    println!("- impulses: {}", stats.impulses_potential.0);
    println!("  - min: {}", stats.impulses_potential.1);
    println!("  - max: {}", stats.impulses_potential.2);
    println!("- all: {}", stats.all_potential.0);
    println!("  - min: {}", stats.all_potential.1);
    println!("  - max: {}", stats.all_potential.2);
    println!("Neurons connections:");
    println!("- Incoming:");
    println!("  - min: {}", stats.incoming_neuron_connections.0);
    println!("  - max: {}", stats.incoming_neuron_connections.1);
    println!("- Outgoing:");
    println!("  - min: {}", stats.outgoing_neuron_connections.0);
    println!("  - max: {}", stats.outgoing_neuron_connections.1);
    println!("Synapses receptors:");
    println!("- min: {}", stats.synapses_receptors.0);
    println!("- max: {}", stats.synapses_receptors.1);
}
