<template>
  <span class="pill" :class="klass">
    <i></i>
    {{ value || '-' }}
  </span>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{ value?: string }>()
const klass = computed(() => ({
  success: props.value === 'SUCCESS',
  danger: props.value === 'FAILED',
  warning: props.value === 'RUNNING' || props.value === 'WAITING',
  info: !props.value
}))
</script>

<style scoped>
.pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 5px 9px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 900;
  color: #334155;
  background: rgba(100, 116, 139, 0.12);
}

.pill i {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: #64748b;
}

.success { color: #047857; background: rgba(49, 196, 141, 0.14); }
.success i { background: #31c48d; }
.danger { color: #b91c1c; background: rgba(240, 82, 82, 0.14); }
.danger i { background: #f05252; }
.warning { color: #c2410c; background: rgba(255, 138, 76, 0.16); }
.warning i { background: #ff8a4c; }
</style>
