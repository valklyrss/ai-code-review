<template>
  <el-container class="app-shell">
    <el-aside width="248px" class="app-aside">
      <div class="brand-card">
        <div class="brand-mark">AI</div>
        <div>
          <div class="brand-title">代码旁路审核</div>
          <div class="brand-subtitle">Git Diff Review Studio</div>
        </div>
      </div>

      <el-menu router :default-active="$route.path" class="nav-menu">
        <el-menu-item index="/">
          <span class="nav-dot dot-cyan"></span>
          <span>驾驶舱</span>
        </el-menu-item>
        <el-menu-item index="/repos">
          <span class="nav-dot dot-green"></span>
          <span>仓库配置</span>
        </el-menu-item>
        <el-menu-item index="/tasks">
          <span class="nav-dot dot-orange"></span>
          <span>审核任务</span>
        </el-menu-item>
        <el-menu-item index="/issues">
          <span class="nav-dot dot-red"></span>
          <span>问题清单</span>
        </el-menu-item>
        <el-menu-item index="/settings/ai">
          <span class="nav-dot dot-blue"></span>
          <span>AI 设置</span>
        </el-menu-item>
        <el-menu-item index="/settings/system">
          <span class="nav-dot dot-lime"></span>
          <span>系统设置</span>
        </el-menu-item>
      </el-menu>

      <div class="aside-footer">
        <span class="pulse"></span>
        <span>旁路运行中</span>
      </div>
    </el-aside>

    <el-container class="content-shell">
      <el-header class="topbar">
        <div>
          <div class="topbar-kicker">Independent Review Platform</div>
          <div class="topbar-title">不用改 GitLab，也能把风险挡在上线前</div>
        </div>
        <div class="topbar-chip">OpenAI-compatible</div>
      </el-header>
      <el-main class="app-main">
        <router-view />
      </el-main>
    </el-container>
  </el-container>
</template>

