<template>
  <h1 class="page-title">仓库配置</h1>
  <div class="toolbar">
    <el-button type="primary" @click="openCreate">新增仓库</el-button>
  </div>

  <div class="panel">
    <el-table :data="repos" v-loading="loading">
      <el-table-column prop="repo_name" label="仓库名称" min-width="120" />
      <el-table-column prop="repo_url" label="仓库地址" min-width="260" show-overflow-tooltip />
      <el-table-column prop="auth_type" label="认证" width="80" />
      <el-table-column prop="branch_pattern" label="分支规则" width="120" />
      <el-table-column prop="scan_interval_seconds" label="间隔" width="80" />
      <el-table-column prop="owner_email" label="负责人邮箱" min-width="150" />
      <el-table-column label="拉取状态" min-width="220">
        <template #default="{ row }">
          <div class="sync-cell">
            <el-tag :type="syncTagType(row.sync_status)">{{ syncText(row.sync_status) }}</el-tag>
            <el-progress
              :percentage="row.sync_progress || 0"
              :status="row.sync_status === 'FAILED' ? 'exception' : row.sync_status === 'SUCCESS' ? 'success' : undefined"
              :stroke-width="8"
            />
            <div class="sync-message">{{ row.sync_message || '-' }}</div>
          </div>
        </template>
      </el-table-column>
      <el-table-column label="启用" width="80">
        <template #default="{ row }">
          <el-tag :type="row.enabled ? 'success' : 'info'">{{ row.enabled ? '是' : '否' }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="操作" width="410">
        <template #default="{ row }">
          <el-button size="small" :disabled="isSyncing(row) || row.sync_status !== 'SUCCESS'" @click="$router.push('/repos/' + row.id)">详情</el-button>
          <el-button size="small" :disabled="isSyncing(row)" @click="openEdit(row)">编辑</el-button>
          <el-button size="small" :disabled="isSyncing(row)" @click="test(row)">测试</el-button>
          <el-button size="small" :disabled="isSyncing(row)" @click="sync(row)">拉取</el-button>
          <el-button size="small" :disabled="isSyncing(row) || row.sync_status !== 'SUCCESS'" @click="scan(row)">扫描</el-button>
          <el-button size="small" type="danger" :disabled="isSyncing(row)" @click="remove(row)">删除</el-button>
        </template>
      </el-table-column>
    </el-table>
  </div>

  <el-dialog v-model="visible" :title="editing ? '编辑仓库' : '新增仓库'" width="720px">
    <RepoForm v-model="form" />
    <template #footer>
      <el-button @click="visible = false">取消</el-button>
      <el-button type="primary" @click="save">保存</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { api, Repo } from '../api'
import RepoForm from './RepoForm.vue'

const repos = ref<Repo[]>([])
const loading = ref(false)
const visible = ref(false)
const editing = ref<Repo | null>(null)
const form = ref<any>({})
let timer: number | undefined

async function load(silent = false) {
  if (!silent) loading.value = true
  try {
    repos.value = await api.repos()
    updatePolling()
  } finally {
    if (!silent) loading.value = false
  }
}

function openCreate() {
  editing.value = null
  form.value = { auth_type: 'HTTP', branch_pattern: '*', scan_interval_seconds: 60, enabled: true }
  visible.value = true
}

function openEdit(row: Repo) {
  editing.value = row
  form.value = { ...row, enabled: !!row.enabled }
  visible.value = true
}

async function save() {
  if (editing.value) {
    await api.updateRepo(editing.value.id, form.value)
  } else {
    await api.createRepo(form.value)
  }
  visible.value = false
  ElMessage.success('已保存，正在拉取仓库')
  await load()
}

async function test(row: Repo) {
  const res: any = await api.testRepo(row.id)
  ElMessage.success(`连接成功，发现 ${res.branches?.length || 0} 个分支`)
}

async function sync(row: Repo) {
  await api.syncRepo(row.id)
  ElMessage.success('已开始拉取仓库')
  await load()
}

async function scan(row: Repo) {
  await api.scanRepo(row.id)
  ElMessage.success('已触发扫描')
}

async function remove(row: Repo) {
  await ElMessageBox.confirm(`删除仓库 ${row.repo_name}?`)
  await api.deleteRepo(row.id)
  await load()
}

function isSyncing(row: Repo) {
  return row.sync_status === 'PENDING' || row.sync_status === 'SYNCING'
}

function syncText(status?: string) {
  const map: Record<string, string> = {
    NOT_SYNCED: '未拉取',
    PENDING: '等待中',
    SYNCING: '拉取中',
    SUCCESS: '已完成',
    FAILED: '失败'
  }
  return map[status || 'NOT_SYNCED'] || status || '未拉取'
}

function syncTagType(status?: string) {
  if (status === 'SUCCESS') return 'success'
  if (status === 'FAILED') return 'danger'
  if (status === 'SYNCING' || status === 'PENDING') return 'warning'
  return 'info'
}

function updatePolling() {
  const hasSyncing = repos.value.some(isSyncing)
  if (hasSyncing && timer === undefined) {
    timer = window.setInterval(() => load(true), 2000)
  }
  if (!hasSyncing && timer !== undefined) {
    window.clearInterval(timer)
    timer = undefined
  }
}

onMounted(() => load())
onUnmounted(() => {
  if (timer !== undefined) window.clearInterval(timer)
})
</script>

<style scoped>
.sync-cell {
  display: grid;
  gap: 6px;
}

.sync-message {
  color: #6b7280;
  font-size: 12px;
  line-height: 1.3;
}
</style>
