// One reading per minute, so these map directly to a time span.
const OPTIONS = [
  { label: '1h', value: 60 },
  { label: '6h', value: 360 },
  { label: '24h', value: 1440 },
  { label: '3d', value: 4320 },
]

export default function RangeControl({ value, onChange }) {
  return (
    <div className="range-control">
      {OPTIONS.map((opt) => (
        <button
          key={opt.value}
          className={opt.value === value ? 'active' : ''}
          onClick={() => onChange(opt.value)}
        >
          {opt.label}
        </button>
      ))}
    </div>
  )
}
