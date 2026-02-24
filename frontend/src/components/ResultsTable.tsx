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

export function ResultsTable({ players, onBack }: Props) {
  const [expanded, setExpanded] = useState<Set<string>>(new Set())

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
        <p className="no-results">No bad deaths found.</p>
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
            <th>Player</th>
            <th>Bad Deaths</th>
            <th>Avg Death Order</th>
            <th>Top-3 Deaths</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {players.map(player => (
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
