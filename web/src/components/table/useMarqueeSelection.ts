import { onBeforeUnmount, onMounted, ref } from 'vue'

export interface MarqueeRect {
  left: number
  top: number
  width: number
  height: number
}

export interface MarqueeOptions {
  hitTest: (box: MarqueeRect) => Iterable<string>
  selected: () => ReadonlySet<string>
  select: (keys: Set<string>) => void
  enabled?: () => boolean
}

const DRAG_THRESHOLD = 4
const EDGE_SCROLL_ZONE = 48
const EDGE_SCROLL_SPEED = 24

export const IGNORED_TARGETS = [
  'button',
  'input',
  'textarea',
  'select',
  'label',
  'summary',
  '[contenteditable]',
  '[data-no-marquee]',
  '.n-button',
  'a:not([draggable="false"])',
  '.n-modal-container',
  '.n-popover',
  '.n-dropdown-menu',
].join(',')

interface Drag {
  startX: number
  startY: number
  x: number
  y: number
  invert: boolean
  initial: ReadonlySet<string>
  active: boolean
  frame: number | null
  scrollFrame: number | null
}

function sameSet(a: ReadonlySet<string>, b: ReadonlySet<string>) {
  if (a.size !== b.size) return false
  for (const value of a) if (!b.has(value)) return false
  return true
}

export function useMarqueeSelection(options: MarqueeOptions) {
  const rect = ref<MarqueeRect | null>(null)
  let drag: Drag | null = null

  function canStart(event: PointerEvent) {
    if (event.button !== 0 || !event.isPrimary || event.pointerType === 'touch') return false
    if (options.enabled && !options.enabled()) return false

    const target = event.target
    if (!(target instanceof Element) || target.closest(IGNORED_TARGETS)) return false

    const { clientWidth, clientHeight } = document.documentElement
    return event.clientX < clientWidth && event.clientY < clientHeight
  }

  function onPointerDown(event: PointerEvent) {
    if (drag || !canStart(event)) return

    drag = {
      startX: event.clientX + window.scrollX,
      startY: event.clientY + window.scrollY,
      x: event.clientX,
      y: event.clientY,
      invert: event.ctrlKey || event.metaKey,
      initial: new Set(options.selected()),
      active: false,
      frame: null,
      scrollFrame: null,
    }

    window.addEventListener('pointermove', onPointerMove)
    window.addEventListener('pointerup', onPointerUp)
    window.addEventListener('pointercancel', stop)
    window.addEventListener('scroll', schedule, { passive: true })
  }

  function onPointerMove(event: PointerEvent) {
    if (!drag) return
    if (event.buttons === 0) return stop()

    drag.x = event.clientX
    drag.y = event.clientY

    if (!drag.active) {
      const moved = Math.hypot(drag.x + window.scrollX - drag.startX, drag.y + window.scrollY - drag.startY)
      if (moved < DRAG_THRESHOLD) return

      drag.active = true
      window.getSelection()?.removeAllRanges()
      document.documentElement.classList.add('marquee-selecting')
      drag.scrollFrame = requestAnimationFrame(edgeScroll)
    }

    schedule()
  }

  function onPointerUp() {
    const wasActive = drag?.active ?? false
    if (wasActive) apply()
    stop()

    if (wasActive) {
      const swallow = (event: MouseEvent) => {
        event.stopPropagation()
        event.preventDefault()
      }
      window.addEventListener('click', swallow, { capture: true, once: true })
      setTimeout(() => window.removeEventListener('click', swallow, { capture: true }), 0)
    }
  }

  function stop() {
    if (drag?.frame != null) cancelAnimationFrame(drag.frame)
    if (drag?.scrollFrame != null) cancelAnimationFrame(drag.scrollFrame)
    drag = null
    rect.value = null

    document.documentElement.classList.remove('marquee-selecting')
    window.removeEventListener('pointermove', onPointerMove)
    window.removeEventListener('pointerup', onPointerUp)
    window.removeEventListener('pointercancel', stop)
    window.removeEventListener('scroll', schedule)
  }

  function schedule() {
    if (!drag?.active || drag.frame !== null) return
    drag.frame = requestAnimationFrame(apply)
  }

  function apply() {
    if (!drag) return
    drag.frame = null

    const originX = drag.startX - window.scrollX
    const originY = drag.startY - window.scrollY
    const box: MarqueeRect = {
      left: Math.min(originX, drag.x),
      top: Math.min(originY, drag.y),
      width: Math.abs(originX - drag.x),
      height: Math.abs(originY - drag.y),
    }
    rect.value = box

    const touched = new Set(options.hitTest(box))

    let next: Set<string>
    if (drag.invert) {
      next = new Set(drag.initial)
      for (const key of touched) {
        if (!next.delete(key)) next.add(key)
      }
    } else {
      next = touched
    }

    if (!sameSet(next, options.selected())) options.select(next)
  }

  function edgeScroll() {
    if (!drag?.active) return

    const height = window.innerHeight
    let delta = 0
    if (drag.y < EDGE_SCROLL_ZONE) {
      delta = -Math.ceil((EDGE_SCROLL_ZONE - Math.max(drag.y, 0)) / EDGE_SCROLL_ZONE * EDGE_SCROLL_SPEED)
    } else if (drag.y > height - EDGE_SCROLL_ZONE) {
      delta = Math.ceil((Math.min(drag.y, height) - (height - EDGE_SCROLL_ZONE)) / EDGE_SCROLL_ZONE * EDGE_SCROLL_SPEED)
    }
    if (delta !== 0) window.scrollBy(0, delta)

    drag.scrollFrame = requestAnimationFrame(edgeScroll)
  }

  onMounted(() => window.addEventListener('pointerdown', onPointerDown))
  onBeforeUnmount(() => {
    window.removeEventListener('pointerdown', onPointerDown)
    stop()
  })

  return { rect }
}
