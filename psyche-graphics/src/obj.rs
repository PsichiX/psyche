use obj_exporter::*;
use psyche_core::brain::BrainActivityMap;
use psyche_core::error::*;
use psyche_core::neuron::Position;
use psyche_core::Scalar;
use std::io::Cursor;
use std::iter::repeat;
use std::string::FromUtf8Error;

#[derive(Debug, Clone)]
pub struct Config {
    pub neurons: Option<Color>,
    pub synapses: Option<Color>,
    pub impulses: Option<Color>,
    pub sensors: Option<Color>,
    pub effectors: Option<Color>,
    pub color_storage: ColorStorage,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            neurons: Some([255, 0, 255].into()),
            synapses: Some([0, 0, 255].into()),
            impulses: Some([192, 192, 255].into()),
            sensors: Some([255, 255, 0].into()),
            effectors: Some([128, 0, 0].into()),
            color_storage: ColorStorage::Nowhere,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ColorStorage {
    Nowhere,
    Normals,
    TexVertices,
}

impl Default for ColorStorage {
    fn default() -> Self {
        ColorStorage::Nowhere
    }
}

/// (R, G, B)
#[derive(Debug, Copy, Clone)]
pub struct Color(u8, u8, u8);

impl From<(u8, u8, u8)> for Color {
    fn from(value: (u8, u8, u8)) -> Self {
        Self(value.0, value.1, value.2)
    }
}

impl From<[u8; 3]> for Color {
    fn from(value: [u8; 3]) -> Self {
        Self(value[0], value[1], value[2])
    }
}

/// generates OBJ string from activity map.
/// NOTE: Colors are stored either in vertices normals or texture vertices.
pub fn generate_string(activity_map: &BrainActivityMap, config: &Config) -> Result<String> {
    match String::from_utf8(generate(activity_map, config)?) {
        Ok(s) => Ok(s),
        Err(e) => Err(utf8_into_error(e)),
    }
}

/// generates OBJ bytes from activity map.
/// NOTE: Colors are stored either in vertices normals or texture vertices.
pub fn generate(activity_map: &BrainActivityMap, config: &Config) -> Result<Vec<u8>> {
    let mut objects = vec![];

    if let Some(ref neurons) = config.neurons {
        if !activity_map.neurons.is_empty() {
            objects.push(Object {
                name: "neurons".to_owned(),
                vertices: activity_map
                    .neurons
                    .iter()
                    .map(|p| Vertex {
                        x: p.x,
                        y: p.y,
                        z: p.z,
                    })
                    .collect(),
                tex_vertices: if config.color_storage == ColorStorage::TexVertices {
                    let Color(r, g, b) = neurons;
                    repeat(TVertex {
                        u: *r as Scalar / 255.0,
                        v: *g as Scalar / 255.0,
                        w: *b as Scalar / 255.0,
                    })
                    .take(activity_map.neurons.len())
                    .collect()
                } else {
                    vec![]
                },
                normals: if config.color_storage == ColorStorage::Normals {
                    let Color(r, g, b) = neurons;
                    repeat(Vertex {
                        x: *r as Scalar / 255.0,
                        y: *g as Scalar / 255.0,
                        z: *b as Scalar / 255.0,
                    })
                    .take(activity_map.neurons.len())
                    .collect()
                } else {
                    vec![]
                },
                geometry: vec![Geometry {
                    material_name: None,
                    shapes: activity_map
                        .neurons
                        .iter()
                        .enumerate()
                        .map(|(i, _)| Shape {
                            primitive: Primitive::Point((i, None, None)),
                            groups: vec![],
                            smoothing_groups: vec![],
                        })
                        .collect(),
                }],
            });
        }
    }

    if let Some(ref synapses) = config.synapses {
        if !activity_map.connections.is_empty() {
            objects.push(Object {
                name: "synapses".to_owned(),
                vertices: activity_map
                    .connections
                    .iter()
                    .flat_map(|(f, t, _)| {
                        vec![
                            Vertex {
                                x: f.x,
                                y: f.y,
                                z: f.z,
                            },
                            Vertex {
                                x: t.x,
                                y: t.y,
                                z: t.z,
                            },
                        ]
                    })
                    .collect(),
                tex_vertices: if config.color_storage == ColorStorage::TexVertices {
                    let Color(r, g, b) = synapses;
                    repeat(TVertex {
                        u: *r as Scalar / 255.0,
                        v: *g as Scalar / 255.0,
                        w: *b as Scalar / 255.0,
                    })
                    .take(activity_map.connections.len())
                    .collect()
                } else {
                    vec![]
                },
                normals: if config.color_storage == ColorStorage::Normals {
                    let Color(r, g, b) = synapses;
                    repeat(Vertex {
                        x: *r as Scalar / 255.0,
                        y: *g as Scalar / 255.0,
                        z: *b as Scalar / 255.0,
                    })
                    .take(activity_map.connections.len())
                    .collect()
                } else {
                    vec![]
                },
                geometry: vec![Geometry {
                    material_name: None,
                    shapes: activity_map
                        .connections
                        .iter()
                        .enumerate()
                        .map(|(i, _)| Shape {
                            primitive: Primitive::Line(
                                (i * 2, None, None),
                                (i * 2 + 1, None, None),
                            ),
                            groups: vec![],
                            smoothing_groups: vec![],
                        })
                        .collect(),
                }],
            });
        }
    }

    if let Some(ref impulses) = config.impulses {
        if !activity_map.impulses.is_empty() {
            let positions = activity_map
                .impulses
                .iter()
                .map(|(s, e, f)| lerp(*s, *e, *f))
                .collect::<Vec<_>>();
            objects.push(Object {
                name: "impulses".to_owned(),
                vertices: positions
                    .iter()
                    .map(|p| Vertex {
                        x: p.x,
                        y: p.y,
                        z: p.z,
                    })
                    .collect(),
                tex_vertices: if config.color_storage == ColorStorage::TexVertices {
                    let Color(r, g, b) = impulses;
                    repeat(TVertex {
                        u: *r as Scalar / 255.0,
                        v: *g as Scalar / 255.0,
                        w: *b as Scalar / 255.0,
                    })
                    .take(positions.len())
                    .collect()
                } else {
                    vec![]
                },
                normals: if config.color_storage == ColorStorage::Normals {
                    let Color(r, g, b) = impulses;
                    repeat(Vertex {
                        x: *r as Scalar / 255.0,
                        y: *g as Scalar / 255.0,
                        z: *b as Scalar / 255.0,
                    })
                    .take(positions.len())
                    .collect()
                } else {
                    vec![]
                },
                geometry: vec![Geometry {
                    material_name: None,
                    shapes: positions
                        .iter()
                        .enumerate()
                        .map(|(i, _)| Shape {
                            primitive: Primitive::Point((i, None, None)),
                            groups: vec![],
                            smoothing_groups: vec![],
                        })
                        .collect(),
                }],
            });
        }
    }

    if let Some(ref sensors) = config.sensors {
        if !activity_map.sensors.is_empty() {
            objects.push(Object {
                name: "sensors".to_owned(),
                vertices: activity_map
                    .sensors
                    .iter()
                    .map(|p| Vertex {
                        x: p.x,
                        y: p.y,
                        z: p.z,
                    })
                    .collect(),
                tex_vertices: if config.color_storage == ColorStorage::TexVertices {
                    let Color(r, g, b) = sensors;
                    repeat(TVertex {
                        u: *r as Scalar / 255.0,
                        v: *g as Scalar / 255.0,
                        w: *b as Scalar / 255.0,
                    })
                    .take(activity_map.sensors.len())
                    .collect()
                } else {
                    vec![]
                },
                normals: if config.color_storage == ColorStorage::Normals {
                    let Color(r, g, b) = sensors;
                    repeat(Vertex {
                        x: *r as Scalar / 255.0,
                        y: *g as Scalar / 255.0,
                        z: *b as Scalar / 255.0,
                    })
                    .take(activity_map.sensors.len())
                    .collect()
                } else {
                    vec![]
                },
                geometry: vec![Geometry {
                    material_name: None,
                    shapes: activity_map
                        .sensors
                        .iter()
                        .enumerate()
                        .map(|(i, _)| Shape {
                            primitive: Primitive::Point((i, None, None)),
                            groups: vec![],
                            smoothing_groups: vec![],
                        })
                        .collect(),
                }],
            });
        }
    }

    if let Some(ref effectors) = config.effectors {
        if !activity_map.effectors.is_empty() {
            objects.push(Object {
                name: "effectors".to_owned(),
                vertices: activity_map
                    .effectors
                    .iter()
                    .map(|p| Vertex {
                        x: p.x,
                        y: p.y,
                        z: p.z,
                    })
                    .collect(),
                tex_vertices: if config.color_storage == ColorStorage::TexVertices {
                    let Color(r, g, b) = effectors;
                    repeat(TVertex {
                        u: *r as Scalar / 255.0,
                        v: *g as Scalar / 255.0,
                        w: *b as Scalar / 255.0,
                    })
                    .take(activity_map.effectors.len())
                    .collect()
                } else {
                    vec![]
                },
                normals: if config.color_storage == ColorStorage::Normals {
                    let Color(r, g, b) = effectors;
                    repeat(Vertex {
                        x: *r as Scalar / 255.0,
                        y: *g as Scalar / 255.0,
                        z: *b as Scalar / 255.0,
                    })
                    .take(activity_map.effectors.len())
                    .collect()
                } else {
                    vec![]
                },
                geometry: vec![Geometry {
                    material_name: None,
                    shapes: activity_map
                        .effectors
                        .iter()
                        .enumerate()
                        .map(|(i, _)| Shape {
                            primitive: Primitive::Point((i, None, None)),
                            groups: vec![],
                            smoothing_groups: vec![],
                        })
                        .collect(),
                }],
            });
        }
    }

    let set = ObjSet {
        material_library: None,
        objects,
    };
    let mut cursor = Cursor::new(vec![]);
    export(&set, &mut cursor)?;
    Ok(cursor.into_inner())
}

fn lerp(start: Position, end: Position, factor: Scalar) -> Position {
    let factor = factor.max(0.0).min(1.0);
    Position {
        x: (end.x - start.x) * factor + start.x,
        y: (end.y - start.y) * factor + start.y,
        z: (end.z - start.z) * factor + start.z,
    }
}

fn utf8_into_error(error: FromUtf8Error) -> Error {
    Error::simple(format!("{}", error))
}
