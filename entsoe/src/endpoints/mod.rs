use crate::{client::Entsoe, errors::EntsoeError, models::acknowledgement::AcknowledgementMarketDocument, models::request::RequestParameters};
use serde::de::DeserializeOwned;
use tracing::debug;

pub mod prices;
pub mod generation;
pub mod load;
pub mod flows;

impl Entsoe {
    /// Build the query, send it, surface API acknowledgements as errors, and
    /// deserialize the XML body into `T`.
    pub(crate) async fn request_doc<T: DeserializeOwned>(
        &self,
        params: &RequestParameters,
    ) -> Result<T, EntsoeError> {
        let url_query = params
            .to_params()
            .into_iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");
        let url = format!("{}?{}", self.base_url(), url_query);
        debug!("request_doc url = {}", url);

        let text = self
            .http()
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        if text.contains("Acknowledgement_MarketDocument") {
            let ack: AcknowledgementMarketDocument =
                quick_xml::de::from_str(&text).map_err(EntsoeError::XmlParse)?;
            return Err(EntsoeError::ApiError { code: ack.reason.code, text: ack.reason.text });
        }
        quick_xml::de::from_str(&text).map_err(EntsoeError::XmlParse)
    }
}