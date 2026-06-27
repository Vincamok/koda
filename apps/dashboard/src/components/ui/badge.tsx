import * as React from 'react'
import { cva, type VariantProps } from 'class-variance-authority'
import { cn } from '@/lib/utils'

const badgeVariants = cva(
  'inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-xs font-medium transition-colors',
  {
    variants: {
      variant: {
        default: 'bg-koda-surface-raised text-koda-text border border-koda-border',
        primary: 'bg-indigo-500/20 text-indigo-300 border border-indigo-500/30',
        success: 'bg-green-500/20 text-green-300 border border-green-500/30',
        warning: 'bg-amber-500/20 text-amber-300 border border-amber-500/30',
        danger: 'bg-red-500/20 text-red-300 border border-red-500/30',
        muted: 'bg-koda-surface-raised/50 text-koda-text-muted border border-koda-border/50',
        // Workspace status variants
        created: 'bg-slate-500/20 text-slate-300 border border-slate-500/30',
        cloning: 'bg-blue-500/20 text-blue-300 border border-blue-500/30',
        ready: 'bg-indigo-500/20 text-indigo-300 border border-indigo-500/30',
        starting: 'bg-amber-500/20 text-amber-300 border border-amber-500/30',
        running: 'bg-green-500/20 text-green-300 border border-green-500/30',
        stopping: 'bg-orange-500/20 text-orange-300 border border-orange-500/30',
        stopped: 'bg-slate-500/20 text-slate-300 border border-slate-500/30',
        reviewing: 'bg-purple-500/20 text-purple-300 border border-purple-500/30',
        closed: 'bg-zinc-500/20 text-zinc-400 border border-zinc-500/30',
        failed: 'bg-red-500/20 text-red-300 border border-red-500/30',
      },
    },
    defaultVariants: {
      variant: 'default',
    },
  },
)

export interface BadgeProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof badgeVariants> {}

function Badge({ className, variant, ...props }: BadgeProps) {
  return (
    <div className={cn(badgeVariants({ variant }), className)} {...props} />
  )
}

export { Badge, badgeVariants }
