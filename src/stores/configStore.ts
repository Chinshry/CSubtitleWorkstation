import { ref } from 'vue'

export const configRevision = ref(0)

export function notifyConfigChanged() {
  configRevision.value += 1
}
