<script lang="ts" setup>
import { computed, ref, watch } from 'vue'
import { fetchImageDataUrl } from '@/services/tauri'
import { normalizeImageUrl } from '@/utils/image'

defineOptions({
  inheritAttrs: false,
})

interface Props {
  src?: string | null
  alt?: string
  referrerpolicy?: ReferrerPolicy
  useProxyOnError?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  src: '',
  alt: '',
  referrerpolicy: 'no-referrer',
  useProxyOnError: true,
})

const proxyCache = new Map<string, Promise<string | null>>()

const currentSrc = ref('')
const hasTriedProxy = ref(false)

const normalizedInputSrc = computed(() => normalizeImageUrl(props.src))

watch(
  normalizedInputSrc,
  (value) => {
    currentSrc.value = value
    hasTriedProxy.value = false
  },
  { immediate: true },
)

function isRemoteHttpUrl(url: string): boolean {
  return url.startsWith('http://') || url.startsWith('https://')
}

function getProxyImage(url: string): Promise<string | null> {
  const cached = proxyCache.get(url)
  if (cached) {
    return cached
  }

  const promise = fetchImageDataUrl(url)
    .then(dataUrl => dataUrl || null)
    .catch((error) => {
      console.warn('[SmartImage] Proxy image fetch failed:', url, error)
      return null
    })

  proxyCache.set(url, promise)
  return promise
}

async function handleError() {
  const fallbackTarget = normalizedInputSrc.value
  if (!props.useProxyOnError || hasTriedProxy.value || !isRemoteHttpUrl(fallbackTarget)) {
    return
  }

  hasTriedProxy.value = true
  const proxiedDataUrl = await getProxyImage(fallbackTarget)
  if (proxiedDataUrl) {
    currentSrc.value = proxiedDataUrl
  } else {
    console.warn('[SmartImage] Image remains unavailable after proxy fallback:', fallbackTarget)
  }
}
</script>

<template>
  <img
    v-if="currentSrc"
    v-bind="$attrs"
    :src="currentSrc"
    :alt="alt"
    :referrerpolicy="referrerpolicy"
    @error="handleError"
  >
</template>
