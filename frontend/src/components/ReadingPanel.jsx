import { parseTimestamp } from '../api'
import { toDisplayTemp, unitSymbol } from '../lib/temperature'
import UnitToggle from './UnitToggle'

function relativeTime(date) {
  if (!date) return null
  const seconds = Math.round((Date.now() - date.getTime()) / 1000)
  if (seconds < 5) return 'just now'
  if (seconds < 60) return `${seconds}s ago`
  const minutes = Math.round(seconds / 60)
  if (minutes < 60) return `${minutes}m ago`
  const hours = Math.round(minutes / 60)
  if (hours < 24) return `${hours}h ago`
  return date.toLocaleString()
}

export default function ReadingPanel({ latest, loading, unit, onUnitChange }) {
  const status = !latest ? (loading ? 'reading…' : 'no data yet') : null
  const when = latest ? relativeTime(parseTimestamp(latest.timestamp)) : null

  return (
    <div className="reading-panel">
      <div className="reading temp">
        <div className="reading-header">
          <p className="reading-label">Temperature</p>
          <UnitToggle unit={unit} onChange={onUnitChange} />
        </div>
        {status ? (
          <p className="reading-empty">{status}</p>
        ) : (
          <>
            <div className="reading-value">
              {toDisplayTemp(latest.temperature, unit).toFixed(1)}
              <span className="reading-unit">{unitSymbol(unit)}</span>
            </div>
            <p className="reading-updated">{when ? `as of ${when}` : 'timestamp unavailable'}</p>
          </>
        )}
      </div>
      <div className="reading humidity">
        <p className="reading-label">Humidity</p>
        {status ? (
          <p className="reading-empty">{status}</p>
        ) : (
          <>
            <div className="reading-value">
              {latest.humidity.toFixed(1)}
              <span className="reading-unit">%</span>
            </div>
            <p className="reading-updated">{when ? `as of ${when}` : 'timestamp unavailable'}</p>
          </>
        )}
      </div>
    </div>
  )
}
