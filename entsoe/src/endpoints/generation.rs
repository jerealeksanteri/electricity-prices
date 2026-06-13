use crate::{client::Entsoe, errors::EntsoeError, models::generation::GlMarketDocument, models::request::RequestParameters};

impl Entsoe {
    /// A75 — actual generation per production type for `params.in_domain`.
    pub async fn get_generation(&self, params: &RequestParameters) -> Result<GlMarketDocument, EntsoeError> {
        self.request_doc(params).await
    }
}
