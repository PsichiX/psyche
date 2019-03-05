use psyche::core::id::ID;

pub trait Named<T> {
    fn id(&self) -> ID<T>;
}

pub trait ItemsManager<T>
where
    T: Named<T>,
{
    fn items(&self) -> &[T];

    fn add(&mut self, item: T) -> ID<T>;

    fn create(&mut self) -> ID<T>;

    fn create_with<F>(&mut self, with: F) -> ID<T>
    where
        F: FnMut(&mut T, &mut Self);

    fn destroy(&mut self, id: ID<T>) -> bool;

    fn with<F, R>(&mut self, id: ID<T>, with: F) -> Option<R>
    where
        F: FnMut(&mut T, &mut Self) -> R;

    fn item(&self, id: ID<T>) -> Option<&T>;

    fn item_mut(&mut self, id: ID<T>) -> Option<&mut T>;
}
