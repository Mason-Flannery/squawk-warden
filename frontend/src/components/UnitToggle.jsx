export default function UnitToggle({ unit, onChange }) {
  return (
    <div className="unit-toggle">
      <button className={unit === 'F' ? 'active' : ''} onClick={() => onChange('F')}>
        °F
      </button>
      <button className={unit === 'C' ? 'active' : ''} onClick={() => onChange('C')}>
        °C
      </button>
    </div>
  )
}
