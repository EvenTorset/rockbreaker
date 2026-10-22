<script setup lang="ts">
import { computed, defineAsyncComponent, onBeforeUnmount, shallowRef, watch } from 'vue'
import { NButton, NCard, NIcon, NSpin } from 'naive-ui'
import { ArrowDownload16Filled, Document24Regular } from '@vicons/fluent'
import type { ItemDetail } from '@/api'
import { formatBytes, formatDateTime } from '@/util/format'
import { createViewerFile } from '@/viewers/reader'
import { findViewer } from '@/viewers/registry'
import type { Viewer, ViewerFile } from '@/viewers/types'
import Col from './Col.vue'
import Dim from './Dim.vue'
import FilePath from './FilePath.vue'
import IconButton from './IconButton.vue'
import PathBar from './PathBar.vue'
import RenameHostingButton from './RenameHostingButton.vue'
import Row from './Row.vue'
import StopHostingButton from './StopHostingButton.vue'
import Tooltip from './Tooltip.vue'

const props = defineProps<{
  item: ItemDetail
}>()

const emit = defineEmits<{
  stopped: []
  renamed: [newId: string]
}>()

type Resolution =
  | { status: 'pending' }
  | { status: 'none' }
  | { status: 'viewer', viewer: Viewer, file: ViewerFile }

const resolution = shallowRef<Resolution>({ status: 'pending' })
const extraDetails = shallowRef<string[]>([])

let latest = 0
let controller: AbortController | undefined

watch(() => props.item.download_url, async () => {
  const call = ++latest
  controller?.abort()
  controller = new AbortController()
  resolution.value = { status: 'pending' }
  extraDetails.value = []

  const file = createViewerFile(props.item, controller.signal)
  const viewer = await findViewer(file)
  if (call !== latest) return

  resolution.value = viewer ? { status: 'viewer', viewer, file } : { status: 'none' }

  void Promise.resolve(viewer?.metadata?.(file)).then((details) => {
    if (call === latest && details) extraDetails.value = details
  }).catch(() => {})
}, { immediate: true })

onBeforeUnmount(() => {
  latest++
  controller?.abort()
})

const preview = computed(() => {
  const current = resolution.value
  return current.status === 'viewer' ? defineAsyncComponent(current.viewer.load) : null
})

const isSharedRoot = computed(() => props.item.is_host && props.item.subpath === '')
const isNested = computed(() => props.item.kind !== 'file' || props.item.subpath !== '')

const details = computed(() => {
  return [
    formatBytes(props.item.file_size, 'Unknown size'),
    props.item.modified !== null ? `Modified ${formatDateTime(props.item.modified)}` : null,
    ...extraDetails.value,
  ].filter(Boolean).join(' · ')
})
</script>

<template>
  <Col align="stretch" gap="16px">
    <PathBar v-if="isNested" :item />

    <div v-if="resolution.status === 'pending'" class="pending">
      <NSpin :size="20" />
    </div>

    <template v-else-if="resolution.status === 'viewer'">
      <Row class="file-bar" justify="space-between" wrap gap="12px">
        <Row gap="12px" class="file-info">
          <NIcon :size="32" :component="Document24Regular" />
          <Col align="flex-start" gap="2px" class="file-text">
            <h2 class="file-name"><FilePath :path="item.file_name" /></h2>
            <Dim>{{ details }}</Dim>
          </Col>
        </Row>
        <Row>
          <Tooltip>
            <template #trigger="{ props: triggerProps }">
              <IconButton v-bind="triggerProps" tag="a" class="accent" :href="item.download_url" download>
                <ArrowDownload16Filled />
              </IconButton>
            </template>
            Download
          </Tooltip>
          <RenameHostingButton v-if="isSharedRoot" :host-id="item.id" @renamed="emit('renamed', $event)" />
          <StopHostingButton v-if="isSharedRoot" :host-id="item.id" @stopped="emit('stopped')" />
        </Row>
      </Row>

      <div class="preview">
        <component :is="preview" :key="item.download_url" :file="resolution.file" />
      </div>
    </template>

    <NCard v-else class="file-view" title="File">
      <template #header-extra>
        <Row v-if="isSharedRoot">
          <RenameHostingButton :host-id="item.id" @renamed="emit('renamed', $event)" />
          <StopHostingButton :host-id="item.id" @stopped="emit('stopped')" />
        </Row>
      </template>
      <Col gap="4px" class="card-body">
        <NIcon :size="40" :component="Document24Regular" />
        <h2 class="file-name"><FilePath :path="item.file_name" /></h2>
        <Dim>{{ details }}</Dim>
        <Row justify="center" gap="8px" wrap class="actions">
          <NButton class="accent" size="large" tag="a" :href="item.download_url" download>
            Download
          </NButton>
        </Row>
      </Col>
    </NCard>
  </Col>
</template>

<style lang="scss" scoped>

.pending {
  display: flex;
  justify-content: center;
  padding: 48px 0;
}

.file-info,
.file-text {
  min-width: 0;
}

.file-name {
  margin: 0;
  min-width: 0;
  max-width: 100%;
  font-size: 1.3rem;
  font-weight: 500;
}

.preview {
  overflow: hidden;
  background-color: var(--color-0-alt);
  border: 1px solid var(--color-2);
  border-radius: 6px;
}

.file-view {
  width: 100%;
  max-width: 500px;
  margin: 1rem auto;
}

.card-body {
  min-width: 0;
  text-align: center;
}

.actions {
  margin-top: 16px;
}

</style>
