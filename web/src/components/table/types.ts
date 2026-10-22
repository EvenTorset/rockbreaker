export type SortDirection = 'asc' | 'desc'

export interface SortState {
  key: string
  direction: SortDirection
}

export interface TableColumn {
  key: string
  title: string
  width?: string
  sortable?: boolean
  hideBelow?: number
  align?: 'start' | 'center' | 'end'
}
