<script setup>
import { cn } from '@/lib/utils.js'

const props = defineProps({
  tone: { type: String, default: 'neutral' },
  class: { type: [String, Array, Object], default: undefined },
})
</script>

<template>
  <span
    data-slot="badge"
    :class="cn(
      'inline-flex h-5 items-center border px-1.5 font-mono text-[11px] leading-none',
      tone === 'danger' && 'border-destructive/70 text-destructive',
      tone === 'warning' && 'border-warning/70 text-warning',
      tone === 'neutral' && 'border-border text-muted-foreground',
      props.class,
    )"
  >
    <slot />
  </span>
</template>
