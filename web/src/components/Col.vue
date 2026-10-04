<script setup lang="ts">
import type { CSSProperties } from 'vue'

withDefaults(defineProps<{
  justify?: CSSProperties['justifyContent']
  align?: CSSProperties['alignItems']
  gap?: string
  wrap?: boolean
}>(), {
  justify: 'flex-start',
  align: 'safe center',
  gap: '4px'
})
</script>

<template>
  <div :style="{
    display: 'flex',
    flexDirection: 'column',
    justifyContent: justify,
    alignItems: align,
    gap,
    flexWrap: wrap ? 'wrap' : 'nowrap',
  }">
    <slot></slot>
  </div>
</template>
