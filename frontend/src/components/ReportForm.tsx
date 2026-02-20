import { useState, useEffect } from 'react'

interface Encounter {
  id: number
  name: string
}

interface Props {
  onSubmit: (reports: string[], encounterId: number) => void
  loading: boolean
}

export function ReportForm({ onSubmit, loading }: Props) {
  const [encounters, setEncounters] = useState<Encounter[]>([])
  const [encounterId, setEncounterId] = useState<number>(0)
  const [reportInputs, setReportInputs] = useState<string[]>([''])

  useEffect(() => {
    fetch('/api/encounters')
      .then(r => r.json())
      .then(data => {
        setEncounters(data.encounters)
        if (data.encounters.length > 0) setEncounterId(data.encounters[0].id)
      })
  }, [])

  function setReport(index: number, value: string) {
    setReportInputs(prev => prev.map((v, i) => (i === index ? value : v)))
  }

  function addReport() {
    setReportInputs(prev => [...prev, ''])
  }

  function removeReport(index: number) {
    setReportInputs(prev => prev.filter((_, i) => i !== index))
  }

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    const codes = reportInputs.map(s => s.trim()).filter(Boolean)
    if (codes.length === 0) return
    onSubmit(codes, encounterId)
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
        <label>Report Codes</label>
        {reportInputs.map((val, i) => (
          <div key={i} className="report-row">
            <input
              type="text"
              placeholder="e.g. AbVphwHqgLJ7ZQ3Y"
              value={val}
              onChange={e => setReport(i, e.target.value)}
            />
            {reportInputs.length > 1 && (
              <button type="button" className="remove-btn" onClick={() => removeReport(i)}>✕</button>
            )}
          </div>
        ))}
        <button type="button" className="add-btn" onClick={addReport}>+ Add report</button>
      </div>

      <button type="submit" className="submit-btn" disabled={loading}>
        {loading ? 'Analyzing…' : 'Analyze'}
      </button>
    </form>
  )
}
