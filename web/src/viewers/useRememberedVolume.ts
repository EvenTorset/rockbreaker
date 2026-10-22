import { onMounted, type Ref } from 'vue'

const STORAGE_KEY = 'rockbreaker:volume'

interface Saved {
  volume: number
  muted: boolean
}

function load(): Saved | null {
  try {
    const saved = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? 'null')
    if (typeof saved?.volume === 'number' && saved.volume >= 0 && saved.volume <= 1) {
      return { volume: saved.volume, muted: saved.muted === true }
    }
  } catch {
    // Storage can be unavailable or hold something else, and then the player just starts out as it normally does
  }
  return null
}

export function useRememberedVolume(media: Ref<HTMLMediaElement | undefined>) {
  onMounted(() => {
    const saved = load()
    if (!saved || !media.value) return

    media.value.volume = saved.volume
    media.value.muted = saved.muted
  })

  function onVolumeChange() {
    if (!media.value) return

    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify({ volume: media.value.volume, muted: media.value.muted }))
    } catch {
      // Not being able to remember it is no reason to get in the way of playing
    }
  }

  return { onVolumeChange }
}
