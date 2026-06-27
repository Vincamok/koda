import * as React from 'react'
import { cn } from '@/lib/utils'

export interface InputProps
  extends React.InputHTMLAttributes<HTMLInputElement> {
  error?: string
}

const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ className, type, error, ...props }, ref) => {
    return (
      <div className="w-full">
        <input
          type={type}
          ref={ref}
          className={cn(
            'flex h-9 w-full rounded border border-koda-border bg-koda-surface-raised px-3 py-1 text-sm text-koda-text shadow-sm transition-colors',
            'placeholder:text-koda-text-muted',
            'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-koda-primary focus-visible:ring-offset-1 focus-visible:ring-offset-koda-surface',
            'disabled:cursor-not-allowed disabled:opacity-50',
            error && 'border-red-500 focus-visible:ring-red-500',
            className,
          )}
          {...props}
        />
        {error && (
          <p className="mt-1 text-xs text-red-400">{error}</p>
        )}
      </div>
    )
  },
)
Input.displayName = 'Input'

export { Input }
