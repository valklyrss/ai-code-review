<template>
  <h1 class="page-title">Dashboard</h1>
  <div class="stats">
    <div class="stat"><span>仓库总数</span><b>{{ repos.length }}</b></div>
    <div class="stat"><span>今日任务</span><b>{{ todayTasks }}</b></div>
    <div class="stat"><span>今日问题</span><b>{{ todayIssues }}</b></div>
    <div class="stat"><span>HIGH</span><b>{{ high }}</b></div>
    <div class="stat"><span>CRITICAL</span><b>{{ critical }}</b></div>
  </div>
  <div class="panel">
    <h3>最近审核任务</h3>
    <el-table :data="tasks" v-loading="loading">
      <el-table-column prop="created_at" label="创建时间" min-width="170" />
      <el-table-column prop="repo_name" label="仓库" />
      <el-table-column prop="branch_name" label="分支" />
      <el-table-column label="状态"><template #default="{row}"><StatusTag :value="row.status" /></template></el-table-column>
      <el-table-column label="风险"><template #default="{row}"><RiskTag :value="row.risk_level" /></template></el-table-column>
      <el-table-column prop="issue_count" label="问题数" />
    </el-table>
  </div>
</template>
<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { api, Repo } from '../api'
import StatusTag from '../components/StatusTag.vue'
import RiskTag from '../components/RiskTag.vue'

const repos = ref<Repo[]>([])
const tasks = ref<any[]>([])
const loading = ref(false)
const todayTasks = ref(0)
const todayIssues = ref(0)
const high = ref(0)
const critical = ref(0)

onMounted(async () => {
  loading.value = true
  try {
    repos.value = await api.repos()
    const taskPage: any = await api.tasks({ page_size: 8 })
    tasks.value = taskPage.items || []
    const issuePage: any = await api.issues({ page_size: 100 })
    const today = new Date().toISOString().slice(0, 10)
    todayTasks.value = tasks.value.filter(t => (t.created_at || '').startsWith(today)).length
    todayIssues.value = (issuePage.items || []).filter((i: any) => (i.created_at || '').startsWith(today)).length
    high.value = (issuePage.items || []).filter((i: any) => i.issue_level === 'HIGH').length
    critical.value = (issuePage.items || []).filter((i: any) => i.issue_level === 'CRITICAL').length
  } finally {
    loading.value = false
  }
})
</script>

