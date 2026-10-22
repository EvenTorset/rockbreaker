import type { Viewer, ViewerFile } from './types'

const viewers: Viewer[] = []

export function registerViewer(viewer: Viewer) {
  if (viewers.some(existing => existing.id === viewer.id)) {
    throw new Error(`A viewer with the ID '${viewer.id}' is already registered.`)
  }

  viewers.push(viewer)
  viewers.sort((a, b) => (b.priority ?? 0) - (a.priority ?? 0))
}

export async function findViewer(file: ViewerFile): Promise<Viewer | null> {
  for (const viewer of viewers) {
    if (file.signal.aborted) return null

    try {
      if (await viewer.test(file)) return viewer
    } catch (error) {
      if (!file.signal.aborted) console.warn(`The '${viewer.id}' viewer failed to test '${file.name}'.`, error)
    }
  }

  return null
}

const modules = import.meta.glob<{ default: Viewer }>('./modules/*/index.ts', { eager: true })

for (const path of Object.keys(modules).sort()) {
  registerViewer(modules[path].default)
}
