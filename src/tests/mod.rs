#![allow(missing_docs)] // TODO need a better solution

#[cfg(feature = "test-e2e")]
pub mod e2e;

#[cfg(all(test, feature = "test-coverage"))]
mod coverage;
