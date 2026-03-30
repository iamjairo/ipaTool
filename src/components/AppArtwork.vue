<template>
  <div class="app-artwork">
    <img
      v-if="imageSrc && !imageFailed"
      :src="imageSrc"
      :alt="altText"
      class="app-artwork-image"
      @error="imageFailed = true"
    >
    <div v-else class="app-artwork-fallback">
      {{ fallbackText }}
    </div>
  </div>
</template>

<script setup>
import { computed, ref, watch } from 'vue'

const props = defineProps({
  src: {
    type: String,
    default: ''
  },
  alt: {
    type: String,
    default: ''
  },
  label: {
    type: String,
    default: ''
  }
})

const imageFailed = ref(false)

watch(() => props.src, () => {
  imageFailed.value = false
})

const imageSrc = computed(() => props.src?.trim() || '')
const altText = computed(() => props.alt?.trim() || props.label?.trim() || 'App artwork')
const fallbackText = computed(() => {
  const text = props.label?.trim() || props.alt?.trim() || '?'
  return text.slice(0, 1).toUpperCase()
})
</script>

<style scoped>
.app-artwork {
  width: 56px;
  height: 56px;
  flex-shrink: 0;
  border-radius: 14px;
  overflow: hidden;
  box-shadow: 0 8px 18px rgba(15, 23, 42, 0.12);
  background: linear-gradient(135deg, rgba(99, 102, 241, 0.16), rgba(59, 130, 246, 0.18));
}

.app-artwork-image,
.app-artwork-fallback {
  width: 100%;
  height: 100%;
}

.app-artwork-image {
  display: block;
  object-fit: cover;
}

.app-artwork-fallback {
  display: flex;
  align-items: center;
  justify-content: center;
  color: #4338ca;
  font-size: 22px;
  font-weight: 700;
}

.dark .app-artwork {
  background: linear-gradient(135deg, rgba(79, 70, 229, 0.26), rgba(14, 165, 233, 0.22));
}

.dark .app-artwork-fallback {
  color: #e0e7ff;
}
</style>
