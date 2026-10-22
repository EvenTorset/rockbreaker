<script setup lang="ts">
import { computed, onBeforeUnmount, provide, ref, shallowRef, watch } from 'vue'
import { useRouter } from 'vue-router'
import { NButton, NCheckbox, NIcon, NInput, NSpin, type InputInst } from 'naive-ui'
import {
  ArrowDownload16Filled,
  Document24Filled,
  Folder24Filled,
  FolderZip16Filled,
  Search16Filled,
} from '@vicons/fluent'
import { searchDirectory, type DirectoryEntry, type ItemDetail, type SearchResult } from '@/api'
import { escapeRegExp } from '@/util/path'
import { downloadUrl, downloadWithPost } from '@/util/download'
import { formatBytes, formatDateTime } from '@/util/format'
import { naturalCompare } from '@/util/sort'
import { itemLocation, rawUrl } from '@/util/urls'
import Dim from '@/components/Dim.vue'
import FilePath from '@/components/FilePath.vue'
import IconButton from '@/components/IconButton.vue'
import PathBar from '@/components/PathBar.vue'
import RenameHostingButton from '@/components/RenameHostingButton.vue'
import Row from '@/components/Row.vue'
import StopHostingButton from '@/components/StopHostingButton.vue'
import Tooltip from '@/components/Tooltip.vue'
import SelectableTable from '@/components/table/SelectableTable.vue'
import SelectionBar from '@/components/table/SelectionBar.vue'
import type { SortState, TableColumn } from '@/components/table/types'

const props = defineProps<{
  item: ItemDetail
  loading?: boolean
}>()

const emit = defineEmits<{
  stopped: []
  renamed: [newId: string]
}>()

const router = useRouter()

const isSharedRoot = computed(() => props.item.is_host && props.item.subpath === '')

const columns: TableColumn[] = [
  { key: 'name', title: 'Name', width: 'minmax(8rem, 1fr)', sortable: true },
  { key: 'size', title: 'Size', width: '110px', sortable: true },
  { key: 'modified', title: 'Modified', width: '200px', sortable: true, hideBelow: 640 },
  { key: 'actions', title: 'Action', width: '84px', align: 'center' },
]

const ROW_HEIGHT = 45
const FOLDER_COLOR = '#f2cc9e'
const FILE_COLOR = 'var(--color-6)'

const sort = ref<SortState>({ key: 'name', direction: 'asc' })
const selected = ref<ReadonlySet<string>>(new Set())

watch(() => props.item, () => {
  selected.value = new Set()
  query.value = ''
})

// MARK: Search

const SEARCH_DELAY_MS = 250

const query = ref('')
const recursive = ref(false)
const results = shallowRef<SearchResult | null>(null)
const searching = ref(false)
const searchError = ref<string | null>(null)

const term = computed(() => query.value.trim())
const isSearchingBelow = computed(() => recursive.value && term.value !== '')

provide('path-highlight', computed(() => {
  return term.value === '' ? null : new RegExp(`${escapeRegExp(term.value)}(?![^/\\\\]*[/\\\\])`, 'gi')
}))

let searchTimer: ReturnType<typeof setTimeout> | undefined
let searchController: AbortController | undefined

function cancelSearch() {
  clearTimeout(searchTimer)
  searchController?.abort()
  searchController = undefined
}

watch([term, recursive, () => props.item], () => {
  cancelSearch()
  results.value = null
  searchError.value = null
  searching.value = isSearchingBelow.value
  if (!isSearchingBelow.value) return

  const subpath = props.item.subpath
  const searched = term.value

  searchTimer = setTimeout(async () => {
    const controller = new AbortController()
    searchController = controller

    try {
      const found = await searchDirectory(props.item.id, subpath, searched, controller.signal)
      if (!controller.signal.aborted) results.value = found
    } catch (error) {
      if (!controller.signal.aborted) searchError.value = error instanceof Error ? error.message : 'Searching failed.'
    } finally {
      if (!controller.signal.aborted) searching.value = false
    }
  }, SEARCH_DELAY_MS)
})

onBeforeUnmount(cancelSearch)

function matchesTerm(entry: DirectoryEntry) {
  return entry.name.toLowerCase().includes(term.value.toLowerCase())
}

function relativePath(entry: DirectoryEntry) {
  const folder = props.item.subpath
  return folder !== '' && entry.subpath.startsWith(`${folder}/`) ? entry.subpath.slice(folder.length + 1) : entry.subpath
}

const shownEntries = computed(() => {
  const entries = props.item.entries ?? []
  if (term.value === '') return entries
  if (isSearchingBelow.value) return results.value?.entries ?? entries.filter(matchesTerm)
  return entries.filter(matchesTerm)
})

const emptyMessage = computed(() => {
  if (term.value === '') return 'This folder is empty.'
  return recursive.value ? `Nothing in this folder or below it matches "${term.value}".` : `Nothing in this folder matches "${term.value}".`
})

function compareEntries(a: DirectoryEntry, b: DirectoryEntry, { key, direction }: SortState) {
  if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1

  const sign = direction === 'asc' ? 1 : -1
  if (key === 'name') return sign * (naturalCompare(a.name, b.name) || naturalCompare(a.subpath, b.subpath))

  const difference = key === 'size'
    ? (a.size ?? 0) - (b.size ?? 0)
    : (a.modified ?? 0) - (b.modified ?? 0)

  return difference !== 0 ? sign * difference : naturalCompare(a.name, b.name)
}

const sortedEntries = computed(() => {
  return shownEntries.value.toSorted((a, b) => compareEntries(a, b, sort.value))
})

const selectedEntries = computed(() => sortedEntries.value.filter(entry => selected.value.has(entry.subpath)))
const singleFileSelected = computed(() => selectedEntries.value.length === 1 && !selectedEntries.value[0].is_dir)

const busy = computed(() => props.loading || searching.value)

function onSearchKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') query.value = ''
}

function open(entry: DirectoryEntry) {
  router.push(itemLocation(props.item.id, entry.subpath))
}

function downloadSelected() {
  const entries = selectedEntries.value

  if (entries.length === 1) {
    downloadUrl(rawUrl(props.item.id, entries[0].subpath))
  } else if (entries.length > 1) {
    downloadWithPost(`/!/api/zip/${encodeURIComponent(props.item.id)}`, entries.map(entry => ['path', entry.subpath]))
  }
}

const searchInput = ref<InputInst | null>(null)

function onKeydown(event: KeyboardEvent) {
  if (
    (event.ctrlKey || event.metaKey)
    && event.key.toLowerCase() === 'f'
    && searchInput.value?.inputElRef !== document.activeElement
  ) {
    event.preventDefault()
    searchInput.value?.select()
  }
}

function onSearchInputMounted() {
  window.addEventListener('keydown', onKeydown)
}

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeydown)
})
</script>

<template>
  <div class="directory-view">
    <Row justify="space-between" wrap gap="12px">
      <PathBar :item class="path" />
      <Row>
        <Tooltip v-if="item.zip_url">
          <template #trigger="{ props }">
            <IconButton v-bind="props" tag="a" class="accent" :href="item.zip_url" download>
              <FolderZip16Filled />
            </IconButton>
          </template>
          Download as ZIP
        </Tooltip>
        <RenameHostingButton v-if="isSharedRoot" :host-id="item.id" @renamed="emit('renamed', $event)" />
        <StopHostingButton v-if="isSharedRoot" :host-id="item.id" @stopped="emit('stopped')" />
      </Row>
    </Row>

    <Row class="search" data-no-marquee gap="16px" wrap>
      <NInput
        ref="searchInput"
        v-model:value="query"
        class="search-input"
        clearable
        placeholder="Search this folder..."
        :input-props="{ 'aria-label': 'Search this folder', spellcheck: false }"
        @keydown="onSearchKeydown"
        @vnode-mounted="onSearchInputMounted"
      >
        <template #prefix>
          <NIcon :component="Search16Filled" />
        </template>
      </NInput>
      <NCheckbox v-model:checked="recursive">Include subfolders</NCheckbox>
    </Row>

    <Dim v-if="searchError" tag="p" class="search-note">{{ searchError }}</Dim>
    <Dim v-else-if="isSearchingBelow && results?.truncated" tag="p" class="search-note">
      Only the first {{ results.entries.length }} matches are shown. Search for something more specific to see the rest.
    </Dim>

    <div class="listing" :class="{ loading: busy }" :aria-busy="busy" :inert="loading">
      <Transition name="spinner">
        <div v-if="busy" class="spinner">
          <NSpin :size="22" />
        </div>
      </Transition>

      <SelectableTable
        v-model:selected="selected"
        v-model:sort="sort"
        :rows="sortedEntries"
        :row-key="entry => entry.subpath"
        :columns
        :row-height="ROW_HEIGHT"
        @open="open"
      >
        <template #cell-name="{ row }">
          <a
            class="entry"
            :class="{ folder: row.is_dir }"
            :href="itemLocation(item.id, row.subpath)"
            draggable="false"
          >
            <NIcon
              :component="row.is_dir ? Folder24Filled : Document24Filled"
              :color="row.is_dir ? FOLDER_COLOR : FILE_COLOR"
            />
            <FilePath :path="isSearchingBelow ? relativePath(row) : row.name" :no-extension="row.is_dir" />
          </a>
        </template>

        <template #cell-size="{ row }">
          <Dim>{{ formatBytes(row.size) }}</Dim>
        </template>

        <template #cell-modified="{ row }">
          <Dim>{{ formatDateTime(row.modified) }}</Dim>
        </template>

        <template #cell-actions="{ row }">
          <Tooltip>
            <template #trigger="{ props }">
              <IconButton
                v-bind="props"
                tag="a"
                :class="{ accent: row.is_dir }"
                :href="rawUrl(item.id, row.subpath)"
                download
              >
                <FolderZip16Filled v-if="row.is_dir" />
                <ArrowDownload16Filled v-else />
              </IconButton>
            </template>
            {{ row.is_dir ? 'Download as ZIP' : 'Download' }}
          </Tooltip>
        </template>

        <template #empty>
          <template v-if="!busy">{{ emptyMessage }}</template>
        </template>
      </SelectableTable>
    </div>
  </div>

  <SelectionBar :count="selected.size" @deselect="selected = new Set()">
    <NButton class="accent" size="small" @click="downloadSelected">
      <template #icon>
        <NIcon :component="singleFileSelected ? ArrowDownload16Filled : FolderZip16Filled" />
      </template>
      {{ singleFileSelected ? 'Download' : 'Download as ZIP' }}
    </NButton>
  </SelectionBar>
</template>

<style lang="scss" scoped>

.directory-view {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.path {
  flex: 1;
  min-width: 0;
}

.search-input {
  flex: 1;
  min-width: 200px;
}

.search-note {
  margin: -8px 0 0;
  font-size: 0.9rem;
}

.listing {
  position: relative;

  &.loading :deep(.selectable-table) {
    opacity: 0.55;
    pointer-events: none;
    transition: opacity 0.2s ease 0.12s;
  }

  :deep(.selectable-table) {
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

.entry {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  max-width: 100%;
  color: inherit;
  text-decoration: none;
  cursor: default;

  &.folder {
    font-weight: 600;

    :deep(.file-path) {
      font-weight: inherit;
    }
  }

  :deep(.n-icon) {
    flex: none;
  }
}

</style>
