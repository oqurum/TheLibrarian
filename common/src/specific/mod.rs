use serde::{Serialize, Deserialize};

pub mod edit;
mod language;
mod thumbnail;
mod source;
mod isbn;
mod ids;

pub use language::*;
pub use thumbnail::*;
pub use source::*;
pub use isbn::*;
pub use ids::*;




#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub enum Either<A, B> {
	Left(A),
	Right(B),
}