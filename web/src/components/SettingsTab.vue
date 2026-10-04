<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { NButton, NCard, NInput, NPopconfirm } from 'naive-ui'
import { fetchSettings, regenerateToken, updateSettings, type SettingsUpdate } from '@/api'
import { useLoader } from '@/composables/useLoader'
import Notify from '@/notify'
import Col from './Col.vue'
import PageState from './PageState.vue'
import Row from './Row.vue'
import SettingSwitch from './SettingSwitch.vue'

const emit = defineEmits<{
  urlBaseChanged: []
}>()

const { data: settings, loading, error, reload } = useLoader(fetchSettings)
const saving = ref<keyof SettingsUpdate | 'token' | null>(null)
const urlBase = ref('')

watch(() => settings.value?.url_base, (value) => {
  if (value !== undefined) urlBase.value = value
})

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
            title="File explorer context menu"
            :description="settings.context_menu.supported
              ? 'Add “Share with Rockbreaker…” to the right-click menu of files and folders.'
              : 'Only available on Windows.'"
            :value="settings.context_menu.enabled"
            :disabled="!settings.context_menu.supported || saving !== null"
            :loading="saving === 'context_menu'"
            @update:value="apply('context_menu', { context_menu: $event })"
          />
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
        </Col>
      </NCard>

      <NCard title="Share links">
        <Col align="stretch" gap="8px">
          <div>
            <h3>URL base</h3>
            <p>
              The address that share links are built on. Set it to wherever this computer can be reached from
              outside, e.g. https://tunnel.example.com
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
