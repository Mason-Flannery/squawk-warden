import {
  ResponsiveContainer,
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
} from 'recharts'
import { parseTimestamp } from '../api'
import { toDisplayTemp, unitSymbol } from '../lib/temperature'

const HUMIDITY_TICKS = [0, 20, 40, 60, 80, 100]

// Recharts' auto domain/ticks from 'dataMin - 2' / 'dataMax + 2' just divides
// whatever decimal range the data happens to have, which is why the labels
// come out as things like 68.57. Round outward to a clean step instead so
// the axis reads in whole numbers and the gridlines land somewhere sensible.
function niceTempAxis(values) {
  if (values.length === 0) return { domain: [0, 10], ticks: [0, 5, 10] }
  const step = 5
  const min = Math.min(...values)
  const max = Math.max(...values)
  const lo = Math.floor(min / step) * step - step
  const hi = Math.ceil(max / step) * step + step
  const ticks = []
  for (let t = lo; t <= hi; t += step) ticks.push(t)
  return { domain: [lo, hi], ticks }
}

function buildChartData(history, unit) {
  // Backend returns newest-first; the chart reads left-to-right oldest-first.
  const ascending = [...history].reverse()
  const withDates = ascending.map((r) => ({
    ...r,
    date: parseTimestamp(r.timestamp),
  }))

  const validDates = withDates.filter((r) => r.date)
  const distinctTimes = new Set(validDates.map((r) => r.date.getTime()))
  const timestampsUsable = validDates.length === withDates.length && distinctTimes.size > 1

  const spanMs = timestampsUsable
    ? withDates[withDates.length - 1].date.getTime() - withDates[0].date.getTime()
    : 0
  const spanDays = spanMs / (1000 * 60 * 60 * 24)

  const data = withDates.map((r, i) => ({
    id: r.id,
    temperature: toDisplayTemp(r.temperature, unit),
    humidity: r.humidity,
    label: timestampsUsable
      ? spanDays > 1
        ? r.date.toLocaleString(undefined, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })
        : r.date.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' })
      : `#${r.id ?? i}`,
  }))

  return { data, timestampsUsable }
}

function ChartTooltip({ active, payload, label, unit }) {
  if (!active || !payload?.length) return null
  return (
    <div
      style={{
        background: '#1a2029',
        border: '1px solid #262e3b',
        borderRadius: 8,
        padding: '8px 12px',
        fontFamily: 'IBM Plex Mono, monospace',
        fontSize: '0.8rem',
      }}
    >
      <div style={{ color: '#8b93a7', marginBottom: 4 }}>{label}</div>
      {payload.map((p) => (
        <div key={p.dataKey} style={{ color: p.color }}>
          {p.dataKey === 'temperature' ? `${p.value.toFixed(1)}${unitSymbol(unit)}` : `${p.value.toFixed(1)}%`}
        </div>
      ))}
    </div>
  )
}

export default function HistoryChart({ history, loading, unit }) {
  if (loading && history.length === 0) {
    return <div className="chart-empty">loading history…</div>
  }

  if (history.length === 0) {
    return <div className="chart-empty">No readings yet. Once the sensor logs some data, it'll show up here.</div>
  }

  const { data, timestampsUsable } = buildChartData(history, unit)
  const tempAxis = niceTempAxis(data.map((d) => d.temperature))

  return (
    <div>
      {!timestampsUsable && (
        <p className="chart-note">
          Timestamps look off on these readings (missing or identical), so this is plotted by reading order instead
          of time. Worth checking that the readings table's timestamp column has a default value.
        </p>
      )}
      <ResponsiveContainer width="100%" height={280}>
        <LineChart data={data} margin={{ top: 8, right: 12, left: -12, bottom: 0 }}>
          <CartesianGrid stroke="#262e3b" vertical={false} />
          <XAxis
            dataKey="label"
            stroke="#8b93a7"
            fontSize={11}
            tickLine={false}
            axisLine={{ stroke: '#262e3b' }}
            minTickGap={32}
          />
          <YAxis
            yAxisId="temp"
            stroke="#e8a33d"
            fontSize={11}
            tickLine={false}
            axisLine={false}
            width={36}
            domain={tempAxis.domain}
            ticks={tempAxis.ticks}
            allowDecimals={false}
          />
          <YAxis
            yAxisId="humidity"
            orientation="right"
            stroke="#4fb0c6"
            fontSize={11}
            tickLine={false}
            axisLine={false}
            width={36}
            domain={[0, 100]}
            ticks={HUMIDITY_TICKS}
            allowDecimals={false}
          />
          <Tooltip content={<ChartTooltip unit={unit} />} />
          <Line
            yAxisId="temp"
            type="monotone"
            dataKey="temperature"
            stroke="#e8a33d"
            strokeWidth={2}
            dot={false}
            isAnimationActive={false}
          />
          <Line
            yAxisId="humidity"
            type="monotone"
            dataKey="humidity"
            stroke="#4fb0c6"
            strokeWidth={2}
            dot={false}
            isAnimationActive={false}
          />
        </LineChart>
      </ResponsiveContainer>
      <div className="legend">
        <span className="legend-item">
          <span className="legend-swatch" style={{ background: '#e8a33d' }} />
          Temperature
        </span>
        <span className="legend-item">
          <span className="legend-swatch" style={{ background: '#4fb0c6' }} />
          Humidity
        </span>
      </div>
    </div>
  )
}
