import { ref } from 'vue'

export const currentInstanceName = ref<string | null>(null)
export const currentLaunchFn = ref<(() => Promise<void>) | null>(null)
export const currentStopFn = ref<(() => Promise<void>) | null>(null)
