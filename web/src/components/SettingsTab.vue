<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { NButton, NCard, NInput, NPopconfirm, NSwitch } from 'naive-ui'
import { ArrowDown16Filled, ArrowUp16Filled } from '@vicons/fluent'
import { fetchSettings, regenerateToken, updateSettings, type SettingsUpdate } from '@/api'
import { useLoader } from '@/composables/useLoader'
import Notify from '@/notify'
import Col from './Col.vue'
import IconButton from './IconButton.vue'
import PageState from './PageState.vue'
import Row from './Row.vue'
import SettingSwitch from './SettingSwitch.vue'

const emit = defineEmits<{
  urlBaseChanged: []
}>()

const { data: settings, loading, error, reload } = useLoader(fetchSettings)
const saving = ref<keyof SettingsUpdate | 'token' | null>(null)
const urlBase = ref('')
const ignorePatterns = ref('')

watch(() => settings.value?.url_base, (value) => {
  if (value !== undefined) urlBase.value = value
})

watch(() => settings.value?.ignore_patterns, (value) => {
  if (value !== undefined) ignorePatterns.value = value.join('\n')
})

function typedPatterns() {
  return ignorePatterns.value.split('\n').map(line => line.trim()).filter(Boolean)
}

const ignorePatternsChanged = computed(() => {
  if (!settings.value) return false
  const typed = typedPatterns()
  const saved = settings.value.ignore_patterns
  return typed.length !== saved.length || typed.some((line, index) => line !== saved[index])
})

const menuOptions = computed(() => settings.value?.context_menu_options ?? [])
const enabledMenuOptions = computed(() => menuOptions.value.filter(option => option.enabled))

const menuHint = computed(() => {
  const [only] = enabledMenuOptions.value
  if (enabledMenuOptions.value.length === 1) {
    return `Only "${only.label}" is on, so it's in the menu by itself as "${only.single_label}", instead of in a "Share with Rockbreaker…" submenu.`
  }
  return 'These show up in the "Share with Rockbreaker…" submenu, in this order. If only one is enabled, it replaces the submenu instead.'
})

function changeMenuOptions(change: (options: { id: string, enabled: boolean }[]) => void) {
  const options = menuOptions.value.map(({ id, enabled }) => ({ id, enabled }))
  change(options)
  return apply('context_menu_options', { context_menu_options: options })
}

function moveMenuOption(index: number, by: -1 | 1) {
  return changeMenuOptions((options) => {
    const [moved] = options.splice(index, 1)
    options.splice(index + by, 0, moved)
  })
}

function toggleMenuOption(index: number, enabled: boolean) {
  return changeMenuOptions((options) => {
    options[index].enabled = enabled
  })
}

const urlBaseChanged = computed(() => !!settings.value && urlBase.value.trim() !== settings.value.url_base)

async function apply(key: keyof SettingsUpdate, update: SettingsUpdate): Promise<boolean> {
  saving.value = key
  try {
    settings.value = await updateSettings(update)
    return true
  } catch (err) {
    Notify.error(err instanceof Error ? err.message : 'Failed to update settings.')
    return false
  } finally {
    saving.value = null
  }
}

async function saveUrlBase() {
  if (!await apply('url_base', { url_base: urlBase.value })) return;
  urlBase.value = settings.value?.url_base ?? urlBase.value
  Notify.success({ content: 'Share link address saved.', duration: 3000 })
  emit('urlBaseChanged')
}

async function saveIgnorePatterns() {
  if (!await apply('ignore_patterns', { ignore_patterns: typedPatterns() })) return
  ignorePatterns.value = settings.value?.ignore_patterns.join('\n') ?? ignorePatterns.value
  Notify.success({ content: 'Ignore list saved.', duration: 3000 })
}

async function generateToken() {
  saving.value = 'token'
  try {
    await regenerateToken()
    Notify.success({ content: 'Generated a new token. This browser keeps working.', duration: 4000 })
  } catch (err) {
    Notify.error(err instanceof Error ? err.message : 'Failed to generate a new token.')
  } finally {
    saving.value = null
  }
}

reload()
</script>

<template>
  <PageState :loading="loading && !settings" :error>
    <Col v-if="settings" align="stretch" gap="16px">
      <NCard title="System">
        <Col align="stretch" gap="20px">
          <SettingSwitch
            title="Run on startup"
            :description="settings.run_on_startup.supported
              ? 'Start Rockbreaker in the background when you log in.'
              : 'Only available on Windows.'"
            :value="settings.run_on_startup.enabled"
            :disabled="!settings.run_on_startup.supported || saving !== null"
            :loading="saving === 'run_on_startup'"
            @update:value="apply('run_on_startup', { run_on_startup: $event })"
          />
          <SettingSwitch
            title="File explorer context menu"
            :description="settings.context_menu.supported
              ? 'Add &quot;Share with Rockbreaker…&quot; to the right-click menu of files and folders.'
              : 'Only available on Windows.'"
            :value="settings.context_menu.enabled"
            :disabled="!settings.context_menu.supported || saving !== null"
            :loading="saving === 'context_menu'"
            @update:value="apply('context_menu', { context_menu: $event })"
          />
          <Col v-if="settings.context_menu.supported" align="stretch" gap="8px" class="menu-options">
            <div>
              <h3>Context menu options</h3>
              <p>{{ menuHint }}</p>
            </div>
            <div
              v-for="(option, index) in menuOptions"
              :key="option.id"
              class="menu-option"
              :class="{ off: !option.enabled }"
            >
              <Row gap="4px">
                <IconButton
                  :disabled="index === 0 || saving !== null"
                  :aria-label="`Move &quot;${option.label}&quot; up`"
                  @click="moveMenuOption(index, -1)"
                >
                  <ArrowUp16Filled />
                </IconButton>
                <IconButton
                  :disabled="index === menuOptions.length - 1 || saving !== null"
                  :aria-label="`Move &quot;${option.label}&quot; down`"
                  @click="moveMenuOption(index, 1)"
                >
                  <ArrowDown16Filled />
                </IconButton>
              </Row>
              <div class="menu-option-text">
                <span class="menu-option-label">{{ option.label }}</span>
                <p>{{ option.description }}</p>
              </div>
              <NSwitch
                :value="option.enabled"
                :disabled="saving !== null || (option.enabled && enabledMenuOptions.length === 1)"
                :aria-label="`Show &quot;${option.label}&quot;`"
                @update:value="toggleMenuOption(index, $event)"
              />
            </div>
          </Col>
        </Col>
      </NCard>

      <NCard title="Share links">
        <Col align="stretch" gap="8px">
          <div>
            <h3>URL base</h3>
            <p>
              The address that share links are built on. Set it to wherever this computer can be reached from
              outside, e.g. <code>https://tunnel.example.com</code>
            </p>
          </div>
          <Row gap="8px" align="flex-start">
            <NInput
              v-model:value="urlBase"
              placeholder="https://tunnel.example.com"
              :disabled="saving === 'url_base'"
              @keyup.enter="urlBaseChanged && saveUrlBase()"
            />
            <NButton
              class="accent"
              :disabled="!urlBaseChanged"
              :loading="saving === 'url_base'"
              @click="saveUrlBase"
            >
              Save
            </NButton>
          </Row>
        </Col>
      </NCard>

      <NCard title="Ignored files">
        <Col align="stretch" gap="20px">
          <SettingSwitch
            title="Ignore files that Git ignores"
            description="Leave out what the .gitignore files in a shared folder say to ignore. Only the files from the shared folder down are looked at, never the ones above it."
            :value="settings!.respect_gitignore"
            :disabled="saving !== null"
            :loading="saving === 'respect_gitignore'"
            @update:value="apply('respect_gitignore', { respect_gitignore: $event })"
          />

          <Col align="stretch" gap="8px">
            <div>
              <h3>Ignore list</h3>
              <p>
                Files and folders that match any of these are left out when they're found inside of a shared folder.
                One pattern per line, written like the lines of a .gitignore file, so <code>node_modules</code>
                matches at any depth, <code>*.log</code> matches by name, <code>/build</code> only the top level,
                and <code>!keep.log</code> brings something back. Something that is shared directly is always shared,
                even if it matches.
              </p>
            </div>
            <NInput
              v-model:value="ignorePatterns"
              class="ignore-list"
              type="textarea"
              :autosize="{ minRows: 4, maxRows: 14 }"
              placeholder=".git&#10;node_modules"
              :disabled="saving === 'ignore_patterns'"
              :input-props="{ spellcheck: false, autocapitalize: 'off', autocorrect: 'off' }"
            />
            <Row justify="flex-end">
              <NButton
                class="accent"
                :disabled="!ignorePatternsChanged"
                :loading="saving === 'ignore_patterns'"
                @click="saveIgnorePatterns"
              >
                Save
              </NButton>
            </Row>
          </Col>
        </Col>
      </NCard>

      <NCard title="Security">
        <Row justify="space-between" gap="16px">
          <div>
            <h3>Host token</h3>
            <p>
              The token that gives a browser access to this dashboard. Generate a new one if it may have leaked.
              The old one stops working immediately.
            </p>
          </div>
          <NPopconfirm
            placement="bottom-end"
            positive-text="Generate"
            negative-text="Cancel"
            @positive-click="generateToken"
          >
            <template #trigger>
              <NButton class="danger" :loading="saving === 'token'" :disabled="saving !== null">
                Generate new token
              </NButton>
            </template>
            Every other browser or saved dashboard link will lose access. Continue?
          </NPopconfirm>
        </Row>
      </NCard>
    </Col>
  </PageState>
</template>

<style lang="scss" scoped>

.menu-options {
  margin-left: 12px;
  padding-left: 16px;
  border-left: 2px solid var(--color-2);
}

.menu-option {
  display: grid;
  grid-template-columns: auto 1fr auto;
  align-items: center;
  gap: 16px;
  padding: 8px 12px;
  background-color: var(--color-1);
  border: 1px solid var(--color-2);
  border-radius: 6px;

  &.off .menu-option-text {
    opacity: 0.55;
  }

  p {
    margin: 2px 0 0;
    font-size: 0.9rem;
  }
}

.menu-option-label {
  font-weight: 600;
}

.ignore-list :deep(textarea) {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, 'Liberation Mono', monospace;
  font-size: 13px;
}

code {
  padding: 0.1em 0.35em;
  background-color: var(--color-1);
  border-radius: 4px;
  font-size: 0.9em;
}

</style>
