<script setup lang="ts" generic="T">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { NIcon } from 'naive-ui'
import { useWindowVirtualizer } from '@tanstack/vue-virtual'
import { ArrowDown16Filled, ArrowUp16Filled } from '@vicons/fluent'
import type { SortState, TableColumn } from './types'
import { IGNORED_TARGETS, useMarqueeSelection, type MarqueeRect } from './useMarqueeSelection'

const props = defineProps<{
  rows: readonly T[]
  rowKey: (row: T) => string
  columns: readonly TableColumn[]
  rowHeight: number
}>()

const emit = defineEmits<{
  open: [row: T]
}>()

defineSlots<{
  [cell: `cell-${string}`]: (props: { row: T, index: number }) => unknown
  empty?: () => unknown
}>()

const selected = defineModel<ReadonlySet<string>>('selected', { default: () => new Set<string>() })
const sort = defineModel<SortState | undefined>('sort')

const INTERACTIVE = 'button, input, select, textarea, label, .n-button, [data-no-select]'
const LONG_PRESS_MS = 450
const LONG_PRESS_SLOP = 8

const root = ref<HTMLElement>()
const body = ref<HTMLElement>()

const entries = computed(() => props.rows.map((row, index) => ({ row, index, key: props.rowKey(row) })))
const keys = computed(() => entries.value.map(entry => entry.key))
const indexOfKey = computed(() => new Map(keys.value.map((key, index) => [key, index])))

const scrollMargin = ref(0)

const virtualizer = useWindowVirtualizer(computed(() => ({
  count: props.rows.length,
  estimateSize: () => props.rowHeight,
  overscan: 8,
  scrollMargin: scrollMargin.value,
})))

const visibleRows = computed(() => {
  return virtualizer.value.getVirtualItems()
    .filter(item => item.index < entries.value.length)
    .map(item => ({ ...entries.value[item.index], offset: item.start - scrollMargin.value }))
})

const totalHeight = computed(() => props.rows.length * props.rowHeight)

function measure() {
  if (body.value) scrollMargin.value = body.value.getBoundingClientRect().top + window.scrollY
}

let resizeObserver: ResizeObserver | undefined

const viewportWidth = ref(window.innerWidth)
const visibleColumns = computed(() => {
  return props.columns.filter(column => column.hideBelow === undefined || viewportWidth.value >= column.hideBelow)
})

const template = computed(() => visibleColumns.value.map(column => column.width ?? 'auto').join(' '))

function onResize() {
  viewportWidth.value = window.innerWidth
  measure()
}

watch(() => props.rows.length, () => nextTick(measure))

const anchor = ref<string | null>(null)

watch(keys, (current) => {
  const present = new Set(current)
  if ([...selected.value].some(key => !present.has(key))) {
    selected.value = new Set([...selected.value].filter(key => present.has(key)))
  }
  if (anchor.value !== null && !present.has(anchor.value)) anchor.value = null
})

function isInteractive(target: EventTarget | null) {
  return target instanceof Element && target.closest(INTERACTIVE) !== null
}

function rowAt(event: Event) {
  const target = event.target
  if (!(target instanceof Element)) return null

  const el = target.closest<HTMLElement>('[data-row-key]')
  if (!el || !root.value?.contains(el)) return null

  const index = indexOfKey.value.get(el.dataset.rowKey ?? '')
  return index === undefined ? null : entries.value[index]
}

function toggle(key: string) {
  const next = new Set(selected.value)
  if (!next.delete(key)) next.add(key)
  selected.value = next
  anchor.value = key
}

function selectWithModifiers(event: MouseEvent, key: string, index: number) {
  const ctrl = event.ctrlKey || event.metaKey
  const anchorIndex = anchor.value === null ? -1 : indexOfKey.value.get(anchor.value) ?? -1

  if (event.shiftKey && anchorIndex !== -1) {
    const range = keys.value.slice(Math.min(anchorIndex, index), Math.max(anchorIndex, index) + 1)

    if (ctrl) {
      const next = new Set(selected.value)
      const select = next.has(anchor.value!)
      for (const rangeKey of range) {
        if (select) next.add(rangeKey)
        else next.delete(rangeKey)
      }
      selected.value = next
    } else {
      selected.value = new Set(range)
    }
    return
  }

  if (ctrl) {
    toggle(key)
  } else {
    selected.value = new Set([key])
    anchor.value = key
  }
}

let pointerType = 'mouse'
let longPressed = false
let longPressTimer: ReturnType<typeof setTimeout> | undefined
let longPressOrigin = { x: 0, y: 0 }

function cancelLongPress() {
  clearTimeout(longPressTimer)
  longPressTimer = undefined
}

function onPointerDown(event: PointerEvent) {
  pointerType = event.pointerType
  longPressed = false
  cancelLongPress()

  if (event.pointerType !== 'touch' || isInteractive(event.target)) return

  const entry = rowAt(event)
  if (!entry) return

  longPressOrigin = { x: event.clientX, y: event.clientY }
  longPressTimer = setTimeout(() => {
    longPressed = true
    toggle(entry.key)
    navigator.vibrate?.(10)
  }, LONG_PRESS_MS)
}

function onPointerMove(event: PointerEvent) {
  if (longPressTimer === undefined) return
  if (Math.hypot(event.clientX - longPressOrigin.x, event.clientY - longPressOrigin.y) > LONG_PRESS_SLOP) {
    cancelLongPress()
  }
}

function onClick(event: MouseEvent) {
  const entry = rowAt(event)
  if (!entry || isInteractive(event.target)) return

  event.preventDefault()

  if (longPressed) {
    longPressed = false
    return
  }

  if (event.detail === 0) return emit('open', entry.row)

  if (pointerType === 'touch') {
    if (selected.value.size > 0) toggle(entry.key)
    else emit('open', entry.row)
    return
  }

  selectWithModifiers(event, entry.key, entry.index)
}

function onDoubleClick(event: MouseEvent) {
  const entry = rowAt(event)
  if (!entry || isInteractive(event.target) || pointerType === 'touch') return
  if (event.ctrlKey || event.metaKey || event.shiftKey) return

  emit('open', entry.row)
}

function onContextMenu(event: MouseEvent) {
  if (pointerType === 'touch' && rowAt(event)) event.preventDefault()
}

function onHeaderClick(column: TableColumn) {
  if (!column.sortable) return

  const current = sort.value
  sort.value = current?.key === column.key
    ? { key: column.key, direction: current.direction === 'asc' ? 'desc' : 'asc' }
    : { key: column.key, direction: 'asc' }
}

function ariaSort(column: TableColumn) {
  if (!column.sortable) return undefined
  if (sort.value?.key !== column.key) return 'none'
  return sort.value.direction === 'asc' ? 'ascending' : 'descending'
}

function onWindowClick(event: MouseEvent) {
  if (selected.value.size === 0 || event.button !== 0) return
  if (event.ctrlKey || event.metaKey || event.shiftKey) return

  const target = event.target
  if (!(target instanceof Element) || !target.isConnected) return
  if (target.closest(`[data-row-key], .header-cell.sortable, ${IGNORED_TARGETS}`)) return

  selected.value = new Set()
}

function onKeyDown(event: KeyboardEvent) {
  if (!(event.ctrlKey || event.metaKey) || event.shiftKey || event.altKey || event.key.toLowerCase() !== 'a') return

  const target = event.target
  if (target instanceof HTMLElement && (target.isContentEditable || target.closest('input, textarea, select'))) return
  if (document.querySelector('.n-modal-container')) return

  event.preventDefault()
  selected.value = new Set(keys.value)
}

onMounted(() => {
  window.addEventListener('keydown', onKeyDown)
  window.addEventListener('click', onWindowClick)
  window.addEventListener('resize', onResize)

  measure()
  resizeObserver = new ResizeObserver(measure)
  resizeObserver.observe(document.body)
})
onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeyDown)
  window.removeEventListener('click', onWindowClick)
  window.removeEventListener('resize', onResize)
  resizeObserver?.disconnect()
  cancelLongPress()
})

function hitTest(box: MarqueeRect) {
  const area = body.value?.getBoundingClientRect()
  if (!area || keys.value.length === 0) return []
  if (box.left >= area.right || box.left + box.width <= area.left) return []

  const first = Math.max(0, Math.floor((box.top - area.top) / props.rowHeight))
  const last = Math.min(keys.value.length - 1, Math.ceil((box.top + box.height - area.top) / props.rowHeight) - 1)

  if (last < first) return []
  return keys.value.slice(first, last + 1)
}

const { rect } = useMarqueeSelection({
  hitTest,
  selected: () => selected.value,
  select: (keys) => {
    selected.value = keys
  },
  enabled: () => props.rows.length > 0,
})
</script>

<template>
  <div
    ref="root"
    class="selectable-table"
    role="grid"
    aria-multiselectable="true"
    :aria-rowcount="rows.length + 1"
    :style="{ '--table-columns': template }"
    @pointerdown="onPointerDown"
    @pointermove="onPointerMove"
    @pointerup="cancelLongPress"
    @pointercancel="cancelLongPress"
    @click="onClick"
    @dblclick="onDoubleClick"
    @contextmenu="onContextMenu"
  >
    <div class="head" role="row">
      <div
        v-for="column in visibleColumns"
        :key="column.key"
        class="cell header-cell"
        :class="[`align-${column.align ?? 'start'}`, { sortable: column.sortable }]"
        role="columnheader"
        :aria-sort="ariaSort(column)"
        :tabindex="column.sortable ? 0 : undefined"
        @click="onHeaderClick(column)"
        @keydown.enter.prevent="onHeaderClick(column)"
        @keydown.space.prevent="onHeaderClick(column)"
      >
        <span>{{ column.title }}</span>
        <NIcon
          v-if="sort?.key === column.key"
          class="sort-icon"
          :size="14"
          :component="sort.direction === 'asc' ? ArrowUp16Filled : ArrowDown16Filled"
        />
      </div>
    </div>

    <div ref="body" class="body" :style="{ height: `${totalHeight}px` }">
      <div
        v-for="entry in visibleRows"
        :key="entry.key"
        class="row"
        :class="{
          selected: selected.has(entry.key),
          first: entry.index === 0,
          last: entry.index === rows.length - 1,
        }"
        :style="{ height: `${rowHeight}px`, transform: `translateY(${entry.offset}px)` }"
        role="row"
        :aria-rowindex="entry.index + 2"
        :aria-selected="selected.has(entry.key)"
        :data-row-key="entry.key"
      >
        <div
          v-for="column in visibleColumns"
          :key="column.key"
          class="cell"
          :class="`align-${column.align ?? 'start'}`"
          role="gridcell"
        >
          <slot :name="`cell-${column.key}`" :row="entry.row" :index="entry.index"></slot>
        </div>
      </div>
    </div>

    <div v-if="rows.length === 0" class="empty">
      <slot name="empty"></slot>
    </div>
  </div>

  <Teleport to="body">
    <div
      v-if="rect"
      class="selection-marquee"
      :style="{
        left: `${rect.left}px`,
        top: `${rect.top}px`,
        width: `${rect.width}px`,
        height: `${rect.height}px`,
      }"
    ></div>
  </Teleport>
</template>

<style lang="scss" scoped>

.selectable-table {
  --corner: 5px;

  background-color: var(--color-0-alt);
  border: 1px solid var(--color-2);
  border-radius: calc(var(--corner) + 1px);
  user-select: none;
  -webkit-user-select: none;
  -webkit-touch-callout: none;
}

.head,
.row {
  display: grid;
  grid-template-columns: var(--table-columns);
  align-items: center;
}

.head {
  position: sticky;
  top: 0;
  z-index: 3;
  background-color: var(--color-1);
  border-bottom: 1px solid var(--color-2);
  border-radius: var(--corner) var(--corner) 0 0;
  color: var(--color-6);
  font-weight: 600;
}

.body {
  position: relative;
}

.row {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  box-sizing: border-box;
  border-top: 1px solid var(--color-2);

  &.first {
    border-top: 0;
  }
}

.cell {
  position: relative;
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
  padding: 8px 12px;
  white-space: nowrap;

  &.align-center {
    justify-content: center;
  }

  &.align-end {
    justify-content: flex-end;
  }
}

.header-cell {
  align-self: stretch;

  &:first-child {
    border-top-left-radius: var(--corner);
  }

  &:last-child {
    border-top-right-radius: var(--corner);
  }

  &.sortable {
    cursor: pointer;

    &:hover {
      background-color: rgb(from var(--color-accent) r g b / 0.12);
    }

    &:focus-visible {
      outline: 2px solid var(--color-accent);
      outline-offset: -2px;
    }
  }
}

.sort-icon {
  flex: none;
  color: var(--color-accent);
}

.row::before {
  content: '';
  position: absolute;
  inset: 0;
  border: 1px solid transparent;
  pointer-events: none;
}

.row.last::before {
  border-bottom-left-radius: var(--corner);
  border-bottom-right-radius: var(--corner);
}

@media (hover: hover) {
  .row:hover::before {
    background-color: rgb(from var(--color-accent) r g b / 0.08);
  }
}

.row.selected::before,
.row.selected:hover::before {
  background-color: rgb(from var(--color-accent) r g b / 0.2);
  border-color: var(--color-accent);
}

.row.selected + .row.selected::before {
  top: -1px;
  border-top-color: transparent;
}

.row.selected:has(+ .row.selected)::before {
  border-bottom-color: transparent;
}

.empty {
  padding: 24px 12px;
  text-align: center;
  color: var(--color-dim);
}

.selection-marquee {
  position: fixed;
  z-index: 90;
  pointer-events: none;
  background-color: rgb(from var(--color-accent) r g b / 0.15);
  border: 1px solid var(--color-accent);
}

</style>
