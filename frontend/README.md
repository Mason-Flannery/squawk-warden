# Climate Station frontend

A small React dashboard for the temperature/humidity sensor: a live readout up
top, and a history chart below.

## Run it

```bash
npm install
npm run dev
```

Opens on http://localhost:5173. It talks to the axum server at
`http://localhost:3000` by default — copy `.env.example` to `.env` and change
`VITE_API_BASE_URL` if your server lives somewhere else.

## What it needs from the backend

- `GET /readings/latest` — already exists, returns one `Reading` as JSON (or
  204/404-ish when the table is empty — the frontend treats "not ok" as "no
  data yet").
- `GET /readings/history?limit=N` — **new**, needed for the chart. See
  `backend-additions.md` in this folder for the Rust code to add.
- CORS enabled for the frontend's origin (`http://localhost:5173` in dev),
  otherwise the browser will block the requests. Also covered in
  `backend-additions.md`.

## Range options assume one reading per minute

The 1h / 6h / 24h / 3d buttons are just reading counts under the hood (60,
360, 1440, 4320) — they only line up with real time spans because the sensor
logs once a minute. If you ever change the collection interval, update the
`value`s in `src/components/RangeControl.jsx` to match, or swap the history
endpoint to filter by an actual timestamp range instead of a row count once
the timestamp column is confirmed reliable (see below).

## About the timestamp

`new_reading` doesn't pass a timestamp to the `INSERT`, so it only gets a real
value if the `readings` table's `timestamp` column has a
`DEFAULT CURRENT_TIMESTAMP` (or similar) in the schema. If it doesn't, every
row's timestamp will be `NULL` or empty.

The chart handles this defensively: if timestamps are missing or all
identical, it falls back to plotting by reading order and shows a small note
so it's obvious something's off. Worth double-checking the actual
`CREATE TABLE readings` statement (not shown in what was shared) to confirm
the column has that default — or just query the table directly:

```bash
sqlite3 your.db "SELECT id, timestamp FROM readings ORDER BY id DESC LIMIT 5;"
```
