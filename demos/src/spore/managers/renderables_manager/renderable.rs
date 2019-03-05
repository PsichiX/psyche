use crate::managers::items_manager::Named;
use cgmath::*;
use psyche::core::id::ID;
use psyche::core::Scalar;
use std::cmp::Ordering;

pub type RenderableID = ID<Renderable>;
pub type Color = [f32; 4];
pub type Vec2 = Vector2<Scalar>;
pub type Size = Vec2;
pub type Length = Scalar;
pub type Thickness = Scalar;
pub type Radius = Scalar;
pub type Angle = Rad<Scalar>;

#[inline]
pub fn angle(value: Scalar) -> Angle {
    Rad::<_>(value)
}

#[derive(Debug, Default, Clone)]
pub struct Renderable {
    id: RenderableID,
    pub depth: Scalar,
    pub transform: Transform,
    pub graphics: Graphics,
}

impl Named<Self> for Renderable {
    #[inline]
    fn id(&self) -> RenderableID {
        self.id
    }
}

impl Renderable {
    #[inline]
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone)]
pub struct Transform {
    pub position: Vec2,
    pub angle: Angle,
}

impl Default for Transform {
    #[inline]
    fn default() -> Self {
        Self {
            position: Vec2::zero(),
            angle: angle(0.0),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Graphics {
    None,
    Rectangle(Color, Size),
    Line(Color, Length, Thickness),
    Circle(Color, Radius),
}

impl Default for Graphics {
    #[inline]
    fn default() -> Self {
        Graphics::None
    }
}

#[derive(Debug, Default, Clone)]
pub struct Hierarchy {
    pub name: String,
    pub renderable: Option<RenderableID>,
    pub children: Vec<Hierarchy>,
}

impl Hierarchy {
    pub fn new(renderable: RenderableID) -> Self {
        Self::with_name("".to_owned(), renderable)
    }

    pub fn named(name: String) -> Self {
        Self {
            name,
            renderable: None,
            children: vec![],
        }
    }

    pub fn with_name(name: String, renderable: RenderableID) -> Self {
        Self {
            name,
            renderable: Some(renderable),
            children: vec![],
        }
    }

    pub fn sort_children(&mut self, renderables: &[Renderable]) {
        self.children.sort_by(|a, b| {
            if let Some(a) = a.renderable {
                if let Some(b) = b.renderable {
                    if let Some(a) = renderables.iter().find(|r| r.id() == a) {
                        if let Some(b) = renderables.iter().find(|r| r.id() == b) {
                            return a.depth.partial_cmp(&b.depth).unwrap();
                        }
                    }
                }
            }
            Ordering::Equal
        });

        for child in &mut self.children {
            child.sort_children(renderables);
        }
    }
}

impl Into<Hierarchy> for () {
    fn into(self) -> Hierarchy {
        Hierarchy::default()
    }
}

impl Into<Hierarchy> for RenderableID {
    fn into(self) -> Hierarchy {
        Hierarchy::new(self)
    }
}

impl Into<Hierarchy> for String {
    fn into(self) -> Hierarchy {
        Hierarchy::named(self)
    }
}

impl Into<Hierarchy> for &str {
    fn into(self) -> Hierarchy {
        Hierarchy::named(self.to_owned())
    }
}

impl Into<Hierarchy> for (String, RenderableID) {
    fn into(self) -> Hierarchy {
        Hierarchy::with_name(self.0, self.1)
    }
}

impl Into<Hierarchy> for (RenderableID, String) {
    fn into(self) -> Hierarchy {
        Hierarchy::with_name(self.1, self.0)
    }
}

impl Into<Hierarchy> for (&str, RenderableID) {
    fn into(self) -> Hierarchy {
        Hierarchy::with_name(self.0.to_owned(), self.1)
    }
}

impl Into<Hierarchy> for (RenderableID, &str) {
    fn into(self) -> Hierarchy {
        Hierarchy::with_name(self.1.to_owned(), self.0)
    }
}

impl Into<Hierarchy> for Vec<Hierarchy> {
    fn into(self) -> Hierarchy {
        let mut result = Hierarchy::default();
        result.children = self;
        result
    }
}

impl Into<Hierarchy> for (RenderableID, Vec<Hierarchy>) {
    fn into(self) -> Hierarchy {
        let mut result = Hierarchy::new(self.0);
        result.children = self.1;
        result
    }
}

impl Into<Hierarchy> for (String, Vec<Hierarchy>) {
    fn into(self) -> Hierarchy {
        let mut result = Hierarchy::named(self.0);
        result.children = self.1;
        result
    }
}

impl Into<Hierarchy> for (&str, Vec<Hierarchy>) {
    fn into(self) -> Hierarchy {
        let mut result = Hierarchy::named(self.0.to_owned());
        result.children = self.1;
        result
    }
}

impl Into<Hierarchy> for (String, RenderableID, Vec<Hierarchy>) {
    fn into(self) -> Hierarchy {
        let mut result = Hierarchy::with_name(self.0, self.1);
        result.children = self.2;
        result
    }
}

impl Into<Hierarchy> for (RenderableID, String, Vec<Hierarchy>) {
    fn into(self) -> Hierarchy {
        let mut result = Hierarchy::with_name(self.1, self.0);
        result.children = self.2;
        result
    }
}

impl Into<Hierarchy> for (&str, RenderableID, Vec<Hierarchy>) {
    fn into(self) -> Hierarchy {
        let mut result = Hierarchy::with_name(self.0.to_owned(), self.1);
        result.children = self.2;
        result
    }
}

impl Into<Hierarchy> for (RenderableID, &str, Vec<Hierarchy>) {
    fn into(self) -> Hierarchy {
        let mut result = Hierarchy::with_name(self.1.to_owned(), self.0);
        result.children = self.2;
        result
    }
}
