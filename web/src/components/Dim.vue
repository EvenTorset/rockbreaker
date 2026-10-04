<script setup lang="ts">
withDefaults(defineProps<{
  tag?: string
  pure?: boolean
}>(), {
  tag: 'span'
})
</script>

<template>
  <component :is="tag" class="dim" :style="{
    display: pure ? 'contents' : undefined
  }">
    <slot></slot>
  </component>
</template>

<style lang="scss" scoped>

.dim {
  color: var(--color-dim) !important;
}

</style>
