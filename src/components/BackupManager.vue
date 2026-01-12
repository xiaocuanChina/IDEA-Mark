<script setup>
import { ref, watch, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { ElMessage, ElMessageBox } from "element-plus";
import BookmarkViewer from "./BookmarkViewer.vue";

const props = defineProps({
  workspacePath: {
    type: String,
    required: true
  }
});

const emit = defineEmits(["restored"]);

const backups = ref([]);
const workspaceFiles = ref([]);
const loading = ref(false);
const backingUp = ref(false);
const projectNames = ref([]);
const selectedProjects = ref([]);
const showBackupDialog = ref(false);
const showDetailDialog = ref(false);
const detailLoading = ref(false);
const detailBookmarks = ref([]);
const detailGroup = ref(null);

// 按时间戳分组备份
const groupedBackups = computed(() => {
  const groups = {};
  
  for (const backup of backups.value) {
    const timestampKey = backup.id.substring(0, 15);
    
    if (!groups[timestampKey]) {
      groups[timestampKey] = {
        timestamp: backup.timestamp,
        timestampKey,
        files: [],
        projects: new Set()
      };
    }
    groups[timestampKey].files.push(backup);
    backup.projects?.forEach(p => groups[timestampKey].projects.add(p));
  }
  
  for (const group of Object.values(groups)) {
    group.projectList = Array.from(group.projects);
  }
  
  return Object.values(groups).sort((a, b) => b.timestampKey.localeCompare(a.timestampKey));
});

async function loadWorkspaceFiles() {
  if (!props.workspacePath) return;
  
  try {
    workspaceFiles.value = await invoke("list_workspace_files", { 
      workspacePath: props.workspacePath 
    });
  } catch (error) {
    console.error("[BackupManager] Failed to load workspace files:", error);
  }
}

async function loadProjectNames() {
  if (!props.workspacePath) return;
  
  try {
    const bookmarks = await invoke("read_bookmarks_from_workspace", {
      workspacePath: props.workspacePath
    });
    const names = [...new Set(bookmarks.map(b => b.project_name).filter(Boolean))];
    projectNames.value = names;
    selectedProjects.value = [...names];
  } catch (error) {
    console.error("[BackupManager] Failed to load project names:", error);
  }
}

async function loadBackups() {
  if (!props.workspacePath) return;
  
  loading.value = true;
  try {
    const allBackups = await invoke("get_backup_list");
    const fileNames = workspaceFiles.value.map(f => f.name);
    backups.value = allBackups.filter(b => fileNames.includes(b.original_file_name));
  } catch (error) {
    console.error("[BackupManager] Failed to load backups:", error);
  } finally {
    loading.value = false;
  }
}

function openBackupDialog() {
  if (projectNames.value.length === 0) {
    ElMessage.warning("没有可备份的项目");
    return;
  }
  selectedProjects.value = [...projectNames.value];
  showBackupDialog.value = true;
}

async function handleBackup() {
  if (selectedProjects.value.length === 0) {
    ElMessage.warning("请至少选择一个项目");
    return;
  }
  
  showBackupDialog.value = false;
  backingUp.value = true;
  
  try {
    let successCount = 0;
    for (const file of workspaceFiles.value) {
      await invoke("backup_bookmark_file", { 
        filePath: file.path,
        projects: selectedProjects.value
      });
      successCount++;
    }
    await loadBackups();
    ElMessage.success(`成功备份 ${selectedProjects.value.length} 个项目`);
  } catch (error) {
    console.error("[BackupManager] Backup failed:", error);
    ElMessage.error("备份失败: " + error);
  } finally {
    backingUp.value = false;
  }
}

async function handleViewDetail(group) {
  detailGroup.value = group;
  detailBookmarks.value = [];
  showDetailDialog.value = true;
  detailLoading.value = true;
  
  try {
    const allBookmarks = [];
    for (const backup of group.files) {
      const bookmarks = await invoke("read_backup_bookmarks", { 
        backupPath: backup.path 
      });
      allBookmarks.push(...bookmarks);
    }
    detailBookmarks.value = allBookmarks;
  } catch (error) {
    console.error("[BackupManager] Failed to load backup detail:", error);
    ElMessage.error("加载备份详情失败: " + error);
  } finally {
    detailLoading.value = false;
  }
}

async function handleRestoreGroup(group) {
  try {
    const isRunning = await invoke("check_idea_running");
    if (isRunning) {
      await ElMessageBox.alert(
        "检测到 IntelliJ IDEA 正在运行！\n\n请先关闭 IDEA 后再进行还原操作，否则可能导致配置文件损坏或还原失败。",
        "IDEA 正在运行",
        {
          confirmButtonText: "我知道了",
          type: "warning"
        }
      );
      return;
    }
  } catch (error) {
    console.error("[BackupManager] Failed to check IDEA status:", error);
  }

  const projectText = group.projectList.length > 0 
    ? group.projectList.join('、') 
    : '未知项目';
    
  try {
    await ElMessageBox.confirm(
      `确定要还原 ${group.timestamp} 的备份吗？\n\n涉及项目：${projectText}\n\n警告：这将覆盖当前的配置文件。`,
      "确认还原",
      {
        confirmButtonText: "确定还原",
        cancelButtonText: "取消",
        type: "warning"
      }
    );
    
    let successCount = 0;
    for (const backup of group.files) {
      const targetFile = workspaceFiles.value.find(f => f.name === backup.original_file_name);
      if (targetFile) {
        await invoke("restore_bookmark_file", { 
          backupPath: backup.path,
          targetPath: targetFile.path 
        });
        successCount++;
      }
    }
    
    ElMessage.success(`还原成功！请重启 IDEA 以生效。`);
    emit("restored");
  } catch (error) {
    if (error !== "cancel") {
      console.error("[BackupManager] Restore failed:", error);
      ElMessage.error("还原失败: " + error);
    }
  }
}

async function handleDeleteGroup(group) {
  try {
    await ElMessageBox.confirm(
      `确定要删除 ${group.timestamp} 的备份吗？`,
      "确认删除",
      {
        confirmButtonText: "删除",
        cancelButtonText: "取消",
        type: "warning"
      }
    );
    
    for (const backup of group.files) {
      await invoke("delete_backup_file", { backupPath: backup.path });
    }
    await loadBackups();
    ElMessage.success("备份已删除");
  } catch (error) {
    if (error !== "cancel") {
      console.error("[BackupManager] Delete failed:", error);
      ElMessage.error("删除失败: " + error);
    }
  }
}

async function init() {
  await loadWorkspaceFiles();
  await Promise.all([loadBackups(), loadProjectNames()]);
}

watch(() => props.workspacePath, init);
onMounted(init);
</script>

<template>
  <div class="backup-manager">
    <div class="header">
      <div class="title-section">
        <el-icon :size="20"><Files /></el-icon>
        <h4>备份与还原</h4>
        <el-tag v-if="projectNames.length > 0" size="small" type="success">
          {{ projectNames.length }} 个项目
        </el-tag>
      </div>
      <el-button 
        type="primary" 
        @click="openBackupDialog" 
        :loading="backingUp"
        :disabled="projectNames.length === 0"
      >
        <el-icon><Upload /></el-icon>
        立即备份
      </el-button>
    </div>

    <el-divider />

    <div v-if="loading" class="loading">
      <el-skeleton :rows="3" animated />
    </div>
    
    <el-empty v-else-if="groupedBackups.length === 0" description="暂无备份记录" :image-size="60" />

    <div v-else class="backup-list">
      <div v-for="group in groupedBackups" :key="group.timestampKey" class="backup-group">
        <div class="group-header">
          <div class="group-info">
            <el-icon><Clock /></el-icon>
            <span class="group-time">{{ group.timestamp }}</span>
          </div>
          <div class="group-actions">
            <el-button size="small" @click="handleViewDetail(group)">
              <el-icon><View /></el-icon>
              查看
            </el-button>
            <el-button type="primary" size="small" @click="handleRestoreGroup(group)">
              <el-icon><RefreshLeft /></el-icon>
              还原
            </el-button>
            <el-button type="danger" size="small" @click="handleDeleteGroup(group)">
              <el-icon><Delete /></el-icon>
            </el-button>
          </div>
        </div>
        <div v-if="group.projectList.length > 0" class="group-projects">
          <el-tag 
            v-for="name in group.projectList" 
            :key="name" 
            size="small"
            effect="plain"
            type="primary"
          >
            {{ name }}
          </el-tag>
        </div>
        <div v-else class="group-projects">
          <el-tag size="small" type="info">旧版备份（无项目信息）</el-tag>
        </div>
      </div>
    </div>

    <div class="tips">
      <el-icon><InfoFilled /></el-icon>
      <span>提示：还原前请确保 IDEA 已关闭，还原后需重启 IDEA 生效</span>
    </div>

    <!-- 备份选择对话框 -->
    <el-dialog v-model="showBackupDialog" title="选择要备份的项目" width="400px">
      <el-checkbox-group v-model="selectedProjects">
        <el-checkbox 
          v-for="name in projectNames" 
          :key="name" 
          :value="name"
          :label="name"
          style="display: block; margin-bottom: 10px;"
        />
      </el-checkbox-group>
      <div class="dialog-actions">
        <el-button size="small" @click="selectedProjects = [...projectNames]">全选</el-button>
        <el-button size="small" @click="selectedProjects = []">清空</el-button>
      </div>
      <template #footer>
        <el-button @click="showBackupDialog = false">取消</el-button>
        <el-button type="primary" @click="handleBackup" :disabled="selectedProjects.length === 0">
          备份 ({{ selectedProjects.length }})
        </el-button>
      </template>
    </el-dialog>

    <!-- 备份详情对话框 -->
    <el-dialog 
      v-model="showDetailDialog" 
      :title="`备份详情 - ${detailGroup?.timestamp || ''}`" 
      width="700px"
      top="5vh"
    >
      <div v-if="detailLoading" class="detail-loading">
        <el-skeleton :rows="5" animated />
      </div>
      <div v-else class="detail-content">
        <BookmarkViewer :bookmarks="detailBookmarks" />
      </div>
      <template #footer>
        <el-button @click="showDetailDialog = false">关闭</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
.backup-manager {
  padding: 5px;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.title-section {
  display: flex;
  align-items: center;
  gap: 8px;
}

h4 {
  margin: 0;
  color: #303133;
  font-size: 16px;
}

.el-divider {
  margin: 15px 0;
}

.backup-list {
  max-height: 300px;
  overflow-y: auto;
}

.backup-group {
  padding: 12px 15px;
  margin-bottom: 8px;
  background: #f5f7fa;
  border-radius: 6px;
  border: 1px solid #e4e7ed;
}

.backup-group:last-child {
  margin-bottom: 0;
}

.group-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.group-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.group-time {
  font-weight: 500;
  color: #303133;
}

.group-actions {
  display: flex;
  gap: 8px;
}

.group-projects {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 10px;
  padding-top: 10px;
  border-top: 1px dashed #dcdfe6;
}

.tips {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 15px;
  padding: 10px;
  background: #f4f4f5;
  border-radius: 4px;
  font-size: 13px;
  color: #909399;
}

.dialog-actions {
  margin-top: 15px;
  padding-top: 10px;
  border-top: 1px solid #ebeef5;
}

/* 详情对话框样式 */
.detail-loading {
  min-height: 200px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.detail-content {
  max-height: 60vh;
  overflow-y: auto;
}

/* 修复全局按钮样式影响弹窗关闭按钮的问题 */
:deep(.el-dialog__headerbtn),
:deep(.el-dialog__headerbtn:hover),
:deep(.el-dialog__headerbtn:active) {
  border: none !important;
  box-shadow: none !important;
  background-color: transparent !important;
  outline: none !important;
}
</style>
