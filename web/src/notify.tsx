import { defineComponent, ref, unref, type VNode } from 'vue'
import { NButton, NIcon, useNotification } from 'naive-ui'
import { type Renderable } from '@/types'
import Content from '@/components/Content.vue'
import { CheckmarkCircle24Regular, ErrorCircle24Regular, Info24Regular, QuestionCircle24Regular, Warning24Regular } from '@vicons/fluent'

type NotifyType = 'error' | 'info' | 'warning' | 'success' | 'default'

export type NotifyAction =
  | Renderable
  | {
      icon?: Renderable
      label: string
      run: () => void
    }

export type NotifyOptions = {
  icon?: Renderable
  title?: Renderable
  content: Renderable
  meta?: Renderable
  actions?: NotifyAction[]
  duration?: number
  closable?: boolean
}

export type NotifyController = {
  type: NotifyType
  icon: Renderable | undefined
  title: Renderable
  content: Renderable
  meta: Renderable
  closable: boolean
  actions: NotifyAction[]
  close: (delay?: number) => void
}

type NotifyInput = NotifyOptions | Renderable

const titleText = {
  default: 'Notification',
  info: 'Information',
  success: 'Success',
  warning: 'Warning',
  error: 'Error',
}

let api: ReturnType<typeof useNotification>

export const NotifyProvider = defineComponent({
  setup(_, { slots }) {
    api = useNotification()
    return () => slots.default?.()
  },
})

function normalizeOptions(type: NotifyType, input: NotifyInput): NotifyOptions {
  if (typeof input === 'string' || (input && typeof input === 'object' && 'component' in input)) {
    const title = titleText[type]
    return { title, content: input as string | VNode }
  }

  const opts = input as NotifyOptions
  if (typeof opts.title !== 'string') {
    opts.title = titleText[type]
  }
  return opts
}

function createNotification(type: NotifyType, input: NotifyInput): NotifyController {
  const options = normalizeOptions(type, input)
  const typeRef = ref<NotifyType>(type)
  const iconRef = ref<Renderable>(options.icon!)
  const titleRef = ref<Renderable>(options.title!)
  const contentRef = ref<Renderable>(options.content!)
  const metaRef = ref<Renderable>(options.meta!)
  const actionsRef = ref<NotifyAction[]>(options.actions ?? [])
  const n = api.create({
    type,
    title: () => <Content content={unref(titleRef)} />,
    content: () => <Content content={unref(contentRef)} />,
    meta: () => <Content content={unref(metaRef)} />,
    closable: options.closable ?? true,
    duration: options.duration ?? 0,
    keepAliveOnHover: true,
    avatar: () =>
      iconRef.value ? <Content content={iconRef.value} />
      : type === 'error' ? <NIcon size={24} component={ErrorCircle24Regular} color='var(--color-6)' />
      : type === 'warning' ? <NIcon size={24} component={Warning24Regular} color='var(--color-6)' />
      : type === 'success' ? <NIcon size={24} component={CheckmarkCircle24Regular} color='var(--color-6)' />
      : type === 'info' ? <NIcon size={24} component={Info24Regular} color='var(--color-6)' />
      : <NIcon size={24} component={QuestionCircle24Regular} color='var(--color-6)' />,
    action: () => actionsRef.value.length ? (
      <div style="display: flex; gap: 6px">
        {actionsRef.value.map(a => {
          if (typeof a === 'object' && a !== null && 'label' in a && 'run' in a) {
            const icon = a.icon
            return <NButton
              class='accent'
              onClick={a.run}
              v-slots={{
                default: () => a.label,
                ...icon && { icon: () => <Content content={icon} /> },
              }}
            />
          }
          return <Content content={a} />
        })}
      </div>
    ) : null
  })

  return {
    get type() { return n.type ?? 'default' },
    set type(val: NotifyType) { typeRef.value = n.type = val ?? 'default' },
    get icon() { return iconRef.value },
    set icon(val: Renderable) { iconRef.value = val },
    get title() { return titleRef.value },
    set title(val: Renderable) { titleRef.value = val },
    get content() { return contentRef.value },
    set content(val: Renderable) { contentRef.value = val },
    get meta() { return metaRef.value },
    set meta(val: Renderable) { metaRef.value = val },
    get closable() { return n.closable ?? true },
    set closable(val: boolean) { n.closable = val },
    get actions() { return actionsRef.value },
    set actions(val: NotifyAction[]) { actionsRef.value = val },
    close: (delay = 0) => {
      if (delay > 0) {
        setTimeout(() => {
          n.destroy()
        }, delay)
      } else {
        n.destroy()
      }
    }
  }
}

interface Notify {
  (options: NotifyOptions): NotifyController
  (message: Renderable): NotifyController
  error(options: NotifyOptions): NotifyController
  error(message: Renderable): NotifyController
  info(options: NotifyOptions): NotifyController
  info(message: Renderable): NotifyController
  warn(options: NotifyOptions): NotifyController
  warn(message: Renderable): NotifyController
  success(options: NotifyOptions): NotifyController
  success(message: Renderable): NotifyController
}

const Notify = ((input: Renderable | NotifyOptions) => createNotification('default', input)) as Notify
Notify.error = (input) => createNotification('error', input)
Notify.info = (input) => createNotification('info', input)
Notify.warn = (input) => createNotification('warning', input)
Notify.success = (input) => createNotification('success', input)

export default Notify
