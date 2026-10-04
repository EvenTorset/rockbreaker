<script setup lang="ts">
import { computed } from 'vue'
import { NCard, NTabPane, NTable, NTabs } from 'naive-ui'
import { fetchDashboard, type DashboardItem } from '@/api'
import { useLoader } from '@/composables/useLoader'
import { copyText } from '@/util/clipboard'
import Col from '@/components/Col.vue'
import Dim from '@/components/Dim.vue'
import FilePath from '@/components/FilePath.vue'
import PageState from '@/components/PageState.vue'
import RenameHostingButton from '@/components/RenameHostingButton.vue'
import Row from '@/components/Row.vue'
import SettingsTab from '@/components/SettingsTab.vue'
import StopHostingButton from '@/components/StopHostingButton.vue'
import { Link16Filled, Share16Filled } from '@vicons/fluent'
import Tooltip from '@/components/Tooltip.vue'
import IconButton from '@/components/IconButton.vue'

const { data, loading, error, reload } = useLoader(fetchDashboard)
const items = computed(() => data.value ?? [])

const SHOWN_PATHS = 3

function typeLabel(item: DashboardItem) {
  if (item.kind === 'file') return 'File'
  if (item.kind === 'directory') return 'Directory'
  return `${item.paths.length} items`
}

reload()
</script>

<template>
  <NTabs type="line" animated default-value="items" @update:value="tab => tab === 'items' && reload()">
    <NTabPane name="items" tab="Hosted Items">
      <PageState :loading="loading && !data" :error>
        <Col align="stretch" gap="16px">
          <Dim tag="p">Manage local paths currently being served.</Dim>

          <NCard v-if="items.length === 0">
            No active shared items found on host daemon.
          </NCard>

          <NTable v-else :single-line="false" size="small">
            <colgroup>
              <col span="1" style="width: 100%;">
              <col span="1" style="width: fit-content;">
              <col span="1" style="width: fit-content;">
              <col span="1" style="width: fit-content;">
            </colgroup>
            <thead>
              <tr>
                <th>Shared Item</th>
                <th>Type</th>
                <th>Links</th>
                <th>Action</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in items" :key="item.id">
                <td class="name-cell">
                  <Col align="flex-start" gap="2px">
                    <a
                      class="host-id"
                      :href="item.share_url"
                      target="_blank"
                      rel="noopener noreferrer"
                    >{{ item.id }}</a>
                    <FilePath v-for="path in item.paths.slice(0, SHOWN_PATHS)" :key="path" :path />
                    <Dim v-if="item.paths.length > SHOWN_PATHS">
                      and {{ item.paths.length - SHOWN_PATHS }} more
                    </Dim>
                  </Col>
                </td>
                <td style="white-space: nowrap;">{{ typeLabel(item) }}</td>
                <td>
                  <Row>
                    <Tooltip>
                      <template #trigger="{ props }">
                        <IconButton v-bind="props" @click="copyText(item.share_url, 'Share URL')">
                          <Share16Filled />
                        </IconButton>
                      </template>
                      Copy share link
                    </Tooltip>
                    <Tooltip>
                      <template #trigger="{ props }">
                        <IconButton v-bind="props" @click="copyText(item.direct_url, 'Direct URL')">
                          <Link16Filled />
                        </IconButton>
                      </template>
                      Copy direct link
                    </Tooltip>
                  </Row>
                </td>
                <td>
                  <Row>
                    <RenameHostingButton :host-id="item.id" size="small" @renamed="reload" />
                    <StopHostingButton :host-id="item.id" size="small" @stopped="reload" />
                  </Row>
                </td>
              </tr>
            </tbody>
          </NTable>
        </Col>
      </PageState>
    </NTabPane>

    <NTabPane name="settings" tab="Settings" display-directive="show:lazy">
      <SettingsTab @url-base-changed="reload" />
    </NTabPane>
  </NTabs>
</template>

<style lang="scss" scoped>

.name-cell {
  max-width: 0;
}

.host-id {
  font-weight: 700;
  color: var(--color-accent);
  text-decoration: none;

  &:hover {
    color: var(--color-accent-hover);
    text-decoration: underline;
  }
}

</style>
