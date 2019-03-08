use crate::managers::items_manager::ItemsManager;
use crate::managers::items_manager::Named;
use psyche::core::brain::{Brain, BrainID};
use psyche::core::Scalar;

impl Named<Self> for Brain {
    fn id(&self) -> BrainID {
        Brain::id(self)
    }
}

#[derive(Debug, Clone, Default)]
pub struct BrainsManager {
    brains: Vec<Brain>,
}

impl BrainsManager {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn process(&mut self, dt: Scalar) {
        for brain in &mut self.brains {
            brain.process(dt).unwrap();
        }
    }
}

impl ItemsManager<Brain> for BrainsManager {
    #[inline]
    fn items(&self) -> &[Brain] {
        &self.brains
    }

    fn add(&mut self, item: Brain) -> BrainID {
        let id = item.id();
        self.brains.push(item);
        id
    }

    fn create(&mut self) -> BrainID {
        self.add(Brain::new())
    }

    fn create_with<F>(&mut self, mut with: F) -> BrainID
    where
        F: FnMut(&mut Brain, &mut Self),
    {
        let mut brain = Brain::new();
        with(&mut brain, self);
        self.add(brain)
    }

    fn destroy(&mut self, id: BrainID) -> bool {
        if let Some(index) = self.brains.iter().position(|r| r.id() == id) {
            self.brains.swap_remove(index);
            true
        } else {
            false
        }
    }

    fn with<F, R>(&mut self, id: BrainID, mut with: F) -> Option<R>
    where
        F: FnMut(&mut Brain, &mut Self) -> R,
    {
        if let Some(index) = self.brains.iter().position(|r| r.id() == id) {
            let mut brain = self.brains.swap_remove(index);
            let result = with(&mut brain, self);
            self.brains.push(brain);
            Some(result)
        } else {
            None
        }
    }

    fn item(&self, id: BrainID) -> Option<&Brain> {
        if let Some(brain) = self.brains.iter().find(|r| r.id() == id) {
            Some(brain)
        } else {
            None
        }
    }

    fn item_mut(&mut self, id: BrainID) -> Option<&mut Brain> {
        if let Some(brain) = self.brains.iter_mut().find(|r| r.id() == id) {
            Some(brain)
        } else {
            None
        }
    }
}
