import { encodePath } from '@/util/urls'
import { getHostToken, setHostToken } from '@/util/hostToken'

export type ItemKind = 'file' | 'directory' | 'collection'

export interface DashboardItem {
  id: string
  kind: ItemKind
  paths: string[]
  share_url: string
  direct_url: string
  shared_at: number | null
}

export interface SettingState {
  supported: boolean
  enabled: boolean
}

export interface ContextMenuOption {
  id: string
  label: string
  single_label: string
  description: string
  enabled: boolean
}

export interface Settings {
  context_menu: SettingState
  context_menu_options: ContextMenuOption[]
  run_on_startup: SettingState
  url_base: string
  ignore_patterns: string[]
  respect_gitignore: boolean
}

export type SettingsUpdate = Partial<{
  context_menu: boolean
  context_menu_options: { id: string, enabled: boolean }[]
  run_on_startup: boolean
  url_base: string
  ignore_patterns: string[]
  respect_gitignore: boolean
}>

export interface DirectoryEntry {
  name: string
  is_dir: boolean
  size: number | null
  modified: number | null
  subpath: string
}

export interface ItemDetail {
  id: string
  kind: ItemKind
  root_name: string
  item_count: number | null
  file_name: string
  is_dir: boolean
  is_host: boolean
  file_size: number | null
  modified: number | null
  subpath: string
  parent_subpath: string | null
  entries?: DirectoryEntry[]
  zip_url?: string
  download_url: string
}

async function request(url: string, init: RequestInit | undefined, connectError: string) {
  const token = getHostToken()
  const headers = new Headers(init?.headers)
  if (token) headers.set('X-Rockbreaker-Token', token)

  try {
    return await fetch(url, { ...init, headers })
  } catch {
    throw new Error(connectError)
  }
}

function postJson(body: unknown): RequestInit {
  return {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  }
}

async function failure(res: Response, fallback: string) {
  const reason = await res.text().catch(() => '')
  return new Error(reason || fallback)
}

export async function fetchDashboard(): Promise<DashboardItem[]> {
  const connectError = 'Could not connect to Rockbreaker host server.'
  const res = await request('/!/api/dashboard', undefined, connectError)
  if (!res.ok) {
    if (res.status !== 403) throw new Error('Failed to load dashboard.')
    throw new Error(getHostToken()
      ? 'The host token was rejected, probably because it was replaced. Open the dashboard again from the Rockbreaker tray icon, or by running Rockbreaker with no arguments.'
      : 'The dashboard is only available through the link Rockbreaker opens. Use the Rockbreaker tray icon, or run Rockbreaker with no arguments.')
  }
  try {
    const data: { items: DashboardItem[] } = await res.json()
    return data.items
  } catch {
    throw new Error(connectError)
  }
}

export async function fetchItem(path: string): Promise<ItemDetail> {
  const connectError = 'Could not retrieve shared item detail.'
  const res = await request(`/!/api/item${path}`, undefined, connectError)
  if (!res.ok) throw new Error('Hosted item not found or link has expired.')
  try {
    return await res.json()
  } catch {
    throw new Error(connectError)
  }
}

export interface SearchResult {
  entries: DirectoryEntry[]
  truncated: boolean
}

export async function searchDirectory(
  id: string,
  subpath: string,
  query: string,
  signal: AbortSignal,
): Promise<SearchResult> {
  const path = subpath ? `/${encodePath(subpath)}` : ''
  const res = await request(`/!/api/search/${id}${path}?q=${encodeURIComponent(query)}`, { signal }, 'Could not search.')
  if (!res.ok) throw new Error('Searching failed.')
  return await res.json()
}

export async function stopHosting(id: string): Promise<void> {
  const res = await request(`/!/api/stop/${id}`, { method: 'POST' }, 'Error connecting to host server.')
  if (!res.ok) throw await failure(res, 'Failed to stop sharing.')
}

export async function revealHosting(id: string): Promise<void> {
  const res = await request(`/!/api/reveal/${id}`, { method: 'POST' }, 'Error connecting to host server.')
  if (!res.ok) throw await failure(res, 'Failed to show the item in the file explorer.')
}

export async function renameHosting(id: string, newId: string): Promise<void> {
  const res = await request(`/!/api/rename/${id}`, postJson({ new_id: newId }), 'Error connecting to host server.')
  if (!res.ok) throw await failure(res, 'Failed to edit the item\'s ID.')
}

export async function fetchSettings(): Promise<Settings> {
  const connectError = 'Could not connect to Rockbreaker host server.'
  const res = await request('/!/api/settings', undefined, connectError)
  if (!res.ok) throw new Error('Failed to load settings.')
  try {
    return await res.json()
  } catch {
    throw new Error(connectError)
  }
}

export async function updateSettings(update: SettingsUpdate): Promise<Settings> {
  const res = await request('/!/api/settings', postJson(update), 'Error connecting to host server.')
  if (!res.ok) throw await failure(res, 'Failed to update settings.')
  try {
    return await res.json()
  } catch {
    throw new Error('Error connecting to host server.')
  }
}

export async function regenerateToken(): Promise<void> {
  const res = await request('/!/api/token/regenerate', { method: 'POST' }, 'Error connecting to host server.')
  if (!res.ok) throw await failure(res, 'Failed to generate a new token.')

  try {
    const data: { token: string } = await res.json()
    setHostToken(data.token)
  } catch {
    throw new Error('Error connecting to host server.')
  }
}
