import { ref } from "vue"

export interface InstanceNavTarget {
  name: string
  version: string
  version_type: string
  loader?: {
    type: "fabric" | "forge" | "neoforge" | "quilt"
    version: string
  }
  icon?: string
}

export const pendingInstance = ref<InstanceNavTarget | null>(null)

export function navigateToInstance(target: InstanceNavTarget) {
  pendingInstance.value = target
}

export function consumePendingInstance(): InstanceNavTarget | null {
  const val = pendingInstance.value
  pendingInstance.value = null
  return val
}
