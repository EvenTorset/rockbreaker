export function getCSSVar(name: string, element: HTMLElement = document.documentElement) {
  return getComputedStyle(element).getPropertyValue(name)
}
