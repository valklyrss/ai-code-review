<template>
  <section class="dashboard-hero">
    <div>
      <p class="hero-kicker">AI Review Command Center</p>
      <h1>把每一次提交，都变成可追踪的风险信号</h1>
      <p class="hero-copy">轮询 Git 仓库、拉取 mirror、分析 diff、沉淀问题清单。这里是你的旁路审核控制台。</p>
    </div>
    <div class="hero-orbit">
      <span></span>
      <span></span>
      <strong>LIVE</strong>
    </div>
  </section>

  <div class="stats">
    <div class="stat stat-cyan"><span>仓库总数</span><b>{{ repos.length }}</b></div>
    <div class="stat stat-green"><span>今日任务</span><b>{{ todayTasks }}</b></div>
    <div class="stat stat-orange"><span>今日问题</span><b>{{ todayIssues }}</b></div>
    <div class="stat stat-red"><span>HIGH</span><b>{{ high }}</b></div>
    <div class="stat stat-blue"><span>CRITICAL</span><b>{{ critical }}</b></div>
  </div>

  <div class="panel">
    <div class="section-head">
      <div>
        <h3>最近审核任务</h3>
        <p>新的风险会在这里冒出来，像仪表盘上的红灯一样清楚。</p>
      </div>
    </div>
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

<style scoped>
.dashboard-hero {
  position: relative;
  min-height: 210px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
  margin-bottom: 16px;
  padding: 24px;
  border-radius: 8px;
  overflow: hidden;
  color: #0f172a;
  background:
    linear-gradient(135deg, rgba(18, 184, 200, 0.22), rgba(49, 196, 141, 0.16) 45%, rgba(255, 138, 76, 0.16)),
    rgba(255, 255, 255, 0.76);
  border: 1px solid rgba(255, 255, 255, 0.82);
  box-shadow: 0 18px 50px rgba(16, 24, 40, 0.12);
  backdrop-filter: blur(18px);
}

.dashboard-hero::after {
  content: "";
  position: absolute;
  inset: auto -8% -45% 35%;
  height: 190px;
  background: repeating-linear-gradient(90deg, rgba(255, 255, 255, 0.32) 0 18px, transparent 18px 36px);
  transform: rotate(-6deg);
}

.hero-kicker {
  margin: 0 0 10px;
  color: #0f766e;
  font-weight: 950;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  font-size: 12px;
}

.dashboard-hero h1 {
  position: relative;
  z-index: 1;
  max-width: 720px;
  margin: 0;
  font-size: 38px;
  line-height: 1.12;
  font-weight: 950;
}

.hero-copy {
  position: relative;
  z-index: 1;
  max-width: 680px;
  margin: 14px 0 0;
  color: #475569;
  font-weight: 700;
}

.hero-orbit {
  position: relative;
  z-index: 1;
  flex: 0 0 142px;
  width: 142px;
  height: 142px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  background: rgba(255, 255, 255, 0.55);
  border: 1px solid rgba(255, 255, 255, 0.75);
}

.hero-orbit span {
  position: absolute;
  border-radius: 50%;
  border: 2px solid rgba(18, 184, 200, 0.38);
  inset: 14px;
  animation: spin 7s linear infinite;
}

.hero-orbit span:nth-child(2) {
  inset: 30px;
  border-color: rgba(255, 138, 76, 0.45);
  animation-duration: 4.5s;
  animation-direction: reverse;
}

.hero-orbit strong {
  width: 68px;
  height: 68px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  color: white;
  background: linear-gradient(135deg, #12b8c8, #31c48d);
  box-shadow: 0 16px 36px rgba(18, 184, 200, 0.34);
}

.section-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
  margin-bottom: 12px;
}

.section-head p {
  margin: 4px 0 0;
  color: #64748b;
  font-weight: 700;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

@media (max-width: 900px) {
  .dashboard-hero {
    display: block;
  }

  .dashboard-hero h1 {
    font-size: 30px;
  }

  .hero-orbit {
    margin-top: 20px;
  }
}
</style>
