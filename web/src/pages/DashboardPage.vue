<script setup lang="ts">
import { computed, ref } from 'vue'
import { NButton, NCard, NIcon, NPopconfirm, NTabPane, NTabs } from 'naive-ui'
import { fetchDashboard, revealHosting, stopHosting, type DashboardItem } from '@/api'
import { useLoader } from '@/composables/useLoader'
import Notify from '@/notify'
import { copyText } from '@/util/clipboard'
import { formatDateTime } from '@/util/format'
import { naturalCompare } from '@/util/sort'
import { itemLocation } from '@/util/urls'
import Col from '@/components/Col.vue'
import Dim from '@/components/Dim.vue'
import FilePath from '@/components/FilePath.vue'
import PageState from '@/components/PageState.vue'
import RenameHostingButton from '@/components/RenameHostingButton.vue'
import Row from '@/components/Row.vue'
import SettingsTab from '@/components/SettingsTab.vue'
import StopHostingButton from '@/components/StopHostingButton.vue'
import SelectableTable from '@/components/table/SelectableTable.vue'
import SelectionBar from '@/components/table/SelectionBar.vue'
import type { SortState, TableColumn } from '@/components/table/types'
import { Dismiss16Filled, FolderOpen16Filled, Link16Filled, Share16Filled } from '@vicons/fluent'
import Tooltip from '@/components/Tooltip.vue'
import IconButton from '@/components/IconButton.vue'

const { data, loading, error, reload } = useLoader(fetchDashboard)
const items = computed(() => data.value ?? [])

const ROW_HEIGHT = 64

const columns: TableColumn[] = [
  { key: 'id', title: 'Shared Item', width: 'minmax(8rem, 1fr)', sortable: true },
  { key: 'type', title: 'Type', width: '110px', sortable: true, hideBelow: 520 },
  { key: 'shared', title: 'Shared', width: '200px', sortable: true, hideBelow: 760 },
  { key: 'links', title: 'Links', width: '96px' },
  { key: 'actions', title: 'Action', width: '140px' },
]

const sort = ref<SortState>({ key: 'shared', direction: 'asc' })
const selected = ref<ReadonlySet<string>>(new Set())

function typeLabel(item: DashboardItem) {
  if (item.kind === 'file') return 'File'
  if (item.kind === 'directory') return 'Directory'
  return `${item.paths.length} items`
}

const sortedItems = computed(() => {
  const { key, direction } = sort.value
  const sign = direction === 'asc' ? 1 : -1

  return items.value
    .map((item, index) => ({ item, index }))
    .toSorted((a, b) => {
      let difference = 0
      if (key === 'id') difference = naturalCompare(a.item.id, b.item.id)
      else if (key === 'type') difference = naturalCompare(typeLabel(a.item), typeLabel(b.item))
      else if (key === 'shared') difference = (a.item.shared_at ?? 0) - (b.item.shared_at ?? 0)

      return difference !== 0 ? sign * difference : sign * (a.index - b.index)
    })
    .map(({ item }) => item)
})

function canReveal(item: DashboardItem) {
  return item.kind !== 'collection'
}

async function reveal(item: DashboardItem) {
  try {
    await revealHosting(item.id)
  } catch (error) {
    Notify.error(error instanceof Error ? error.message : 'Failed to show the item in the file explorer.')
  }
}

function collectionSummary(item: DashboardItem) {
  return item.paths.map(path => path.split(/[\\/]/).filter(Boolean).pop() ?? path).join(', ')
}

function pageLink(item: DashboardItem) {
  return import.meta.env.DEV ? itemLocation(item.id, '') : item.share_url
}

function openItem(item: DashboardItem) {
  window.open(pageLink(item), '_blank', 'noopener,noreferrer')
}

async function stopSelected() {
  const ids = [...selected.value]
  const results = await Promise.allSettled(ids.map(stopHosting))
  const failed = ids.filter((_, index) => results[index].status === 'rejected')

  if (failed.length === 0) {
    Notify.success({ content: `Stopped sharing ${ids.length} ${ids.length === 1 ? 'item' : 'items'}.`, duration: 3000 })
  } else {
    Notify.error(`Failed to stop sharing ${failed.length} of ${ids.length} items.`)
  }

  selected.value = new Set(failed)
  reload()
}

reload()
</script>

<template>
  <NTabs type="line" animated default-value="items" @update:value="tab => tab === 'items' && reload()">
    <NTabPane name="items" tab="Hosted Items">
      <PageState :loading="loading && !data" :error>
        <Col align="stretch" gap="16px">
          <Dim tag="p">Manage local paths currently being served.</Dim>

          <NCard v-if="items.length === 0">
            No active shared items found on host daemon.
          </NCard>

          <SelectableTable
            v-else
            v-model:selected="selected"
            v-model:sort="sort"
            :rows="sortedItems"
            :row-key="item => item.id"
            :columns
            :row-height="ROW_HEIGHT"
            @open="openItem"
          >
            <template #cell-id="{ row: item }">
              <Col align="flex-start" gap="2px" class="name-cell">
                <a
                  class="host-id"
                  :href="pageLink(item)"
                  target="_blank"
                  rel="noopener noreferrer"
                  draggable="false"
                >{{ item.id }}</a>
                <FilePath v-if="item.paths.length === 1" :path="item.paths[0]" />
                <Tooltip v-else>
                  <template #trigger="{ props }">
                    <Dim v-bind="props" class="collection-summary">{{ collectionSummary(item) }}</Dim>
                  </template>
                  <ul class="path-list">
                    <li v-for="path in item.paths" :key="path">
                      <FilePath :path />
                    </li>
                  </ul>
                </Tooltip>
              </Col>
            </template>

            <template #cell-type="{ row: item }">
              {{ typeLabel(item) }}
            </template>

            <template #cell-shared="{ row: item }">
              <Dim>{{ formatDateTime(item.shared_at, 'Unknown') }}</Dim>
            </template>

            <template #cell-links="{ row: item }">
              <Row>
                <Tooltip>
                  <template #trigger="{ props }">
                    <IconButton v-bind="props" @click="copyText(item.share_url, 'Share URL')">
                      <Share16Filled />
                    </IconButton>
                  </template>
                  Copy share link
                </Tooltip>
                <Tooltip>
                  <template #trigger="{ props }">
                    <IconButton v-bind="props" @click="copyText(item.direct_url, 'Direct URL')">
                      <Link16Filled />
                    </IconButton>
                  </template>
                  Copy direct link
                </Tooltip>
              </Row>
            </template>

            <template #cell-actions="{ row: item }">
              <Row>
                <Tooltip v-if="canReveal(item)">
                  <template #trigger="{ props }">
                    <IconButton v-bind="props" @click="reveal(item)">
                      <FolderOpen16Filled />
                    </IconButton>
                  </template>
                  Reveal in file explorer
                </Tooltip>
                <RenameHostingButton :host-id="item.id" size="small" @renamed="reload" />
                <StopHostingButton :host-id="item.id" size="small" @stopped="reload" />
              </Row>
            </template>
          </SelectableTable>
        </Col>
      </PageState>
    </NTabPane>

    <NTabPane name="settings" tab="Settings" display-directive="show:lazy">
      <SettingsTab @url-base-changed="reload" />
    </NTabPane>
  </NTabs>

  <SelectionBar :count="selected.size" @deselect="selected = new Set()">
    <NPopconfirm
      placement="top"
      positive-text="Stop sharing"
      negative-text="Cancel"
      @positive-click="stopSelected"
    >
      <template #trigger>
        <NButton class="danger" size="small">
          <template #icon>
            <NIcon :component="Dismiss16Filled" />
          </template>
          Stop sharing
        </NButton>
      </template>
      Are you sure you want to stop sharing {{ selected.size === 1 ? 'this item' : `these ${selected.size} items` }}?
    </NPopconfirm>
  </SelectionBar>
</template>

<style lang="scss" scoped>

.name-cell {
  min-width: 0;
  max-width: 100%;
}

.host-id {
  font-weight: 700;
  color: var(--color-accent);
  text-decoration: none;
  cursor: default;
}

.path-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin: 0;
  padding: 0;
  list-style: none;
  font-size: 13px;

  li {
    display: flex;
    max-width: min(36rem, calc(100dvw - 80px));
  }
}

.collection-summary {
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

</style>
