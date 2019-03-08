pub struct Switch<T> {
    index: usize,
    options: Vec<T>,
}

impl<T> Switch<T>
where
    T: Clone,
{
    pub fn new(options: usize, value: T) -> Self {
        Self {
            index: 0,
            options: vec![value; options],
        }
    }

    pub fn with_options(options: Vec<T>) -> Self {
        Self { index: 0, options }
    }

    pub fn switch(&mut self) {
        if !self.options.is_empty() {
            self.index = (self.index + 1) % self.options.len();
        }
    }

    pub fn iterate(&mut self) -> Option<(&T, &mut T)> {
        if !self.options.is_empty() {
            let prev = self.index;
            self.index = (self.index + 1) % self.options.len();
            let next = self.index;
            unsafe {
                let prev_option_ptr = self.options.as_ptr().offset(prev as isize);
                let next_option_ptr = self.options.as_mut_ptr().offset(next as isize);
                Some((
                    prev_option_ptr.as_ref().unwrap(),
                    next_option_ptr.as_mut().unwrap(),
                ))
            }
        } else {
            None
        }
    }

    pub fn get(&self) -> Option<&T> {
        if !self.options.is_empty() {
            Some(&self.options[self.index])
        } else {
            None
        }
    }

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
