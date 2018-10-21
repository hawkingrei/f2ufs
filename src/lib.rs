#![feature(integer_atomics)]
pub mod f2fs;
pub mod trace;
pub type block_t = u32;
pub type nid_t = u32;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
