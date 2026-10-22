<script setup lang="ts">
import { darkTheme, NConfigProvider, NNotificationProvider, type GlobalThemeOverrides } from 'naive-ui'
import { RouterView } from 'vue-router'
import { getCSSVar } from '@/util/getCSSVar'
import { ref } from 'vue'
import { NotifyProvider } from './notify'
import Logo from './components/Logo.vue'

function genNaiveTheme(): GlobalThemeOverrides {
  return {
    common: {
      primaryColor: getCSSVar('--color-accent'),
      primaryColorHover: getCSSVar('--color-accent-hover'),
      primaryColorPressed: getCSSVar('--color-accent-pressed'),
      primaryColorSuppl: getCSSVar('--color-accent-suppl'),
      inputColor: getCSSVar('--color-0'),
      textColorBase: getCSSVar('--color-5'),
      textColor1: getCSSVar('--color-4'),
      textColor2: getCSSVar('--color-5'),
      textColor3: getCSSVar('--color-6'),
      fontWeight: '500',
      fontSize: '16px',
    },
    Button: {
      textColorPrimary: getCSSVar('--color-6'),
    },
    Input: {
      border: '1px solid var(--color-2)',
      placeholderColor: 'var(--color-4)',
      iconColor: 'var(--color-4)',
    },
    Split: {
      resizableTriggerColor: 'color-mix(in oklch, var(--color-1), var(--color-2))',
      resizableTriggerColorHover: 'var(--color-accent)',
    },
    Notification: {
      color: 'var(--color-1)',
      iconColor: 'var(--color-4)',
      iconColorInfo: 'color-mix(in srgb, var(--color-accent) 75%, var(--color-1))',
      iconColorWarning: 'color-mix(in srgb, var(--color-alert) 60%, var(--color-1))',
      iconColorError: 'color-mix(in srgb, var(--color-danger) 75%, var(--color-1))',
      iconColorSuccess: 'color-mix(in srgb, var(--color-success) 60%, var(--color-1))',
      headerTextColor: 'var(--color-6)',
      closeMargin: '0 calc(-1 * var(--notify-border-width)) 0 0',
      padding: '0 0 16px 0',
      width: 'fit-content',
    },
    Skeleton: {
      color: getCSSVar('--color-2'),
      colorEnd: getCSSVar('--color-3'),
    },
    InternalSelectMenu: {
      color: getCSSVar('--color-1'),
      borderRadius: '6px',
      optionColorActive: getCSSVar('--color-2'),
      optionColorPending: getCSSVar('--color-2'),
      optionColorActivePending: getCSSVar('--color-2'),
      optionTextColorActive: getCSSVar('--color-6'),
    },
    Upload: {
      draggerColor: getCSSVar('--color-0'),
      draggerBorder: '1px dashed var(--color-3)',
    },
    Checkbox: {
      labelFontWeight: 500,
      border: '1px solid var(--color-4)',
      checkMarkColor: '#fff',
    },
    Switch: {
      railColor: getCSSVar('--color-2'),
    },
    Breadcrumb: {
      itemTextColor: getCSSVar('--color-4'),
      itemTextColorHover: getCSSVar('--color-6'),
      itemTextColorPressed: getCSSVar('--color-6'),
      itemTextColorActive: getCSSVar('--color-6'),
      itemColorHover: 'rgb(from var(--color-accent) r g b / 0.15)',
      itemColorPressed: 'rgb(from var(--color-accent) r g b / 0.25)',
      separatorColor: getCSSVar('--color-3'),
    },
    Table: {
      tdColor: getCSSVar('--color-0-alt'),
      thColor: getCSSVar('--color-1'),
      borderColor: getCSSVar('--color-2'),
      borderRadius: '6px',
    },
  }
}

const naiveThemeOverrides = ref<GlobalThemeOverrides>(genNaiveTheme())
</script>

<template>
  <NConfigProvider
    :theme="darkTheme"
    :theme-overrides="naiveThemeOverrides"
  >
    <NNotificationProvider placement="bottom-right" :container-style="{ bottom: 'calc(72px + env(safe-area-inset-bottom, 0px))' }">
      <NotifyProvider>
        <div class="app-container">
          <header class="app-header">
            <Logo />
          </header>
          <main>
            <RouterView />
          </main>
        </div>
      </NotifyProvider>
    </NNotificationProvider>
  </NConfigProvider>
</template>

<style>
@import url('@/assets/notify.scss');
</style>

<style lang="scss" scoped>

.app-container {
  width: 100%;
  max-width: 1200px;
  margin: 0 auto;
  padding: 2rem 1.5rem calc(6rem + env(safe-area-inset-bottom, 0px));
}

.app-header {
  margin-bottom: 2rem;
  padding-bottom: 1rem;
  border-bottom: 1px solid var(--color-2);

  h1 {
    font-size: 4em;
    font-weight: 300;
  }
}

</style>
