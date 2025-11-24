import type { ButtonHTMLAttributes, ReactNode } from 'react'

type ButtonVariant = 'primary' | 'secondary' | 'ghost'

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant
  children: ReactNode
}

export function Button({
  variant = 'primary',
  children,
  style,
  ...rest
}: ButtonProps) {
  const base: React.CSSProperties = {
    borderRadius: 999,
    padding: '6px 12px',
    fontSize: 14,
    border: '1px solid transparent',
    cursor: 'pointer',
  }

  const palette: Record<ButtonVariant, React.CSSProperties> = {
    primary: {
      backgroundColor: '#4f46e5',
      color: '#f9fafb',
      borderColor: '#4338ca',
    },
    secondary: {
      backgroundColor: '#020617',
      color: '#e5e7eb',
      borderColor: '#374151',
    },
    ghost: {
      backgroundColor: 'transparent',
      color: '#9ca3af',
      borderColor: 'transparent',
    },
  }

  return (
    <button style={{ ...base, ...palette[variant], ...style }} {...rest}>
      {children}
    </button>
  )
}
