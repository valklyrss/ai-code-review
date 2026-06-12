<template>
  <div class="dashboard-page">
    <section class="command-board">
      <div class="command-copy">
        <p class="eyebrow">AI REVIEW OPS</p>
        <h1>今日代码风险态势</h1>
        <p>聚合仓库、审核任务和问题等级，快速判断今天是否有值得立刻处理的高风险变更。</p>
        <div class="hero-actions">
          <el-button type="primary" @click="$router.push('/repos')">管理仓库</el-button>
          <el-button @click="$router.push('/tasks')">查看任务</el-button>
          <el-button @click="$router.push('/settings/ai')">AI 设置</el-button>
        </div>
      </div>

      <div class="risk-radar">
        <div class="radar-ring ring-a"></div>
        <div class="radar-ring ring-b"></div>
        <div class="radar-ring ring-c"></div>
        <div class="radar-sweep"></div>
        <div class="radar-core">
          <span>HIGH+</span>
          <strong>{{ high + critical }}</strong>
        </div>
      </div>
    </section>

    <div class="metric-grid">
      <div class="metric-card cyan">
        <span>仓库总数</span>
        <strong>{{ repos.length }}</strong>
        <em>已接入旁路审核</em>
      </div>
      <div class="metric-card green">
        <span>今日任务</span>
        <strong>{{ todayTasks }}</strong>
        <em>自动与手动触发</em>
      </div>
      <div class="metric-card orange">
        <span>今日问题</span>
        <strong>{{ todayIssues }}</strong>
        <em>待确认与待修复</em>
      </div>
      <div class="metric-card red">
        <span>HIGH</span>
        <strong>{{ high }}</strong>
        <em>建议优先处理</em>
      </div>
      <div class="metric-card blue">
        <span>CRITICAL</span>
        <strong>{{ critical }}</strong>
        <em>生产事故风险</em>
      </div>
    </div>

    <div class="dashboard-grid">
      <section class="panel flow-panel">
        <div class="section-head">
          <div>
            <h3>审核流水线</h3>
            <p>从 Git 发现提交，到 AI 分析，再到问题沉淀。</p>
          </div>
        </div>
        <div class="flow-line">
          <div class="flow-node active">
            <b>01</b>
            <span>拉取仓库</span>
          </div>
          <div class="flow-node active">
            <b>02</b>
            <span>计算 Diff</span>
          </div>
          <div class="flow-node">
            <b>03</b>
            <span>AI 审核</span>
          </div>
          <div class="flow-node">
            <b>04</b>
            <span>邮件告警</span>
          </div>
        </div>
      </section>

      <section class="panel health-panel">
        <div class="section-head">
          <div>
            <h3>任务分布</h3>
            <p>最近任务的结果概览。</p>
          </div>
        </div>
        <div class="health-bars">
          <div>
            <span>成功</span>
            <b>{{ successCount }}</b>
            <i :style="{ width: barWidth(successCount) }"></i>
          </div>
          <div>
            <span>失败</span>
            <b>{{ failedCount }}</b>
            <i class="danger" :style="{ width: barWidth(failedCount) }"></i>
          </div>
          <div>
            <span>等待/运行</span>
            <b>{{ pendingCount }}</b>
            <i class="warning" :style="{ width: barWidth(pendingCount) }"></i>
          </div>
        </div>
      </section>
    </div>

    <section class="panel task-panel">
      <div class="section-head">
        <div>
          <h3>最近审核任务</h3>
          <p>点击任务列表页可以继续查看文件 diff、问题和邮件状态。</p>
        </div>
        <el-button @click="$router.push('/tasks')">全部任务</el-button>
      </div>
      <el-table :data="tasks" v-loading="loading">
        <el-table-column label="创建时间" min-width="150">
          <template #default="{ row }">{{ formatTime(row.created_at) }}</template>
        </el-table-column>
        <el-table-column prop="repo_name" label="仓库" />
        <el-table-column prop="branch_name" label="分支" />
        <el-table-column label="状态">
          <template #default="{ row }"><StatusTag :value="row.status" /></template>
        </el-table-column>
        <el-table-column label="风险">
          <template #default="{ row }"><RiskTag :value="row.risk_level" /></template>
        </el-table-column>
        <el-table-column prop="issue_count" label="问题数" />
      </el-table>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { api, Repo } from '../api'
import StatusTag from '../components/StatusTag.vue'
import RiskTag from '../components/RiskTag.vue'
import { formatTime } from '../utils/time'

const repos = ref<Repo[]>([])
const tasks = ref<any[]>([])
const loading = ref(false)
const todayTasks = ref(0)
const todayIssues = ref(0)
const high = ref(0)
const critical = ref(0)

const successCount = computed(() => tasks.value.filter(t => t.status === 'SUCCESS').length)
const failedCount = computed(() => tasks.value.filter(t => t.status === 'FAILED').length)
const pendingCount = computed(() => tasks.value.filter(t => t.status === 'WAITING' || t.status === 'RUNNING').length)
const maxTaskBucket = computed(() => Math.max(successCount.value, failedCount.value, pendingCount.value, 1))

function barWidth(value: number) {
  return `${Math.max(10, Math.round((value / maxTaskBucket.value) * 100))}%`
}

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
.dashboard-page {
  display: grid;
  gap: 16px;
}

.command-board {
  position: relative;
  min-height: 260px;
  display: grid;
  grid-template-columns: minmax(0, 1fr) 300px;
  gap: 24px;
  align-items: center;
  overflow: hidden;
  padding: 26px;
  border-radius: 8px;
  background:
    linear-gradient(135deg, rgba(18, 184, 200, 0.22), rgba(63, 131, 248, 0.14) 44%, rgba(255, 138, 76, 0.18)),
    rgba(255, 255, 255, 0.78);
  border: 1px solid rgba(255, 255, 255, 0.86);
  box-shadow: 0 20px 54px rgba(16, 24, 40, 0.12);
  backdrop-filter: blur(18px);
}

.command-board::before {
  content: "";
  position: absolute;
  width: 520px;
  height: 520px;
  right: -180px;
  top: -210px;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(49, 196, 141, 0.32), transparent 62%);
}

.command-board::after {
  content: "";
  position: absolute;
  left: 22px;
  right: 22px;
  bottom: 18px;
  height: 54px;
  background: repeating-linear-gradient(90deg, rgba(255,255,255,0.38) 0 16px, transparent 16px 34px);
  transform: skewX(-14deg);
  opacity: 0.55;
}

.command-copy {
  position: relative;
  z-index: 1;
}

.eyebrow {
  margin: 0 0 12px;
  color: #0f766e;
  font-size: 12px;
  font-weight: 950;
  letter-spacing: 0.1em;
}

.command-copy h1 {
  max-width: 620px;
  margin: 0;
  font-size: 44px;
  line-height: 1.08;
  font-weight: 950;
}

.command-copy p:not(.eyebrow) {
  max-width: 620px;
  margin: 14px 0 0;
  color: #475569;
  font-weight: 800;
}

.hero-actions {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
  margin-top: 22px;
}

.risk-radar {
  position: relative;
  z-index: 1;
  width: 240px;
  height: 240px;
  justify-self: center;
  border-radius: 50%;
  display: grid;
  place-items: center;
  background:
    radial-gradient(circle, rgba(255,255,255,0.92) 0 24%, rgba(255,255,255,0.45) 25% 100%),
    conic-gradient(from 45deg, rgba(18,184,200,0.2), rgba(49,196,141,0.2), rgba(255,138,76,0.22), rgba(18,184,200,0.2));
  border: 1px solid rgba(255, 255, 255, 0.82);
}

.radar-ring {
  position: absolute;
  border-radius: 50%;
  border: 1px solid rgba(18, 184, 200, 0.32);
}

.ring-a { inset: 22px; }
.ring-b { inset: 58px; border-color: rgba(255, 138, 76, 0.36); }
.ring-c { inset: 94px; border-color: rgba(49, 196, 141, 0.38); }

.radar-sweep {
  position: absolute;
  inset: 8px;
  border-radius: 50%;
  background: conic-gradient(from 0deg, rgba(18,184,200,0.32), transparent 28%);
  animation: radarSpin 5s linear infinite;
}

.radar-core {
  position: relative;
  z-index: 1;
  width: 96px;
  height: 96px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  align-content: center;
  color: white;
  background: linear-gradient(135deg, #12b8c8, #3f83f8);
  box-shadow: 0 18px 42px rgba(18, 184, 200, 0.32);
}

.radar-core span {
  font-size: 12px;
  font-weight: 950;
}

.radar-core strong {
  font-size: 34px;
  line-height: 1;
}

.metric-grid {
  display: grid;
  grid-template-columns: repeat(5, minmax(140px, 1fr));
  gap: 12px;
}

.metric-card {
  position: relative;
  overflow: hidden;
  min-height: 132px;
  padding: 16px;
  border-radius: 8px;
  background: rgba(255,255,255,0.78);
  border: 1px solid rgba(255,255,255,0.86);
  box-shadow: 0 14px 34px rgba(16, 24, 40, 0.08);
}

.metric-card::before {
  content: "";
  position: absolute;
  right: -28px;
  top: -28px;
  width: 96px;
  height: 96px;
  border-radius: 50%;
  background: color-mix(in srgb, var(--tone) 24%, transparent);
}

.metric-card span,
.metric-card em,
.metric-card strong {
  position: relative;
  z-index: 1;
}

.metric-card span {
  display: block;
  color: #64748b;
  font-weight: 900;
}

.metric-card strong {
  display: block;
  margin: 12px 0 8px;
  font-size: 34px;
  line-height: 1;
}

.metric-card em {
  color: #64748b;
  font-style: normal;
  font-weight: 700;
  font-size: 12px;
}

.cyan { --tone: #12b8c8; }
.green { --tone: #31c48d; }
.orange { --tone: #ff8a4c; }
.red { --tone: #f05252; }
.blue { --tone: #3f83f8; }

.dashboard-grid {
  display: grid;
  grid-template-columns: minmax(0, 1.25fr) minmax(300px, 0.75fr);
  gap: 16px;
}

.section-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
  margin-bottom: 14px;
}

.section-head p {
  margin: 4px 0 0;
  color: #64748b;
  font-weight: 700;
}

.flow-line {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 10px;
}

.flow-node {
  position: relative;
  min-height: 96px;
  padding: 14px;
  border-radius: 8px;
  background: rgba(248, 250, 252, 0.78);
  border: 1px solid rgba(15, 23, 42, 0.08);
}

.flow-node::after {
  content: "";
  position: absolute;
  top: 28px;
  right: -10px;
  width: 10px;
  height: 2px;
  background: rgba(18, 184, 200, 0.35);
}

.flow-node:last-child::after {
  display: none;
}

.flow-node b {
  display: inline-grid;
  place-items: center;
  width: 34px;
  height: 34px;
  border-radius: 50%;
  color: white;
  background: #94a3b8;
}

.flow-node.active b {
  background: linear-gradient(135deg, #12b8c8, #31c48d);
}

.flow-node span {
  display: block;
  margin-top: 12px;
  font-weight: 900;
}

.health-bars {
  display: grid;
  gap: 14px;
}

.health-bars div {
  position: relative;
  padding-bottom: 16px;
}

.health-bars span {
  color: #64748b;
  font-weight: 800;
}

.health-bars b {
  float: right;
}

.health-bars i {
  position: absolute;
  left: 0;
  bottom: 0;
  height: 8px;
  border-radius: 999px;
  background: linear-gradient(90deg, #12b8c8, #31c48d);
}

.health-bars i.danger {
  background: linear-gradient(90deg, #f05252, #ff8a4c);
}

.health-bars i.warning {
  background: linear-gradient(90deg, #ff8a4c, #faca15);
}

.task-panel {
  margin-bottom: 12px;
}

@keyframes radarSpin {
  to {
    transform: rotate(360deg);
  }
}

@media (max-width: 1100px) {
  .command-board,
  .dashboard-grid {
    grid-template-columns: 1fr;
  }

  .risk-radar {
    justify-self: start;
  }

  .metric-grid {
    grid-template-columns: repeat(2, 1fr);
  }

  .flow-line {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>
