use crate::{client::Entsoe, errors::EntsoeError, models::request::RequestParameters, models::prices::PublicationMarketDocument};

impl Entsoe {
    pub async fn get_prices(&self, params: &RequestParameters) -> Result<PublicationMarketDocument, EntsoeError> {
        self.request_doc(params).await
    }
}
