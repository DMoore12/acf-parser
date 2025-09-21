pub mod errors;
pub mod parser;

pub mod prelude {
    #[doc(hidden)]
    pub use crate::parser::{parse_acf, Acf};

    #[doc(hidden)]
    pub use crate::errors::AcfError;
}
