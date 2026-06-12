<template>
  <h1 class="page-title">审核任务</h1>
  <div class="toolbar">
    <el-input v-model="filters.repo_id" placeholder="仓库ID" style="width:180px" clearable />
    <el-select v-model="filters.status" placeholder="状态" clearable style="width:140px"><el-option v-for="s in ['WAITING','RUNNING','SUCCESS','FAILED']" :key="s" :label="s" :value="s" /></el-select>
    <el-select v-model="filters.result" placeholder="结果" clearable style="width:120px"><el-option v-for="s in ['PASS','WARN','FAIL']" :key="s" :label="s" :value="s" /></el-select>
    <el-select v-model="filters.risk_level" placeholder="风险" clearable style="width:140px"><el-option v-for="s in ['INFO','LOW','MEDIUM','HIGH','CRITICAL']" :key="s" :label="s" :value="s" /></el-select>
    <el-button @click="load">查询</el-button>
  </div>
  <div class="panel">
    <el-table :data="items" v-loading="loading">
      <el-table-column prop="created_at" label="创建时间" min-width="170" />
      <el-table-column prop="repo_name" label="仓库" />
      <el-table-column prop="branch_name" label="分支" />
      <el-table-column label="old"><template #default="{row}">{{ short(row.old_commit_id) }}</template></el-table-column>
      <el-table-column label="new"><template #default="{row}">{{ short(row.new_commit_id) }}</template></el-table-column>
      <el-table-column label="状态"><template #default="{row}"><StatusTag :value="row.status" /></template></el-table-column>
      <el-table-column prop="result" label="结果" />
      <el-table-column label="风险"><template #default="{row}"><RiskTag :value="row.risk_level" /></template></el-table-column>
      <el-table-column prop="issue_count" label="问题" />
      <el-table-column prop="high_count" label="HIGH" />
      <el-table-column prop="critical_count" label="CRITICAL" />
      <el-table-column label="操作" width="150"><template #default="{row}"><el-button size="small" @click="$router.push('/tasks/'+row.id)">详情</el-button><el-button size="small" @click="retry(row)">重试</el-button></template></el-table-column>
    </el-table>
    <el-pagination layout="prev, pager, next, total" :total="total" :page-size="20" @current-change="changePage" />
  </div>
</template>
<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { api } from '../api'
import StatusTag from '../components/StatusTag.vue'
import RiskTag from '../components/RiskTag.vue'
const items = ref<any[]>([]); const total = ref(0); const loading = ref(false); const page = ref(1)
const filters = reactive<any>({})
const short = (s?: string) => s ? s.slice(0, 8) : '-'
async function load() { loading.value = true; try { const res: any = await api.tasks({ ...filters, page: page.value, page_size: 20 }); items.value = res.items; total.value = res.total } finally { loading.value = false } }
async function retry(row: any) { await api.retryTask(row.id); await load() }
function changePage(p: number) { page.value = p; load() }
onMounted(load)
</script>
