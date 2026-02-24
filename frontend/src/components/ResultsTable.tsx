import { useState } from 'react'

interface DeathDetail {
  date: string
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

interface Props {
  players: PlayerResult[]
  onBack: () => void
}

type SortKey = 'name' | 'bad_deaths' | 'avg_death_order' | 'early_deaths'
type SortDir = 'asc' | 'desc'

export function ResultsTable({ players, onBack }: Props) {
  const [expanded, setExpanded] = useState<Set<string>>(new Set())
  const [sortKey, setSortKey] = useState<SortKey>('bad_deaths')
  const [sortDir, setSortDir] = useState<SortDir>('desc')

  function handleSort(key: SortKey) {
    if (key === sortKey) {
      setSortDir(d => d === 'asc' ? 'desc' : 'asc')
    } else {
      setSortKey(key)
      setSortDir(key === 'name' ? 'asc' : 'desc')
    }
  }

  function sortIndicator(key: SortKey) {
    if (key !== sortKey) return ' ↕'
    return sortDir === 'asc' ? ' ↑' : ' ↓'
  }

  const sorted = [...players].sort((a, b) => {
    const mul = sortDir === 'asc' ? 1 : -1
    if (sortKey === 'name') return mul * a.name.localeCompare(b.name)
    return mul * (a[sortKey] - b[sortKey])
  })

  function toggle(name: string) {
    setExpanded(prev => {
      const next = new Set(prev)
      next.has(name) ? next.delete(name) : next.add(name)
      return next
    })
  }

  if (players.length === 0) {
    return (
      <div>
        <p className="no-results">No results found.</p>
        <button className="back-btn" onClick={onBack}>← Back</button>
      </div>
    )
  }

  return (
    <div>
      <div className="results-header">
        <h2>Bad Deaths Summary</h2>
        <button className="back-btn" onClick={onBack}>← Back</button>
      </div>
      <table className="results-table">
        <thead>
          <tr>
            <th className="sortable" onClick={() => handleSort('name')}>Player{sortIndicator('name')}</th>
            <th className="sortable" onClick={() => handleSort('bad_deaths')}>Bad Deaths{sortIndicator('bad_deaths')}</th>
            <th className="sortable" onClick={() => handleSort('avg_death_order')}>Avg Death Order{sortIndicator('avg_death_order')}</th>
            <th className="sortable" onClick={() => handleSort('early_deaths')}>Top-3 Deaths{sortIndicator('early_deaths')}</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {sorted.map(player => (
            <>
              <tr key={player.name} className="player-row" onClick={() => toggle(player.name)}>
                <td className="player-name">{player.name}</td>
                <td>{player.bad_deaths}</td>
                <td>{player.avg_death_order.toFixed(1)}</td>
                <td>{player.early_deaths}</td>
                <td className="expand-cell">{expanded.has(player.name) ? '▲' : '▼'}</td>
              </tr>
              {expanded.has(player.name) && (() => {
                const byDate = player.details.reduce((acc, d) => {
                  (acc[d.date] ??= []).push(d)
                  return acc
                }, {} as Record<string, DeathDetail[]>)
                return Object.entries(byDate).sort().flatMap(([date, fights]) => [
                  <tr key={`${player.name}-${date}-header`} className="detail-row">
                    <td colSpan={5} className="detail-date-header">{date}</td>
                  </tr>,
                  ...fights.map((d, i) => (
                    <tr key={`${player.name}-${date}-${i}`} className="detail-row">
                      <td colSpan={5} className="detail-cell">
                        Fight {d.fight_id} — died {d.death_order}/{d.out_of} to <strong>{d.ability_name}</strong>
                      </td>
                    </tr>
                  ))
                ])
              })()}
            </>
          ))}
        </tbody>
      </table>
    </div>
  )
}
