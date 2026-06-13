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
    pub out_domain: Option<Domain>,
    pub in_domain: Option<Domain>,
    pub out_bidding_zone_domain: Option<Domain>,
    pub period_start: PeriodTimestamp,
    pub period_end: PeriodTimestamp,
    pub process_type: Option<ProcessType>,
    pub psr_type: Option<PsrType>,
    pub authorization: Authorization,
}

impl RequestParameters {
    pub(crate) fn to_params(&self) -> Vec<(&str, String)> {
        let mut v: Vec<(&str, String)> = vec![
            ("documentType", self.document_type.to_string()),
            ("periodStart", self.period_start.to_string()),
            ("periodEnd", self.period_end.to_string()),
            (self.authorization.key.as_str(), self.authorization.value.clone()),
        ];
        if let Some(d) = &self.out_domain { v.push(("out_Domain", d.to_string())); }
        if let Some(d) = &self.in_domain { v.push(("in_Domain", d.to_string())); }
        if let Some(d) = &self.out_bidding_zone_domain { v.push(("outBiddingZone_Domain", d.to_string())); }
        if let Some(p) = &self.process_type { v.push(("processType", p.to_string())); }
        if let Some(p) = &self.psr_type { v.push(("psrType", p.to_string())); }
        v
    }
}

#[derive(Debug)]
pub struct PeriodTimestamp(pub DateTime<Utc>);

impl fmt::Display for PeriodTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.format("%Y%m%d%H%M"))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DocumentType {
    A44,
    A75,
    A65,
    A11,
}

impl FromStr for DocumentType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "A44" => Ok(DocumentType::A44),
            "A75" => Ok(DocumentType::A75),
            "A65" => Ok(DocumentType::A65),
            "A11" => Ok(DocumentType::A11),
            _ => Err(()),
        }
    }
}

impl DocumentType {
    fn as_str(&self) -> &'static str {
        match self {
            Self::A44 => "A44",
            Self::A75 => "A75",
            Self::A65 => "A65",
            Self::A11 => "A11",
        }
    }
}

impl fmt::Display for DocumentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ProcessType {
    Realised,
    DayAhead,
}

impl ProcessType {
    fn as_str(&self) -> &'static str {
        match self {
            ProcessType::Realised => "A16",
            ProcessType::DayAhead => "A01",
        }
    }
}

impl fmt::Display for ProcessType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PsrType(&'static str, &'static str); // (code, human name)

impl PsrType {
    pub fn from_code(code: &str) -> PsrType {
        // Return fully `'static` (code, name) pairs so `PsrType` stays `Copy`
        // with no per-call allocation/leak. Unknown codes collapse to B20/Other.
        match code {
            "B01" => PsrType("B01", "Biomass"),
            "B02" => PsrType("B02", "Fossil Brown coal/Lignite"),
            "B04" => PsrType("B04", "Fossil Gas"),
            "B05" => PsrType("B05", "Fossil Hard coal"),
            "B06" => PsrType("B06", "Fossil Oil"),
            "B09" => PsrType("B09", "Geothermal"),
            "B10" => PsrType("B10", "Hydro Pumped Storage"),
            "B11" => PsrType("B11", "Hydro Run-of-river"),
            "B12" => PsrType("B12", "Hydro Water Reservoir"),
            "B14" => PsrType("B14", "Nuclear"),
            "B15" => PsrType("B15", "Other renewable"),
            "B16" => PsrType("B16", "Solar"),
            "B17" => PsrType("B17", "Waste"),
            "B18" => PsrType("B18", "Wind Offshore"),
            "B19" => PsrType("B19", "Wind Onshore"),
            "B20" => PsrType("B20", "Other"),
            _ => PsrType("B20", "Other"),
        }
    }

    pub fn code(&self) -> &str {
        self.0
    }

    pub fn name(&self) -> &'static str {
        self.1
    }
}

impl fmt::Display for PsrType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
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

#[derive(Debug, Clone, Copy)]
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
    EE,
    RU,
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
            Self::EE   => "10Y1001A1001A39I",
            Self::RU   => "10Y1001A1001A49F",
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
            Self::EE   => "EE",
            Self::RU   => "RU",
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
            "10YNO-3--------J" => Ok(Domain::NO3),
            "10YNO-4--------9" => Ok(Domain::NO4),
            "10Y1001A1001A48H" => Ok(Domain::NO5),
            "10YDK-1--------W" => Ok(Domain::DK1),
            "10YDK-2--------M" => Ok(Domain::DK2),
            "10Y1001A1001A39I" => Ok(Domain::EE),
            "10Y1001A1001A49F" => Ok(Domain::RU),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Domain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_type_strings() {
        assert_eq!(ProcessType::Realised.to_string(), "A16");
        assert_eq!(ProcessType::DayAhead.to_string(), "A01");
    }

    #[test]
    fn psr_type_human_names() {
        assert_eq!(PsrType::from_code("B19").name(), "Wind Onshore");
        assert_eq!(PsrType::from_code("B14").name(), "Nuclear");
        assert_eq!(PsrType::from_code("ZZZ").name(), "Other");
    }

    #[test]
    fn domain_ee_ru_added() {
        assert_eq!(Domain::EE.to_string(), "10Y1001A1001A39I");
        assert_eq!(Domain::RU.to_string(), "10Y1001A1001A49F");
    }

    #[test]
    fn domain_from_str_no3_no4_fixed() {
        assert!(matches!(Domain::from_str("10YNO-3--------J"), Ok(Domain::NO3)));
        assert!(matches!(Domain::from_str("10YNO-4--------9"), Ok(Domain::NO4)));
    }

    #[test]
    fn params_emit_optional_fields_only_when_set() {
        let p = RequestParameters {
            document_type: DocumentType::A75,
            out_domain: None,
            in_domain: Some(Domain::FI),
            out_bidding_zone_domain: None,
            period_start: PeriodTimestamp("2024-01-01T00:00:00Z".parse().unwrap()),
            period_end: PeriodTimestamp("2024-01-02T00:00:00Z".parse().unwrap()),
            process_type: Some(ProcessType::Realised),
            psr_type: None,
            authorization: Authorization::new("tok".into()),
        };
        let params = p.to_params();
        let keys: Vec<&str> = params.iter().map(|(k, _)| *k).collect();
        assert!(keys.contains(&"documentType"));
        assert!(keys.contains(&"processType"));
        assert!(keys.contains(&"in_Domain"));
        assert!(!keys.contains(&"out_Domain"));        // None -> omitted
        assert!(!keys.contains(&"psrType"));           // None -> omitted
    }
}