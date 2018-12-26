pub mod buffer;
pub mod mp;
pub mod sp;

pub use self::buffer::Buffer;
pub use self::mp::mc as mpmc;
pub use self::sp::mc as spmc;
pub use self::sp::sc as spsc;
/// The return type for `try_recv` methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TryRecv<T> {
    /// Received a value.
    Data(T),
    /// Not received a value because the buffer is empty.
    Empty,
    /// Lost the race to a concurrent operation. Try again.
    Retry,
}

impl<T> TryRecv<T> {
    /// Applies a function to the content of `TryRecv::Data`.
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> TryRecv<U> {
        match self {
            TryRecv::Data(v) => TryRecv::Data(f(v)),
            TryRecv::Empty => TryRecv::Empty,
            TryRecv::Retry => TryRecv::Retry,
        }
    }
}