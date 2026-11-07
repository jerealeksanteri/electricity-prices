use thiserror::Error;

#[derive(Debug, Error)]
pub enum EntsoeError {
    /// Always constructed via `From<reqwest::Error>`, which strips the request
    /// URL — it carries the `securityToken` query parameter.
    #[error("HTTP request failed: {0}")]
    Http(reqwest::Error),

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

impl From<reqwest::Error> for EntsoeError {
    fn from(e: reqwest::Error) -> Self {
        EntsoeError::Http(e.without_url())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// reqwest embeds the request URL (including the `securityToken` query
    /// parameter) in its error messages. That must never surface through
    /// `EntsoeError`, since these errors end up in logs and UI.
    #[tokio::test]
    async fn http_error_does_not_leak_url_or_token() {
        let err = reqwest::Client::new()
            .get("http://127.0.0.1:1/api?securityToken=SUPER-SECRET-TOKEN")
            .send()
            .await
            .expect_err("port 1 should refuse the connection");
        let msg = EntsoeError::from(err).to_string();
        assert!(!msg.contains("SUPER-SECRET-TOKEN"), "token leaked: {msg}");
        assert!(!msg.contains("securityToken"), "query leaked: {msg}");
    }
}
