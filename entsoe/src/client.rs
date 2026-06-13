use reqwest::Client;
use crate::models::request::Authorization;

const BASE_URL: &str = "https://web-api.tp.entsoe.eu/api";

pub struct Entsoe {
    http: Client,
    authorization: Authorization
}

impl Entsoe {
    pub fn new(auth: Authorization) -> Self {
        Self {
            http: Client::new(),
            authorization: auth
        }
    }

    pub(crate) fn http(&self) -> &Client {
        &self.http
    }

    pub(crate) fn base_url(&self) -> &str {
        BASE_URL
    }
    
    pub(crate) fn get_authorization(&self) -> String {
        self.authorization.get_authorization()
    }
}

impl Default for Entsoe {
    fn default() -> Self {
        Self::new(Default::default())
    }
}