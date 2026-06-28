'use client'

import * as React from 'react'
import { ChevronDown } from 'lucide-react'
import { cn } from '@/lib/utils'

export interface SelectOption {
  value: string
  label: string
}

export interface SelectProps
  extends React.SelectHTMLAttributes<HTMLSelectElement> {
  options: SelectOption[]
  placeholder?: string
  onValueChange?: (value: string) => void
  error?: string
}

const Select = React.forwardRef<HTMLSelectElement, SelectProps>(
  ({ className, options, placeholder, onValueChange, error, onChange, ...props }, ref) => {
    const handleChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
      onChange?.(e)
      onValueChange?.(e.target.value)
    }

    return (
      <div className="relative w-full">
        <select
          ref={ref}
          className={cn(
            'flex h-9 w-full appearance-none rounded border border-koda-border bg-koda-surface-raised px-3 py-1 pr-8 text-sm text-koda-text shadow-sm transition-colors',
            'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-koda-primary focus-visible:ring-offset-1 focus-visible:ring-offset-koda-surface',
            'disabled:cursor-not-allowed disabled:opacity-50',
            error && 'border-red-500 focus-visible:ring-red-500',
            className,
          )}
          onChange={handleChange}
          {...props}
        >
          {placeholder && (
            <option value="" disabled>
              {placeholder}
            </option>
          )}
          {options.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
        <ChevronDown className="pointer-events-none absolute right-2.5 top-1/2 h-4 w-4 -translate-y-1/2 text-koda-text-muted" />
        {error && (
          <p className="mt-1 text-xs text-red-400">{error}</p>
        )}
      </div>
    )
  },
)
Select.displayName = 'Select'

export { Select }
