<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import 'popupable/styles.css'
import 'popupable'
import Dim from '@/components/Dim.vue'
import type { ViewerFile } from '../../types'

/** Small images are scaled up until they're at least this many pixels wide and tall, if there's room. */
const MIN_SIZE = 128

const props = defineProps<{
  file: ViewerFile
}>()

const container = ref<HTMLElement>()
const failed = ref(false)
const natural = ref<{ width: number, height: number } | null>(null)
const available = ref(0)

let observer: ResizeObserver | undefined

onMounted(() => {
  if (!container.value) return

  available.value = container.value.clientWidth
  observer = new ResizeObserver(([entry]) => {
    available.value = entry.contentRect.width
  })
  observer.observe(container.value)
})

onBeforeUnmount(() => observer?.disconnect())

function onLoad(event: Event) {
  const image = event.target as HTMLImageElement
  natural.value = { width: image.naturalWidth, height: image.naturalHeight }
}

const scale = computed(() => {
  if (!natural.value || props.file.extension === 'svg') return 1

  const { width, height } = natural.value
  if (width === 0 || height === 0) return 1

  const wanted = Math.ceil(MIN_SIZE / Math.min(width, height))
  const fits = Math.floor(available.value / width)
  return Math.max(1, Math.min(wanted, fits))
})

const size = computed(() => {
  if (!natural.value || scale.value === 1) return undefined

  return {
    width: `${natural.value.width * scale.value}px`,
    height: `${natural.value.height * scale.value}px`,
  }
})
</script>

<template>
  <div ref="container" class="image-viewer">
    <Dim v-if="failed" class="failed">This image couldn't be displayed.</Dim>
    <img
      v-else
      :src="file.previewUrl"
      :alt="file.name"
      :class="{ scaled: scale > 1 }"
      :style="size"
      data-popupable
      data-popupable-zoomable
      data-popupable-transparent
      :data-popupable-no-upscale="scale > 1 ? undefined : ''"
      @load="onLoad"
      @error="failed = true"
    >
  </div>
</template>

<style lang="scss" scoped>

.image-viewer {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 160px;
  padding: 16px;
  background-color: var(--color-0-alt);
}

img {
  display: block;
  flex: none;
  max-width: 100%;
  max-height: 80vh;
  background: var(--checkerboard);

  &.scaled {
    max-height: none;
    image-rendering: pixelated;
  }
}

.failed {
  padding: 24px;
}

</style>
