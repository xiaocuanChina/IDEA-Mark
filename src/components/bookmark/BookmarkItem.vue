<script setup>
defineProps({
  bookmark: {
    type: Object,
    required: true,
  },
});

defineEmits(['delete']);
</script>

<template>
  <div class="bookmark-card">
    <div class="card-header">
      <h4>{{ bookmark.title }}</h4>
      <button class="delete-btn" @click="$emit('delete', bookmark.id)">删除</button>
    </div>
    <div class="card-meta">
      <code>{{ bookmark.file_path }}:{{ bookmark.line_number }}</code>
      <span class="date">{{ bookmark.created_at ? new Date(bookmark.created_at).toLocaleString('zh-CN') : '' }}</span>
    </div>
    <div class="card-content">{{ bookmark.content }}</div>
  </div>
</template>

<style scoped>
.bookmark-card {
  border: 1px solid #e0e0e0;
  padding: 15px;
  border-radius: 8px;
  margin-bottom: 15px;
  background: white;
  box-shadow: 0 2px 4px rgba(0,0,0,0.05);
  transition: transform 0.2s, box-shadow 0.2s;
}

.bookmark-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 8px rgba(0,0,0,0.1);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.card-header h4 {
  margin: 0;
  color: #2c3e50;
  font-size: 1.1em;
}

.card-meta {
  font-size: 0.85em;
  color: #666;
  margin-bottom: 10px;
  display: flex;
  justify-content: space-between;
  background-color: #f5f5f5;
  padding: 4px 8px;
  border-radius: 4px;
}

.card-content {
  white-space: pre-wrap;
  color: #444;
  line-height: 1.5;
}

.delete-btn {
  background-color: #ff4444;
  color: white;
  border: none;
  padding: 5px 12px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.85em;
  transition: background-color 0.2s;
}

.delete-btn:hover {
  background-color: #cc0000;
}
</style>
