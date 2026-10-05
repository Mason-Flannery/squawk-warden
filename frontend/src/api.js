// Empty string means "same origin as the page" — correct once axum is
// serving the built frontend itself. Only set VITE_API_BASE_URL when running
// `npm run dev` against a separately-running axum server on another port.
const API_BASE = import.meta.env.VITE_API_BASE_URL || ''

async function getJson(path) {
  const res = await fetch(`${API_BASE}${path}`)
  if (!res.ok) {
    throw new Error(`Server responded ${res.status}`)
  }
  return res.json()
}

export function fetchLatest() {
  return getJson('/readings/latest')
}

export function fetchHistory(limit = 200) {
  return getJson(`/readings/history?limit=${limit}`)
}

// SQLite's CURRENT_TIMESTAMP produces "YYYY-MM-DD HH:MM:SS" (UTC, no
// timezone marker). Browsers are inconsistent about parsing that shape
// directly, so normalize it into a real ISO string before handing it to
// Date. Returns null if the value can't be made sense of, which callers
// use to fall back to reading order instead of a broken time axis.
export function parseTimestamp(raw) {
  if (!raw) return null
  const isoish = raw.includes('T') ? raw : `${raw.replace(' ', 'T')}Z`
  const date = new Date(isoish)
  return Number.isNaN(date.getTime()) ? null : date
}
