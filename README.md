# Voltti

**Voltti** is a Rust + Dioxus dashboard for the Finnish electricity market,
showing real-time and forecast data from ENTSO-E: day-ahead spot prices
(in c/kWh), the generation mix, load forecast, and cross-border flows.

## Run locally

```bash
cp .env.example .env          # add your ENTSO_E_TOKEN
npm run css                   # compile Tailwind (or: npm run css:watch)
dx serve                      # needs dioxus-cli ^0.6
```

## Build the container

```bash
docker build -t voltti .
docker run -e ENTSO_E_TOKEN=... -p 8080:8080 voltti
```
