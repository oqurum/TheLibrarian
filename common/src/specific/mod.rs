use serde::{Serialize, Deserialize};

mod thumbnail;
mod source;
mod isbn;
mod ids;

pub use thumbnail::*;
pub use source::*;
pub use isbn::*;
pub use ids::*;




#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Either<A, B> {
	Left(A),
	Right(B),
}