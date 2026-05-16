<template>
  <div class="sidebar">
    <div class="header">
      <h3>飞鸽多开</h3>
      <el-button type="primary" size="small" @click="addAccount" :icon="Plus">
        添加
      </el-button>
    </div>

    <div class="account-list">
      <div
        v-for="account in accounts"
        :key="account.id"
        class="account-item"
        :class="{ active: account.id === activeId }"
        @click="switchTo(account)"
      >
        <div class="account-info">
          <el-icon :size="16" class="status-dot">
            <Monitor />
          </el-icon>
          <span class="account-name">{{ account.name }}</span>
        </div>
        <div class="account-actions">
          <el-tag
            :type="account.opened ? 'success' : 'info'"
            size="small"
            effect="dark"
          >
            {{ account.opened ? "在线" : "离线" }}
          </el-tag>
          <el-button
            v-if="!account.opened"
            type="success"
            size="small"
            text
            @click.stop="openAccount(account)"
          >
            打开
          </el-button>
          <el-button
            v-else
            type="danger"
            size="small"
            text
            @click.stop="closeAccount(account)"
          >
            关闭
          </el-button>
        </div>
      </div>
    </div>

    <div class="footer">
      <span class="count">已开 {{ openedCount }} 个</span>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Plus, Monitor } from "@element-plus/icons-vue";

const accounts = ref([]);
const activeId = ref(null);
let nextIndex = 1;

const openedCount = computed(
  () => accounts.value.filter((a) => a.opened).length
);

function addAccount() {
  const id = `shop_${nextIndex}`;
  accounts.value.push({
    name: `店铺${nextIndex}`,
    id,
    opened: false,
  });
  nextIndex++;
}

async function openAccount(account) {
  try {
    await invoke("open_account", {
      accountId: account.id,
      accountName: account.name,
    });
    account.opened = true;
    activeId.value = account.id;
  } catch (e) {
    console.error("打开失败:", e);
  }
}

async function switchTo(account) {
  if (!account.opened) return;
  try {
    await invoke("switch_account", { accountId: account.id });
    activeId.value = account.id;
  } catch (e) {
    console.error("切换失败:", e);
  }
}

async function closeAccount(account) {
  try {
    await invoke("close_account", { accountId: account.id });
    account.opened = false;
    if (activeId.value === account.id) {
      activeId.value = null;
    }
  } catch (e) {
    console.error("关闭失败:", e);
  }
}
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: "Microsoft YaHei", "PingFang SC", sans-serif;
  background: #f0f2f5;
  overflow: hidden;
}

.sidebar {
  width: 280px;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #fff;
  border-right: 1px solid #e4e7ed;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  border-bottom: 1px solid #e4e7ed;
  background: #409eff;
  color: #fff;
}

.header h3 {
  font-size: 16px;
  font-weight: 600;
}

.account-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.account-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 12px;
  margin-bottom: 4px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  border: 1px solid transparent;
}

.account-item:hover {
  background: #ecf5ff;
}

.account-item.active {
  background: #ecf5ff;
  border-color: #409eff;
}

.account-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.account-name {
  font-size: 14px;
  color: #303133;
}

.account-actions {
  display: flex;
  align-items: center;
  gap: 4px;
}

.status-dot {
  color: #909399;
}

.account-item.active .status-dot {
  color: #409eff;
}

.footer {
  padding: 12px 16px;
  border-top: 1px solid #e4e7ed;
  text-align: center;
}

.count {
  font-size: 12px;
  color: #909399;
}
</style>