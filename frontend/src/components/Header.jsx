export default function Header({ connected }) {
  return (
    <div className="header">
      <div>
        <h1>Climate Station</h1>
        <p>Temperature & humidity monitoring</p>
      </div>
      <div className="status">
        <span className={`status-dot ${connected === null ? '' : connected ? 'live' : 'down'}`} />
        {connected === null ? 'connecting…' : connected ? 'live' : 'sensor unreachable'}
      </div>
    </div>
  )
}
