import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export interface UpdateInfo {
  version: string;
  notes: string;
  pub_date: string;
}

export function useUpdater() {
  const isChecking = ref(false);
  const isInstalling = ref(false);
  const updateAvailable = ref(false);
  const updateInfo = ref<UpdateInfo | null>(null);
  const error = ref<string | null>(null);

  const currentVersion = ref('');

  async function getCurrentVersion(): Promise<string> {
    try {
      const version = await invoke<string>('get_app_version');
      currentVersion.value = version;
      return version;
    } catch (e) {
      console.error('Failed to get current version:', e);
      return '0.0.0';
    }
  }

  async function checkForUpdates(): Promise<boolean> {
    isChecking.value = true;
    error.value = null;
    updateAvailable.value = false;
    updateInfo.value = null;

    try {
      const result = await invoke<{ version: string; notes: string; pub_date: string } | null>('check_for_updates');

      if (result) {
        updateAvailable.value = true;
        updateInfo.value = result;
        return true;
      }

      return false;
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      console.error('Failed to check for updates:', e);
      return false;
    } finally {
      isChecking.value = false;
    }
  }

  async function installUpdate(): Promise<boolean> {
    isInstalling.value = true;
    error.value = null;

    try {
      await invoke('install_update');
      return true;
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      console.error('Failed to install update:', e);
      return false;
    } finally {
      isInstalling.value = false;
    }
  }

  const isNewerVersion = computed(() => {
    if (!updateInfo.value || !currentVersion.value) return false;

    const current = currentVersion.value.split('.').map(Number);
    const available = updateInfo.value.version.split('.').map(Number);

    for (let i = 0; i < 3; i++) {
      if ((available[i] || 0) > (current[i] || 0)) return true;
      if ((available[i] || 0) < (current[i] || 0)) return false;
    }

    return false;
  });

  return {
    isChecking,
    isInstalling,
    updateAvailable,
    updateInfo,
    error,
    currentVersion,
    isNewerVersion,
    getCurrentVersion,
    checkForUpdates,
    installUpdate,
  };
}
