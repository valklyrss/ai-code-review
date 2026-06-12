<template>
  <h1 class="page-title">仓库详情</h1>

  <div class="toolbar">
    <el-button @click="$router.push('/repos')">返回</el-button>
    <el-select v-model="branch" placeholder="选择分支" style="width: 240px" @change="loadCommits">
      <el-option v-for="item in branches" :key="item.branch_name" :label="item.branch_name" :value="item.branch_name" />
    </el-select>
    <el-input-number v-model="limit" :min="20" :max="500" :step="20" />
    <el-button @click="loadCommits">刷新</el-button>
  </div>

  <div class="panel" v-loading="loading">
    <el-descriptions v-if="repo" :column="2" border style="margin-bottom: 16px">
      <el-descriptions-item label="仓库名称">{{ repo.repo_name }}</el-descriptions-item>
      <el-descriptions-item label="仓库地址">{{ repo.repo_url }}</el-descriptions-item>
      <el-descriptions-item label="拉取状态">{{ repo.sync_status }}</el-descriptions-item>
      <el-descriptions-item label="当前分支">{{ branch || '-' }}</el-descriptions-item>
    </el-descriptions>

    <el-table :data="commits" row-key="commit_id">
      <el-table-column label="提交树" min-width="360">
        <template #default="{ row }">
          <div class="commit-tree">
            <span class="graph">{{ row.graph }}</span>
            <code>{{ row.short_id }}</code>
            <span class="subject">{{ row.subject }}</span>
          </div>
        </template>
      </el-table-column>
      <el-table-column label="作者" min-width="180">
        <template #default="{ row }">
          <div>{{ row.author_name }}</div>
          <div class="muted">{{ row.author_email }}</div>
        </template>
      </el-table-column>
      <el-table-column prop="commit_time" label="提交时间" min-width="190" />
      <el-table-column label="扫描记录" min-width="220">
        <template #default="{ row }">
          <div v-if="tasksByCommit[row.commit_id]?.length" class="task-list">
            <el-tag
              v-for="task in tasksByCommit[row.commit_id]"
              :key="task.id"
              class="task-tag"
              :type="task.status === 'SUCCESS' ? 'success' : task.status === 'FAILED' ? 'danger' : 'warning'"
              @click="$router.push('/tasks/' + task.id)"
            >
              {{ task.status }} / {{ task.result || '-' }} / {{ task.risk_level || '-' }}
            </el-tag>
          </div>
          <span v-else class="muted">未扫描</span>
        </template>
      </el-table-column>
      <el-table-column label="操作" width="120">
        <template #default="{ row }">
          <el-button size="small" type="primary" @click="scanCommit(row)">扫描</el-button>
        </template>
      </el-table-column>
    </el-table>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import { ElMessage } from 'element-plus'
import { api, Repo } from '../api'

interface Branch {
  branch_name: string
  commit_id: string
}

interface GraphCommit {
  graph: string
  commit_id: string
  short_id: string
  author_name: string
  author_email: string
  commit_time: string
  subject: string
}

const route = useRoute()
const repoId = computed(() => route.params.id as string)
const repo = ref<Repo | null>(null)
const branches = ref<Branch[]>([])
const branch = ref('')
const limit = ref(100)
const commits = ref<GraphCommit[]>([])
const tasks = ref<any[]>([])
const loading = ref(false)

const tasksByCommit = computed(() => {
  const map: Record<string, any[]> = {}
  for (const task of tasks.value) {
    if (!map[task.new_commit_id]) map[task.new_commit_id] = []
    map[task.new_commit_id].push(task)
  }
  return map
})

async function loadCommits() {
  loading.value = true
  try {
    const res: any = await api.repoCommits(repoId.value, { branch: branch.value || undefined, limit: limit.value })
    repo.value = res.repo
    branches.value = res.branches || []
    branch.value = res.selected_branch || branch.value
    commits.value = res.commits || []
    tasks.value = res.tasks || []
  } finally {
    loading.value = false
  }
}

async function scanCommit(row: GraphCommit) {
  const res: any = await api.scanCommit(repoId.value, row.commit_id, { branch: branch.value })
  ElMessage.success('已创建审核任务')
  tasks.value.unshift({
    id: res.task_id,
    new_commit_id: row.commit_id,
    status: 'WAITING',
    result: null,
    risk_level: null
  })
}

onMounted(loadCommits)
</script>

<style scoped>
.commit-tree {
  display: flex;
  align-items: center;
  gap: 8px;
  font-family: ui-monospace, SFMono-Regular, Consolas, "Liberation Mono", monospace;
  white-space: pre;
}

.graph {
  color: #2563eb;
  min-width: 42px;
}

.subject {
  white-space: normal;
}

.muted {
  color: #6b7280;
  font-size: 12px;
}

.task-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.task-tag {
  cursor: pointer;
}
</style>
