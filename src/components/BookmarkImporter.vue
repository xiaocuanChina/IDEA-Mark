<script setup>
import { ref } from 'vue';

const projectPath = ref('');
const emit = defineEmits(['import']);

function handleImport() {
  if (!projectPath.value) {
    alert('请输入Idea项目路径或bookmarks.xml文件路径');
    return;
  }
  emit('import', projectPath.value);
}
</script>

<template>
  <div class="importer-section">
    <h3>导入 Idea 书签</h3>
    <div class="import-controls">
      <input 
        v-model="projectPath" 
        placeholder="例如: C:/Users/Dev/Project/.idea/bookmarks.xml"
        class="path-input"
      />
      <button @click="handleImport" class="import-btn">导入 / 同步</button>
    </div>
    <p class="hint">输入 IntelliJ IDEA 项目的 bookmarks.xml 路径以导入书签。</p>
  </div>
</template>

<style scoped>
.importer-section {
  background: #fff;
  padding: 20px;
  border-radius: 8px;
  border: 1px solid #eee;
  margin-bottom: 20px;
  box-shadow: 0 1px 3px rgba(0,0,0,0.05);
}

h3 {
  margin-top: 0;
  margin-bottom: 15px;
  font-size: 1.1em;
  color: #333;
}

.import-controls {
  display: flex;
  gap: 10px;
}

.path-input {
  flex: 1;
  padding: 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
}

.import-btn {
  padding: 10px 20px;
  background-color: #3498db;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  white-space: nowrap;
}

.import-btn:hover {
  background-color: #2980b9;
}

.hint {
  font-size: 0.85em;
  color: #888;
  margin-top: 10px;
  margin-bottom: 0;
}
</style>
