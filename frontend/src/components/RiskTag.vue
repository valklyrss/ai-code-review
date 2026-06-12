<template>
  <span class="risk" :class="klass">{{ value || 'INFO' }}</span>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{ value?: string }>()
const klass = computed(() => ({
  critical: props.value === 'CRITICAL',
  high: props.value === 'HIGH',
  medium: props.value === 'MEDIUM',
  low: props.value === 'LOW',
  info: !props.value || props.value === 'INFO'
}))
</script>

<style scoped>
.risk {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 64px;
  padding: 5px 9px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 950;
  color: #334155;
  background: rgba(100, 116, 139, 0.12);
}

.critical {
  color: #7f1d1d;
  background: linear-gradient(135deg, rgba(240, 82, 82, 0.24), rgba(255, 138, 76, 0.18));
}

.high {
  color: #b91c1c;
  background: rgba(240, 82, 82, 0.16);
}

.medium {
  color: #c2410c;
  background: rgba(255, 138, 76, 0.16);
}

.low {
  color: #047857;
  background: rgba(49, 196, 141, 0.14);
}
</style>
