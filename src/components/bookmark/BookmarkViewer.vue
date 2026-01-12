<script setup>
import { computed } from "vue";

const props = defineProps({
  bookmarks: {
    type: Array,
    required: true,
  },
});

// 按项目分组，再按文件分组
const groupedBookmarks = computed(() => {
  const groups = {};
  props.bookmarks.forEach(b => {
    const proj = b.project_name || '未知项目';
    if (!groups[proj]) {
      groups[proj] = { files: {}, count: 0 };
    }
    const filePath = b.file_path || '未知文件';
    if (!groups[proj].files[filePath]) {
      groups[proj].files[filePath] = [];
    }
    groups[proj].files[filePath].push(b);
    groups[proj].count++;
  });
  return groups;
});

// 获取助记符显示
function getMnemonicTag(mnemonic) {
  if (!mnemonic) return null;
  const num = parseInt(mnemonic);
  if (!isNaN(num) && num >= 0 && num <= 9) {
    return ['⓪', '①', '②', '③', '④', '⑤', '⑥', '⑦', '⑧', '⑨'][num];
  }
  return mnemonic;
}

// 截取文件名
function getFileName(filePath) {
  if (!filePath) return '';
  return filePath.split(/[/\\]/).pop();
}
</script>

<template>
  <div class="bookmark-viewer">
    <el-empty v-if="bookmarks.length === 0" description="暂无书签数据，请选择 IDEA 版本" />
    
    <el-collapse v-else v-model="activeNames" accordion>
      <el-collapse-item 
        v-for="(group, projectName) in groupedBookmarks" 
        :key="projectName"
        :name="projectName"
      >
        <template #title>
          <div class="project-title">
            <el-icon><Folder /></el-icon>
            <span class="project-name">{{ projectName }}</span>
            <el-tag size="small" type="info">{{ group.count }}</el-tag>
          </div>
        </template>
        
        <div class="file-groups">
          <div v-for="(bookmarkList, filePath) in group.files" :key="filePath" class="file-group">
            <div class="file-header">
              <el-icon><Document /></el-icon>
              <span class="file-path">{{ getFileName(filePath) }}</span>
              <el-tag size="small" type="success" v-if="bookmarkList.length > 1">{{ bookmarkList.length }}</el-tag>
            </div>
            <div class="bookmark-items">
              <div v-for="(bookmark, idx) in bookmarkList" :key="idx" class="bookmark-row">
                <el-tag size="small" type="primary" class="line-tag">行 {{ bookmark.line_number }}</el-tag>
                <span v-if="bookmark.mnemonic" class="mnemonic">{{ getMnemonicTag(bookmark.mnemonic) }}</span>
                <span class="description">{{ bookmark.description || '无描述' }}</span>
              </div>
            </div>
          </div>
        </div>
      </el-collapse-item>
    </el-collapse>
  </div>
</template>

<script>
export default {
  data() {
    return {
      activeNames: []
    };
  },
  watch: {
    bookmarks: {
      immediate: true,
      handler(val) {
        if (val && val.length > 0) {
          const firstProject = val[0]?.project_name || '未知项目';
          this.activeNames = firstProject;
        }
      }
    }
  }
};
</script>

<style scoped>
.bookmark-viewer {
  width: 100%;
  padding: 10px;
  box-sizing: border-box;
  overflow: hidden; /* Hide scrollbar, ensure content fits via box-sizing */
}

.project-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 500;
  flex: 1; /* Occupy remaining space */
  min-width: 0; /* Allow flex item to shrink below content size */
  overflow: hidden;
}

.project-name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-groups {
  padding: 10px;
}

.file-group {
  margin-bottom: 12px;
  border: 1px solid #ebeef5;
  border-radius: 6px;
  overflow: hidden;
}

.file-group:last-child {
  margin-bottom: 0;
}

.file-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: #f5f7fa;
  border-bottom: 1px solid #ebeef5;
}

.file-path {
  flex: 1;
  color: #409eff;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.bookmark-items {
  padding: 8px 12px;
}

.bookmark-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 0;
  border-bottom: 1px dashed #ebeef5;
}

.bookmark-row:last-child {
  border-bottom: none;
}

.line-tag {
  flex-shrink: 0;
}

.mnemonic {
  font-size: 1.2em;
  color: #e6a23c;
  flex-shrink: 0;
}

.description {
  flex: 1;
  color: #606266;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

:deep(.el-collapse-item__header) {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  padding: 0 15px;
  border-radius: 6px;
  height: 48px;
  overflow: visible;
  box-sizing: border-box; /* Ensure padding is included in width */
}

:deep(.el-collapse-item__header.is-active) {
  border-radius: 6px 6px 0 0;
}

:deep(.el-collapse-item__header .el-collapse-item__arrow) {
  color: white;
  margin-left: auto;
  flex-shrink: 0;
}

:deep(.el-collapse-item__wrap) {
  border: 1px solid #ebeef5;
  border-top: none;
  border-radius: 0 0 6px 6px;
  overflow: hidden;
}

:deep(.el-collapse-item__content) {
  padding: 0;
}

:deep(.el-collapse-item) {
  margin-bottom: 10px;
  border: none;
}

:deep(.el-collapse) {
  border: none;
}
</style>
