//! # entsoe
//!
//! A small async client for the [ENTSO-E Transparency Platform] REST API.
//!
//! The [`Entsoe`] client exposes one method per supported document type:
//!
//! - [`Entsoe::get_prices`] — A44, day-ahead prices
//! - [`Entsoe::get_generation`] — A75, actual generation per production type
//! - [`Entsoe::get_load_forecast`] — A65, total load forecast
//! - [`Entsoe::get_flows`] — A11, physical cross-border flows
//!
//! Each takes a [`RequestParameters`] (document type, domains, period, and the
//! security token) and returns the parsed XML market document.
//!
//! [ENTSO-E Transparency Platform]: https://transparency.entsoe.eu/

mod client;
mod errors;
mod models;
mod endpoints;

pub use client::Entsoe;
pub use errors::EntsoeError;
pub use models::{
    Authorization,
    RequestParameters,
    PublicationMarketDocument,
    DocumentType,
    PeriodTimestamp,
    Domain,
    ContractMarketAgreementType,
    TimeSeries,
    TimeInterval,
    Point,
    DomainMrid,
    Period,
    // Generation / Load types
    GlMarketDocument,
    GlTimeSeries,
    GlPeriod,
    GlPoint,
    MktPsrType,
    // Flow types
    FlowMarketDocument,
    FlowTimeSeries,
    FlowPeriod,
    FlowPoint,
};
pub use models::request::{ProcessType, PsrType};