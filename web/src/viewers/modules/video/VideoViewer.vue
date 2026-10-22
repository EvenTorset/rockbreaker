<script setup lang="ts">
import { ref } from 'vue'
import Dim from '@/components/Dim.vue'
import type { ViewerFile } from '../../types'
import { useRememberedVolume } from '../../useRememberedVolume'

defineProps<{
  file: ViewerFile
}>()

const failed = ref(false)
const media = ref<HTMLMediaElement>()
const { onVolumeChange } = useRememberedVolume(media)
</script>

<template>
  <div class="video-viewer">
    <Dim v-if="failed" class="failed">
      Your browser can't play this video. You can still download it.
    </Dim>
    <video
      v-else
      ref="media"
      @volumechange="onVolumeChange"
      :src="file.previewUrl"
      controls
      preload="metadata"
      @error="failed = true"
    ></video>
  </div>
</template>

<style lang="scss" scoped>

.video-viewer {
  display: flex;
  justify-content: center;
  background-color: #000;
}

video {
  display: block;
  width: 100%;
  max-height: 80vh;
}

.failed {
  align-self: center;
  padding: 48px 24px;
  text-align: center;
}

</style>
