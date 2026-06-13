use crate::{client::Entsoe, errors::EntsoeError, models::generation::GlMarketDocument, models::request::RequestParameters};

impl Entsoe {
    /// A65 — total load forecast for `params.out_bidding_zone_domain`.
    pub async fn get_load_forecast(&self, params: &RequestParameters) -> Result<GlMarketDocument, EntsoeError> {
        self.request_doc(params).await
    }
}