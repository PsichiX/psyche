//! Tool used to sequentialy switch between many options of same data.

/// Collection that contains several versions/options of data so you can switch between them in
/// sequential manner. It can be used to produce next data frame based on previous data frame.
///
/// # Example
/// ```
/// use psyche_utils::switch::Switch;
///
/// let mut switch = Switch::new(2, vec![1, 2, 4]);
/// if let Some((prev, next)) = switch.iterate() {
///     for i in 0..prev.len() {
///         // next frame item equals sum of two neighbors.
///         let start = i.max(1) - 1;
///         let end = (i + 2).min(prev.len());
///         next[i] = (start..end).map(|i| prev[i]).sum();
///     }
/// }
/// assert_eq!(switch.get().unwrap(), &vec![3, 7, 6]);
/// ```
pub struct Switch<T> {
    index: usize,
    options: Vec<T>,
}

impl<T> Switch<T> {
    /// Creates new switch with number of options and cloned value applied for each of them.
    ///
    /// # Arguments
    /// * `options` - Number of options that switch will hold.
    /// * `value` - Initial value applied for each of options.
    ///
    /// # Return
    /// Instance of switch.
    ///
    /// # Example
    /// ```
    /// use psyche_utils::switch::Switch;
    ///
    /// let switch = Switch::new(2, vec![1, 2, 4]);
    /// ```
    pub fn new(options: usize, value: T) -> Self
    where
        T: Clone,
    {
        Self {
            index: 0,
            options: vec![value; options],
        }
    }

    /// Creates new switch with initial options values.
    ///
    /// # Note
    /// Make sure that your options values have same length if they are for example arrays or
    /// vectors or any other collection that needs to have same length across each iteration.
    ///
    /// # Arguments
    /// * `options` - Initial values applied for options.
    ///
    /// # Return
    /// Instance of switch.
    ///
    /// # Example
    /// ```
    /// use psyche_utils::switch::Switch;
    ///
    /// let switch = Switch::with_options(vec![
    ///     vec![1, 2, 4],
    ///     vec![3, 7, 6],
    /// ]);
    /// ```
    pub fn with_options(options: Vec<T>) -> Self {
        Self { index: 0, options }
    }

    /// Gets currently active option index.
    ///
    /// # Return
    /// Index of currently active switch option.
    ///
    /// # Example
    /// ```
    /// use psyche_utils::switch::Switch;
    ///
    /// let mut switch = Switch::new(2, 0);
    /// assert_eq!(switch.index(), 0);
    /// switch.switch();
    /// assert_eq!(switch.index(), 1);
    /// switch.switch();
    /// assert_eq!(switch.index(), 0);
    /// ```
    pub fn index(&self) -> usize {
        self.index
    }

    /// Gets number of options that holds.
    ///
    /// # Return
    /// Number of switch options.
    ///
    /// # Example
    /// ```
    /// use psyche_utils::switch::Switch;
    ///
    /// let mut switch = Switch::new(2, 0);
    /// assert_eq!(switch.count(), 2);
    /// ```
    pub fn count(&self) -> usize {
        self.options.len()
    }

    /// Switches to next option.
    ///
    /// # Example
    /// ```
    /// use psyche_utils::switch::Switch;
    ///
    /// let mut switch = Switch::with_options(vec![0, 1]);
    /// assert_eq!(*switch.get().unwrap(), 0);
    /// switch.switch();
    /// assert_eq!(*switch.get().unwrap(), 1);
    /// ```
    pub fn switch(&mut self) {
        if !self.options.is_empty() {
            self.index = (self.index + 1) % self.options.len();
        }
    }

    /// Switches to next option and returns pair of _previous_ and _next_ options.
    ///
    /// # Return
    /// Pair of _previous_ and _next_ options if holds more than one.
    ///
    /// # Example
    /// ```
    /// use psyche_utils::switch::Switch;
    ///
    /// let mut switch = Switch::with_options(vec![0, 1]);
    /// assert_eq!(switch.iterate().unwrap(), (&0, &mut 1));
    /// ```
    pub fn iterate(&mut self) -> Option<(&T, &mut T)> {
        if !self.options.is_empty() {
            let prev = self.index;
            self.index = (self.index + 1) % self.options.len();
            let next = self.index;
            if prev != next {
                unsafe {
                    let prev_option_ptr = self.options.as_ptr().offset(prev as isize);
                    let next_option_ptr = self.options.as_mut_ptr().offset(next as isize);
                    return Some((
                        prev_option_ptr.as_ref().unwrap(),
                        next_option_ptr.as_mut().unwrap(),
                    ));
                }
            }
        }
        None
    }

    /// Gets currently active option if any.
    ///
    /// # Return
    /// Reference to currently active option.
    ///
    /// # Example
    /// ```
    /// use psyche_utils::switch::Switch;
    ///
    /// let mut switch = Switch::with_options(vec![0, 1]);
    /// assert_eq!(switch.get().unwrap(), &0);
    /// ```
    pub fn get(&self) -> Option<&T> {
        if !self.options.is_empty() {
            Some(&self.options[self.index])
        } else {
            None
        }
    }

    /// Gets currently active option if any.
    ///
    /// # Return
    /// Mutable reference to currently active option.
    ///
    /// # Example
    /// ```
    /// use psyche_utils::switch::Switch;
    ///
    /// let mut switch = Switch::with_options(vec![0, 1]);
    /// assert_eq!(switch.get_mut().unwrap(), &mut 0);
    /// ```
    pub fn get_mut(&mut self) -> Option<&mut T> {
        if !self.options.is_empty() {
            Some(&mut self.options[self.index])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_switch() {
        let mut switch = Switch::with_options(vec![1, 2]);
        assert_eq!(*switch.get().unwrap(), 1);
        switch.switch();
        assert_eq!(*switch.get().unwrap(), 2);
        switch.switch();
        assert_eq!(*switch.get().unwrap(), 1);
        switch.switch();
        assert_eq!(*switch.get().unwrap(), 2);
        assert_eq!(switch.iterate().unwrap(), (&2, &mut 1))
    }
}
