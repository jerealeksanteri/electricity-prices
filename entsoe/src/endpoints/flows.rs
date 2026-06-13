use crate::{client::Entsoe, errors::EntsoeError, models::flows::FlowMarketDocument, models::request::RequestParameters};

impl Entsoe {
    /// A11 — physical flow from `params.out_domain` to `params.in_domain`.
    pub async fn get_flows(&self, params: &RequestParameters) -> Result<FlowMarketDocument, EntsoeError> {
        self.request_doc(params).await
    }
}