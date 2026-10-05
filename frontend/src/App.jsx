import { useEffect, useState, useCallback } from 'react'
import Header from './components/Header.jsx'
import ReadingPanel from './components/ReadingPanel.jsx'
import RangeControl from './components/RangeControl.jsx'
import HistoryChart from './components/HistoryChart.jsx'
import { fetchLatest, fetchHistory } from './api.js'

const POLL_MS = 15000 // readings land once a minute; this just needs to notice within a few seconds of that
const HISTORY_REFRESH_MS = 60000

export default function App() {
  const [latest, setLatest] = useState(null)
  const [connected, setConnected] = useState(null)
  const [latestLoading, setLatestLoading] = useState(true)
  const [error, setError] = useState(null)

  const [range, setRange] = useState(360) // 6h at one reading/minute
  const [unit, setUnit] = useState('F')
  const [history, setHistory] = useState([])
  const [historyLoading, setHistoryLoading] = useState(true)

  const loadLatest = useCallback(async () => {
    try {
      const data = await fetchLatest()
      setLatest(data)
      setConnected(true)
      setError(null)
    } catch (err) {
      setConnected(false)
      setError(err.message)
    } finally {
      setLatestLoading(false)
    }
  }, [])

  const loadHistory = useCallback(async (limit) => {
    setHistoryLoading(true)
    try {
      const data = await fetchHistory(limit)
      setHistory(data)
    } catch (err) {
      setError(err.message)
    } finally {
      setHistoryLoading(false)
    }
  }, [])

  useEffect(() => {
    loadLatest()
    const id = setInterval(loadLatest, POLL_MS)
    return () => clearInterval(id)
  }, [loadLatest])

  useEffect(() => {
    loadHistory(range)
    const id = setInterval(() => loadHistory(range), HISTORY_REFRESH_MS)
    return () => clearInterval(id)
  }, [range, loadHistory])

  return (
    <div className="page">
      <Header connected={connected} />
      <ReadingPanel latest={latest} loading={latestLoading} unit={unit} onUnitChange={setUnit} />

      <div className="chart-panel">
        <div className="chart-panel-header">
          <h2>History</h2>
          <RangeControl value={range} onChange={setRange} />
        </div>
        <HistoryChart history={history} loading={historyLoading} unit={unit} />
      </div>

      {error && (
        <div className="error-banner">
          Can't reach the sensor API ({error}). Make sure the axum server is running on the address in
          VITE_API_BASE_URL and that it allows requests from this page's origin (CORS).
        </div>
      )}
    </div>
  )
}
