const STORAGE_KEY = 'rockbreaker-host-token'

let memoryToken: string | null = null

export function getHostToken(): string | null {
  if (memoryToken) return memoryToken
  try {
    return localStorage.getItem(STORAGE_KEY)
  } catch {
    return null
  }
}

export function setHostToken(token: string) {
  memoryToken = token
  try {
    localStorage.setItem(STORAGE_KEY, token)
  } catch {
    // Kept in memory only
  }
}
