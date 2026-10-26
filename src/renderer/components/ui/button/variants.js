import { cva } from 'class-variance-authority'

export const buttonVariants = cva(
  'inline-flex items-center justify-center gap-2 border text-[13px] font-medium outline-none transition-colors duration-100 disabled:pointer-events-none disabled:opacity-45 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring [&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0',
  {
    variants: {
      variant: {
        default: 'border-primary bg-primary text-primary-foreground hover:bg-primary/85',
        secondary: 'border-border bg-secondary text-secondary-foreground hover:bg-accent',
        outline: 'border-border bg-background text-foreground hover:bg-accent',
        ghost: 'border-transparent bg-transparent text-muted-foreground hover:border-border hover:bg-accent hover:text-foreground',
        destructive: 'border-destructive bg-destructive text-destructive-foreground hover:bg-destructive/85',
      },
      size: {
        default: 'min-h-8 h-8 px-3 py-1.5',
        sm: 'min-h-7 h-7 px-2.5 text-xs',
        icon: 'size-8 p-0',
      },
    },
    defaultVariants: {
      variant: 'default',
      size: 'default',
    },
  },
)
