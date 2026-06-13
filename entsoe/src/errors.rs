use thiserror::Error;

#[derive(Debug, Error)]
pub enum EntsoeError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Unexpected API response: {0}")]
    Parse(String),

    #[error("XML parse error: {0}")]
    XmlParse(quick_xml::DeError),

    #[error("Config error: {0}")]
    Config(String),

    /// The API returned an Acknowledgement_MarketDocument indicating no data was found
    /// or another API-level error. `code` is the reason code (e.g. "999") and
    /// `text` is the human-readable reason message.
    #[error("API error (code {code}): {text}")]
    ApiError { code: String, text: String },
}
