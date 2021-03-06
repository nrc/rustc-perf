//! Result and error types representing the outcome of compiling a function.

use crate::verifier::VerifierErrors;
use failure_derive::Fail;

/// A compilation error.
///
/// When Cranelift fails to compile a function, it will return one of these error codes.
#[derive(Fail, Debug, PartialEq, Eq)]
pub enum CodegenError {
    /// A list of IR verifier errors.
    ///
    /// This always represents a bug, either in the code that generated IR for Cranelift, or a bug
    /// in Cranelift itself.
    #[fail(display = "Verifier errors:\n{}", _0)]
    Verifier(#[cause] VerifierErrors),

    /// An implementation limit was exceeded.
    ///
    /// Cranelift can compile very large and complicated functions, but the [implementation has
    /// limits][limits] that cause compilation to fail when they are exceeded.
    ///
    /// [limits]: https://cranelift.readthedocs.io/en/latest/ir.html#implementation-limits
    #[fail(display = "Implementation limit exceeded")]
    ImplLimitExceeded,

    /// The code size for the function is too large.
    ///
    /// Different target ISAs may impose a limit on the size of a compiled function. If that limit
    /// is exceeded, compilation fails.
    #[fail(display = "Code for function is too large")]
    CodeTooLarge,
}

/// A convenient alias for a `Result` that uses `CodegenError` as the error type.
pub type CodegenResult<T> = Result<T, CodegenError>;

impl From<VerifierErrors> for CodegenError {
    fn from(e: VerifierErrors) -> Self {
        CodegenError::Verifier(e)
    }
}
