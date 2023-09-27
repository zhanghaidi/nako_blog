pub mod art;
pub mod attach;
pub mod cate;
pub mod comment;
pub mod friendlink;
pub mod guestbook;
pub mod page;
pub mod setting;
pub mod tag;
pub mod user;

#[inline]
pub fn default<T: Default>() -> T {
    std::default::Default::default()
}
