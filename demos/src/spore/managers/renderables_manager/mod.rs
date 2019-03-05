#![allow(dead_code)]

pub mod renderable;

use crate::managers::items_manager::{ItemsManager, Named};
use piston_window::{ellipse, line, rectangle, Context, G2d, Transformed};
use psyche::core::Scalar;
use renderable::*;

#[derive(Debug, Clone, Default)]
pub struct RenderablesManager {
    renderables: Vec<Renderable>,
    root: Option<Hierarchy>,
}

impl RenderablesManager {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn root(&self) -> Option<&Hierarchy> {
        if let Some(ref root) = self.root {
            Some(root)
        } else {
            None
        }
    }

    #[inline]
    pub fn root_mut(&mut self) -> Option<&mut Hierarchy> {
        if let Some(ref mut root) = self.root {
            Some(root)
        } else {
            None
        }
    }

    #[inline]
    pub fn set_root(&mut self, node: Option<Hierarchy>) {
        self.root = node;
    }

    pub fn hierarchy(&self, path: &str) -> Option<&Hierarchy> {
        if path.is_empty() {
            return self.root();
        }

        match &self.root {
            Some(root) => {
                let mut last = root;
                for part in path.split('/') {
                    if let Some(node) = last.children.iter().find(|n| n.name == part) {
                        last = node;
                    } else {
                        return None;
                    }
                }
                Some(last)
            }
            _ => None,
        }
    }

    pub fn hierarchy_mut(&mut self, path: &str) -> Option<&mut Hierarchy> {
        if path.is_empty() {
            return self.root_mut();
        }

        match &mut self.root {
            Some(root) => {
                let mut last = root;
                for part in path.split('/') {
                    if let Some(node) = last.children.iter_mut().find(|n| n.name == part) {
                        last = node;
                    } else {
                        return None;
                    }
                }
                Some(last)
            }
            _ => None,
        }
    }

    #[inline]
    pub fn refresh(&mut self) {
        if let Some(root) = &mut self.root {
            root.sort_children(&self.renderables);
        } else {
            self.renderables
                .sort_by(|a, b| a.depth.partial_cmp(&b.depth).unwrap());
        }
    }

    pub fn render(&self, context: Context, gfx: &mut G2d) {
        if let Some(ref hierarchy) = self.root {
            self.draw_hierarchy_node(context.transform, gfx, hierarchy);
        } else {
            for renderable in &self.renderables {
                self.draw_renderable(context.transform, gfx, renderable);
            }
        }
    }

    fn draw_hierarchy_node(
        &self,
        transform: [[Scalar; 3]; 2],
        gfx: &mut G2d,
        hierarchy: &Hierarchy,
    ) {
        let transform = hierarchy
            .renderable
            .map(|r| {
                self.item(r)
                    .map(|r| self.draw_renderable(transform, gfx, r))
                    .unwrap_or(transform)
            })
            .unwrap_or(transform);
        for child in &hierarchy.children {
            self.draw_hierarchy_node(transform, gfx, child);
        }
    }

    fn draw_renderable(
        &self,
        transform: [[Scalar; 3]; 2],
        gfx: &mut G2d,
        renderable: &Renderable,
    ) -> [[Scalar; 3]; 2] {
        let transform = transform
            .trans(
                renderable.transform.position.x,
                renderable.transform.position.y,
            )
            .rot_rad(renderable.transform.angle.0);
        match renderable.graphics {
            Graphics::Rectangle(color, size) => {
                let hw = size.x * 0.5;
                let hh = size.y * 0.5;
                rectangle(
                    color,
                    rectangle::rectangle_by_corners(-hw, -hh, hw, hh),
                    transform,
                    gfx,
                );
            }
            Graphics::Line(color, length, thickness) => {
                let hl = length * 0.5;
                line(color, thickness, [-hl, 0.0, hl, 0.0], transform, gfx);
            }
            Graphics::Circle(color, radius) => {
                ellipse(color, ellipse::circle(0.0, 0.0, radius), transform, gfx);
            }
            _ => {}
        }
        transform
    }
}

impl ItemsManager<Renderable> for RenderablesManager {
    #[inline]
    fn items(&self) -> &[Renderable] {
        &self.renderables
    }

    fn add(&mut self, item: Renderable) -> RenderableID {
        let id = item.id();
        self.renderables.push(item);
        id
    }

    fn create(&mut self) -> RenderableID {
        self.add(Renderable::new())
    }

    fn create_with<F>(&mut self, mut with: F) -> RenderableID
    where
        F: FnMut(&mut Renderable, &mut Self),
    {
        let mut renderable = Renderable::new();
        with(&mut renderable, self);
        self.add(renderable)
    }

    fn destroy(&mut self, id: RenderableID) -> bool {
        if let Some(index) = self.renderables.iter().position(|r| r.id() == id) {
            self.renderables.swap_remove(index);
            true
        } else {
            false
        }
    }

    fn with<F, R>(&mut self, id: RenderableID, mut with: F) -> Option<R>
    where
        F: FnMut(&mut Renderable, &mut Self) -> R,
    {
        if let Some(index) = self.renderables.iter().position(|r| r.id() == id) {
            let mut renderable = self.renderables.swap_remove(index);
            let result = with(&mut renderable, self);
            self.renderables.push(renderable);
            Some(result)
        } else {
            None
        }
    }

    #[inline]
    fn item(&self, id: RenderableID) -> Option<&Renderable> {
        self.renderables.iter().find(|r| r.id() == id)
    }

    #[inline]
    fn item_mut(&mut self, id: RenderableID) -> Option<&mut Renderable> {
        self.renderables.iter_mut().find(|r| r.id() == id)
    }
}
