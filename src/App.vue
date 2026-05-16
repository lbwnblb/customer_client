<template>
  <div class="container">
    <h1>飞鸽客服多开管理</h1>
    <el-button type="primary" @click="addAccount">添加账号</el-button>
    <div class="account-list">
      <el-card v-for="(account, index) in accounts" :key="index" class="account-card">
        <span>{{ account.name }}</span>
        <div class="card-actions">
          <el-tag :type="account.opened ? 'success' : 'info'" size="small">
            {{ account.opened ? '已打开' : '未打开' }}
          </el-tag>
          <el-button type="success" size="small" @click="openWindow(account)">打开</el-button>
        </div>
      </el-card>
    </div>
  </div>
</template>

<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const accounts = ref([
  { name: "店铺1", id: "shop_1", opened: false },
  { name: "店铺2", id: "shop_2", opened: false },
]);

function addAccount() {
  const index = accounts.value.length + 1;
  accounts.value.push({ name: `店铺${index}`, id: `shop_${index}`, opened: false });
}

async function openWindow(account) {
  try {
    const result = await invoke("open_workspace_window", {
      accountId: account.id,
      accountName: account.name,
    });
    account.opened = true;
    console.log(result);
  } catch (error) {
    console.error("打开窗口失败:", error);
  }
}
</script>

<style scoped>
.container { padding: 20px; }
.account-list { margin-top: 20px; }
.account-card { margin-bottom: 10px; }
.account-card :deep(.el-card__body) { display: flex; justify-content: space-between; align-items: center; }
.card-actions { display: flex; align-items: center; gap: 10px; }
</style>
