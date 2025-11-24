import { FormEvent } from 'react'

interface InputBarProps {
  value: string
  onChange: (value: string) => void
  onSend: (value: string) => void
  disabled?: boolean
  placeholder?: string
}

export function InputBar({
  value,
  onChange,
  onSend,
  disabled,
  placeholder,
}: InputBarProps) {
  function handleSubmit(e: FormEvent) {
    e.preventDefault()
    const trimmed = value.trim()
    if (!trimmed) return
    onSend(trimmed)
  }

  return (
    <form
      onSubmit={handleSubmit}
      style={{
        display: 'flex',
        gap: 8,
        paddingTop: 8,
        borderTop: '1px solid #1f2937',
      }}
    >
      <textarea
        value={value}
        placeholder={placeholder ?? 'Posez une question à GenAptitude…'}
        onChange={(e) => onChange(e.target.value)}
        disabled={disabled}
        rows={2}
        style={{
          flex: 1,
          resize: 'none',
          borderRadius: 8,
          border: '1px solid #374151',
          padding: 8,
          fontSize: 14,
          backgroundColor: '#020617',
          color: '#f9fafb',
        }}
      />
      <button
        type="submit"
        disabled={disabled || !value.trim()}
        style={{
          borderRadius: 999,
          padding: '8px 16px',
          border: 'none',
          backgroundColor: disabled ? '#4b5563' : '#4f46e5',
          color: '#f9fafb',
          fontWeight: 500,
          cursor: disabled ? 'not-allowed' : 'pointer',
        }}
      >
        Envoyer
      </button>
    </form>
  )
}
