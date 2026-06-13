# fi-energy-dashboard — Design

**Date:** 2026-06-13
**Status:** Approved (design); ready for implementation plan

A Finnish electricity market dashboard showing real-time and forecast data from
ENTSO-E. A Dioxus 0.6 fullstack app fetches data server-side via server
functions, caches responses in-memory (moka), and renders a dark-themed
multi-page dashboard with ECharts charts.

## Decisions locked during brainstorming

- **Dioxus 0.6**, not 0.5. The `use_server_future(...)?` + `Suspense` pattern in
  the original spec is the 0.6-native idiom.
- **Replace in place**: convert the existing `electricity-prices` repo (currently
  a hello-world) into the `fi-energy-dashboard` app + a Cargo workspace. The
  existing `entsoe/` crate becomes a path-dependency member.
- **Real data only**: read `ENTSO_E_TOKEN`; **panic with a clear message if it is
  missing**. No demo/mock data mode.
- **Full real endpoints in the `entsoe` crate**: the three missing ENTSO-E
  endpoints are implemented in the crate, not the app.
- **echarts.min.js vendored locally** into `assets/` (not a CDN script tag).
- **Commits are handled by the user.** Do not run `git commit`/`git push`.

## Critical context: the `entsoe` crate only does prices

The local `entsoe/` crate currently implements exactly one endpoint:
`get_prices` (document type `A44`, day-ahead prices). It has **no** generation,
load-forecast, or cross-border-flow support, and its `RequestParameters` is
hardcoded to the six price-query params. The bulk of this project is extending
that crate.

| Server fn | ENTSO-E doc | In crate today? | Notes |
|---|---|---|---|
| `get_spot_prices` | A44 | yes | maps cleanly to `PricePoint` |
| `get_generation_mix` | A75 (Actual Gen per Type) | **add** | needs `processType`, `psrType`; `GL_MarketDocument`, `quantity` points |
| `get_consumption_forecast` | A65 (Load Forecast) | **add** | needs `outBiddingZone_Domain`, `processType=A01` |
| `get_cross_border_flows` | A11 (Physical Flows) | **add** | one query per border direction; net import/export |

Additional gaps: the `Domain` enum has no **EE** or **RU** (needed for FI↔EE /
FI↔RU flows; note FI–RU commercial flow has been ~0 since 2022 → that border
usually returns "no data", handled gracefully). The existing `Domain::from_str`
has a bug (NO3 and NO4 both map to `NO1`) which will be fixed.

## Architecture

### Workspace & build
- Root `Cargo.toml` defines the `fi-energy-dashboard` package **and** a
  `[workspace]` with `entsoe` as a member (path dependency).
- Dioxus 0.6 fullstack with `web` + `server` feature flags. `dx serve` for dev,
  `dx build --release` for prod.
- `src/main.rs`:
  - Client: `dioxus::launch(App)`.
  - `#[cfg(feature = "server")]`: load `.env` (dotenvy), read `ENTSO_E_TOKEN`
    (panic if missing) and `BIND_ADDR` (default `0.0.0.0:8080`), construct
    `Arc<EntsoeCache>`, build an Axum `Router`, attach the Dioxus app via
    `serve_dioxus_application(...)`, and `.layer(Extension(cache))`.

### `entsoe` crate extension
- `RequestParameters` gains optional `process_type`, `psr_type`,
  `out_bidding_zone_domain`; per-query typed constructors keep prices working
  unchanged. `to_params()` only emits the fields that are set.
- New enums: `ProcessType` (e.g. `A16` Realised, `A01` Day-ahead) and `PsrType`
  (B-codes → human names: Wind Onshore/Offshore, Solar, Hydro variants, Nuclear,
  Fossil Gas/Coal, Biomass, Waste, etc.).
- New models:
  - `models/generation.rs` + `models/load.rs`: `GL_MarketDocument` with
    `quantity`-based `Point`s; generation TimeSeries carries `MktPSRType.psrType`.
  - Flows reuse a `quantity` variant of `Publication_MarketDocument`.
- New endpoints, each reusing the existing acknowledgement-error handling:
  `get_generation` (A75), `get_load_forecast` (A65), `get_flows` (A11).
- `Domain`: add `EE` (`10Y1001A1001A39I`), `RU` (`10Y1001A1001A49F`); fix
  NO3/NO4 `FromStr`.

### App server layer (`src/server/`)
- `mod.rs`: shared data types — `PricePoint { timestamp: DateTime<Utc>,
  price_eur_mwh: f64 }`, `GenerationMix { timestamp, sources: Vec<GenerationSource> }`,
  `GenerationSource { source_type: String, value_mw: f64 }`,
  `ForecastPoint { timestamp, value_mw: f64 }`,
  `FlowPoint { from_area, to_area, value_mw, timestamp }`. All derive
  `Serialize, Deserialize, Clone, Debug`. Also a `thiserror` `ServerError` enum.
- `cache.rs`: `EntsoeCache` holds the `entsoe::Entsoe` client + **four**
  `moka::future::Cache`s — one per data type, because they store different value
  types (a single `Cache<String, V>` cannot hold all four). Keys remain
  `"{data_type}:{area}"` strings.
  - `spot_prices`: custom `moka::Expiry` returning `next_price_publish()`.
  - `generation`: 10 min TTL. `forecast`: 30 min TTL. `flows`: 10 min TTL.
  - `next_price_publish() -> std::time::Duration`: next **13:15 Europe/Helsinki**
    via `chrono-tz` (the original spec said both "13:15 CET" and "Europe/Helsinki";
    resolved in favor of Helsinki). Hour/minute are constants for easy tuning.
- `entso.rs`: four `#[server]` functions, default area
  `"10YFI-1--------U"`. Each: `extract::<Extension<Arc<EntsoeCache>>>()` → cache
  lookup (early return on hit) → call the crate → map the XML document into the
  shared type (position+resolution → timestamps; psrType → names; flows netted
  per border) → insert into cache → return. All errors mapped through
  `ServerFnError`. Flows query FI↔SE3, FI↔EE, FI↔NO4, FI↔RU (both directions,
  netted); RU "no data" → 0.

### UI (`src/components/`)
- `app.rs`: Router + the `Route` enum (Nav layout; routes `/`, `/prices`,
  `/generation`, `/forecast`, `/flows`). Renders a `document::Script` for the
  vendored `echarts.min.js` and the Tailwind stylesheet.
- `nav.rs`: dark (`bg-gray-900`) horizontal top nav, "FI Energy Dashboard" title
  on the left, links to all five pages, active route highlighted.
- Reusable `Skeleton` (loading) and `ErrorBanner { msg }`.
- `pages/`: each page uses `use_server_future(|| fetch(...))?` + `Suspense` with
  an explicit `None | Some(Ok(d)) | Some(Err(e))` match (Skeleton / chart /
  ErrorBanner). No `unwrap()`.
  - Overview: current-hour spot price (large), up/down trend badge vs yesterday
    avg, today sparkline, quick-stat cards (generation total, consumption forecast).
  - Prices: 24h day-ahead bar chart, bars color-coded (green <5 c/kWh, yellow
    5–15, red >15), current hour highlighted, min/max/avg stats.
  - Generation: donut/pie of current mix by source, MW labels, colored legend.
  - Forecast: line chart of next-24h consumption forecast.
  - Flows: table/bar of cross-border flows with import/export direction indicators.
- `charts/` (`price_chart`, `generation_pie`, `flow_map`): build a
  `charming::Chart` from props, `serde_json`-serialize it, and in a `use_effect`
  call `document::eval` to run `echarts.init(el).setOption(json)` on a
  100%-width div (unique id per instance). No authored JS.

### Styling
- Tailwind via the `dx` asset pipeline: `assets/tailwind.css`, `tailwind.config.js`.
- Dark theme: `bg-gray-900` page, `bg-gray-800` cards (rounded, subtle border,
  padding). Overview uses a responsive grid (`grid-cols-1 md:grid-cols-2`).

### Docker
Multi-stage:
1. `rust:latest`: install `dioxus-cli`, `dx build --release`.
2. `debian:bookworm-slim`: copy the server binary + `assets/` (incl. vendored
   `echarts.min.js`). Expose `8080`.

## Error handling
- Server-fn errors mapped through `ServerFnError`; domain errors via `thiserror`
  (`ServerError`, plus the crate's `EntsoeError`).
- Client components handle `None`, `Some(Ok)`, `Some(Err)` explicitly; no
  `unwrap()` in production paths.

## Build order
1. Workspace + Cargo.toml + Dioxus.toml + Tailwind scaffold (blank app compiles).
2. Extend `entsoe` crate (params, enums, models, endpoints) with unit tests on
   XML parsing using sample payloads.
3. App server layer (shared types, cache, four server functions).
4. Components: app/router/nav + Skeleton/ErrorBanner.
5. Charts + vendored echarts.min.js.
6. Pages wired to server functions.
7. Dockerfile.

## Risks
- **echarts.min.js** must be fetched once into `assets/` (vendoring chosen). Pull
  via `curl` from jsdelivr during implementation; if network is blocked, stop and
  report.
- The three new ENTSO-E endpoints can't be verified against the live API without
  a token. XML parsing is unit-tested against sample payloads; live field mapping
  is best-effort until run with a real `ENTSO_E_TOKEN`.

## Out of scope
- User authentication.
- Any hand-authored JavaScript files (all interactivity via Dioxus / `eval`).
- Hardcoded tokens.
