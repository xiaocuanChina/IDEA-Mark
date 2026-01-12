<script setup>
defineProps({
  stats: {
    type: Object,
    required: true
  },
  loading: {
    type: Boolean,
    default: false
  },
  canRefresh: {
    type: Boolean,
    default: false
  }
});

const emit = defineEmits(['refresh']);
</script>

<template>
  <el-card class="stats-card" shadow="hover">
    <div class="stats-content">
      <div class="stats-group">
        <el-statistic title="总书签" :value="stats.total">
          <template #prefix>
            <el-icon><Collection /></el-icon>
          </template>
        </el-statistic>
        
        <el-divider direction="vertical" />
        
        <el-statistic title="项目数" :value="stats.projects">
          <template #prefix>
            <el-icon><FolderOpened /></el-icon>
          </template>
        </el-statistic>
        
        <el-divider direction="vertical" />
        
        <el-statistic title="助记符" :value="stats.withMnemonic">
          <template #prefix>
            <el-icon><Star /></el-icon>
          </template>
        </el-statistic>
      </div>
      
      <el-button 
        type="primary" 
        :icon="Refresh"
        :loading="loading"
        :disabled="!canRefresh"
        @click="emit('refresh')"
        class="refresh-btn"
      >
        刷新
      </el-button>
    </div>
  </el-card>
</template>

<script>
import { Refresh } from '@element-plus/icons-vue';
export default {
  data() {
    return { Refresh };
  }
};
</script>

<style scoped>
.stats-content {
  display: flex;
  align-items: center;
  gap: 16px;
}

.stats-group {
  display: flex;
  align-items: center;
  gap: 16px;
  flex: 1;
  min-width: 0;
}

.refresh-btn {
  flex-shrink: 0;
}

.stats-card :deep(.el-card__body) {
  padding: 12px 16px;
}

:deep(.el-statistic) {
  min-width: 70px;
}

:deep(.el-statistic__head) {
  font-size: 13px;
}

:deep(.el-statistic__content) {
  font-size: 22px;
}

:deep(.el-divider--vertical) {
  height: 36px;
}
</style>
