use reqwest_middleware::reqwest::{Request, Response};
use reqwest_middleware::Error;

/// Describes:
///
/// - which request outcomes should be retried
/// - how the request should be updated to retry
#[async_trait::async_trait]
pub trait Chainer {
    type State: Sync + Send + Default;

    /// Inspect the result of the previous request, to decide whether to make
    /// another request.
    ///
    /// - If another request is required, update the previous request to form the
    /// next request in the chain, and return `Ok(None)`.
    /// - If the response is ready, return it inside `Ok(Some(response))`.
    /// - If an error occurs and you cannot continue, return `Err(error)`.
    ///
    /// Returning a response, or an error, will result in termination of the chain.
    ///
    /// Information is available from:
    ///
    /// - self (global state, instantiated at middleware creation)
    /// - result (the result of the previous request)
    /// - state (local state, instantiated for each request chain)
    ///
    /// Global side effects can be managed via interior mutability of `self`.
    async fn chain(
        &self,
        result: Result<Response, Error>,
        state: &mut Self::State,
        request: &mut Request,
    ) -> Result<Option<Response>, Error>;

    /// Safety valve to protect against infinite chaining.
    ///
    /// This value may be overriden by the user.
    fn max_chain_length(&self) -> u32 {
        /// We limit the number of retries to avoid stack-overflow issues due to the recursion.
        ///
        /// This can be increased by the user.
        const DEFAULT_MAXIMUM_CHAIN_LENGTH: u32 = 7;
        DEFAULT_MAXIMUM_CHAIN_LENGTH
    }
}

pub struct ChainMiddleware<T> {
    inner: T,
}

impl<T> ChainMiddleware<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    pub(crate) fn inner(&self) -> &T {
        &self.inner
    }
}
