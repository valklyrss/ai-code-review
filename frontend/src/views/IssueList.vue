<template>
  <h1 class="page-title">问题清单</h1>
  <div class="toolbar">
    <el-select v-model="filters.level" placeholder="等级" clearable style="width:140px"><el-option v-for="s in ['INFO','LOW','MEDIUM','HIGH','CRITICAL']" :key="s" :value="s" :label="s" /></el-select>
    <el-select v-model="filters.status" placeholder="状态" clearable style="width:170px"><el-option v-for="s in statuses" :key="s" :value="s" :label="s" /></el-select>
    <el-input v-model="filters.repo_id" placeholder="仓库ID" clearable style="width:180px" />
    <el-button @click="load">查询</el-button>
  </div>
  <div class="panel">
    <el-table :data="items" v-loading="loading">
      <el-table-column label="等级" width="110"><template #default="{row}"><RiskTag :value="row.issue_level" /></template></el-table-column>
      <el-table-column prop="file_path" label="文件" min-width="220" show-overflow-tooltip />
      <el-table-column prop="line_no" label="行" width="80" />
      <el-table-column prop="title" label="标题" min-width="180" />
      <el-table-column prop="description" label="说明" min-width="260" show-overflow-tooltip />
      <el-table-column label="状态" width="170"><template #default="{row}"><el-select v-model="row.status" @change="(s: string) => update(row, s)"><el-option v-for="s in statuses" :key="s" :value="s" :label="s" /></el-select></template></el-table-column>
    </el-table>
    <el-pagination layout="prev, pager, next, total" :total="total" :page-size="20" @current-change="changePage" />
  </div>
</template>
<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { useRoute } from 'vue-router'
import { api } from '../api'
import RiskTag from '../components/RiskTag.vue'
const statuses = ['TODO','CONFIRMED','FIXED','FALSE_POSITIVE','IGNORED']
const filters = reactive<any>({})
const items = ref<any[]>([]); const total = ref(0); const page = ref(1); const loading = ref(false)
const route = useRoute()
async function load() { loading.value = true; try { const res: any = await api.issues({ ...filters, page: page.value, page_size: 20 }); items.value = res.items; total.value = res.total } finally { loading.value = false } }
async function update(row: any, status: string) { await api.updateIssueStatus(row.id, status) }
function changePage(p: number) { page.value = p; load() }
onMounted(() => {
  if (route.query.level) filters.level = route.query.level
  if (route.query.status) filters.status = route.query.status
  if (route.query.repo_id) filters.repo_id = route.query.repo_id
  if (route.query.active) filters.active = route.query.active
  if (route.query.serious) filters.serious = route.query.serious
  if (route.query.date) filters.date = route.query.date
  load()
})
</script>
