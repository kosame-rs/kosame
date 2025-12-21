pub trait Execute<T> {
    type Result;

    fn execute(self, target: &mut T) -> Self::Result;
}
