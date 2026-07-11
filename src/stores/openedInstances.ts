import { ref } from 'vue'

export interface InstanceInfo {
  name: string
  version: string
  version_type: string
  loader?: string
}

export const openedInstances = ref<InstanceInfo[]>([])

export function addOpenedInstance(inst: InstanceInfo) {
  if (!openedInstances.value.find(i => i.name === inst.name)) {
    openedInstances.value.push(inst)
  }
}

export function removeOpenedInstance(name: string) {
  openedInstances.value = openedInstances.value.filter(i => i.name !== name)
}
