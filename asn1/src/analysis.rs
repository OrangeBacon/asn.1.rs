//! Type checking and name resolution for ASN.1.
//! The following analysis passes are defined:
//! - Local: analyse each module in isolation to get its imports and exports.
//! - Global: resolve dependencies between modules (imports and exports).
//! - Name: resolve names and references within each module.
//! - Type: resolve types across all modules.
//! - Value: parse and analyse values now that the type of the value is known.
//! Note that modules can depend upon each other and must be checked at the
//! same time so circular and recursive dependency resolution can take place.
//!
//! The analysis passes are all based on the fact that the provided CST from the
//! parser is valid as far as the parser can tell.  Any parse errors that put
//! tokens in weird locations within the cst due to error recovery, etc (not
//! currently implemented) should ensure that either analysis does not run or
//! the recovered ast is valid.  Otherwise an internal compiler error will be
//! thrown.  If the error is that a structure that should not be present is,
//! however it could not be detected in parsing, then it likely will be thrown
//! as a type error for the user to fix and analysis to continue.

mod environment;
mod error;
mod global;
mod local;

pub mod context;

pub use error::AnalysisError;
