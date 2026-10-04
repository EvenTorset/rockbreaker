<script setup lang="ts">
import type { TooltipAnchor, TooltipEvent, TooltipSide } from 'easy-tooltips'
import { computed, ref, type CSSProperties } from 'vue'

const props = withDefaults(defineProps<{
  side?: TooltipSide
  distance?: number
  disabled?: boolean
  style?: CSSProperties
  anchor?: TooltipAnchor
  disableOnClick?: boolean
}>(), {
  disableOnClick: true,
})

defineEmits<{
  'tooltip-open': [event: TooltipEvent]
  'tooltip-close': [event: TooltipEvent]
  'tooltip-move': [event: TooltipEvent]
}>()

const disabledInternal = ref(false)

function cssPropertiesToString(styleObj: CSSProperties): string {
  return Object.entries(styleObj)
    .filter(([_, value]) => value != null && value !== '')
    .map(([key, value]) =>
      `${key.replace(/([A-Z])/g, '-$1').toLowerCase()}:${value};`
    )
    .join(' ')
}

const styleString = computed(() => cssPropertiesToString(props.style ?? {}))
</script>

<template>
  <slot
    name="trigger"
    :props="{
      onPointerdown: (e: PointerEvent) => {
        if (disableOnClick && e.pointerType === 'mouse') {
          disabledInternal = true
        }
      },
      onPointerleave: (_e: PointerEvent) => disabledInternal = false,
      ...!disabled && !disabledInternal && {
        'data-easy-tooltip-src': 'next',
        'data-easy-tooltip-class': 'tooltip',
        'data-easy-tooltip-prefer': side,
        'data-easy-tooltip-anchor': anchor,
        onEasyTooltipOpen: (e: TooltipEvent) => $emit('tooltip-open', e),
        onEasyTooltipClose: (e: TooltipEvent) => $emit('tooltip-close', e),
        onEasyTooltipMove: (e: TooltipEvent) => $emit('tooltip-move', e),
        ...typeof distance === 'number' ? {
          'data-easy-tooltip-style': `--easy-tooltip-distance: ${distance}px;${styleString}`,
        } : {
          'data-easy-tooltip-style': props.style ? `${styleString}` : undefined,
        },
      }
    }"
  ></slot>
  <template>
    <slot></slot>
  </template>
</template>
