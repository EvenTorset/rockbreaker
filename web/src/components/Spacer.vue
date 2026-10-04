<script setup lang="ts">
import type { CSSProperties } from 'vue'

withDefaults(defineProps<{
  flex?: CSSProperties['flex']
  max?: CSSProperties['max-width']
  bridge?: boolean
}>(), {
  flex: 1,
  max: 'auto',
})
</script>

<template>
  <div :class="{ bridge }" :style="{ flex, maxWidth: max }"></div>
</template>

<style lang="scss" scoped>

.bridge {
  margin: 0 8px;
  height: 1px;
  align-self: center;
  background-color: var(--color-2);
}

</style>