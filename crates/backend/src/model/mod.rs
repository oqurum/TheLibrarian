use rusqlite::{Row, types::FromSql};

mod auth;
mod book;
mod book_person;
mod book_tag;
mod edit;
mod image;
mod member;
mod person;
mod person_alt;
mod tag;

pub use auth::*;
pub use book::*;
pub use book_person::*;
pub use book_tag::*;
pub use edit::*;
pub use self::image::*;
pub use member::*;
pub use person::*;
pub use person_alt::*;
pub use tag::*;



pub trait TableRow<'a> where Self: Sized {
    fn create(row: &mut AdvRow<'a>) -> rusqlite::Result<Self>;

    fn from_row(value: &'a Row<'a>) -> rusqlite::Result<Self> {
        Self::create(&mut AdvRow {
            index: 0,
            row: value
        })
    }
}


pub struct AdvRow<'a> {
    index: usize,
    row: &'a Row<'a>,
}

impl<'a> AdvRow<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn next<T: FromSql>(&mut self) -> rusqlite::Result<T> {
        self.index += 1;

        self.row.get(self.index - 1)
    }

    pub fn next_opt<T: FromSql>(&mut self) -> rusqlite::Result<Option<T>> {
        self.next()
    }
}