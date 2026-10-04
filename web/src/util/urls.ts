export function encodePath(path: string) {
  return path.split('/').map(encodeURIComponent).join('/')
}

export function itemLocation(id: string, subpath: string) {
  return subpath ? `/${id}/${encodePath(subpath)}` : `/${id}`
}

export function rawUrl(id: string, subpath: string) {
  return `/!/raw/${id}/${encodePath(subpath)}`
}

export function isValidHostId(id: string) {
  return /^[a-zA-Z0-9_-]+$/.test(id)
}
