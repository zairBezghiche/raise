interface SuggestionPanelProps {
    suggestions: string[]
    onSelect: (value: string) => void
  }
  
  export function SuggestionPanel({
    suggestions,
    onSelect,
  }: SuggestionPanelProps) {
    if (!suggestions.length) return null
  
    return (
      <div
        style={{
          display: 'flex',
          flexWrap: 'wrap',
          gap: 8,
          marginBottom: 8,
        }}
      >
        {suggestions.map((s) => (
          <button
            key={s}
            type="button"
            onClick={() => onSelect(s)}
            style={{
              borderRadius: 999,
              padding: '4px 10px',
              border: '1px solid #374151',
              backgroundColor: '#020617',
              color: '#9ca3af',
              fontSize: 12,
              cursor: 'pointer',
            }}
          >
            {s}
          </button>
        ))}
      </div>
    )
  }
  