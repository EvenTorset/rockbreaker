export function downloadUrl(url: string) {
  const link = document.createElement('a')
  link.href = url
  link.download = ''
  link.style.display = 'none'
  document.body.append(link)
  link.click()
  link.remove()
}

export function downloadWithPost(url: string, fields: [name: string, value: string][]) {
  const form = document.createElement('form')
  form.method = 'post'
  form.action = url
  form.style.display = 'none'

  for (const [name, value] of fields) {
    const input = document.createElement('input')
    input.type = 'hidden'
    input.name = name
    input.value = value
    form.append(input)
  }

  document.body.append(form)
  form.submit()
  setTimeout(() => form.remove(), 1000)
}
