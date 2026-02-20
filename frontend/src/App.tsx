import { useState } from 'react'
import { ReportForm } from './components/ReportForm'
import { ResultsTable } from './components/ResultsTable'
import './App.css'

interface DeathDetail {
  fight_id: number
  death_order: number
  out_of: number
  ability_name: string
}

interface PlayerResult {
  name: string
  bad_deaths: number
  avg_death_order: number
  early_deaths: number
  details: DeathDetail[]
}

export default function App() {
  const [results, setResults] = useState<PlayerResult[] | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  async function handleSubmit(reports: string[], encounterId: number) {
    setLoading(true)
    setError(null)
    try {
      const res = await fetch('/api/analyze', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ reports, encounter_id: encounterId }),
      })
      if (!res.ok) {
        const text = await res.text()
        throw new Error(text || res.statusText)
      }
      const data = await res.json()
      setResults(data.players)
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="container">
      <h1>WCL Deaths</h1>
      {error && <div className="error">{error}</div>}
      {results === null ? (
        <ReportForm onSubmit={handleSubmit} loading={loading} />
      ) : (
        <ResultsTable players={results} onBack={() => setResults(null)} />
      )}
    </div>
  )
}
