import { ref, type Ref } from 'vue'

export function useLoader<T>(load: () => Promise<T>) {
  const data = ref<T | null>(null) as Ref<T | null>
  const loading = ref(true)
  const error = ref<string | null>(null)
  let latest = 0

  async function reload() {
    const call = ++latest
    loading.value = true
    error.value = null
    try {
      const result = await load()
      if (call !== latest) return
      data.value = result
    } catch (err) {
      if (call !== latest) return
      data.value = null
      error.value = err instanceof Error ? err.message : String(err)
    } finally {
      if (call === latest) loading.value = false
    }
  }

  return { data, loading, error, reload }
}
