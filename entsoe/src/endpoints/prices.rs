use crate::{
    client::Entsoe,
    errors::EntsoeError,
    models::acknowledgement::AcknowledgementMarketDocument,
    models::request::RequestParameters,
    models::prices::PublicationMarketDocument
};
use tracing::debug;

impl Entsoe {
    pub async fn get_prices(&self, params: &RequestParameters) -> Result<PublicationMarketDocument, EntsoeError> {

        debug!("Getting prices from Ensoe");
        let url_query = params.to_params()
            .into_iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        let url = format!("{}?{}", self.base_url(), url_query);
        debug!("url = {}", url);
        debug!("params = {:?}", params);
        debug!("query = {:?}", url_query);
        debug!("authorization = {:?}", self.get_authorization());

        let text = self
            .http()
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        debug!("text = {}", text);

        // If the response is an Acknowledgement_MarketDocument the API is
        // signalling an error (e.g. "No matching data found"). Surface that as
        // a proper EntsoeError instead of a confusing XML parse failure.
        if text.contains("Acknowledgement_MarketDocument") {
            let ack: AcknowledgementMarketDocument =
                quick_xml::de::from_str(&text).map_err(EntsoeError::XmlParse)?;
            return Err(EntsoeError::ApiError {
                code: ack.reason.code,
                text: ack.reason.text,
            });
        }

        quick_xml::de::from_str(&text).map_err(EntsoeError::XmlParse)
    }
}