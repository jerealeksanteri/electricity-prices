use std::str::FromStr;
use std::fmt;
use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct Authorization {
    key: String,
    value: String,
}

impl Authorization {
    pub fn new(value: String) -> Self {
        Self {
            key: "securityToken".to_string(),
            value
        }
    }

    pub(crate) fn get_authorization(&self) -> String {
        format!("&{}={}", self.key, self.value)
    }
}

impl Default for Authorization {
    fn default() -> Self {
        Self {
            key: "securityToken".to_string(),
            value: String::new(),
        }
    }
}

#[derive(Debug)]
pub struct RequestParameters {
    pub document_type: DocumentType,
    pub out_domain: Domain,
    pub in_domain: Domain,
    pub period_start: PeriodTimestamp,
    pub period_end: PeriodTimestamp,
    pub authorization: Authorization,
}

impl RequestParameters {
    pub(crate) fn to_params(&self) -> Vec<(&str, String)> {
        vec![
            ("documentType", self.document_type.to_string()),
            ("out_Domain", self.out_domain.to_string()),
            ("in_Domain", self.in_domain.to_string()),
            ("periodStart", self.period_start.to_string()),
            ("periodEnd", self.period_end.to_string()),
            (self.authorization.key.as_str(), self.authorization.value.clone()),
        ]
    }
}

#[derive(Debug)]
pub struct PeriodTimestamp(pub DateTime<Utc>);

impl fmt::Display for PeriodTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.format("%Y%m%d%H%M"))
    }
}

#[derive(Debug)]
pub enum DocumentType {
    A44,
}

impl FromStr for DocumentType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "A44" => Ok(DocumentType::A44),
            _ => Err(()),
        }
    }
}

impl DocumentType {
    fn as_str(&self) -> &'static str {
        match self {
            Self::A44 => "A44",
        }
    }
}
impl fmt::Display for DocumentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug)]
pub enum ContractMarketAgreementType {
    Daily,
    IntraDay
}

impl ContractMarketAgreementType {
    fn as_str(&self) -> &str {
        match self {
            ContractMarketAgreementType::Daily => "A01",
            ContractMarketAgreementType::IntraDay => "A07",
        }
    }
}

impl fmt::Display for ContractMarketAgreementType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for ContractMarketAgreementType {
    type Err = ();
    fn from_str(s: &str) -> Result<ContractMarketAgreementType, Self::Err> {
        match s {
            "A01" => Ok(ContractMarketAgreementType::Daily),
            "A07" => Ok(ContractMarketAgreementType::IntraDay),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub enum Domain {
    FI,
    SE1,
    SE2,
    SE3,
    SE4,
    NO1,
    NO1A,
    NO2,
    NO3,
    NO4,
    NO5,
    DK1,
    DK2,
}

impl Domain {
    fn as_str(&self) -> &'static str {
        match self {
            Self::FI   => "10YFI-1--------U",
            Self::SE1  => "10Y1001A1001A44P",
            Self::SE2  => "10Y1001A1001A45N",
            Self::SE3  => "10Y1001A1001A46L",
            Self::SE4  => "10Y1001A1001A47J",
            Self::NO1  => "10YNO-1--------2",
            Self::NO1A => "10Y1001A1001A64J",
            Self::NO2  => "10YNO-2--------T",
            Self::NO3  => "10YNO-3--------J",
            Self::NO4  => "10YNO-4--------9",
            Self::NO5  => "10Y1001A1001A48H",
            Self::DK1  => "10YDK-1--------W",
            Self::DK2  => "10YDK-2--------M",
        }
    }

    /// Human-readable bidding-zone code (e.g. "FI", "SE1").
    pub fn name(&self) -> &'static str {
        match self {
            Self::FI   => "FI",
            Self::SE1  => "SE1",
            Self::SE2  => "SE2",
            Self::SE3  => "SE3",
            Self::SE4  => "SE4",
            Self::NO1  => "NO1",
            Self::NO1A => "NO1A",
            Self::NO2  => "NO2",
            Self::NO3  => "NO3",
            Self::NO4  => "NO4",
            Self::NO5  => "NO5",
            Self::DK1  => "DK1",
            Self::DK2  => "DK2",
        }
    }
}

impl FromStr for Domain {
    type Err = ();
    fn from_str(input: &str) -> Result<Domain, Self::Err> {

        match input {
            "10YFI-1--------U" => Ok(Domain::FI),
            "10Y1001A1001A44P" => Ok(Domain::SE1),
            "10Y1001A1001A45N" => Ok(Domain::SE2),
            "10Y1001A1001A46L" => Ok(Domain::SE3),
            "10Y1001A1001A47J" => Ok(Domain::SE4),
            "10YNO-1--------2" => Ok(Domain::NO1),
            "10Y1001A1001A64J" => Ok(Domain::NO1A),
            "10YNO-2--------T" => Ok(Domain::NO2),
            "10YNO-3--------J" => Ok(Domain::NO1),
            "10YNO-4--------9" => Ok(Domain::NO1),
            "10Y1001A1001A48H" => Ok(Domain::NO5),
            "10YDK-1--------W" => Ok(Domain::DK1),
            "10YDK-2--------M" => Ok(Domain::DK2),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Domain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}