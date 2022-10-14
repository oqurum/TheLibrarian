use tokio_postgres::{types::FromSql, Row};

mod auth;
mod book;
mod book_person;
mod book_tag;
mod collection;
mod collection_item;
mod edit;
mod image;
mod member;
mod metadata_search;
mod person;
mod person_alt;
mod search_global;
mod search_servers;
mod server_link;
mod tag;

use crate::Result;
pub use auth::*;
pub use book::*;
pub use book_person::*;
pub use book_tag::*;
pub use collection::*;
pub use collection_item::*;
pub use edit::*;

pub use self::image::*;
pub use member::*;
pub use metadata_search::*;
pub use person::*;
pub use person_alt::*;
pub use search_global::*;
pub use search_servers::*;
pub use server_link::*;
pub use tag::*;

pub trait TableRow
where
    Self: Sized,
{
    fn create(row: &mut AdvRow) -> Result<Self>;

    fn from_row(value: Row) -> Result<Self> {
        Self::create(&mut AdvRow {
            index: 0,
            row: value,
        })
    }
}

pub struct AdvRow {
    index: usize,
    row: Row,
}

impl AdvRow {
    #[allow(clippy::should_implement_trait)]
    pub fn next<'a, T: FromSql<'a>>(&'a mut self) -> Result<T> {
        self.index += 1;

        Ok(self.row.try_get(self.index - 1)?)
    }

    pub fn next_opt<'a, T: FromSql<'a>>(&'a mut self) -> Result<Option<T>> {
        self.next()
    }
}

pub fn row_int_to_usize(value: Row) -> Result<usize> {
    Ok(value.try_get::<_, i32>(0)? as usize)
}

pub fn row_bigint_to_usize(value: Row) -> Result<usize> {
    Ok(value.try_get::<_, i64>(0)? as usize)
}

fn boxed_to_dyn_vec<V: ?Sized>(value: &[Box<V>]) -> Vec<&V> {
    value.iter().map(|v| &**v).collect()
}
