<script setup lang="ts">
import { ref, nextTick, computed } from 'vue';
import { ListTodo, Plus, Trash2, X, Check } from 'lucide-vue-next';
import { useTodoStore } from '@/stores/todo';

const todoStore = useTodoStore();

const newTodoText = ref('');
const editingId = ref<string | null>(null);
const editText = ref('');
const editInputRef = ref<HTMLInputElement | null>(null);

const sortedItems = computed(() => {
  return [...todoStore.items].sort((a, b) => {
    if (a.done === b.done) return b.createdAt - a.createdAt;
    return a.done ? 1 : -1;
  });
});

function handleAdd() {
  if (!newTodoText.value.trim()) return;
  todoStore.addTodo(newTodoText.value);
  newTodoText.value = '';
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter') {
    handleAdd();
  }
}

function startEdit(item: { id: string; text: string }) {
  editingId.value = item.id;
  editText.value = item.text;
  void nextTick(() => {
    editInputRef.value?.focus();
  });
}

function commitEdit() {
  if (editingId.value) {
    todoStore.updateText(editingId.value, editText.value);
  }
  editingId.value = null;
  editText.value = '';
}

function cancelEdit() {
  editingId.value = null;
  editText.value = '';
}

function handleEditKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter') {
    commitEdit();
  } else if (event.key === 'Escape') {
    cancelEdit();
  }
}

function clearCompleted() {
  todoStore.clearCompleted();
}
</script>

<template>
  <Teleport to="body">
    <div
      class="fixed inset-y-0 right-0 z-50 flex w-80 transform flex-col border-l border-border bg-elevated shadow-lg transition-transform duration-200 ease-out"
      :class="todoStore.isPanelOpen ? 'translate-x-0' : 'translate-x-full'"
    >
      <div class="flex h-12 items-center justify-between border-b border-border px-3">
        <div class="flex items-center gap-2 text-sm font-medium text-primary">
          <ListTodo class="h-4 w-4 text-accent" />
          {{ $t('todo.title') }}
        </div>
        <div class="flex items-center gap-1">
          <button
            v-if="todoStore.doneItems.length > 0"
            type="button"
            class="rounded-md p-1.5 text-xs text-secondary transition-colors hover:bg-raised hover:text-primary"
            :title="$t('common.clear')"
            @click="clearCompleted"
          >
            <Check class="h-4 w-4" />
          </button>
          <button
            type="button"
            class="rounded-md p-1.5 text-secondary transition-colors hover:bg-raised hover:text-primary"
            :aria-label="$t('common.close')"
            @click="todoStore.closePanel()"
          >
            <X class="h-4 w-4" />
          </button>
        </div>
      </div>

      <div class="flex-1 overflow-y-auto p-3">
        <ul v-if="sortedItems.length > 0" class="space-y-1">
          <li
            v-for="item in sortedItems"
            :key="item.id"
            class="group flex items-start gap-2 rounded-md p-1.5 transition-colors hover:bg-raised"
          >
            <input
              :id="`todo-${item.id}`"
              v-model="item.done"
              type="checkbox"
              class="mt-1 h-4 w-4 rounded border-border text-accent focus:ring-accent"
              @change="todoStore.toggleDone(item.id)"
            />
            <div class="min-w-0 flex-1">
              <input
                v-if="editingId === item.id"
                ref="editInputRef"
                v-model="editText"
                type="text"
                class="w-full rounded border border-border bg-base px-1.5 py-0.5 text-sm text-primary outline-none focus:border-accent"
                @keydown="handleEditKeydown"
                @blur="commitEdit"
              />
              <label
                v-else
                :for="`todo-${item.id}`"
                class="cursor-pointer text-sm text-secondary"
                :class="{ 'text-tertiary line-through': item.done }"
                @click="startEdit(item)"
              >
                {{ item.text }}
              </label>
            </div>
            <button
              type="button"
              class="rounded p-1 text-tertiary opacity-0 transition-colors group-hover:opacity-100 hover:text-danger"
              :title="$t('common.delete')"
              @click="todoStore.removeTodo(item.id)"
            >
              <Trash2 class="h-3.5 w-3.5" />
            </button>
          </li>
        </ul>

        <div v-else class="flex flex-1 flex-col items-center justify-center gap-2 py-8 text-secondary">
          <ListTodo class="h-10 w-10 opacity-20" />
          <p class="text-sm">{{ $t('todo.noItems') }}</p>
        </div>
      </div>

      <div class="border-t border-border p-3">
        <div class="flex gap-2">
          <input
            v-model="newTodoText"
            type="text"
            class="flex-1 rounded-md border border-border bg-base px-3 py-2 text-sm text-primary outline-none placeholder:text-tertiary focus:border-accent"
            :placeholder="$t('todo.placeholder')"
            @keydown="handleKeydown"
          />
          <button
            type="button"
            class="flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-accent text-white transition-colors hover:bg-accent-hover disabled:opacity-50"
            :disabled="!newTodoText.trim()"
            @click="handleAdd"
          >
            <Plus class="h-4 w-4" />
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
