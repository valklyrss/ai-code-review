<template>
  <el-container class="layout">
    <el-aside width="220px" class="aside">
      <div class="brand">AI 代码旁路审核</div>
      <el-menu router :default-active="$route.path">
        <el-menu-item index="/">Dashboard</el-menu-item>
        <el-menu-item index="/repos">仓库配置</el-menu-item>
        <el-menu-item index="/tasks">审核任务</el-menu-item>
        <el-menu-item index="/issues">问题清单</el-menu-item>
      </el-menu>
    </el-aside>
    <el-main>
      <router-view />
    </el-main>
  </el-container>
</template>

