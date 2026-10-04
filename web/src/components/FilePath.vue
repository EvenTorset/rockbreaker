<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import Dim from './Dim.vue'
import Tooltip from './Tooltip.vue'

const props = defineProps<{
  path: string
}>()

const lastSlash = computed(() => Math.max(props.path.lastIndexOf('/'), props.path.lastIndexOf('\\')))
const lastDot = computed(() => props.path.lastIndexOf('.'))

const parts = computed(() => {
  const path = props.path
  const dirEnd = lastSlash.value + 1
  const extStart = lastDot.value > dirEnd ? lastDot.value : path.length

  const points = new Set([0, dirEnd, extStart, path.length])
  const bounds = [...points].sort((a, b) => a - b)

  const out: { text: string, dim: boolean }[] = []
  for (let i = 0; i < bounds.length - 1; i++) {
    const start = bounds[i]
    const end = bounds[i + 1]
    if (start === end) continue
    out.push({
      text: path.slice(start, end),
      dim: end <= dirEnd || start >= extStart,
    })
  }
  return out
})

const pathEl = ref<HTMLElement>()
const isOverflowing = ref(false)

function checkOverflow() {
  if (!pathEl.value) return;
  isOverflowing.value = pathEl.value.scrollWidth > pathEl.value.clientWidth
}

let observer: ResizeObserver | undefined

onMounted(() => {
  checkOverflow()
  observer = new ResizeObserver(checkOverflow)
  if (pathEl.value) observer.observe(pathEl.value)
})

onBeforeUnmount(() => {
  observer?.disconnect()
})
</script>

<template>
  <Tooltip
    anchor="pin-x"
    :disabled="!isOverflowing"
    :style="{
      '--easy-tooltip-max-width': 'calc(100dvw - 40px)',
    }"
  >
    <template #trigger="{ props }">
      <span ref="pathEl" v-bind="props" class="file-path"><span class="bidi">&#8294;</span>
        <template v-for="(part, i) of parts" :key="i">
          <Dim v-if="part.dim">{{ part.text }}</Dim>
          <span v-else>{{ part.text }}</span>
          <span class="bidi">&lrm;</span>
        </template><span class="bidi">&#8297;</span>
      </span>
    </template>
    {{ path }}
  </Tooltip>
</template>

<style lang="scss" scoped>

.file-path {
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 100%;
  min-width: 32px;
  white-space: nowrap;
  direction: rtl;
  text-align: left;
  font-weight: 500;

  & .bidi {
    user-select: none;
  }
}

</style>
