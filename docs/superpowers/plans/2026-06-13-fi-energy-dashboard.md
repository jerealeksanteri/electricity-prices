# fi-energy-dashboard Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Dioxus 0.6 fullstack Finnish electricity dashboard that fetches ENTSO-E prices, generation, load forecast, and cross-border flows server-side, caches them in moka, and renders dark-themed pages with ECharts charts.

**Architecture:** Cargo workspace: the repo root is the `fi-energy-dashboard` app, `entsoe/` is a path-dependency member crate. The `entsoe` crate is extended with three new endpoints (A75 generation, A65 load, A11 flows). The app's `server` module wraps the crate behind `#[server]` functions guarded by an `EntsoeCache` (moka). The `components` module renders pages via `use_server_future` + `Suspense`; charts are `charming` options serialized to JSON and injected into vendored `echarts.min.js` via `document::eval`.

**Tech Stack:** Rust 2024, Dioxus 0.6 (fullstack/router), Axum, moka 0.12 (future), chrono + chrono-tz, charming, thiserror/anyhow, dotenvy, serde, Tailwind via `dx`.

---

## Conventions for this plan

- **Commits are handled by the user.** Each "Commit" step lists the suggested `git add`/message, but DO NOT run it — tell the user the task is ready to commit and continue.
- The spec lives at `docs/superpowers/specs/2026-06-13-fi-energy-dashboard-design.md`.
- Default bidding zone constant used everywhere: `"10YFI-1--------U"` (Finland).
- Some exact Dioxus 0.6 / charming import paths carry minor uncertainty (flagged in the spec). Where a step says "if it does not compile, adjust the import", treat the surrounding intent as authoritative and fix the path against the compiler — this is not a placeholder, it is expected API-path reconciliation.
- Verification commands:
  - Crate unit tests: `cargo test -p entsoe`
  - App server-side type check: `cargo check --features server`
  - App client-side type check: `cargo check --features web`
  - Full dev run (manual): `dx serve` (requires `dioxus-cli` + a real `ENTSO_E_TOKEN` in `.env`)

---

## File Structure

**Workspace / config**
- `Cargo.toml` — workspace root + `fi-energy-dashboard` package, features `web`/`server`.
- `Dioxus.toml` — app name, asset dir.
- `tailwind.config.js`, `assets/tailwind.css` — Tailwind input.
- `assets/echarts.min.js` — vendored ECharts (downloaded once).
- `.env.example` — documents `ENTSO_E_TOKEN`, `BIND_ADDR`.
- `Dockerfile` — multi-stage build.

**`entsoe` crate (extend)**
- `entsoe/src/models/request.rs` — add `ProcessType`, `PsrType`, optional params, EE/RU domains, NO3/NO4 fix.
- `entsoe/src/models/generation.rs` — `GlMarketDocument` (A75/A65 shared shape) + psrType.
- `entsoe/src/models/flows.rs` — flow `Publication_MarketDocument` (quantity variant).
- `entsoe/src/models/mod.rs` — re-exports.
- `entsoe/src/endpoints/generation.rs` — `get_generation` (A75).
- `entsoe/src/endpoints/load.rs` — `get_load_forecast` (A65).
- `entsoe/src/endpoints/flows.rs` — `get_flows` (A11).
- `entsoe/src/endpoints/mod.rs` — module wiring.
- `entsoe/src/lib.rs` — re-exports.

**App (`src/`)**
- `src/main.rs` — client launch + server bootstrap (Axum + Extension).
- `src/server/mod.rs` — shared types + `ServerError` + module wiring + helpers.
- `src/server/cache.rs` — `EntsoeCache`, `next_price_publish()`.
- `src/server/entso.rs` — four `#[server]` functions + XML→domain mapping.
- `src/components/mod.rs` — module wiring.
- `src/components/app.rs` — `App`, `Route`, document head.
- `src/components/nav.rs` — `Nav` layout.
- `src/components/common.rs` — `Skeleton`, `ErrorBanner`.
- `src/components/pages/{mod,overview,prices,generation,forecast,flows}.rs`.
- `src/components/charts/{mod,price_chart,generation_pie,flow_map}.rs`.

---

## Phase 1 — Workspace & scaffold

### Task 1: Workspace root Cargo.toml

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Replace `Cargo.toml` with the workspace + app manifest**

```toml
[package]
name = "fi-energy-dashboard"
version = "0.1.0"
edition = "2024"

[dependencies]
dioxus = { version = "0.6", features = ["router"] }
entsoe = { path = "entsoe" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
chrono-tz = "0.10"
charming = "0.4"
thiserror = "2"
anyhow = "1"

# server-only deps
moka = { version = "0.12", features = ["future"], optional = true }
axum = { version = "0.7", optional = true }
tokio = { version = "1", features = ["full"], optional = true }
dotenvy = { version = "0.15", optional = true }
tracing-subscriber = { version = "0.3", optional = true }

[features]
default = []
web = ["dioxus/web"]
server = ["dioxus/server", "dep:moka", "dep:axum", "dep:tokio", "dep:dotenvy", "dep:tracing-subscriber"]

[workspace]
members = ["entsoe"]
```

- [ ] **Step 2: Verify the workspace resolves**

Run: `cargo metadata --no-deps --format-version 1 >/dev/null && echo OK`
Expected: `OK` (downloads index; no manifest errors). If `axum 0.7` conflicts with Dioxus 0.6's pinned axum, align the version to whatever `dioxus-fullstack` re-exports and prefer `dioxus::prelude` axum re-exports in later tasks.

- [ ] **Step 3: Commit (user runs)**

```bash
git add Cargo.toml
git commit -m "chore: convert repo into fi-energy-dashboard workspace"
```

### Task 2: Dioxus.toml + Tailwind + assets scaffold

**Files:**
- Create: `Dioxus.toml`
- Create: `tailwind.config.js`
- Create: `assets/tailwind.css`
- Create: `.env.example`

- [ ] **Step 1: Create `Dioxus.toml`**

```toml
[application]
name = "fi-energy-dashboard"

[web.app]
title = "FI Energy Dashboard"

[web.resource]
style = ["/assets/tailwind.css"]
script = []

[web.watcher]
watch_path = ["src", "assets"]
```

- [ ] **Step 2: Create `tailwind.config.js`**

```js
/** @type {import('tailwindcss').Config} */
module.exports = {
  mode: "all",
  content: ["./src/**/*.{rs,html}", "./dist/**/*.html"],
  theme: { extend: {} },
  plugins: [],
};
```

- [ ] **Step 3: Create `assets/tailwind.css`**

```css
@tailwind base;
@tailwind components;
@tailwind utilities;
```

- [ ] **Step 4: Create `.env.example`**

```
# ENTSO-E API security token (required). Request one at https://transparency.entsoe.eu/
ENTSO_E_TOKEN=
# Optional bind address for the server (default 0.0.0.0:8080)
BIND_ADDR=0.0.0.0:8080
```

- [ ] **Step 5: Commit (user runs)**

```bash
git add Dioxus.toml tailwind.config.js assets/tailwind.css .env.example
git commit -m "chore: add Dioxus + Tailwind scaffold"
```

### Task 3: Minimal compiling app shell

**Files:**
- Modify: `src/main.rs`
- Create: `src/components/mod.rs`
- Create: `src/components/app.rs`

- [ ] **Step 1: Create `src/components/app.rs` (placeholder App)**

```rust
use dioxus::prelude::*;

#[component]
pub fn App() -> Element {
    rsx! {
        div { class: "min-h-screen bg-gray-900 text-gray-100 p-8",
            h1 { class: "text-2xl font-bold", "FI Energy Dashboard" }
        }
    }
}
```

- [ ] **Step 2: Create `src/components/mod.rs`**

```rust
pub mod app;
```

- [ ] **Step 3: Replace `src/main.rs`**

```rust
mod components;
#[cfg(feature = "server")]
mod server;

use components::app::App;

fn main() {
    #[cfg(feature = "server")]
    server_main();

    #[cfg(not(feature = "server"))]
    dioxus::launch(App);
}

#[cfg(feature = "server")]
fn server_main() {
    // Real bootstrap is implemented in Phase 3 (Task 14). For now just launch.
    dioxus::launch(App);
}
```

- [ ] **Step 4: Verify both targets type-check**

Run: `cargo check --features web` then `cargo check --features server`
Expected: both succeed (warnings about unused `server` module are fine until Phase 3).

- [ ] **Step 5: Commit (user runs)**

```bash
git add src/main.rs src/components/
git commit -m "feat: minimal compiling Dioxus app shell"
```

---

## Phase 2 — Extend the `entsoe` crate

### Task 4: Request params — ProcessType, PsrType, optional fields, EE/RU, NO fix

**Files:**
- Modify: `entsoe/src/models/request.rs`
- Test: `entsoe/src/models/request.rs` (inline `#[cfg(test)]`)

- [ ] **Step 1: Write failing tests at the bottom of `request.rs`**

```rust
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
        assert!(!keys.contains(&"out_Domain"));        // None → omitted
        assert!(!keys.contains(&"psrType"));           // None → omitted
    }
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p entsoe request::tests`
Expected: FAIL (compile errors — `ProcessType`, `PsrType`, new fields, `DocumentType::A75` not defined).

- [ ] **Step 3: Extend `request.rs`**

Replace the `RequestParameters` struct + its `to_params`, extend `DocumentType`, and add `ProcessType`/`PsrType`. Keep existing `PeriodTimestamp`, `Authorization`, `ContractMarketAgreementType` as-is. New/changed code:

```rust
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

#[derive(Debug, Clone, Copy)]
pub enum ProcessType { Realised, DayAhead }
impl ProcessType {
    fn as_str(&self) -> &'static str {
        match self { ProcessType::Realised => "A16", ProcessType::DayAhead => "A01" }
    }
}
impl fmt::Display for ProcessType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.as_str()) }
}

#[derive(Debug, Clone, Copy)]
pub struct PsrType(pub &'static str, &'static str); // (code, human name)
impl PsrType {
    pub fn from_code(code: &str) -> PsrType {
        let name = match code {
            "B01" => "Biomass", "B02" => "Fossil Brown coal/Lignite", "B04" => "Fossil Gas",
            "B05" => "Fossil Hard coal", "B06" => "Fossil Oil", "B09" => "Geothermal",
            "B10" => "Hydro Pumped Storage", "B11" => "Hydro Run-of-river",
            "B12" => "Hydro Water Reservoir", "B14" => "Nuclear", "B15" => "Other renewable",
            "B16" => "Solar", "B17" => "Waste", "B18" => "Wind Offshore",
            "B19" => "Wind Onshore", "B20" => "Other", _ => "Other",
        };
        // Leak-free: codes are short static-ish; store owned via Box::leak only if needed.
        PsrType(Box::leak(code.to_string().into_boxed_str()), name)
    }
    pub fn code(&self) -> &str { self.0 }
    pub fn name(&self) -> &'static str { self.1 }
}
impl fmt::Display for PsrType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
}
```

Extend `DocumentType`:

```rust
#[derive(Debug, Clone, Copy)]
pub enum DocumentType { A44, A75, A65, A11 }

impl DocumentType {
    fn as_str(&self) -> &'static str {
        match self {
            Self::A44 => "A44", Self::A75 => "A75",
            Self::A65 => "A65", Self::A11 => "A11",
        }
    }
}
```
(Update the existing `FromStr for DocumentType` to map all four, or delete it if unused — it is only used in tests.)

Add EE/RU to `Domain` (enum + `as_str` + `name` + `from_str`) and fix NO3/NO4:

```rust
// in enum Domain { ... add: }
    EE,
    RU,
// in as_str:
    Self::EE => "10Y1001A1001A39I",
    Self::RU => "10Y1001A1001A49F",
// in name:
    Self::EE => "EE",
    Self::RU => "RU",
// in from_str: fix and add
    "10YNO-3--------J" => Ok(Domain::NO3),
    "10YNO-4--------9" => Ok(Domain::NO4),
    "10Y1001A1001A39I" => Ok(Domain::EE),
    "10Y1001A1001A49F" => Ok(Domain::RU),
```

> Note: `PsrType::from_code` uses `Box::leak` to keep a `&'static` code with zero added deps. psrType codes come from a fixed ENTSO-E vocabulary (≤ ~25 distinct values), so the leak is bounded. If you prefer no leak, change `PsrType` to own a `String` and adjust the two call sites — either is acceptable.

- [ ] **Step 4: Run tests to verify pass**

Run: `cargo test -p entsoe request::tests`
Expected: PASS (5 tests).

- [ ] **Step 5: Update the existing prices endpoint call site**

`entsoe/src/endpoints/prices.rs` builds no `RequestParameters` itself (the caller does), so no change there. But any other internal constructor of `RequestParameters` must add the new `None` fields. Run `cargo check -p entsoe` and fix any struct-literal that misses fields.

Run: `cargo check -p entsoe`
Expected: success.

- [ ] **Step 6: Commit (user runs)**

```bash
git add entsoe/src/models/request.rs entsoe/src/endpoints/prices.rs
git commit -m "feat(entsoe): flexible request params, ProcessType/PsrType, EE/RU domains"
```

### Task 5: GL_MarketDocument model (generation + load)

**Files:**
- Create: `entsoe/src/models/generation.rs`
- Modify: `entsoe/src/models/mod.rs`
- Test: inline in `generation.rs`

- [ ] **Step 1: Write failing parse test**

Create `entsoe/src/models/generation.rs`:

```rust
use serde::Deserialize;

/// GL_MarketDocument is used by A75 (actual generation per type) and
/// A65 (load forecast). Quantities are MW. Generation series carry an
/// `MktPSRType.psrType`; load series do not.
#[derive(Debug, Deserialize)]
#[serde(rename = "GL_MarketDocument")]
pub struct GlMarketDocument {
    #[serde(rename = "mRID")]
    pub mrid: String,
    #[serde(rename = "type")]
    pub doc_type: String,
    #[serde(rename = "TimeSeries", default)]
    pub time_series: Vec<GlTimeSeries>,
}

#[derive(Debug, Deserialize)]
pub struct GlTimeSeries {
    #[serde(rename = "mRID")]
    pub mrid: String,
    #[serde(rename = "businessType")]
    pub business_type: String,
    #[serde(rename = "MktPSRType", default)]
    pub mkt_psr_type: Option<MktPsrType>,
    #[serde(rename = "Period")]
    pub period: GlPeriod,
}

#[derive(Debug, Deserialize)]
pub struct MktPsrType {
    #[serde(rename = "psrType")]
    pub psr_type: String,
}

#[derive(Debug, Deserialize)]
pub struct GlPeriod {
    #[serde(rename = "timeInterval")]
    pub time_interval: super::prices::TimeInterval,
    pub resolution: String,
    #[serde(rename = "Point", default)]
    pub points: Vec<GlPoint>,
}

#[derive(Debug, Deserialize)]
pub struct GlPoint {
    pub position: u32,
    pub quantity: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_GEN: &str = r#"
    <GL_MarketDocument>
      <mRID>abc</mRID>
      <type>A75</type>
      <TimeSeries>
        <mRID>1</mRID>
        <businessType>A01</businessType>
        <MktPSRType><psrType>B14</psrType></MktPSRType>
        <Period>
          <timeInterval><start>2024-01-01T00:00Z</start><end>2024-01-01T01:00Z</end></timeInterval>
          <resolution>PT60M</resolution>
          <Point><position>1</position><quantity>2400.0</quantity></Point>
        </Period>
      </TimeSeries>
    </GL_MarketDocument>"#;

    #[test]
    fn parses_generation_doc() {
        let doc: GlMarketDocument = quick_xml::de::from_str(SAMPLE_GEN).unwrap();
        assert_eq!(doc.doc_type, "A75");
        assert_eq!(doc.time_series.len(), 1);
        let ts = &doc.time_series[0];
        assert_eq!(ts.mkt_psr_type.as_ref().unwrap().psr_type, "B14");
        assert_eq!(ts.period.points[0].quantity, 2400.0);
    }
}
```

- [ ] **Step 2: Register module** in `entsoe/src/models/mod.rs`:

```rust
pub mod generation;
pub use generation::{GlMarketDocument, GlTimeSeries, GlPeriod, GlPoint, MktPsrType};
```

- [ ] **Step 3: Run to verify pass**

Run: `cargo test -p entsoe generation::tests`
Expected: PASS. If `TimeInterval` path is wrong, confirm it is `crate::models::prices::TimeInterval` (it is `pub` there).

- [ ] **Step 4: Commit (user runs)**

```bash
git add entsoe/src/models/generation.rs entsoe/src/models/mod.rs
git commit -m "feat(entsoe): GL_MarketDocument model for generation and load"
```

### Task 6: Flow document model (A11)

**Files:**
- Create: `entsoe/src/models/flows.rs`
- Modify: `entsoe/src/models/mod.rs`
- Test: inline

- [ ] **Step 1: Write failing parse test**

Create `entsoe/src/models/flows.rs`:

```rust
use serde::Deserialize;
use super::prices::{TimeInterval, DomainMrid};

/// A11 physical-flow Publication_MarketDocument. Like prices but Points carry
/// `quantity` (MW) instead of `price.amount`.
#[derive(Debug, Deserialize)]
#[serde(rename = "Publication_MarketDocument")]
pub struct FlowMarketDocument {
    #[serde(rename = "mRID")]
    pub mrid: String,
    #[serde(rename = "TimeSeries", default)]
    pub time_series: Vec<FlowTimeSeries>,
}

#[derive(Debug, Deserialize)]
pub struct FlowTimeSeries {
    #[serde(rename = "in_Domain.mRID")]
    pub in_domain_mrid: DomainMrid,
    #[serde(rename = "out_Domain.mRID")]
    pub out_domain_mrid: DomainMrid,
    #[serde(rename = "Period")]
    pub period: FlowPeriod,
}

#[derive(Debug, Deserialize)]
pub struct FlowPeriod {
    #[serde(rename = "timeInterval")]
    pub time_interval: TimeInterval,
    pub resolution: String,
    #[serde(rename = "Point", default)]
    pub points: Vec<FlowPoint>,
}

#[derive(Debug, Deserialize)]
pub struct FlowPoint {
    pub position: u32,
    pub quantity: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
    <Publication_MarketDocument>
      <mRID>x</mRID>
      <TimeSeries>
        <in_Domain.mRID codingScheme="A01">10YFI-1--------U</in_Domain.mRID>
        <out_Domain.mRID codingScheme="A01">10Y1001A1001A46L</out_Domain.mRID>
        <Period>
          <timeInterval><start>2024-01-01T00:00Z</start><end>2024-01-01T01:00Z</end></timeInterval>
          <resolution>PT60M</resolution>
          <Point><position>1</position><quantity>500.0</quantity></Point>
        </Period>
      </TimeSeries>
    </Publication_MarketDocument>"#;

    #[test]
    fn parses_flow_doc() {
        let doc: FlowMarketDocument = quick_xml::de::from_str(SAMPLE).unwrap();
        assert_eq!(doc.time_series.len(), 1);
        assert_eq!(doc.time_series[0].period.points[0].quantity, 500.0);
        assert_eq!(doc.time_series[0].in_domain_mrid.value, "10YFI-1--------U");
    }
}
```

- [ ] **Step 2: Register module** in `entsoe/src/models/mod.rs`:

```rust
pub mod flows;
pub use flows::{FlowMarketDocument, FlowTimeSeries, FlowPeriod, FlowPoint};
```

- [ ] **Step 3: Run to verify pass**

Run: `cargo test -p entsoe flows::tests`
Expected: PASS.

- [ ] **Step 4: Commit (user runs)**

```bash
git add entsoe/src/models/flows.rs entsoe/src/models/mod.rs
git commit -m "feat(entsoe): A11 physical-flow document model"
```

### Task 7: New endpoints — generation, load, flows

**Files:**
- Create: `entsoe/src/endpoints/generation.rs`
- Create: `entsoe/src/endpoints/load.rs`
- Create: `entsoe/src/endpoints/flows.rs`
- Modify: `entsoe/src/endpoints/mod.rs`
- Modify: `entsoe/src/lib.rs`

- [ ] **Step 1: Add a shared request helper to the client**

The existing `get_prices` inlines URL building + acknowledgement handling. To avoid duplication, add a private helper on `Entsoe` in `entsoe/src/endpoints/mod.rs`:

```rust
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
```

Then refactor `entsoe/src/endpoints/prices.rs` `get_prices` to call it:

```rust
use crate::{client::Entsoe, errors::EntsoeError, models::request::RequestParameters, models::prices::PublicationMarketDocument};

impl Entsoe {
    pub async fn get_prices(&self, params: &RequestParameters) -> Result<PublicationMarketDocument, EntsoeError> {
        self.request_doc(params).await
    }
}
```

- [ ] **Step 2: Create `entsoe/src/endpoints/generation.rs`**

```rust
use crate::{client::Entsoe, errors::EntsoeError, models::generation::GlMarketDocument, models::request::RequestParameters};

impl Entsoe {
    /// A75 — actual generation per production type for `params.in_domain`.
    pub async fn get_generation(&self, params: &RequestParameters) -> Result<GlMarketDocument, EntsoeError> {
        self.request_doc(params).await
    }
}
```

- [ ] **Step 3: Create `entsoe/src/endpoints/load.rs`**

```rust
use crate::{client::Entsoe, errors::EntsoeError, models::generation::GlMarketDocument, models::request::RequestParameters};

impl Entsoe {
    /// A65 — total load forecast for `params.out_bidding_zone_domain`.
    pub async fn get_load_forecast(&self, params: &RequestParameters) -> Result<GlMarketDocument, EntsoeError> {
        self.request_doc(params).await
    }
}
```

- [ ] **Step 4: Create `entsoe/src/endpoints/flows.rs`**

```rust
use crate::{client::Entsoe, errors::EntsoeError, models::flows::FlowMarketDocument, models::request::RequestParameters};

impl Entsoe {
    /// A11 — physical flow from `params.out_domain` to `params.in_domain`.
    pub async fn get_flows(&self, params: &RequestParameters) -> Result<FlowMarketDocument, EntsoeError> {
        self.request_doc(params).await
    }
}
```

- [ ] **Step 5: Export the new public types in `entsoe/src/lib.rs`**

Add to the `pub use models::{...}` list (and keep existing exports):

```rust
pub use models::{
    GlMarketDocument, GlTimeSeries, GlPeriod, GlPoint, MktPsrType,
    FlowMarketDocument, FlowTimeSeries, FlowPeriod, FlowPoint,
};
pub use models::request::{ProcessType, PsrType};
```

(Merge with the existing `pub use` block rather than duplicating names.)

- [ ] **Step 6: Verify the crate builds and all tests pass**

Run: `cargo test -p entsoe`
Expected: PASS (request, generation, flows test modules), no warnings about the new endpoints.

- [ ] **Step 7: Commit (user runs)**

```bash
git add entsoe/src/endpoints/ entsoe/src/lib.rs
git commit -m "feat(entsoe): generation/load/flows endpoints via shared request_doc"
```

---

## Phase 3 — App server layer

### Task 8: Shared data types + ServerError

**Files:**
- Create: `src/server/mod.rs`
- Test: inline (serde round-trip)

- [ ] **Step 1: Write `src/server/mod.rs` with types + a failing round-trip test**

```rust
pub mod cache;
pub mod entso;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PricePoint {
    pub timestamp: DateTime<Utc>,
    pub price_eur_mwh: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GenerationSource {
    pub source_type: String,
    pub value_mw: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GenerationMix {
    pub timestamp: DateTime<Utc>,
    pub sources: Vec<GenerationSource>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ForecastPoint {
    pub timestamp: DateTime<Utc>,
    pub value_mw: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FlowPoint {
    pub from_area: String,
    pub to_area: String,
    pub value_mw: f64,
    pub timestamp: DateTime<Utc>,
}

/// Default Finnish bidding zone EIC.
pub const FI_AREA: &str = "10YFI-1--------U";

#[cfg(feature = "server")]
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("ENTSO-E client error: {0}")]
    Entsoe(#[from] entsoe::EntsoeError),
    #[error("unknown area code: {0}")]
    UnknownArea(String),
    #[error("no data returned for {0}")]
    NoData(String),
    #[error("malformed timestamp: {0}")]
    BadTimestamp(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn price_point_round_trips() {
        let p = PricePoint { timestamp: "2024-01-01T00:00:00Z".parse().unwrap(), price_eur_mwh: 42.5 };
        let json = serde_json::to_string(&p).unwrap();
        let back: PricePoint = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }
}
```

- [ ] **Step 2: Run the round-trip test**

Run: `cargo test --features server server::tests`
Expected: FAIL first if `cache`/`entso` modules don't exist yet. To keep this task self-contained, temporarily comment the `pub mod cache;`/`pub mod entso;` lines, run the test (PASS), then re-add them before Task 9. (Tasks 9–10 create those modules.)

- [ ] **Step 3: Commit (user runs)**

```bash
git add src/server/mod.rs
git commit -m "feat(server): shared data types and ServerError"
```

### Task 9: Cache with dynamic price TTL

**Files:**
- Create: `src/server/cache.rs`
- Test: inline

- [ ] **Step 1: Write `src/server/cache.rs` with `next_price_publish` + a failing test**

```rust
use std::sync::Arc;
use std::time::Duration;

use chrono::{TimeZone, Utc};
use chrono_tz::Europe::Helsinki;
use moka::future::Cache;
use moka::Expiry;

use entsoe::{Authorization, Entsoe};

use super::{ForecastPoint, GenerationMix, PricePoint, FlowPoint};

/// Local publish hour/minute for day-ahead prices (Europe/Helsinki).
const PUBLISH_HOUR: u32 = 13;
const PUBLISH_MIN: u32 = 15;

/// Duration from now until the next 13:15 Europe/Helsinki.
pub fn next_price_publish() -> Duration {
    let now = Utc::now().with_timezone(&Helsinki);
    let today_target = Helsinki
        .with_ymd_and_hms(now.date_naive().year(), now.date_naive().month(), now.date_naive().day(), PUBLISH_HOUR, PUBLISH_MIN, 0)
        .single();
    use chrono::Datelike;
    let target = match today_target {
        Some(t) if t > now => t,
        _ => {
            let tomorrow = (now + chrono::Duration::days(1)).date_naive();
            Helsinki
                .with_ymd_and_hms(tomorrow.year(), tomorrow.month(), tomorrow.day(), PUBLISH_HOUR, PUBLISH_MIN, 0)
                .single()
                .expect("valid Helsinki datetime")
        }
    };
    (target.with_timezone(&Utc) - now.with_timezone(&Utc))
        .to_std()
        .unwrap_or(Duration::from_secs(60))
}

/// Per-entry expiry that pins price entries to the next publish time.
struct PriceExpiry;
impl<K, V> Expiry<K, V> for PriceExpiry {
    fn expire_after_create(&self, _key: &K, _value: &V, _now: std::time::Instant) -> Option<Duration> {
        Some(next_price_publish())
    }
}

/// In-memory cache fronting the ENTSO-E client. One typed cache per data type
/// (values differ), keyed by `"{data_type}:{area}"`.
pub struct EntsoeCache {
    pub client: Entsoe,
    pub token: String,
    spot_prices: Cache<String, Vec<PricePoint>>,
    generation: Cache<String, GenerationMix>,
    forecast: Cache<String, Vec<ForecastPoint>>,
    flows: Cache<String, Vec<FlowPoint>>,
}

impl EntsoeCache {
    pub fn new(token: String) -> Self {
        Self {
            client: Entsoe::new(Authorization::new(token.clone())),
            token,
            spot_prices: Cache::builder().expire_after(PriceExpiry).build(),
            generation: Cache::builder().time_to_live(Duration::from_secs(10 * 60)).build(),
            forecast: Cache::builder().time_to_live(Duration::from_secs(30 * 60)).build(),
            flows: Cache::builder().time_to_live(Duration::from_secs(10 * 60)).build(),
        }
    }

    pub async fn get_prices(&self, area: &str) -> Option<Vec<PricePoint>> {
        self.spot_prices.get(&format!("spot_prices:{area}")).await
    }
    pub async fn put_prices(&self, area: &str, v: Vec<PricePoint>) {
        self.spot_prices.insert(format!("spot_prices:{area}"), v).await;
    }
    pub async fn get_generation(&self, area: &str) -> Option<GenerationMix> {
        self.generation.get(&format!("generation:{area}")).await
    }
    pub async fn put_generation(&self, area: &str, v: GenerationMix) {
        self.generation.insert(format!("generation:{area}"), v).await;
    }
    pub async fn get_forecast(&self, area: &str) -> Option<Vec<ForecastPoint>> {
        self.forecast.get(&format!("forecast:{area}")).await
    }
    pub async fn put_forecast(&self, area: &str, v: Vec<ForecastPoint>) {
        self.forecast.insert(format!("forecast:{area}"), v).await;
    }
    pub async fn get_flows(&self, area: &str) -> Option<Vec<FlowPoint>> {
        self.flows.get(&format!("flows:{area}")).await
    }
    pub async fn put_flows(&self, area: &str, v: Vec<FlowPoint>) {
        self.flows.insert(format!("flows:{area}"), v).await;
    }
}

pub type SharedCache = Arc<EntsoeCache>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_publish_is_within_24h_and_positive() {
        let d = next_price_publish();
        assert!(d.as_secs() > 0);
        assert!(d.as_secs() <= 24 * 3600 + 60);
    }

    #[tokio::test]
    async fn cache_round_trips_prices() {
        let cache = EntsoeCache::new("tok".into());
        assert!(cache.get_prices("FI").await.is_none());
        cache.put_prices("FI", vec![PricePoint { timestamp: Utc::now(), price_eur_mwh: 1.0 }]).await;
        assert_eq!(cache.get_prices("FI").await.unwrap().len(), 1);
    }
}
```

Note: add `use chrono::Datelike;` at the top (already referenced) — it is needed for `.year()/.month()/.day()`. Consolidate the two `use chrono::Datelike;` references into the top import block.

- [ ] **Step 2: Run the cache tests**

Run: `cargo test --features server cache::tests`
Expected: PASS (2 tests). If `Expiry` import path differs in moka 0.12, it is `moka::Expiry` (re-exported); adjust if the compiler points elsewhere.

- [ ] **Step 3: Commit (user runs)**

```bash
git add src/server/cache.rs
git commit -m "feat(server): EntsoeCache with dynamic price TTL"
```

### Task 10: Mapping helpers (XML doc → domain types)

**Files:**
- Modify: `src/server/entso.rs` (create with mapping fns + tests first; server fns added in Task 11)
- Test: inline

- [ ] **Step 1: Create `src/server/entso.rs` with pure mapping helpers + failing tests**

```rust
use chrono::{DateTime, Duration, Utc};

use entsoe::{
    FlowMarketDocument, GlMarketDocument, PsrType, PublicationMarketDocument,
};

use super::{ForecastPoint, GenerationMix, GenerationSource, PricePoint, ServerError};

/// Parse an ISO-8601 interval start like `2024-01-01T00:00Z` into UTC.
fn parse_start(s: &str) -> Result<DateTime<Utc>, ServerError> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|_| DateTime::parse_from_str(s, "%Y-%m-%dT%H:%MZ").map(|dt| dt.with_timezone(&Utc)))
        .map_err(|_| ServerError::BadTimestamp(s.to_string()))
}

/// Minutes implied by an ENTSO-E resolution string (`PT60M`, `PT15M`, `PT30M`, `P1D`).
fn resolution_minutes(res: &str) -> i64 {
    match res {
        "PT15M" => 15,
        "PT30M" => 30,
        "PT60M" | "PT1H" => 60,
        "P1D" => 24 * 60,
        _ => 60,
    }
}

fn point_time(start: DateTime<Utc>, position: u32, res_min: i64) -> DateTime<Utc> {
    start + Duration::minutes((position as i64 - 1) * res_min)
}

/// A44 day-ahead prices → flat `PricePoint` series.
pub fn map_prices(doc: &PublicationMarketDocument) -> Result<Vec<PricePoint>, ServerError> {
    let mut out = Vec::new();
    for ts in &doc.time_series {
        let start = parse_start(&ts.period.time_interval.start)?;
        let res = resolution_minutes(&ts.period.resolution);
        for p in &ts.period.points {
            out.push(PricePoint {
                timestamp: point_time(start, p.position, res),
                price_eur_mwh: p.price_amount,
            });
        }
    }
    if out.is_empty() {
        return Err(ServerError::NoData("prices".into()));
    }
    out.sort_by_key(|p| p.timestamp);
    Ok(out)
}

/// A75 generation → latest value per production type.
pub fn map_generation(doc: &GlMarketDocument) -> Result<GenerationMix, ServerError> {
    let mut latest_ts: Option<DateTime<Utc>> = None;
    let mut sources: Vec<GenerationSource> = Vec::new();
    for ts in &doc.time_series {
        let start = parse_start(&ts.period.time_interval.start)?;
        let res = resolution_minutes(&ts.period.resolution);
        let Some(last) = ts.period.points.last() else { continue };
        let t = point_time(start, last.position, res);
        latest_ts = Some(latest_ts.map_or(t, |cur| cur.max(t)));
        let name = ts
            .mkt_psr_type
            .as_ref()
            .map(|m| PsrType::from_code(&m.psr_type).name().to_string())
            .unwrap_or_else(|| "Other".to_string());
        sources.push(GenerationSource { source_type: name, value_mw: last.quantity });
    }
    let timestamp = latest_ts.ok_or_else(|| ServerError::NoData("generation".into()))?;
    sources.sort_by(|a, b| b.value_mw.partial_cmp(&a.value_mw).unwrap_or(std::cmp::Ordering::Equal));
    Ok(GenerationMix { timestamp, sources })
}

/// A65 load forecast → flat `ForecastPoint` series.
pub fn map_forecast(doc: &GlMarketDocument) -> Result<Vec<ForecastPoint>, ServerError> {
    let mut out = Vec::new();
    for ts in &doc.time_series {
        let start = parse_start(&ts.period.time_interval.start)?;
        let res = resolution_minutes(&ts.period.resolution);
        for p in &ts.period.points {
            out.push(ForecastPoint { timestamp: point_time(start, p.position, res), value_mw: p.quantity });
        }
    }
    if out.is_empty() {
        return Err(ServerError::NoData("forecast".into()));
    }
    out.sort_by_key(|p| p.timestamp);
    Ok(out)
}

/// Sum all points in an A11 flow document to a single directional total (MW·h proxy).
/// Returns 0.0 when the document is empty (e.g. FI–RU "no data").
pub fn sum_flow(doc: &FlowMarketDocument) -> f64 {
    doc.time_series
        .iter()
        .flat_map(|ts| ts.period.points.iter())
        .map(|p| p.quantity)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolution_minutes_known_values() {
        assert_eq!(resolution_minutes("PT60M"), 60);
        assert_eq!(resolution_minutes("PT15M"), 15);
        assert_eq!(resolution_minutes("weird"), 60);
    }

    #[test]
    fn point_time_advances_by_position() {
        let start = parse_start("2024-01-01T00:00Z").unwrap();
        assert_eq!(point_time(start, 1, 60), start);
        assert_eq!(point_time(start, 3, 60), start + Duration::hours(2));
    }

    #[test]
    fn map_prices_sample() {
        let xml = r#"<Publication_MarketDocument><mRID>a</mRID><revisionNumber>1</revisionNumber><type>A44</type><createdDateTime>x</createdDateTime><period.timeInterval><start>2024-01-01T00:00Z</start><end>2024-01-02T00:00Z</end></period.timeInterval><TimeSeries><mRID>1</mRID><auction.type>A01</auction.type><businessType>A62</businessType><in_Domain.mRID codingScheme="A01">10YFI-1--------U</in_Domain.mRID><out_Domain.mRID codingScheme="A01">10YFI-1--------U</out_Domain.mRID><currency_Unit.name>EUR</currency_Unit.name><price_Measure_Unit.name>MWH</price_Measure_Unit.name><curveType>A01</curveType><Period><timeInterval><start>2024-01-01T00:00Z</start><end>2024-01-01T02:00Z</end></timeInterval><resolution>PT60M</resolution><Point><position>1</position><price.amount>10.0</price.amount></Point><Point><position>2</position><price.amount>20.0</price.amount></Point></Period></TimeSeries></Publication_MarketDocument>"#;
        let doc: PublicationMarketDocument = quick_xml::de::from_str(xml).unwrap();
        let pts = map_prices(&doc).unwrap();
        assert_eq!(pts.len(), 2);
        assert_eq!(pts[0].price_eur_mwh, 10.0);
        assert_eq!(pts[1].timestamp, pts[0].timestamp + Duration::hours(1));
    }
}
```

> Note: `quick_xml` is a dependency of `entsoe`, not the app. For these app-side tests add `quick-xml = "0.40"` to the app's `[dev-dependencies]` in `Cargo.toml`:
> ```toml
> [dev-dependencies]
> quick-xml = { version = "0.40", features = ["serialize"] }
> ```

- [ ] **Step 2: Run the mapping tests**

Run: `cargo test --features server entso::tests`
Expected: PASS (3 tests).

- [ ] **Step 3: Commit (user runs)**

```bash
git add src/server/entso.rs Cargo.toml
git commit -m "feat(server): XML document to domain-type mapping helpers"
```

### Task 11: Server functions

**Files:**
- Modify: `src/server/entso.rs` (append the four `#[server]` functions)

- [ ] **Step 1: Append server functions to `src/server/entso.rs`**

```rust
use dioxus::prelude::*;
use std::sync::Arc;

use super::FI_AREA;

#[cfg(feature = "server")]
use entsoe::{Domain, DocumentType, PeriodTimestamp, ProcessType, RequestParameters};
#[cfg(feature = "server")]
use std::str::FromStr;
#[cfg(feature = "server")]
use super::cache::EntsoeCache;

#[cfg(feature = "server")]
fn window() -> (PeriodTimestamp, PeriodTimestamp) {
    // 24h back to 36h forward covers "today + day-ahead".
    let now = chrono::Utc::now();
    (
        PeriodTimestamp(now - chrono::Duration::hours(24)),
        PeriodTimestamp(now + chrono::Duration::hours(36)),
    )
}

#[cfg(feature = "server")]
async fn cache_ctx() -> Result<Arc<EntsoeCache>, ServerFnError> {
    use axum::Extension;
    let Extension(cache): Extension<Arc<EntsoeCache>> = extract().await?;
    Ok(cache)
}

#[cfg(feature = "server")]
fn domain(area: &str) -> Result<Domain, ServerFnError> {
    Domain::from_str(area).map_err(|_| ServerFnError::new(format!("unknown area {area}")))
}

#[server]
pub async fn get_spot_prices(area: String) -> Result<Vec<super::PricePoint>, ServerFnError> {
    let cache = cache_ctx().await?;
    if let Some(hit) = cache.get_prices(&area).await {
        return Ok(hit);
    }
    let d = domain(&area)?;
    let (start, end) = window();
    let params = RequestParameters {
        document_type: DocumentType::A44,
        out_domain: Some(d_clone(&area)?),
        in_domain: Some(domain(&area)?),
        out_bidding_zone_domain: None,
        period_start: start,
        period_end: end,
        process_type: None,
        psr_type: None,
        authorization: entsoe::Authorization::new(cache.token.clone()),
    };
    let doc = cache.client.get_prices(&params).await.map_err(to_sfe)?;
    let mapped = map_prices(&doc).map_err(to_sfe)?;
    cache.put_prices(&area, mapped.clone()).await;
    let _ = d;
    Ok(mapped)
}

#[server]
pub async fn get_generation_mix(area: String) -> Result<super::GenerationMix, ServerFnError> {
    let cache = cache_ctx().await?;
    if let Some(hit) = cache.get_generation(&area).await {
        return Ok(hit);
    }
    let (start, end) = window();
    let params = RequestParameters {
        document_type: DocumentType::A75,
        out_domain: None,
        in_domain: Some(domain(&area)?),
        out_bidding_zone_domain: None,
        period_start: start,
        period_end: end,
        process_type: Some(ProcessType::Realised),
        psr_type: None,
        authorization: entsoe::Authorization::new(cache.token.clone()),
    };
    let doc = cache.client.get_generation(&params).await.map_err(to_sfe)?;
    let mapped = map_generation(&doc).map_err(to_sfe)?;
    cache.put_generation(&area, mapped.clone()).await;
    Ok(mapped)
}

#[server]
pub async fn get_consumption_forecast(area: String) -> Result<Vec<super::ForecastPoint>, ServerFnError> {
    let cache = cache_ctx().await?;
    if let Some(hit) = cache.get_forecast(&area).await {
        return Ok(hit);
    }
    let (start, end) = window();
    let params = RequestParameters {
        document_type: DocumentType::A65,
        out_domain: None,
        in_domain: None,
        out_bidding_zone_domain: Some(domain(&area)?),
        period_start: start,
        period_end: end,
        process_type: Some(ProcessType::DayAhead),
        psr_type: None,
        authorization: entsoe::Authorization::new(cache.token.clone()),
    };
    let doc = cache.client.get_load_forecast(&params).await.map_err(to_sfe)?;
    let mapped = map_forecast(&doc).map_err(to_sfe)?;
    cache.put_forecast(&area, mapped.clone()).await;
    Ok(mapped)
}

#[server]
pub async fn get_cross_border_flows(area: String) -> Result<Vec<super::FlowPoint>, ServerFnError> {
    let cache = cache_ctx().await?;
    if let Some(hit) = cache.get_flows(&area).await {
        return Ok(hit);
    }
    // FI borders: SE3, EE, NO4, RU. Net = import(neighbour→FI) − export(FI→neighbour).
    let borders = [("SE3", Domain::SE3), ("EE", Domain::EE), ("NO4", Domain::NO4), ("RU", Domain::RU)];
    let fi = domain(&area)?;
    let (start, end) = window();
    let now = chrono::Utc::now();
    let mut out = Vec::new();
    for (label, nb) in borders {
        let import = sum_dir(&cache, &nb, &fi, &start, &end).await;
        let export = sum_dir(&cache, &fi, &nb, &start, &end).await;
        let net = import - export; // >0 import into FI
        out.push(super::FlowPoint {
            from_area: if net >= 0.0 { label.to_string() } else { "FI".to_string() },
            to_area: if net >= 0.0 { "FI".to_string() } else { label.to_string() },
            value_mw: net.abs(),
            timestamp: now,
        });
    }
    let _ = fi;
    cache.put_flows(&area, out.clone()).await;
    Ok(out)
}

#[cfg(feature = "server")]
async fn sum_dir(
    cache: &EntsoeCache,
    out_d: &Domain,
    in_d: &Domain,
    start: &PeriodTimestamp,
    end: &PeriodTimestamp,
) -> f64 {
    let params = RequestParameters {
        document_type: DocumentType::A11,
        out_domain: Some(clone_domain(out_d)),
        in_domain: Some(clone_domain(in_d)),
        out_bidding_zone_domain: None,
        period_start: PeriodTimestamp(start.0),
        period_end: PeriodTimestamp(end.0),
        process_type: None,
        psr_type: None,
        authorization: entsoe::Authorization::new(cache.token.clone()),
    };
    match cache.client.get_flows(&params).await {
        Ok(doc) => sum_flow(&doc),
        Err(_) => 0.0, // e.g. FI–RU "no matching data"
    }
}

#[cfg(feature = "server")]
fn clone_domain(d: &Domain) -> Domain {
    Domain::from_str(&d.to_string()).expect("round-trips")
}

#[cfg(feature = "server")]
fn d_clone(area: &str) -> Result<Domain, ServerFnError> { domain(area) }

#[cfg(feature = "server")]
fn to_sfe(e: impl std::fmt::Display) -> ServerFnError {
    ServerFnError::new(e.to_string())
}
```

> Implementation notes for the executor:
> - `Domain` is not `Clone` in the crate today. The plan above sidesteps that with `clone_domain`/`d_clone` (round-trip through `from_str`). Cleaner: add `#[derive(Clone, Copy)]` to `Domain` in `entsoe/src/models/request.rs` and delete `clone_domain`/`d_clone`, calling `domain(&area)?` directly. **Prefer adding `Clone, Copy` to `Domain`** and simplifying — it removes the helpers. If you do, also drop the `let _ = d;`/`let _ = fi;` lines.
> - `extract()` / `ServerFnError` come from `dioxus::prelude`. If `extract` is not in scope, import `use dioxus::fullstack::prelude::extract;` or `use server_fn::axum::extract;` per the compiler.
> - The default-area helper for pages is the constant `FI_AREA`.

- [ ] **Step 2: (Recommended) Add `Clone, Copy` to `Domain`** in `entsoe/src/models/request.rs`:

```rust
#[derive(Debug, Clone, Copy)]
pub enum Domain { /* ...unchanged variants... */ }
```
Then simplify the server fns: replace `Some(d_clone(&area)?)`/`Some(domain(&area)?)` pairs with a single `let d = domain(&area)?;` reused as `Some(d)`, replace `clone_domain(out_d)` with `*out_d`, and delete `clone_domain`, `d_clone`, and the `let _ = d;`/`let _ = fi;` lines.

- [ ] **Step 3: Type-check server build**

Run: `cargo check --features server`
Expected: success. Resolve any `extract`/`ServerFnError` import path against the compiler (see note).

- [ ] **Step 4: Type-check web build (server fns compile to client stubs)**

Run: `cargo check --features web`
Expected: success — `#[server]` generates a client-side caller; the `#[cfg(feature="server")]` bodies/helpers are excluded on web.

- [ ] **Step 5: Commit (user runs)**

```bash
git add src/server/entso.rs entsoe/src/models/request.rs
git commit -m "feat(server): four cached ENTSO-E server functions"
```

### Task 12: Wire server bootstrap (Axum + Extension) in main.rs

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Replace `server_main` with the real Axum bootstrap**

```rust
#[cfg(feature = "server")]
fn server_main() {
    use std::sync::Arc;
    use axum::Extension;
    use dioxus::prelude::*;
    use server::cache::EntsoeCache;

    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let token = std::env::var("ENTSO_E_TOKEN")
        .expect("ENTSO_E_TOKEN must be set (request one at https://transparency.entsoe.eu/)");
    let bind = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let cache = Arc::new(EntsoeCache::new(token));

    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(async move {
        let addr: std::net::SocketAddr = bind.parse().expect("valid BIND_ADDR");
        let router = axum::Router::new()
            .serve_dioxus_application(ServeConfig::new().unwrap(), App)
            .layer(Extension(cache));
        let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
        tracing::info!("listening on http://{addr}");
        axum::serve(listener, router.into_make_service()).await.expect("serve");
    });
}
```

> Note: `serve_dioxus_application` and `ServeConfig` come from Dioxus 0.6's fullstack axum integration (`use dioxus::prelude::*;` should bring them in; if not, `use dioxus_fullstack::prelude::*;`). The exact builder signature (`ServeConfig::new()` vs `ServeConfigBuilder`) may differ slightly by patch version — reconcile against `cargo check --features server`. The load-bearing requirement: the Dioxus app is served and `Extension(cache)` is layered so `extract::<Extension<Arc<EntsoeCache>>>()` works in server fns.

- [ ] **Step 2: Type-check**

Run: `cargo check --features server`
Expected: success.

- [ ] **Step 3: Commit (user runs)**

```bash
git add src/main.rs
git commit -m "feat(server): Axum bootstrap with EntsoeCache extension"
```

---

## Phase 4 — Router, nav, common components

### Task 13: Common components (Skeleton, ErrorBanner)

**Files:**
- Create: `src/components/common.rs`
- Modify: `src/components/mod.rs`

- [ ] **Step 1: Create `src/components/common.rs`**

```rust
use dioxus::prelude::*;

#[component]
pub fn Skeleton() -> Element {
    rsx! {
        div { class: "animate-pulse space-y-4",
            div { class: "h-6 w-1/3 rounded bg-gray-700" }
            div { class: "h-64 w-full rounded bg-gray-800" }
        }
    }
}

#[component]
pub fn ErrorBanner(msg: String) -> Element {
    rsx! {
        div { class: "rounded border border-red-700 bg-red-900/40 p-4 text-red-200",
            span { class: "font-semibold", "Error: " }
            "{msg}"
        }
    }
}

#[component]
pub fn Card(title: String, children: Element) -> Element {
    rsx! {
        div { class: "rounded-lg border border-gray-700 bg-gray-800 p-5",
            h2 { class: "mb-3 text-sm font-medium uppercase tracking-wide text-gray-400", "{title}" }
            {children}
        }
    }
}
```

- [ ] **Step 2: Register in `src/components/mod.rs`**

```rust
pub mod app;
pub mod common;
pub mod nav;
pub mod pages;
pub mod charts;
```

- [ ] **Step 3: Type-check web build** (other modules created in following tasks; comment unregistered modules out, or do this step after Tasks 14–20). For now verify just this file compiles by temporarily limiting `mod.rs` to `pub mod app; pub mod common;`.

Run: `cargo check --features web`
Expected: success.

- [ ] **Step 4: Commit (user runs)**

```bash
git add src/components/common.rs src/components/mod.rs
git commit -m "feat(ui): Skeleton, ErrorBanner, Card components"
```

### Task 14: Nav layout

**Files:**
- Create: `src/components/nav.rs`

- [ ] **Step 1: Create `src/components/nav.rs`**

```rust
use dioxus::prelude::*;
use super::app::Route;

#[component]
pub fn Nav() -> Element {
    let links = [
        (Route::Overview {}, "Overview"),
        (Route::Prices {}, "Prices"),
        (Route::Generation {}, "Generation"),
        (Route::Forecast {}, "Forecast"),
        (Route::Flows {}, "Flows"),
    ];
    let current: Route = use_route();
    rsx! {
        nav { class: "flex items-center gap-6 bg-gray-900 px-6 py-4 text-gray-100 border-b border-gray-800",
            span { class: "text-lg font-bold text-emerald-400", "⚡ FI Energy Dashboard" }
            div { class: "flex gap-2",
                for (route, label) in links {
                    Link {
                        to: route.clone(),
                        class: if current == route { "px-3 py-1 rounded bg-gray-700 text-white" } else { "px-3 py-1 rounded text-gray-300 hover:bg-gray-800" },
                        "{label}"
                    }
                }
            }
        }
        main { class: "min-h-screen bg-gray-900 p-6 text-gray-100",
            Outlet::<Route> {}
        }
    }
}
```

- [ ] **Step 2: (verified together with Task 15 since it references `Route`).** Defer compile to Task 15.

- [ ] **Step 3: Commit (user runs)**

```bash
git add src/components/nav.rs
git commit -m "feat(ui): top nav layout with active-route highlight"
```

### Task 15: App + Router + document head

**Files:**
- Modify: `src/components/app.rs`

- [ ] **Step 1: Replace `src/components/app.rs`**

```rust
use dioxus::prelude::*;

use super::nav::Nav;
use super::pages::{
    overview::Overview, prices::Prices, generation::Generation,
    forecast::Forecast, flows::Flows,
};

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[layout(Nav)]
    #[route("/")]
    Overview,
    #[route("/prices")]
    Prices,
    #[route("/generation")]
    Generation,
    #[route("/forecast")]
    Forecast,
    #[route("/flows")]
    Flows,
}

const TAILWIND: Asset = asset!("/assets/tailwind.css");
const ECHARTS: Asset = asset!("/assets/echarts.min.js");

#[component]
pub fn App() -> Element {
    rsx! {
        document::Stylesheet { href: TAILWIND }
        document::Script { src: ECHARTS }
        Router::<Route> {}
    }
}
```

> Note: page component names must match the `Route` variants (`Overview`, `Prices`, `Generation`, `Forecast`, `Flows`). Tasks 16–20 define them with exactly these names.

- [ ] **Step 2: Defer compile until pages exist (Tasks 16–20).** After Task 20, run `cargo check --features web`.

- [ ] **Step 3: Commit (user runs)**

```bash
git add src/components/app.rs
git commit -m "feat(ui): Router, Route enum, vendored asset head"
```

---

## Phase 5 — Charts + vendored ECharts

### Task 16: Vendor echarts.min.js + chart eval helper

**Files:**
- Create: `assets/echarts.min.js` (download)
- Create: `src/components/charts/mod.rs`

- [ ] **Step 1: Download ECharts into assets**

Run:
```bash
curl -fsSL https://cdn.jsdelivr.net/npm/echarts@5/dist/echarts.min.js -o assets/echarts.min.js && ls -l assets/echarts.min.js
```
Expected: a ~1MB file. **If network is blocked, STOP and tell the user** — vendoring was their explicit choice and the charts cannot render without it.

- [ ] **Step 2: Create `src/components/charts/mod.rs` with the eval helper**

```rust
pub mod price_chart;
pub mod generation_pie;
pub mod flow_map;

use dioxus::prelude::*;

/// Render a charming chart (already serialized to ECharts-option JSON) into a
/// 100%-width div identified by `id`, using the vendored global `echarts`.
pub fn render_echarts(id: &str, option_json: &str) {
    let script = format!(
        r#"
        (function() {{
          if (typeof echarts === 'undefined') return;
          var el = document.getElementById('{id}');
          if (!el) return;
          var chart = echarts.getInstanceByDom(el) || echarts.init(el);
          chart.setOption({option_json});
          window.addEventListener('resize', function() {{ chart.resize(); }});
        }})();
        "#
    );
    let _ = document::eval(&script);
}
```

> Note: `charming::Chart` implements `Serialize`, so chart components produce `option_json` via `serde_json::to_string(&chart)`. `document::eval` is the Dioxus 0.6 eval API; if the import differs, use `dioxus::prelude::eval`.

- [ ] **Step 3: Commit (user runs)**

```bash
git add assets/echarts.min.js src/components/charts/mod.rs
git commit -m "feat(charts): vendor echarts.min.js and add eval render helper"
```

### Task 17: Price bar chart

**Files:**
- Create: `src/components/charts/price_chart.rs`

- [ ] **Step 1: Create `src/components/charts/price_chart.rs`**

```rust
use dioxus::prelude::*;
use charming::{Chart, component::Axis, element::AxisType, series::Bar};

use crate::server::PricePoint;
use super::render_echarts;

/// Convert €/MWh to c/kWh for color thresholds (1 €/MWh = 0.1 c/kWh).
fn eur_mwh_to_c_kwh(p: f64) -> f64 { p / 10.0 }

fn bar_color(c_kwh: f64) -> &'static str {
    if c_kwh < 5.0 { "#22c55e" } else if c_kwh <= 15.0 { "#eab308" } else { "#ef4444" }
}

#[component]
pub fn PriceChart(data: Vec<PricePoint>) -> Element {
    let id = use_memo(|| format!("price-chart-{}", generation_id()));

    use_effect(move || {
        let labels: Vec<String> = data.iter()
            .map(|p| p.timestamp.format("%H:%M").to_string())
            .collect();
        let values: Vec<f64> = data.iter().map(|p| p.price_eur_mwh).collect();
        // Per-bar color via itemStyle is verbose in charming 0.4; for color we
        // post-process the JSON: build a base chart, then inject colored data.
        let chart = Chart::new()
            .x_axis(Axis::new().type_(AxisType::Category).data(labels.clone()))
            .y_axis(Axis::new().type_(AxisType::Value))
            .series(Bar::new().data(values.clone()));
        let mut json = serde_json::to_string(&chart).unwrap_or_else(|_| "{}".into());
        // Replace the plain series data with {value,itemStyle:{color}} objects.
        let colored: Vec<String> = values.iter().map(|v| {
            format!("{{\"value\":{},\"itemStyle\":{{\"color\":\"{}\"}}}}", v, bar_color(eur_mwh_to_c_kwh(*v)))
        }).collect();
        let colored_arr = format!("[{}]", colored.join(","));
        // naive replace of the first data array in the series
        if let Some(pos) = json.find("\"data\":[") {
            // find matching end bracket of that array
            let start = pos + "\"data\":[".len() - 1;
            if let Some(end_rel) = json[start..].find(']') {
                let end = start + end_rel + 1;
                // Only replace the *series* data (last data array). Use rfind instead.
                let _ = (start, end);
            }
        }
        // Simpler + robust: replace the last "data":[...] occurrence (the series).
        if let Some(pos) = json.rfind("\"data\":[") {
            let start = pos + "\"data\":".len();
            if let Some(end_rel) = json[start..].find(']') {
                let end = start + end_rel + 1;
                json.replace_range(start..end, &colored_arr);
            }
        }
        render_echarts(&id(), &json);
    });

    rsx! { div { id: "{id}", class: "h-80 w-full" } }
}

fn generation_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(0);
    N.fetch_add(1, Ordering::Relaxed)
}
```

> Note: per-bar colors in charming 0.4 are awkward to express through the typed API, so the chart's series `data` array is swapped for `{value,itemStyle:{color}}` objects in the serialized JSON. If a future charming version exposes `Bar::data` with `DataPoint`/`itemStyle`, prefer the typed path. The JSON-splice is contained to this component.

- [ ] **Step 2: Defer compile until pages exist; verified in Task 20.**

- [ ] **Step 3: Commit (user runs)**

```bash
git add src/components/charts/price_chart.rs
git commit -m "feat(charts): color-coded day-ahead price bar chart"
```

### Task 18: Generation donut chart

**Files:**
- Create: `src/components/charts/generation_pie.rs`

- [ ] **Step 1: Create `src/components/charts/generation_pie.rs`**

```rust
use dioxus::prelude::*;
use charming::{Chart, series::Pie};

use crate::server::GenerationMix;
use super::render_echarts;

#[component]
pub fn GenerationPie(data: GenerationMix) -> Element {
    let id = use_memo(|| format!("gen-pie-{}", super::price_chart::__noop_id()));

    use_effect(move || {
        // charming Pie takes (name, value) pairs.
        let pairs: Vec<(String, f64)> = data.sources.iter()
            .map(|s| (s.source_type.clone(), s.value_mw))
            .collect();
        let chart = Chart::new()
            .series(Pie::new().radius(vec!["40%", "70%"]).data(pairs));
        let json = serde_json::to_string(&chart).unwrap_or_else(|_| "{}".into());
        render_echarts(&id(), &json);
    });

    rsx! { div { id: "{id}", class: "h-80 w-full" } }
}
```

> Note: `super::price_chart::__noop_id()` is a stand-in — instead expose the `generation_id()` counter from `charts/mod.rs` as `pub fn next_id() -> u64` and call `super::next_id()` here and in `price_chart.rs`. Do that refactor: move `generation_id` into `mod.rs` as `pub fn next_id`, update both call sites. (This removes the awkward cross-module reference.)

- [ ] **Step 2: Apply the `next_id` refactor** in `charts/mod.rs`:

```rust
pub fn next_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(0);
    N.fetch_add(1, Ordering::Relaxed)
}
```
Then `price_chart.rs` uses `super::next_id()` and `generation_pie.rs` uses `super::next_id()`; delete the local `generation_id`/`__noop_id`.

- [ ] **Step 3: Commit (user runs)**

```bash
git add src/components/charts/generation_pie.rs src/components/charts/mod.rs src/components/charts/price_chart.rs
git commit -m "feat(charts): generation donut chart + shared id helper"
```

### Task 19: Forecast line chart + flow chart

**Files:**
- Create: `src/components/charts/flow_map.rs`
- Modify: `src/components/charts/mod.rs` (add `pub mod` for a forecast chart, or reuse)

- [ ] **Step 1: Add a forecast line chart.** Append to `charts/mod.rs`:

```rust
pub mod forecast_line;
```

Create `src/components/charts/forecast_line.rs`:

```rust
use dioxus::prelude::*;
use charming::{Chart, component::Axis, element::AxisType, series::Line};

use crate::server::ForecastPoint;
use super::render_echarts;

#[component]
pub fn ForecastLine(data: Vec<ForecastPoint>) -> Element {
    let id = use_memo(|| format!("forecast-{}", super::next_id()));
    use_effect(move || {
        let labels: Vec<String> = data.iter().map(|p| p.timestamp.format("%H:%M").to_string()).collect();
        let values: Vec<f64> = data.iter().map(|p| p.value_mw).collect();
        let chart = Chart::new()
            .x_axis(Axis::new().type_(AxisType::Category).data(labels))
            .y_axis(Axis::new().type_(AxisType::Value))
            .series(Line::new().data(values).smooth(true));
        let json = serde_json::to_string(&chart).unwrap_or_else(|_| "{}".into());
        render_echarts(&id(), &json);
    });
    rsx! { div { id: "{id}", class: "h-80 w-full" } }
}
```

- [ ] **Step 2: Create `src/components/charts/flow_map.rs`** (horizontal bar of net flows)

```rust
use dioxus::prelude::*;
use charming::{Chart, component::Axis, element::AxisType, series::Bar};

use crate::server::FlowPoint;
use super::render_echarts;

#[component]
pub fn FlowChart(data: Vec<FlowPoint>) -> Element {
    let id = use_memo(|| format!("flows-{}", super::next_id()));
    let rows = data.clone();
    use_effect(move || {
        let labels: Vec<String> = rows.iter()
            .map(|f| format!("{}→{}", f.from_area, f.to_area))
            .collect();
        let values: Vec<f64> = rows.iter().map(|f| f.value_mw).collect();
        let chart = Chart::new()
            .x_axis(Axis::new().type_(AxisType::Value))
            .y_axis(Axis::new().type_(AxisType::Category).data(labels))
            .series(Bar::new().data(values));
        let json = serde_json::to_string(&chart).unwrap_or_else(|_| "{}".into());
        render_echarts(&id(), &json);
    });
    rsx! { div { id: "{id}", class: "h-80 w-full" } }
}
```

- [ ] **Step 3: Commit (user runs)**

```bash
git add src/components/charts/flow_map.rs src/components/charts/forecast_line.rs src/components/charts/mod.rs
git commit -m "feat(charts): forecast line + cross-border flow charts"
```

---

## Phase 6 — Pages

### Task 20: Pages module + all five pages

**Files:**
- Create: `src/components/pages/mod.rs`
- Create: `src/components/pages/{overview,prices,generation,forecast,flows}.rs`

- [ ] **Step 1: Create `src/components/pages/mod.rs`**

```rust
pub mod overview;
pub mod prices;
pub mod generation;
pub mod forecast;
pub mod flows;
```

- [ ] **Step 2: Create `src/components/pages/prices.rs`**

```rust
use dioxus::prelude::*;

use crate::server::{entso::get_spot_prices, FI_AREA, PricePoint};
use crate::components::common::{Skeleton, ErrorBanner, Card};
use crate::components::charts::price_chart::PriceChart;

#[component]
pub fn Prices() -> Element {
    let data = use_server_future(|| get_spot_prices(FI_AREA.to_string()))?;
    rsx! {
        h1 { class: "mb-4 text-2xl font-bold", "Day-ahead prices" }
        Suspense { fallback: rsx! { Skeleton {} },
            {match data() {
                Some(Ok(d)) => rsx! {
                    PriceStats { data: d.clone() }
                    Card { title: "Next 24h (€/MWh)".to_string(), PriceChart { data: d } }
                },
                Some(Err(e)) => rsx! { ErrorBanner { msg: e.to_string() } },
                None => rsx! { Skeleton {} },
            }}
        }
    }
}

#[component]
fn PriceStats(data: Vec<PricePoint>) -> Element {
    let (min, max, avg) = stats(&data);
    rsx! {
        div { class: "mb-4 grid grid-cols-3 gap-4",
            StatCard { label: "Min".to_string(), value: format!("{min:.1} €/MWh") }
            StatCard { label: "Avg".to_string(), value: format!("{avg:.1} €/MWh") }
            StatCard { label: "Max".to_string(), value: format!("{max:.1} €/MWh") }
        }
    }
}

#[component]
fn StatCard(label: String, value: String) -> Element {
    rsx! {
        div { class: "rounded-lg border border-gray-700 bg-gray-800 p-4",
            div { class: "text-xs uppercase text-gray-400", "{label}" }
            div { class: "text-xl font-semibold", "{value}" }
        }
    }
}

fn stats(data: &[PricePoint]) -> (f64, f64, f64) {
    if data.is_empty() { return (0.0, 0.0, 0.0); }
    let mut min = f64::MAX; let mut max = f64::MIN; let mut sum = 0.0;
    for p in data { min = min.min(p.price_eur_mwh); max = max.max(p.price_eur_mwh); sum += p.price_eur_mwh; }
    (min, max, sum / data.len() as f64)
}
```

- [ ] **Step 3: Create `src/components/pages/generation.rs`**

```rust
use dioxus::prelude::*;

use crate::server::{entso::get_generation_mix, FI_AREA};
use crate::components::common::{Skeleton, ErrorBanner, Card};
use crate::components::charts::generation_pie::GenerationPie;

#[component]
pub fn Generation() -> Element {
    let data = use_server_future(|| get_generation_mix(FI_AREA.to_string()))?;
    rsx! {
        h1 { class: "mb-4 text-2xl font-bold", "Generation mix" }
        Suspense { fallback: rsx! { Skeleton {} },
            {match data() {
                Some(Ok(d)) => rsx! { Card { title: "Current generation by source (MW)".to_string(), GenerationPie { data: d } } },
                Some(Err(e)) => rsx! { ErrorBanner { msg: e.to_string() } },
                None => rsx! { Skeleton {} },
            }}
        }
    }
}
```

- [ ] **Step 4: Create `src/components/pages/forecast.rs`**

```rust
use dioxus::prelude::*;

use crate::server::{entso::get_consumption_forecast, FI_AREA};
use crate::components::common::{Skeleton, ErrorBanner, Card};
use crate::components::charts::forecast_line::ForecastLine;

#[component]
pub fn Forecast() -> Element {
    let data = use_server_future(|| get_consumption_forecast(FI_AREA.to_string()))?;
    rsx! {
        h1 { class: "mb-4 text-2xl font-bold", "Consumption forecast" }
        Suspense { fallback: rsx! { Skeleton {} },
            {match data() {
                Some(Ok(d)) => rsx! { Card { title: "Next 24h load forecast (MW)".to_string(), ForecastLine { data: d } } },
                Some(Err(e)) => rsx! { ErrorBanner { msg: e.to_string() } },
                None => rsx! { Skeleton {} },
            }}
        }
    }
}
```

- [ ] **Step 5: Create `src/components/pages/flows.rs`**

```rust
use dioxus::prelude::*;

use crate::server::{entso::get_cross_border_flows, FI_AREA, FlowPoint};
use crate::components::common::{Skeleton, ErrorBanner, Card};
use crate::components::charts::flow_map::FlowChart;

#[component]
pub fn Flows() -> Element {
    let data = use_server_future(|| get_cross_border_flows(FI_AREA.to_string()))?;
    rsx! {
        h1 { class: "mb-4 text-2xl font-bold", "Cross-border flows" }
        Suspense { fallback: rsx! { Skeleton {} },
            {match data() {
                Some(Ok(d)) => rsx! {
                    Card { title: "Net physical flows (MW)".to_string(), FlowChart { data: d.clone() } }
                    FlowTable { data: d }
                },
                Some(Err(e)) => rsx! { ErrorBanner { msg: e.to_string() } },
                None => rsx! { Skeleton {} },
            }}
        }
    }
}

#[component]
fn FlowTable(data: Vec<FlowPoint>) -> Element {
    rsx! {
        table { class: "mt-4 w-full text-left text-sm",
            thead { tr { class: "text-gray-400",
                th { class: "py-2", "Direction" }
                th { class: "py-2", "Flow" }
                th { class: "py-2", "MW" }
            } }
            tbody {
                for f in data {
                    tr { class: "border-t border-gray-800",
                        td { class: "py-2", "{f.from_area} → {f.to_area}" }
                        td { class: "py-2",
                            if f.to_area == "FI" {
                                span { class: "text-emerald-400", "Import" }
                            } else {
                                span { class: "text-amber-400", "Export" }
                            }
                        }
                        td { class: "py-2", "{f.value_mw:.0}" }
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 6: Create `src/components/pages/overview.rs`**

```rust
use dioxus::prelude::*;

use crate::server::{entso::{get_spot_prices, get_generation_mix, get_consumption_forecast}, FI_AREA, PricePoint};
use crate::components::common::{Skeleton, ErrorBanner, Card};
use crate::components::charts::price_chart::PriceChart;

#[component]
pub fn Overview() -> Element {
    let prices = use_server_future(|| get_spot_prices(FI_AREA.to_string()))?;
    let gen = use_server_future(|| get_generation_mix(FI_AREA.to_string()))?;
    let fc = use_server_future(|| get_consumption_forecast(FI_AREA.to_string()))?;

    rsx! {
        h1 { class: "mb-4 text-2xl font-bold", "Overview" }
        Suspense { fallback: rsx! { Skeleton {} },
            div { class: "grid grid-cols-1 gap-4 md:grid-cols-2",
                Card { title: "Current spot price".to_string(),
                    {match prices() {
                        Some(Ok(d)) => rsx! { CurrentPrice { data: d } },
                        Some(Err(e)) => rsx! { ErrorBanner { msg: e.to_string() } },
                        None => rsx! { Skeleton {} },
                    }}
                }
                Card { title: "Generation total".to_string(),
                    {match gen() {
                        Some(Ok(d)) => rsx! {
                            div { class: "text-3xl font-bold",
                                {format!("{:.0} MW", d.sources.iter().map(|s| s.value_mw).sum::<f64>())}
                            }
                        },
                        Some(Err(e)) => rsx! { ErrorBanner { msg: e.to_string() } },
                        None => rsx! { Skeleton {} },
                    }}
                }
                Card { title: "Peak forecast (next 24h)".to_string(),
                    {match fc() {
                        Some(Ok(d)) => rsx! {
                            div { class: "text-3xl font-bold",
                                {format!("{:.0} MW", d.iter().map(|p| p.value_mw).fold(0.0_f64, f64::max))}
                            }
                        },
                        Some(Err(e)) => rsx! { ErrorBanner { msg: e.to_string() } },
                        None => rsx! { Skeleton {} },
                    }}
                }
                Card { title: "Today's prices".to_string(),
                    {match prices() {
                        Some(Ok(d)) => rsx! { PriceChart { data: d } },
                        Some(Err(e)) => rsx! { ErrorBanner { msg: e.to_string() } },
                        None => rsx! { Skeleton {} },
                    }}
                }
            }
        }
    }
}

#[component]
fn CurrentPrice(data: Vec<PricePoint>) -> Element {
    let now = chrono::Utc::now();
    let current = data.iter().min_by_key(|p| (p.timestamp - now).num_seconds().abs());
    let yest_avg = if data.is_empty() { 0.0 } else { data.iter().map(|p| p.price_eur_mwh).sum::<f64>() / data.len() as f64 };
    match current {
        Some(p) => {
            let up = p.price_eur_mwh >= yest_avg;
            rsx! {
                div { class: "text-4xl font-bold", {format!("{:.1} €/MWh", p.price_eur_mwh)} }
                div { class: if up { "text-red-400" } else { "text-emerald-400" },
                    if up { "▲ above avg" } else { "▼ below avg" }
                }
            }
        }
        None => rsx! { ErrorBanner { msg: "no price data".to_string() } },
    }
}
```

- [ ] **Step 7: Full type-check, both targets**

Run: `cargo check --features web` then `cargo check --features server`
Expected: both succeed. Fix any remaining import-path mismatches (Dioxus `Suspense`/`use_server_future` are in `dioxus::prelude`). `use_server_future`'s `?` requires the component to return `Element` (it does).

- [ ] **Step 8: Commit (user runs)**

```bash
git add src/components/pages/
git commit -m "feat(ui): overview, prices, generation, forecast, flows pages"
```

### Task 21: Manual run smoke test

**Files:** none (verification only)

- [ ] **Step 1: Ensure `dioxus-cli` is installed**

Run: `dx --version || cargo install dioxus-cli@^0.6`
Expected: prints a 0.6.x version.

- [ ] **Step 2: Provide a token and run**

Create `.env` from `.env.example` with a real `ENTSO_E_TOKEN`, then:
Run: `dx serve`
Expected: builds web + server, serves on `http://localhost:8080`. Open it, click through all five nav links. Charts render (echarts global present), Skeletons show during load, no panics in the server log. If a page shows an `ErrorBanner`, read the message — likely an ENTSO-E field-mapping mismatch to reconcile against the live XML (see spec risk).

- [ ] **Step 3:** No commit (verification only).

---

## Phase 7 — Docker

### Task 22: Multi-stage Dockerfile

**Files:**
- Create: `Dockerfile`
- Create: `.dockerignore`

- [ ] **Step 1: Create `.dockerignore`**

```
target
.git
.idea
dist
node_modules
.env
```

- [ ] **Step 2: Create `Dockerfile`**

```dockerfile
# ---- Build stage ----
FROM rust:latest AS builder
WORKDIR /app

# dioxus-cli for `dx build`
RUN cargo install dioxus-cli@^0.6 --locked

COPY . .
# Builds web assets + the server binary with the `server` feature.
RUN dx build --release --features server

# Locate the produced server binary and assets (dx places them under target/dx/<app>/release/web/)
# Normalize into /app/out for the runtime stage.
RUN mkdir -p /app/out \
    && cp -r target/dx/fi-energy-dashboard/release/web/* /app/out/ 2>/dev/null || true \
    && find target -type f -name fi-energy-dashboard -maxdepth 6 -perm -u+x -exec cp {} /app/out/server \; 2>/dev/null || true

# ---- Runtime stage ----
FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/out/ /app/
COPY --from=builder /app/assets/ /app/assets/

ENV BIND_ADDR=0.0.0.0:8080
EXPOSE 8080
CMD ["/app/server"]
```

> Note: Dioxus 0.6's `dx build` output layout (`target/dx/<name>/release/web/`) can vary by patch version. The `cp`/`find` normalization is defensive; after the first real build, pin the exact paths from the build log and simplify the COPYs. The runtime needs the server binary, the web public assets, and `assets/` (including `echarts.min.js`).

- [ ] **Step 3: Build the image (optional, requires Docker + network)**

Run: `docker build -t fi-energy-dashboard .`
Expected: image builds. Run with `docker run -e ENTSO_E_TOKEN=... -p 8080:8080 fi-energy-dashboard`.

- [ ] **Step 4: Commit (user runs)**

```bash
git add Dockerfile .dockerignore
git commit -m "chore: multi-stage Dockerfile"
```

---

## Self-review notes (addressed)

- **Spec coverage:** prices/generation/forecast/flows endpoints (Tasks 5–7, 11), cache TTLs incl. dynamic price expiry (Task 9), `next_price_publish` Helsinki (Task 9), shared types (Task 8), `use_server_future`+`Suspense` pages with explicit None/Ok/Err (Tasks 20), Skeleton/ErrorBanner (Task 13), nav active-route (Task 14), color-coded price bars + stats (Tasks 17, 20), donut (Task 18), forecast line (Task 19), flow table+chart with import/export (Tasks 19, 20), dark theme/grid (Tasks 13, 20), Axum Extension state (Task 12), env/panic handling (Task 12), Dockerfile (Task 22), vendored echarts (Task 16), no-unwrap client paths (pages use explicit matches).
- **Known reconciliations (not placeholders):** exact Dioxus 0.6 fullstack import paths (`serve_dioxus_application`, `extract`, `ServeConfig`), charming 0.4 per-bar color (handled via JSON splice), and `dx build` output layout — each has a concrete primary approach plus a compiler-driven fallback.
- **Type consistency:** server fns live in `crate::server::entso`; pages import `crate::server::entso::{...}` and `crate::server::{FI_AREA, PricePoint, ...}`; chart components named `PriceChart`/`GenerationPie`/`ForecastLine`/`FlowChart`; page components named exactly as `Route` variants.
```
