import { useState, useEffect } from 'react'

interface Encounter {
  id: number
  name: string
}

interface SubmitParams {
  guildName: string
  guildServerSlug: string
  guildServerRegion: string
  startTime: number
  endTime: number
  encounterId: number
}

interface Props {
  onSubmit: (params: SubmitParams) => void
  loading: boolean
}

export function ReportForm({ onSubmit, loading }: Props) {
  const [encounters, setEncounters] = useState<Encounter[]>([])
  const [encounterId, setEncounterId] = useState<number>(0)
  const [guildName, setGuildName] = useState('')
  const [guildServerSlug, setGuildServerSlug] = useState('')
  const [guildServerRegion, setGuildServerRegion] = useState('us')
  const [startDate, setStartDate] = useState('')
  const [endDate, setEndDate] = useState('')

  useEffect(() => {
    fetch('/api/encounters')
      .then(r => r.json())
      .then(data => {
        setEncounters(data.encounters)
        if (data.encounters.length > 0) setEncounterId(data.encounters[0].id)
      })
  }, [])

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    if (!guildName || !guildServerSlug || !startDate || !endDate) return
    onSubmit({
      guildName,
      guildServerSlug,
      guildServerRegion,
      startTime: new Date(startDate).getTime(),
      endTime: new Date(endDate).getTime() + 86400000 + 28800000, // end of day in Pacific (UTC-8)
      encounterId,
    })
  }

  return (
    <form onSubmit={handleSubmit} className="form">
      <div className="field">
        <label>Encounter</label>
        <select value={encounterId} onChange={e => setEncounterId(Number(e.target.value))}>
          {encounters.map(enc => (
            <option key={enc.id} value={enc.id}>{enc.name}</option>
          ))}
        </select>
      </div>

      <div className="field">
        <label>Guild Name</label>
        <input
          type="text"
          placeholder="e.g. Boyfriends"
          value={guildName}
          onChange={e => setGuildName(e.target.value)}
          required
        />
      </div>

      <div className="field">
        <label>Server</label>
        <input
          type="text"
          placeholder="e.g. tichondrius"
          value={guildServerSlug}
          onChange={e => setGuildServerSlug(e.target.value)}
          required
        />
      </div>

      <div className="field">
        <label>Region</label>
        <select value={guildServerRegion} onChange={e => setGuildServerRegion(e.target.value)}>
          <option value="us">US</option>
          <option value="eu">EU</option>
          <option value="kr">KR</option>
          <option value="tw">TW</option>
        </select>
      </div>

      <div className="field">
        <label>Start Date</label>
        <input
          type="date"
          value={startDate}
          onChange={e => setStartDate(e.target.value)}
          required
        />
      </div>

      <div className="field">
        <label>End Date</label>
        <input
          type="date"
          value={endDate}
          onChange={e => setEndDate(e.target.value)}
          required
        />
      </div>

      <button type="submit" className="submit-btn" disabled={loading}>
        {loading ? 'Analyzing…' : 'Analyze'}
      </button>
    </form>
  )
}
