<script setup lang="ts">
import { onBeforeUnmount, ref, watch } from 'vue'
import { NButton, NIcon } from 'naive-ui'
import { Dismiss16Filled } from '@vicons/fluent'

const SHOW_DELAY_MS = 250

const props = defineProps<{
  count: number
}>()

defineEmits<{
  deselect: []
}>()

const shown = ref(false)
let timer: ReturnType<typeof setTimeout> | undefined

watch(() => props.count > 0, (hasSelection) => {
  clearTimeout(timer)

  if (hasSelection) {
    timer = setTimeout(() => {
      shown.value = true
    }, SHOW_DELAY_MS)
  } else {
    shown.value = false
  }
}, { immediate: true })

onBeforeUnmount(() => clearTimeout(timer))
</script>

<template>
  <Teleport to="body">
    <Transition name="selection-bar">
      <div
        v-if="shown && count > 0"
        class="selection-bar"
        role="toolbar"
        aria-label="Selection"
        data-no-marquee
      >
        <span class="count" aria-live="polite">{{ count }} selected</span>
        <NButton size="small" @click="$emit('deselect')">
          <template #icon>
            <NIcon :component="Dismiss16Filled" />
          </template>
          Deselect all
        </NButton>
        <slot></slot>
      </div>
    </Transition>
  </Teleport>
</template>

<style lang="scss" scoped>

.selection-bar {
  position: fixed;
  z-index: 100;
  bottom: calc(16px + env(safe-area-inset-bottom, 0px));
  left: 0;
  right: 0;
  margin: 0 auto;
  width: fit-content;
  max-width: calc(100vw - 24px);
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
  padding: 8px 8px 8px 16px;
  background-color: var(--color-1);
  border: 1px solid var(--color-3);
  border-radius: 10px;
  box-shadow: 0 8px 28px rgb(0 0 0 / 0.55);
}

.count {
  margin-right: 8px;
  color: var(--color-6);
  font-weight: 600;
  white-space: nowrap;
}

.selection-bar-enter-active,
.selection-bar-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}

.selection-bar-enter-from,
.selection-bar-leave-to {
  opacity: 0;
  transform: translateY(12px);
}

@media (prefers-reduced-motion: reduce) {
  .selection-bar-enter-active,
  .selection-bar-leave-active {
    transition: opacity 0.15s ease;
  }

  .selection-bar-enter-from,
  .selection-bar-leave-to {
    transform: none;
  }
}

</style>
