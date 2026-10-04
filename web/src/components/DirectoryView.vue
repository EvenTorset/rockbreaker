<script setup lang="ts">
import { NIcon, NSpin, NTable } from 'naive-ui'
import { Document24Filled, Folder24Filled } from '@vicons/fluent'
import { RouterLink } from 'vue-router'
import type { ItemDetail } from '@/api'
import { itemLocation, rawUrl } from '@/util/urls'
import Col from '@/components/Col.vue'
import Dim from '@/components/Dim.vue'
import Row from '@/components/Row.vue'
import RenameHostingButton from '@/components/RenameHostingButton.vue'
import StopHostingButton from '@/components/StopHostingButton.vue'
import { computed } from 'vue'
import { naturalCompare } from '@/util/sort.ts'
import FilePath from '@/components/FilePath.vue'
import IconButton from '@/components/IconButton.vue'
import Tooltip from '@/components/Tooltip.vue'
import { ArrowDownload16Filled, FolderZip16Filled } from '@vicons/fluent'

const props = defineProps<{
  item: ItemDetail
  loading?: boolean
}>()

const emit = defineEmits<{
  stopped: []
  renamed: [newId: string]
}>()

const sortedEntries = computed(() => {
  if (!props.item.entries) return []

  return props.item.entries.toSorted((a, b) => {
    if (a.is_dir !== b.is_dir) {
      return a.is_dir ? -1 : 1
    }

    return naturalCompare(a.name, b.name)
  })
})
</script>

<template>
  <Col align="stretch" gap="16px">
    <Row justify="space-between" wrap gap="12px">
      <Col align="flex-start" gap="2px" class="heading">
        <h2>Index of /{{ item.subpath }}</h2>
        <Dim v-if="item.kind === 'collection'">Shared items: {{ item.item_count }}</Dim>
        <Dim v-else>Shared folder: {{ item.root_name }}</Dim>
      </Col>
      <Row>
        <Tooltip v-if="item.zip_url">
          <template #trigger="{ props }">
            <IconButton v-bind="props" tag="a" class="accent" :href="item.zip_url" download>
              <FolderZip16Filled />
            </IconButton>
          </template>
          Download as ZIP
        </Tooltip>
        <RenameHostingButton v-if="item.is_host" :host-id="item.id" @renamed="emit('renamed', $event)" />
        <StopHostingButton v-if="item.is_host" :host-id="item.id" @stopped="emit('stopped')" />
      </Row>
    </Row>

    <div class="listing" :class="{ loading }" :aria-busy="loading" :inert="loading">
      <Transition name="spinner">
        <div v-if="loading" class="spinner">
          <NSpin :size="22" />
        </div>
      </Transition>

      <NTable :single-line="false" size="small">
        <colgroup>
          <col span="1" style="width: 100%;">
          <col span="1" style="width: fit-content;">
          <col span="1" style="width: fit-content;">
        </colgroup>
        <thead>
          <tr>
            <th>Name</th>
            <th>Size</th>
            <th>Action</th>
          </tr>
        </thead>
        <tbody>
          <tr v-if="item.parent_subpath !== null">
            <td colspan="3">
              <RouterLink :to="itemLocation(item.id, item.parent_subpath)" class="entry folder-link">
                <NIcon :component="Folder24Filled" />
                .. (Parent Directory)
              </RouterLink>
            </td>
          </tr>

          <tr v-for="entry in sortedEntries" :key="entry.name">
            <td>
              <RouterLink
                v-if="entry.is_dir"
                :to="itemLocation(item.id, entry.subpath)"
                class="entry folder-link"
              >
                <NIcon :component="Folder24Filled" />
                {{ entry.name }}
              </RouterLink>
              <span v-else class="entry">
                <NIcon :component="Document24Filled" />
                <FilePath :path="entry.name" />
              </span>
            </td>
            <td style="white-space: nowrap;"><Dim>{{ entry.size }}</Dim></td>
            <td>
              <Row justify="safe center">
                <Tooltip>
                  <template #trigger="{ props }">
                    <IconButton v-bind="props" tag="a" :class="{ accent: entry.is_dir }" :href="rawUrl(item.id, entry.subpath)" download>
                      <FolderZip16Filled v-if="entry.is_dir" />
                      <ArrowDownload16Filled v-else />
                    </IconButton>
                  </template>
                  {{ entry.is_dir ? 'Download as ZIP' : 'Download' }}
                </Tooltip>
              </Row>
            </td>
          </tr>
        </tbody>
      </NTable>
    </div>
  </Col>
</template>

<style lang="scss" scoped>

.listing {
  position: relative;

  &.loading :deep(.n-table) {
    opacity: 0.55;
    pointer-events: none;
    transition: opacity 0.2s ease 0.12s;
  }

  :deep(.n-table) {
    transition: opacity 0.15s ease;
  }
}

.spinner {
  position: absolute;
  top: 12px;
  left: 0;
  right: 0;
  z-index: 1;
  display: flex;
  justify-content: center;
  pointer-events: none;
}

.spinner-enter-active {
  transition: opacity 0.2s ease 0.12s;
}

.spinner-leave-active {
  transition: opacity 0.15s ease;
}

.spinner-enter-from,
.spinner-leave-to {
  opacity: 0;
}

h1 {
  font-size: 1.4rem;
  word-break: break-all;
}

.heading {
  min-width: 0;
}

.entry {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.folder-link {
  color: var(--color-accent);
  text-decoration: none;
  font-weight: 600;

  &:hover {
    text-decoration: underline;
  }
}

</style>
