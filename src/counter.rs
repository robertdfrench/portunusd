pub struct RoundRobin<'slice, T> {
    slice: &'slice [T],
    counter: usize
}

impl<'slice, T> RoundRobin<'slice, T> {
    pub fn new(slice: &'slice [T]) -> Self {
        let counter = slice.len();
        Self{ slice, counter }
    }

    pub fn next<'rr>(&'rr mut self) -> &'slice T {
        self.counter += 1;
        if self.counter >= self.slice.len() {
            self.counter = 0;
        }
        &self.slice[self.counter]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn loops_around() {
        let data = vec![1,2,3];
        let mut rr = RoundRobin::new(&data);
        assert_eq!(rr.next(), &1);
        assert_eq!(rr.next(), &2);
        assert_eq!(rr.next(), &3);
        assert_eq!(rr.next(), &1);
    }
}
