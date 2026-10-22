<script setup lang="ts">
import { ref, watch } from 'vue'
import Dim from '@/components/Dim.vue'
import { formatBytes } from '@/util/format'
import { decodeText } from '../../sniff'
import type { ViewerFile } from '../../types'
import ViewerState from '../../ViewerState.vue'

const MAX_BYTES = 512 * 1024

const props = defineProps<{
  file: ViewerFile
}>()

const text = ref('')
const loading = ref(true)
const error = ref<string | null>(null)
const truncated = ref(false)

watch(() => props.file, async (file) => {
  loading.value = true
  error.value = null

  try {
    const bytes = await file.readSlice(0, MAX_BYTES)
    text.value = decodeText(bytes)
    truncated.value = file.size !== null ? file.size > MAX_BYTES : bytes.length === MAX_BYTES
  } catch (err) {
    if (!file.signal.aborted) error.value = err instanceof Error ? err.message : 'Could not read the file.'
  } finally {
    loading.value = false
  }
}, { immediate: true })
</script>

<template>
  <ViewerState :loading :error>
    <div class="text-viewer">
      <Dim v-if="truncated" tag="p" class="notice">
        Showing the first {{ formatBytes(MAX_BYTES) }}. Download the file to see all of it.
      </Dim>
      <pre tabindex="0">{{ text }}</pre>
    </div>
  </ViewerState>
</template>

<style lang="scss" scoped>

.notice {
  padding: 8px 16px;
  border-bottom: 1px solid var(--color-2);
}

pre {
  margin: 0;
  padding: 16px;
  max-height: 80vh;
  overflow: auto;
  color: var(--color-6);
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, 'Liberation Mono', monospace;
  font-size: 13px;
  line-height: 1.5;
  tab-size: 4;
}

</style>
