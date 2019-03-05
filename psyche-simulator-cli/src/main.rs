extern crate clap;
extern crate psyche;
extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;

mod timeline;

use clap::{App, Arg, ArgMatches, SubCommand};
use core::str::from_utf8;
use psyche::core::brain::{activity, Brain, BrainActivityStats};
use psyche::core::brain_builder::BrainBuilder;
use psyche::core::config::Config;
use psyche::core::error::*;
use psyche::core::Scalar;
use psyche::graphics::obj::generate;
use psyche::serde::json::{brain_builder_from_json, brain_builder_to_json, brain_from_json};
use psyche::serde::yaml::{brain_builder_from_yaml, brain_builder_to_yaml, brain_from_yaml};
use rand::{thread_rng, Rng};
use std::fs::{read, write};
use std::path::Path;
use std::time::Instant;
use timeline::{ActionType, Timeline};

fn main() -> Result<()> {
    let matches = App::new("Psyche AI Simulator CLI")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("snapshot")
                .short("s")
                .long("snapshot")
                .value_name("FILE")
                .help("Brain snapshot file path")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::with_name("builder")
                .short("b")
                .long("builder")
                .value_name("FILE")
                .help("Brain builder config file path")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::with_name("timeline")
                .short("t")
                .long("timeline")
                .value_name("FILE")
                .help("Simulation timeline file path")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::with_name("fps")
                .short("f")
                .long("fps")
                .value_name("INTEGER")
                .help("Simulation frames per second")
                .takes_value(true)
                .default_value("60"),
        )
        .arg(
            Arg::with_name("output_dir")
                .short("o")
                .long("output_dir")
                .value_name("PATH")
                .help("Simulation output files path")
                .takes_value(true)
                .default_value("./"),
        )
        .arg(
            Arg::with_name("name")
                .short("n")
                .long("name")
                .value_name("NAME")
                .help("Simulation output files prefix name")
                .takes_value(true)
                .default_value("activity"),
        )
        .arg(
            Arg::with_name("ignore-neurons")
                .long("ignore-neurons")
                .help("Ignore rendering neurons"),
        )
        .arg(
            Arg::with_name("ignore-connections")
                .long("ignore-connections")
                .help("Ignore rendering connections"),
        )
        .arg(
            Arg::with_name("ignore-impulses")
                .long("ignore-impulses")
                .help("Ignore rendering impulses"),
        )
        .arg(
            Arg::with_name("ignore-sensors")
                .long("ignore-sensors")
                .help("Ignore rendering sensors"),
        )
        .arg(
            Arg::with_name("ignore-effectors")
                .long("ignore-effectors")
                .help("Ignore rendering effectors"),
        )
        .arg(
            Arg::with_name("dry")
                .short("r")
                .long("dry")
                .help("Dry mode (without rendering to files)"),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Verbose output (print triggered actions and brain stats)"),
        )
        .subcommand(
            SubCommand::with_name("template")
                .about("Create default specified config file")
                .arg(
                    Arg::with_name("format")
                        .short("f")
                        .long("format")
                        .help("Template file format (json, yaml)")
                        .default_value("json"),
                )
                .arg(
                    Arg::with_name("type")
                        .short("t")
                        .long("type")
                        .help("Template file type (builder, timeline)")
                        .default_value("builder"),
                )
                .arg(
                    Arg::with_name("output")
                        .short("o")
                        .long("output")
                        .value_name("PATH")
                        .help("Template output file path")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("template") {
        main_template(matches)
    } else {
        main_simulation(matches)
    }
}

fn main_template(matches: &ArgMatches) -> Result<()> {
    let format = matches.value_of("format").unwrap();
    let type_ = matches.value_of("type").unwrap();
    if let Some(output) = matches.value_of("output") {
        match type_ {
            "builder" => match format {
                "json" => write(
                    output,
                    brain_builder_to_json(&make_default_brain_builder(Config::default()), true)
                        .unwrap(),
                )
                .unwrap(),
                "yaml" => write(
                    output,
                    brain_builder_to_yaml(&make_default_brain_builder(Config::default())).unwrap(),
                )
                .unwrap(),
                name => panic!("Unsupported template format: {}", name),
            },
            "timeline" => match format {
                "json" => write(output, Timeline::default().to_json().unwrap()).unwrap(),
                "yaml" => write(output, Timeline::default().to_yaml().unwrap()).unwrap(),
                name => panic!("Unsupported template format: {}", name),
            },
            name => panic!("Unsupported template type: {}", name),
        }
    } else {
        panic!("There is no specified output file path!");
    }

    Ok(())
}

fn main_simulation(matches: ArgMatches) -> Result<()> {
    let mut brain = make_brain(&matches);
    let timeline = make_timeline(&matches);
    let fps = matches.value_of("fps").unwrap().parse::<usize>().unwrap();
    let output_dir = Path::new(matches.value_of("output_dir").unwrap())
        .to_str()
        .unwrap()
        .to_owned();
    let name = matches.value_of("name").unwrap().to_owned();
    let render_neurons = !matches.is_present("ignore-neurons");
    let render_connections = !matches.is_present("ignore-connections");
    let render_impulses = !matches.is_present("ignore-impulses");
    let render_sensors = !matches.is_present("ignore-sensors");
    let render_effectors = !matches.is_present("ignore-effectors");
    let dry = matches.is_present("dry");
    let verbose = matches.is_present("verbose");

    let mut rng = thread_rng();
    let delta_time = 1.0 / fps as Scalar;
    let mut last_time = 0.0;
    let mut next_time = delta_time;
    let mut frame = 0;
    let generator_config = Default::default();
    let timer = Instant::now();
    while let Some(actions) = timeline.perform(last_time, next_time) {
        println!(
            "Rendering {} -> {} ({:?})",
            last_time,
            next_time,
            timer.elapsed()
        );
        if verbose {
            println!("- performing actions:");
            for action in &actions {
                println!("  - {:?}", action);
            }
        } else {
            println!("- performing actions");
        }
        for action in actions {
            match action.action_type {
                ActionType::TriggerSensorByID(id, (min, max)) => {
                    drop(brain.sensor_trigger_impulse(
                        id,
                        if min < max {
                            rng.gen_range(min, max)
                        } else {
                            max
                        },
                    ));
                }
                ActionType::TriggerSensorByIndex(index, (min, max)) => {
                    let ids = brain.get_sensors();
                    if index < ids.len() {
                        drop(brain.sensor_trigger_impulse(
                            ids[index],
                            if min < max {
                                rng.gen_range(min, max)
                            } else {
                                max
                            },
                        ));
                    }
                }
                ActionType::TriggerRandomSensorsByPercentage(percentage, (min, max)) => {
                    let ids = brain.get_sensors();
                    for _ in 0..((ids.len() as Scalar * percentage) as usize) {
                        let index = rng.gen_range(0, ids.len()) % ids.len();
                        if index < ids.len() {
                            drop(brain.sensor_trigger_impulse(
                                ids[index],
                                if min < max {
                                    rng.gen_range(min, max)
                                } else {
                                    max
                                },
                            ));
                        }
                    }
                }
                ActionType::TriggerRandomSensorsByAmount(count, (min, max)) => {
                    let ids = brain.get_sensors();
                    for _ in 0..count {
                        let index = rng.gen_range(0, ids.len()) % ids.len();
                        if index < ids.len() {
                            drop(brain.sensor_trigger_impulse(
                                ids[index],
                                if min < max {
                                    rng.gen_range(min, max)
                                } else {
                                    max
                                },
                            ));
                        }
                    }
                }
                ActionType::IgniteRandomSynapsesByPercentage(percentage, (min, max)) => {
                    let count = (brain.synapses_count() as Scalar * percentage) as usize;
                    brain.ignite_random_synapses(count, min..max);
                }
                ActionType::IgniteRandomSynapsesByAmount(count, (min, max)) => {
                    brain.ignite_random_synapses(count, min..max);
                }
                _ => {}
            }
        }
        println!("- processing brain");
        brain.process(delta_time)?;
        if verbose {
            print_stats(brain.build_activity_stats());
        }
        if !dry {
            println!("- writing snapshot");
            write(
                format!("{}/{}-all-{}.obj", output_dir, name, frame),
                generate(&brain.build_activity_map(activity::ALL), &generator_config)?,
            )
            .unwrap();
            if render_neurons {
                write(
                    format!("{}/{}-neurons-{}.obj", output_dir, name, frame),
                    generate(
                        &brain.build_activity_map(activity::NEURONS),
                        &generator_config,
                    )?,
                )
                .unwrap();
            }
            if render_connections {
                write(
                    format!("{}/{}-connections-{}.obj", output_dir, name, frame),
                    generate(
                        &brain.build_activity_map(activity::CONNECTIONS),
                        &generator_config,
                    )?,
                )
                .unwrap();
            }
            if render_impulses {
                write(
                    format!("{}/{}-impulses-{}.obj", output_dir, name, frame),
                    generate(
                        &brain.build_activity_map(activity::IMPULSES),
                        &generator_config,
                    )?,
                )
                .unwrap();
            }
            if render_sensors {
                write(
                    format!("{}/{}-sensors-{}.obj", output_dir, name, frame),
                    generate(
                        &brain.build_activity_map(activity::SENSORS),
                        &generator_config,
                    )?,
                )
                .unwrap();
            }
            if render_effectors {
                write(
                    format!("{}/{}-effectors-{}.obj", output_dir, name, frame),
                    generate(
                        &brain.build_activity_map(activity::EFFECTORS),
                        &generator_config,
                    )?,
                )
                .unwrap();
            }
        }

        last_time = next_time;
        next_time += delta_time;
        frame += 1;
    }

    Ok(())
}

fn make_brain(matches: &ArgMatches) -> Brain {
    if let Some(snapshot) = matches.value_of("snapshot") {
        if snapshot.ends_with(".json") {
            brain_from_json(from_utf8(&read(snapshot).unwrap()).unwrap()).unwrap()
        } else if snapshot.ends_with(".yaml") {
            brain_from_yaml(from_utf8(&read(snapshot).unwrap()).unwrap()).unwrap()
        } else {
            panic!(
                "Snapshot file with no specified format extension: {}",
                snapshot
            )
        }
    } else if let Some(builder) = matches.value_of("builder") {
        if builder.ends_with(".json") {
            brain_builder_from_json(from_utf8(&read(builder).unwrap()).unwrap())
                .unwrap()
                .build()
        } else if builder.ends_with(".yaml") {
            brain_builder_from_yaml(from_utf8(&read(builder).unwrap()).unwrap())
                .unwrap()
                .build()
        } else {
            panic!(
                "Brain builder file with no specified format extension: {}",
                builder
            )
        }
    } else {
        let mut config = Config::default();
        config.propagation_speed = 50.0;
        config.synapse_reconnection_range = Some(15.0);
        config.neuron_potential_decay = 0.1;
        config.synapse_propagation_decay = 0.01;
        config.synapse_new_connection_receptors = Some(2.0);
        make_default_brain_builder(config).build()
    }
}

fn make_timeline(matches: &ArgMatches) -> Timeline {
    if let Some(timeline) = matches.value_of("timeline") {
        if timeline.ends_with(".json") {
            Timeline::from_json(from_utf8(&read(timeline).unwrap()).unwrap()).unwrap()
        } else if timeline.ends_with(".yaml") {
            Timeline::from_yaml(from_utf8(&read(timeline).unwrap()).unwrap()).unwrap()
        } else {
            panic!(
                "Timeline file with no specified format extension: {}",
                timeline
            )
        }
    } else {
        Default::default()
    }
}

fn print_stats(stats: BrainActivityStats) {
    println!("- brain activity stats:");
    println!("  Count:");
    println!("  - neurons: {}", stats.neurons_count);
    println!("  - synapses: {}", stats.synapses_count);
    println!("  - impulses: {}", stats.impulses_count);
    println!("  Potential:");
    println!("  - neurons: {}", stats.neurons_potential.0);
    println!("    - min: {}", stats.neurons_potential.1.start);
    println!("    - max: {}", stats.neurons_potential.1.end);
    println!("  - impulses: {}", stats.impulses_potential.0);
    println!("    - min: {}", stats.impulses_potential.1.start);
    println!("    - max: {}", stats.impulses_potential.1.end);
    println!("  - all: {}", stats.all_potential.0);
    println!("    - min: {}", stats.all_potential.1.start);
    println!("    - max: {}", stats.all_potential.1.end);
    println!("  Neurons connections:");
    println!("  - Incoming:");
    println!("    - min: {}", stats.incoming_neuron_connections.start);
    println!("    - max: {}", stats.incoming_neuron_connections.end);
    println!("  - Outgoing:");
    println!("    - min: {}", stats.outgoing_neuron_connections.start);
    println!("    - max: {}", stats.outgoing_neuron_connections.end);
    println!("  Synapses receptors:");
    println!("  - min: {}", stats.synapses_receptors.start);
    println!("  - max: {}", stats.synapses_receptors.end);
}

fn make_default_brain_builder(config: Config) -> BrainBuilder {
    BrainBuilder::new()
        .config(config)
        .neurons(600)
        .connections(1000)
        .min_neurogenesis_range(5.0)
        .max_neurogenesis_range(15.0)
        .radius(50.0)
        .sensors(50)
        .effectors(25)
}
