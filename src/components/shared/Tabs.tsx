import { useState } from 'react'

export interface TabItem {
  id: string
  label: string
  content: React.ReactNode
}

interface TabsProps {
  items: TabItem[]
  initialId?: string
}

export function Tabs({ items, initialId }: TabsProps) {
  const [activeId, setActiveId] = useState(
    () => initialId ?? items[0]?.id,
  )

  const active = items.find((t) => t.id === activeId) ?? items[0]

  return (
    <div>
      <div
        style={{
          display: 'flex',
          gap: 8,
          borderBottom: '1px solid #1f2937',
          marginBottom: 8,
        }}
      >
        {items.map((tab) => (
          <button
            key={tab.id}
            type="button"
            onClick={() => setActiveId(tab.id)}
            style={{
              border: 'none',
              background: 'transparent',
              padding: '4px 8px',
              cursor: 'pointer',
              fontSize: 13,
              color:
                tab.id === active?.id ? '#f9fafb' : '#9ca3af',
              borderBottom:
                tab.id === active?.id
                  ? '2px solid #4f46e5'
                  : '2px solid transparent',
            }}
          >
            {tab.label}
          </button>
        ))}
      </div>
      <div>{active?.content}</div>
    </div>
  )
}
