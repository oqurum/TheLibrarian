mod thumbnail;
mod source;
mod isbn;

use serde::{Serialize, Deserialize};
pub use thumbnail::*;
pub use source::*;
pub use isbn::*;




#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Either<A, B> {
	Left(A),
	Right(B),
}