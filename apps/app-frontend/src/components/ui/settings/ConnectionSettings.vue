<script setup lang="ts">
import { Toggle } from '@erteam/ui'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import {
	connectionSettings,
	measureRealPing,
	refreshAllPings,
	type ConnectionItem,
} from '@/store/launcherPreferences'
import i18n from '@/i18n.config'

const isRu = computed(() => (i18n.global.locale.value || '').startsWith('ru'))

const showAddModal = ref(false)
const newServerTitle = ref('')
const newServerHost = ref('')
const newServerType = ref<'direct' | 'system' | 'custom'>('custom')
const isCheckingPing = ref(false)

function onGlobalKeydown(e: KeyboardEvent) {
	if (e.key === 'Escape' && showAddModal.value) {
		closeAddModal()
	}
}

onMounted(async () => {
	window.addEventListener('keydown', onGlobalKeydown)
	await refreshAllPings()
})

onUnmounted(() => {
	window.removeEventListener('keydown', onGlobalKeydown)
})

async function handleAddServer() {
	if (!newServerHost.value.trim()) return
	const host = newServerHost.value.trim()
	const title = newServerTitle.value.trim() || host
	isCheckingPing.value = true
	const realPing = await measureRealPing(host)
	isCheckingPing.value = false

	const newItem: ConnectionItem = {
		id: 'custom_' + Date.now(),
		title,
		subtitle: host,
		host,
		locked: false,
		pingMs: realPing,
		active: false,
		type: newServerType.value,
	}

	connectionSettings.value.items.push(newItem)
	newServerTitle.value = ''
	newServerHost.value = ''
	showAddModal.value = false
}

function selectConnection(item: ConnectionItem) {
	connectionSettings.value.items.forEach((c) => {
		c.active = c.id === item.id
	})
}

function removeConnection(id: string) {
	const idx = connectionSettings.value.items.findIndex((c) => c.id === id)
	if (idx !== -1 && !connectionSettings.value.items[idx].locked) {
		connectionSettings.value.items.splice(idx, 1)
	}
}

function closeAddModal() {
	showAddModal.value = false
}
</script>

<template>
	<div class="flex flex-col gap-6 max-w-2xl select-none">
		<div class="flex items-center gap-2 text-sm text-[var(--er-text-secondary)]">
			<span>{{ isRu ? 'Настройки' : 'Settings' }}</span>
			<span>&gt;</span>
			<div class="flex items-center gap-1.5 text-[var(--er-text)] font-medium">
				<svg class="w-4 h-4 text-gray-300" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
					<circle cx="12" cy="12" r="10"></circle>
					<line x1="2" y1="12" x2="22" y2="12"></line>
					<path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path>
				</svg>
				<span>{{ isRu ? 'Подключения' : 'Connections' }}</span>
			</div>
		</div>

		<div>
			<div class="flex items-center gap-2">
				<svg class="w-5 h-5 text-gray-300" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
					<polyline points="7 15 12 20 17 15"></polyline>
					<polyline points="7 9 12 4 17 9"></polyline>
				</svg>
				<h2 class="m-0 text-xl font-bold text-contrast">
					{{ isRu ? 'Тип подключения' : 'Connection type' }}
				</h2>
			</div>
			<p class="m-0 mt-1.5 text-sm text-secondary leading-relaxed">
				{{ isRu ? 'Метод подключения лаунчера к сети для получения обновлений контента и загрузки версий.' : 'How the launcher connects to the network to fetch content updates and version files.' }}
			</p>
		</div>

		<div class="px-4 py-3 rounded-2xl bg-surface-2 border border-divider flex items-center gap-3 text-sm text-secondary">
			<svg class="w-5 h-5 text-secondary shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
				<circle cx="12" cy="12" r="10"></circle>
				<line x1="12" y1="16" x2="12" y2="12"></line>
				<line x1="12" y1="8" x2="12.01" y2="8"></line>
			</svg>
			<span>{{ isRu ? 'Не используется в игре, только для лаунчера.' : 'Not used in the Minecraft game, only for the launcher itself.' }}</span>
		</div>

		<div
			class="flex items-center justify-between p-4 rounded-2xl bg-surface-2 border border-divider cursor-pointer hover:bg-surface-3 transition-colors"
			@click="connectionSettings.autoSelect = !connectionSettings.autoSelect"
		>
			<div class="flex flex-col">
				<div class="flex items-center gap-2 text-sm font-semibold text-contrast">
					<svg class="w-4 h-4 text-brand fill-current" viewBox="0 0 24 24">
						<polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"></polygon>
					</svg>
					<span>{{ isRu ? 'Автоматический выбор' : 'Automatic selection' }}</span>
				</div>
				<span class="text-xs text-secondary mt-0.5">
					{{ isRu ? 'Используется наилучшее прямое подключение' : 'Best direct connection in use' }}
				</span>
			</div>
			<div @click.stop>
				<Toggle id="auto-connection-toggle" v-model="connectionSettings.autoSelect" />
			</div>
		</div>

		<div class="w-full h-px bg-divider"></div>

		<div class="flex flex-col gap-2.5">
			<div
				v-for="item in connectionSettings.items"
				:key="item.id"
				class="group/conn px-4 py-3.5 rounded-2xl bg-surface-2 border-2 border-solid border-white/10 flex items-center justify-between transition-all cursor-pointer hover:bg-surface-3"
				:class="{
					'!border-purple-500 !bg-[rgba(172,81,251,0.18)]': item.active && !connectionSettings.autoSelect,
					'opacity-65': connectionSettings.autoSelect && !item.active
				}"
				@click="selectConnection(item)"
			>
				<div class="flex items-center gap-3.5">
					<div class="flex items-center gap-1.5 text-secondary">
						<svg v-if="item.locked" class="w-4 h-4 text-secondary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect>
							<path d="M7 11V7a5 5 0 0 1 10 0v4"></path>
						</svg>
						<svg class="w-4 h-4 text-secondary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<circle cx="12" cy="12" r="10"></circle>
							<line x1="2" y1="12" x2="22" y2="12"></line>
							<path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path>
						</svg>
					</div>
					<div class="flex flex-col">
						<span class="text-sm font-semibold text-contrast">{{ item.title }}</span>
						<span class="text-xs text-secondary mt-0.5">{{ item.subtitle }}</span>
					</div>
				</div>

				<div class="flex items-center gap-3">
					<div
						class="px-2.5 py-1 rounded-full border flex items-center gap-1.5 text-xs font-mono"
						:class="item.pingMs < 300 ? 'bg-emerald-950/40 border-emerald-800/40 text-emerald-400' : 'bg-amber-950/40 border-amber-800/40 text-amber-400'"
					>
						<div
							class="w-2 h-2 rounded-full"
							:class="item.pingMs < 300 ? 'bg-emerald-400 animate-pulse' : 'bg-amber-400'"
						></div>
						<span>{{ item.pingMs }} {{ isRu ? 'мс' : 'ms' }}</span>
					</div>

					<button
						v-if="!item.locked"
						class="opacity-0 group-hover/conn:opacity-100 p-1.5 rounded-lg text-red-400 hover:text-red-300 hover:bg-red-500/10 transition-all bg-transparent border-0 cursor-pointer"
						:title="isRu ? 'Удалить сервер' : 'Delete connection'"
						@click.stop="removeConnection(item.id)"
					>
						<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							<polyline points="3 6 5 6 21 6"></polyline>
							<path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
						</svg>
					</button>
				</div>
			</div>
		</div>

		<button
			class="w-full py-3 rounded-2xl bg-surface-2 hover:bg-surface-3 text-secondary hover:text-contrast border border-divider text-sm font-semibold transition-all cursor-pointer flex items-center justify-center gap-2"
			@click="showAddModal = true"
		>
			<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
				<line x1="12" y1="5" x2="12" y2="19"></line>
				<line x1="5" y1="12" x2="19" y2="12"></line>
			</svg>
			<span>{{ isRu ? 'Добавить подключение' : 'Add connection' }}</span>
		</button>

		<Teleport to="body">
			<Transition name="fade">
				<div
					v-if="showAddModal"
					class="fixed inset-0 z-[200] flex items-center justify-center bg-black/75 backdrop-blur-md p-4 overflow-y-auto select-none"
					@click.self="closeAddModal"
				>
					<div
						class="w-full max-w-md bg-bg-raised border border-white/10 rounded-3xl p-6 shadow-2xl flex flex-col gap-5 text-contrast my-auto"
						@click.stop
					>
				<div class="flex items-center justify-between pb-3 border-b border-divider">
					<div class="flex items-center gap-3">
						<div class="w-9 h-9 rounded-2xl bg-brand/15 text-brand flex items-center justify-center border border-brand/20">
							<svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
								<circle cx="12" cy="12" r="10"></circle>
								<line x1="2" y1="12" x2="22" y2="12"></line>
								<path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1 4-10z"></path>
							</svg>
						</div>
						<div>
							<h3 class="text-base font-bold text-contrast m-0">
								{{ isRu ? 'Добавить подключение' : 'Add connection' }}
							</h3>
							<p class="text-xs text-secondary m-0">
								{{ isRu ? 'Введите адрес сервера или хоста' : 'Enter server address or host' }}
							</p>
						</div>
					</div>

					<button
						class="w-8 h-8 rounded-xl bg-transparent hover:bg-surface-3 text-secondary hover:text-contrast flex items-center justify-center transition-colors border-0 cursor-pointer"
						:title="isRu ? 'Закрыть (Esc)' : 'Close (Esc)'"
						@click="closeAddModal"
					>
						<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<line x1="18" y1="6" x2="6" y2="18"></line>
							<line x1="6" y1="6" x2="18" y2="18"></line>
						</svg>
					</button>
				</div>

				<div class="flex flex-col gap-3">
					<div class="flex flex-col gap-1.5">
						<label class="text-xs font-semibold text-secondary">
							{{ isRu ? 'Название (опционально)' : 'Title (optional)' }}
						</label>
						<input
							v-model="newServerTitle"
							type="text"
							:placeholder="isRu ? 'Например: Мой сервер EndRage' : 'e.g. My EndRage Server'"
							class="w-full bg-surface-1 border border-divider focus:border-brand rounded-xl px-3.5 py-2.5 text-sm text-contrast placeholder:text-secondary/60 outline-none transition-colors"
						/>
					</div>

					<div class="flex flex-col gap-1.5">
						<label class="text-xs font-semibold text-secondary">
							{{ isRu ? 'Адрес сервера или сайта' : 'Server address or domain' }}
						</label>
						<input
							v-model="newServerHost"
							type="text"
							:placeholder="isRu ? 'mc.end-rage.ru или 127.0.0.1' : 'mc.end-rage.ru or 127.0.0.1'"
							class="w-full bg-surface-1 border border-divider focus:border-brand rounded-xl px-3.5 py-2.5 text-sm text-contrast placeholder:text-secondary/60 outline-none transition-colors"
							@keydown.enter="handleAddServer"
						/>
					</div>
				</div>

				<div class="flex items-center justify-end gap-3 pt-2 border-t border-divider">
					<button
						class="px-4 py-2.5 rounded-xl bg-transparent hover:bg-surface-3 text-secondary hover:text-contrast text-xs font-semibold border-0 cursor-pointer transition-colors"
						@click="closeAddModal"
					>
						{{ isRu ? 'Отмена' : 'Cancel' }}
					</button>
					<button
						class="px-6 py-2.5 rounded-xl bg-brand hover:brightness-110 active:scale-95 text-white font-bold text-xs uppercase tracking-wider border-0 cursor-pointer shadow-lg transition-all flex items-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
						:disabled="isCheckingPing || !newServerHost.trim()"
						@click="handleAddServer"
					>
						<div v-if="isCheckingPing" class="w-3.5 h-3.5 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
						<span>{{ isRu ? 'Добавить и проверить' : 'Add & Test' }}</span>
					</button>
				</div>
			</div>
		</div>
	</Transition>
</Teleport>
	</div>
</template>
