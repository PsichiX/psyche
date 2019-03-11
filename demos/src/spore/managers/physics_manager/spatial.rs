use super::body::BodyID;
use psyche::core::Scalar;
use spade::{BoundingRect, SpatialObject};

#[derive(Debug, Clone)]
pub struct SpatialData {
    pub body: BodyID,
    pub position: [Scalar; 2],
    pub radius: Scalar,
    pub rect: BoundingRect<[Scalar; 2]>,
}

impl SpatialData {
    pub fn new(body: BodyID, position: [Scalar; 2], radius: Scalar) -> Self {
        Self {
            body,
            position,
            radius,
            rect: BoundingRect::from_corners(
                &[position[0] - radius, position[1] - radius],
                &[position[0] + radius, position[1] + radius],
            ),
        }
    }
}

impl SpatialObject for SpatialData {
    type Point = [Scalar; 2];

    fn mbr(&self) -> BoundingRect<[Scalar; 2]> {
        self.rect
    }

    fn distance2(&self, point: &Self::Point) -> Scalar {
        let dx = point[0] - self.position[0];
        let dy = point[1] - self.position[1];
        (dx * dx + dy * dy - self.radius * self.radius).max(0.0)
    }

    fn contains(&self, point: &Self::Point) -> bool {
        let dx = point[0] - self.position[0];
        let dy = point[1] - self.position[1];
        dx * dx + dy * dy <= self.radius * self.radius
    }
}
