<script setup>
import BookmarkDashboard from "./components/BookmarkDashboard.vue";
</script>

<template>
  <el-container class="app-container">
    <el-header class="app-header">
      <div class="header-content">
        <el-icon :size="28"><Reading /></el-icon>
        <h1>Idea 书签管理</h1>
      </div>
      <p class="subtitle">自动读取并管理您的 IntelliJ IDEA 书签</p>
    </el-header>
    
    <el-main>
      <BookmarkDashboard />
    </el-main>
  </el-container>
</template>

<style scoped>
.app-container {
  min-height: 100vh;
  background: linear-gradient(180deg, #f5f7fa 0%, #e8ecf1 100%);
}

.app-header {
  text-align: center;
  padding: 30px 20px 20px;
  height: auto !important;
}

.header-content {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: #409eff;
}

.header-content h1 {
  margin: 0;
  font-size: 1.8em;
  color: #303133;
}

.subtitle {
  color: #909399;
  margin: 8px 0 0;
  font-size: 0.95em;
}

:deep(.el-main) {
  padding: 0 20px 30px;
  overflow-x: hidden;
}
</style>
