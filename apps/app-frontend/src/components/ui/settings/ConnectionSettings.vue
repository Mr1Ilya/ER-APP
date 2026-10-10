<script setup lang="ts">
import { computed, ref } from 'vue'
import { connectionSettings } from '@/store/launcherPreferences'
import i18n from '@/i18n.config'

const isRu = computed(() => (i18n.global.locale.value || '').startsWith('ru'))

const showAddModal = ref(false)
const newProxyHost = ref('')
const newProxyPort = ref('')
const newProxyType = ref<'http' | 'socks5'>('http')

function handleAddProxy() {
	if (!newProxyHost.value || !newProxyPort.value) return
	const id = 'custom_' + Date.now()
	connectionSettings.value.items.push({
		id,
		title: `${newProxyType.value.toUpperCase()} Прокси`,
		subtitle: `${newProxyHost.value}:${newProxyPort.value}`,
		locked: false,
		pingMs: Math.floor(Math.random() * 80) + 120,
		active: false,
		type: 'custom',
		proxyUrl: `${newProxyType.value}://${newProxyHost.value}:${newProxyPort.value}`,
	})
	newProxyHost.value = ''
	newProxyPort.value = ''
	showAddModal.value = false
}

function selectConnection(item: any) {
	connectionSettings.value.items.forEach((c) => {
		c.active = c.id === item.id
	})
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

		<div class="px-4 py-3 rounded-2xl bg-[#1e2025]/70 border border-[#2b2e38] flex items-center gap-3 text-sm text-secondary">
			<svg class="w-5 h-5 text-gray-400 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
				<circle cx="12" cy="12" r="10"></circle>
				<line x1="12" y1="16" x2="12" y2="12"></line>
				<line x1="12" y1="8" x2="12.01" y2="8"></line>
			</svg>
			<span>{{ isRu ? 'Не используется в игре, только для лаунчера.' : 'Not used in the Minecraft game, only for the launcher itself.' }}</span>
		</div>

		<div class="flex items-center gap-4 py-1">
			<button
				class="w-12 h-6 rounded-full transition-colors relative cursor-pointer border-0"
				:class="connectionSettings.autoSelect ? 'bg-[#3b82f6]' : 'bg-[#2b2e38]'"
				@click="connectionSettings.autoSelect = !connectionSettings.autoSelect"
			>
				<div
					class="w-5 h-5 rounded-full bg-white transition-transform duration-200 absolute top-0.5"
					:class="connectionSettings.autoSelect ? 'left-6.5' : 'left-0.5'"
				></div>
			</button>
			<div class="flex flex-col">
				<div class="flex items-center gap-1.5 text-sm font-semibold text-contrast">
					<span class="text-amber-400">★</span>
					<span>{{ isRu ? 'Автоматический выбор' : 'Automatic selection' }}</span>
				</div>
				<span class="text-xs text-secondary mt-0.5">
					{{ isRu ? 'Используется прямое подключение' : 'Direct connection in use' }}
				</span>
			</div>
		</div>

		<div class="w-full h-px bg-[#262830]"></div>

		<div class="flex flex-col gap-2.5">
			<div
				v-for="item in connectionSettings.items"
				:key="item.id"
				class="px-4 py-3.5 rounded-2xl bg-[#16181d] border border-[#23252d] flex items-center justify-between transition-all cursor-pointer hover:border-[#333642]"
				:class="{ 'opacity-65': connectionSettings.autoSelect && !item.active }"
				@click="selectConnection(item)"
			>
				<div class="flex items-center gap-3.5">
					<div class="flex items-center gap-1.5 text-gray-500">
						<svg v-if="item.locked" class="w-4 h-4 text-gray-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect>
							<path d="M7 11V7a5 5 0 0 1 10 0v4"></path>
						</svg>
						<svg class="w-4 h-4 text-gray-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
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

				<div class="px-2.5 py-1 rounded-full bg-[#1e2320] border border-[#27382d] flex items-center gap-1.5 text-xs font-mono text-[#4ade80]">
					<div class="w-2 h-2 rounded-full bg-[#22c55e]"></div>
					<span>{{ item.pingMs }} {{ isRu ? 'мс' : 'ms' }}</span>
				</div>
			</div>
		</div>

		<button
			class="w-full py-3 rounded-2xl bg-[#1e2026] hover:bg-[#282a33] text-gray-300 hover:text-white border border-[#2b2e38] text-sm font-semibold transition-all cursor-pointer flex items-center justify-center gap-2"
			@click="showAddModal = true"
		>
			<span>{{ isRu ? 'Добавить подключение +' : 'Add connection +' }}</span>
		</button>

		<div
			v-if="showAddModal"
			class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4"
		>
			<div class="w-full max-w-md rounded-3xl bg-[#141518] border border-[#2a2d36] p-6 shadow-2xl flex flex-col gap-5">
				<div class="flex items-center justify-between">
					<h3 class="m-0 text-lg font-bold text-white">{{ isRu ? 'Новое подключение' : 'New connection' }}</h3>
					<button
						class="text-gray-400 hover:text-white bg-transparent border-0 cursor-pointer"
						@click="showAddModal = false"
					>
						✕
					</button>
				</div>

				<div class="flex flex-col gap-3">
					<label class="text-xs font-semibold text-gray-400">{{ isRu ? 'Протокол' : 'Protocol' }}</label>
					<div class="flex gap-2">
						<button
							class="flex-1 py-2 rounded-xl text-xs font-bold transition-colors border cursor-pointer"
							:class="newProxyType === 'http' ? 'bg-[#3b82f6] text-white border-[#3b82f6]' : 'bg-[#1e2026] text-gray-400 border-[#2b2e38]'"
							@click="newProxyType = 'http'"
						>
							HTTP / HTTPS
						</button>
						<button
							class="flex-1 py-2 rounded-xl text-xs font-bold transition-colors border cursor-pointer"
							:class="newProxyType === 'socks5' ? 'bg-[#3b82f6] text-white border-[#3b82f6]' : 'bg-[#1e2026] text-gray-400 border-[#2b2e38]'"
							@click="newProxyType = 'socks5'"
						>
							SOCKS5
						</button>
					</div>

					<label class="text-xs font-semibold text-gray-400 mt-2">{{ isRu ? 'Хост или IP' : 'Host / IP' }}</label>
					<input
						v-model="newProxyHost"
						type="text"
						placeholder="127.0.0.1"
						class="w-full px-3.5 py-2.5 rounded-xl bg-[#1e2026] border border-[#2b2e38] text-sm text-white focus:outline-none focus:border-[#3b82f6]"
					/>

					<label class="text-xs font-semibold text-gray-400 mt-2">{{ isRu ? 'Порт' : 'Port' }}</label>
					<input
						v-model="newProxyPort"
						type="text"
						placeholder="1080"
						class="w-full px-3.5 py-2.5 rounded-xl bg-[#1e2026] border border-[#2b2e38] text-sm text-white focus:outline-none focus:border-[#3b82f6]"
					/>
				</div>

				<div class="flex items-center gap-3 justify-end mt-2">
					<button
						class="px-5 py-2.5 rounded-xl bg-transparent hover:bg-white/5 text-sm font-semibold text-gray-300 border border-transparent cursor-pointer"
						@click="showAddModal = false"
					>
						{{ isRu ? 'Отмена' : 'Cancel' }}
					</button>
					<button
						class="px-5 py-2.5 rounded-xl bg-[#3b82f6] hover:bg-[#2563eb] text-sm font-semibold text-white border-0 cursor-pointer shadow-md"
						@click="handleAddProxy"
					>
						{{ isRu ? 'Сохранить' : 'Save' }}
					</button>
				</div>
			</div>
		</div>
	</div>
</template>
