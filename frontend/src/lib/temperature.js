// Readings are always stored and sent as Celsius; this only affects display.
export function toDisplayTemp(celsius, unit) {
  return unit === 'F' ? (celsius * 9) / 5 + 32 : celsius
}

export function unitSymbol(unit) {
  return unit === 'F' ? '°F' : '°C'
}
