import type { ReactNode } from 'react'

interface CardProps {
  title?: string
  children: ReactNode
}

export function Card({ title, children }: CardProps) {
  return (
    <section
      style={{
        borderRadius: 12,
        border: '1px solid #1f2937',
        backgroundColor: '#020617',
        padding: 12,
      }}
    >
      {title && (
        <h3
          style={{
            fontSize: 14,
            margin: 0,
            marginBottom: 8,
          }}
        >
          {title}
        </h3>
      )}
      {children}
    </section>
  )
}
