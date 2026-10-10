<script setup lang="ts">
import { computed, ref } from 'vue'
import {
	addCustomHomeGroup,
	homeGroups,
	moveHomeGroup,
	toggleHomeGroupVisibility,
	type HomeGroupConfig,
} from '@/store/launcherPreferences'
import i18n from '@/i18n.config'

const isRu = computed(() => (i18n.global.locale.value || '').startsWith('ru'))

const editingGroup = ref<HomeGroupConfig | null>(null)
const tempRows = ref<1 | 2 | 3 | 4>(2)
const tempMultiplayer = ref(true)
const tempSingleplayer = ref(true)

const showCreateModal = ref(false)
const newGroupTitle = ref('')

function openGroupSettings(group: HomeGroupConfig) {
	editingGroup.value = group
	tempRows.value = group.rows || 2
	tempMultiplayer.value = group.activityMultiplayer ?? true
	tempSingleplayer.value = group.activitySingleplayer ?? true
}

function closeGroupSettings() {
	editingGroup.value = null
}

function saveGroupSettings() {
	if (!editingGroup.value) return
	editingGroup.value.rows = tempRows.value
	editingGroup.value.activityMultiplayer = tempMultiplayer.value
	editingGroup.value.activitySingleplayer = tempSingleplayer.value
	editingGroup.value = null
}

function handleCreateGroup() {
	if (!newGroupTitle.value.trim()) return
	addCustomHomeGroup(newGroupTitle.value.trim())
	newGroupTitle.value = ''
	showCreateModal.value = false
}

let draggedIndex = -1

function onDragStart(index: number) {
	draggedIndex = index
}

function onDragOver(e: DragEvent) {
	e.preventDefault()
}

function onDrop(dropIndex: number) {
	if (draggedIndex === -1 || draggedIndex === dropIndex) return
	if (draggedIndex === 0 || dropIndex === 0) return
	moveHomeGroup(draggedIndex, dropIndex)
	draggedIndex = -1
}
</script>

<template>
	<div class="flex flex-col gap-6 max-w-2xl select-none">
		<div class="flex items-start gap-3">
			<div class="text-gray-300 mt-1">
				<svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
					<path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path>
					<polyline points="9 22 9 12 15 12 15 22"></polyline>
				</svg>
			</div>
			<div>
				<h2 class="m-0 text-xl font-bold text-contrast">
					{{ isRu ? 'Главная страница' : 'Home page' }}
				</h2>
				<p class="m-0 mt-1.5 text-sm text-secondary leading-relaxed">
					{{ isRu ? 'Изменяет расположение групп на главной странице и их видимость.' : 'Configure the layout, ordering, and visibility of groups on the play page.' }}
				</p>
			</div>
		</div>

		<div>
			<h3 class="m-0 text-sm font-bold text-contrast mb-3">
				{{ isRu ? 'Отображаемые группы' : 'Displayed groups' }}
			</h3>

			<div class="flex flex-col gap-2">
				<div
					v-for="(group, index) in homeGroups"
					:key="group.id"
					class="group/row px-4 py-3 rounded-2xl bg-[#16181d] border border-[#23252d] flex items-center justify-between transition-all hover:border-[#333642]"
					:class="{
						'opacity-50': !group.visible,
						'cursor-grab': !group.locked,
					}"
					:draggable="!group.locked"
					@dragstart="onDragStart(index)"
					@dragover="onDragOver"
					@drop="onDrop(index)"
				>
					<div class="flex items-center gap-3">
						<div
							v-if="!group.locked"
							class="text-gray-500 hover:text-gray-300 cursor-grab active:cursor-grabbing p-1 -ml-1"
							title="Перетащите для изменения порядка"
						>
							<svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
								<circle cx="8" cy="6" r="1.5" />
								<circle cx="16" cy="6" r="1.5" />
								<circle cx="8" cy="12" r="1.5" />
								<circle cx="16" cy="12" r="1.5" />
								<circle cx="8" cy="18" r="1.5" />
								<circle cx="16" cy="18" r="1.5" />
							</svg>
						</div>
						<div v-else class="text-gray-600 p-1 -ml-1">
							<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect>
								<path d="M7 11V7a5 5 0 0 1 10 0v4"></path>
							</svg>
						</div>

						<div class="text-gray-400">
							<svg v-if="group.icon === 'news'" class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<path d="M4 22h16a2 2 0 0 0 2-2V4a2 2 0 0 0-2-2H8a2 2 0 0 0-2 2v16a2 2 0 0 1-2 2Zm0 0a2 2 0 0 1-2-2v-9c0-1.1.9-2 2-2h2"></path>
								<path d="M18 14h-8"></path>
								<path d="M15 18h-5"></path>
								<path d="M10 6h8v4h-8V6Z"></path>
							</svg>
							<svg v-else-if="group.icon === 'zap'" class="w-4 h-4 text-amber-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon>
							</svg>
							<svg v-else-if="group.icon === 'creeper'" class="w-4 h-4 text-emerald-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
								<path d="M8 8h2v3H8zM14 8h2v3h-2zM10 13h4v4h-4zM8 17h2v2H8zM14 17h2v2h-2z"></path>
							</svg>
							<svg v-else-if="group.icon === 'pin'" class="w-4 h-4 text-sky-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<line x1="12" y1="17" x2="12" y2="22"></line>
								<path d="M5 17h14v-1.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V6h1a2 2 0 0 0 0-4H8a2 2 0 0 0 0 4h1v4.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24Z"></path>
							</svg>
							<svg v-else class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
							</svg>
						</div>

						<span class="text-sm font-semibold text-contrast">{{ group.title }}</span>

						<div
							v-if="group.badge !== undefined"
							class="px-2 py-0.5 rounded-full bg-[#20222a] text-xs font-semibold text-gray-400"
						>
							{{ group.badge }}
						</div>
					</div>

					<div class="flex items-center gap-1.5 opacity-80 group-hover/row:opacity-100 transition-opacity">
						<button
							v-if="!group.locked"
							class="p-1.5 rounded-lg text-gray-400 hover:text-white hover:bg-white/10 transition-colors bg-transparent border-0 cursor-pointer"
							:title="group.visible ? 'Скрыть группу' : 'Показать группу'"
							@click="toggleHomeGroupVisibility(group.id)"
						>
							<svg v-if="group.visible" class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path>
								<circle cx="12" cy="12" r="3"></circle>
							</svg>
							<svg v-else class="w-4 h-4 text-red-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"></path>
								<line x1="1" y1="1" x2="23" y2="23"></line>
							</svg>
						</button>

						<button
							v-if="!group.locked"
							class="p-1.5 rounded-lg text-gray-400 hover:text-white hover:bg-white/10 transition-colors bg-transparent border-0 cursor-pointer"
							title="Настройка отображения"
							@click="openGroupSettings(group)"
						>
							<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<circle cx="12" cy="12" r="3"></circle>
								<path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
							</svg>
						</button>
					</div>
				</div>
			</div>

			<button
				class="w-full mt-3 py-3 rounded-2xl bg-[#1e2026] hover:bg-[#282a33] text-gray-300 hover:text-white border border-[#2b2e38] text-sm font-semibold transition-all cursor-pointer flex items-center justify-center gap-2"
				@click="showCreateModal = true"
			>
				<span>{{ isRu ? 'Создать группу +' : 'Create group +' }}</span>
			</button>
		</div>

		<div
			v-if="editingGroup"
			class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4"
			@keydown.esc="closeGroupSettings"
		>
			<div class="w-full max-w-xl rounded-3xl bg-[#141518] border border-[#2a2d36] p-7 shadow-2xl flex flex-col gap-6">
				<div class="flex items-start justify-between">
					<div class="flex items-start gap-3.5">
						<div class="text-amber-400 mt-0.5">
							<svg class="w-6 h-6 fill-current" viewBox="0 0 24 24">
								<polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon>
							</svg>
						</div>
						<div>
							<h3 class="m-0 text-xl font-bold text-white">{{ editingGroup.title }}</h3>
							<p class="m-0 mt-0.5 text-xs text-gray-400">
								{{ isRu ? 'Настройка отображения' : 'Display settings' }}
							</p>
						</div>
					</div>

					<button
						class="text-xs text-gray-400 hover:text-white bg-transparent border-0 cursor-pointer flex items-center gap-1"
						@click="closeGroupSettings"
					>
						<span>✕</span>
						<span class="text-[10px] uppercase font-mono tracking-wider opacity-60">esc</span>
					</button>
				</div>

				<div class="grid grid-cols-2 gap-4">
					<div
						class="rounded-2xl p-4 bg-[#1a1c22] border transition-all cursor-pointer flex flex-col justify-between h-32"
						:class="tempRows === 1 ? 'border-[#3b82f6]' : 'border-[#26282f] hover:border-[#333642]'"
						@click="tempRows = 1"
					>
						<div class="flex flex-col gap-1.5">
							<div class="h-3 w-16 bg-gray-600/40 rounded"></div>
							<div class="flex gap-2">
								<div class="flex-1 h-7 rounded-lg bg-[#22252e] border border-[#2d303b] flex items-center justify-between px-2">
									<div class="w-6 h-1.5 rounded bg-gray-500/50"></div>
									<div class="w-2 h-2 rounded bg-gray-500/50"></div>
								</div>
								<div class="flex-1 h-7 rounded-lg bg-[#22252e] border border-[#2d303b] flex items-center justify-between px-2">
									<div class="w-6 h-1.5 rounded bg-gray-500/50"></div>
									<div class="w-2 h-2 rounded bg-gray-500/50"></div>
								</div>
							</div>
						</div>

						<div class="flex items-center gap-2 pt-2">
							<div
								class="w-3.5 h-3.5 rounded-full border flex items-center justify-center"
								:class="tempRows === 1 ? 'border-[#3b82f6] bg-[#3b82f6]' : 'border-gray-600 bg-transparent'"
							>
								<div v-if="tempRows === 1" class="w-1.5 h-1.5 rounded-full bg-white"></div>
							</div>
							<span class="text-xs" :class="tempRows === 1 ? 'text-white font-medium' : 'text-gray-400'">
								{{ isRu ? 'только 1 ряд' : 'only 1 row' }}
							</span>
						</div>
					</div>

					<div
						class="rounded-2xl p-4 bg-[#1a1c22] border transition-all cursor-pointer flex flex-col justify-between h-32"
						:class="tempRows === 2 ? 'border-[#3b82f6]' : 'border-[#26282f] hover:border-[#333642]'"
						@click="tempRows = 2"
					>
						<div class="flex flex-col gap-1.5">
							<div class="flex gap-2">
								<div class="flex-1 h-5 rounded-md bg-[#22252e] border border-[#2d303b] flex items-center justify-between px-2">
									<div class="w-6 h-1.5 rounded bg-gray-500/50"></div>
									<div class="w-2 h-2 rounded bg-gray-500/50"></div>
								</div>
								<div class="flex-1 h-5 rounded-md bg-[#22252e] border border-[#2d303b] flex items-center justify-between px-2">
									<div class="w-6 h-1.5 rounded bg-gray-500/50"></div>
									<div class="w-2 h-2 rounded bg-gray-500/50"></div>
								</div>
							</div>
							<div class="flex gap-2">
								<div class="flex-1 h-5 rounded-md bg-[#22252e] border border-[#2d303b] flex items-center justify-between px-2">
									<div class="w-6 h-1.5 rounded bg-gray-500/50"></div>
									<div class="w-2 h-2 rounded bg-gray-500/50"></div>
								</div>
								<div class="flex-1 h-5 rounded-md bg-[#22252e] border border-[#2d303b] flex items-center justify-between px-2">
									<div class="w-6 h-1.5 rounded bg-gray-500/50"></div>
									<div class="w-2 h-2 rounded bg-gray-500/50"></div>
								</div>
							</div>
						</div>

						<div class="flex items-center gap-2 pt-2">
							<div
								class="w-3.5 h-3.5 rounded-full border flex items-center justify-center"
								:class="tempRows === 2 ? 'border-[#3b82f6] bg-[#3b82f6]' : 'border-gray-600 bg-transparent'"
							>
								<div v-if="tempRows === 2" class="w-1.5 h-1.5 rounded-full bg-white"></div>
							</div>
							<span class="text-xs" :class="tempRows === 2 ? 'text-white font-medium' : 'text-gray-400'">
								{{ isRu ? 'до 2 рядов' : 'up to 2 rows' }}
							</span>
						</div>
					</div>

					<div
						class="rounded-2xl p-4 bg-[#1a1c22] border transition-all cursor-pointer flex flex-col justify-between h-32"
						:class="tempRows === 3 ? 'border-[#3b82f6]' : 'border-[#26282f] hover:border-[#333642]'"
						@click="tempRows = 3"
					>
						<div class="flex flex-col gap-1">
							<div v-for="r in 3" :key="r" class="flex gap-2">
								<div class="flex-1 h-3 rounded bg-[#22252e] border border-[#2d303b] flex items-center justify-between px-1.5">
									<div class="w-4 h-1 rounded bg-gray-500/50"></div>
									<div class="w-1.5 h-1.5 rounded bg-gray-500/50"></div>
								</div>
								<div class="flex-1 h-3 rounded bg-[#22252e] border border-[#2d303b] flex items-center justify-between px-1.5">
									<div class="w-4 h-1 rounded bg-gray-500/50"></div>
									<div class="w-1.5 h-1.5 rounded bg-gray-500/50"></div>
								</div>
							</div>
						</div>

						<div class="flex items-center gap-2 pt-2">
							<div
								class="w-3.5 h-3.5 rounded-full border flex items-center justify-center"
								:class="tempRows === 3 ? 'border-[#3b82f6] bg-[#3b82f6]' : 'border-gray-600 bg-transparent'"
							>
								<div v-if="tempRows === 3" class="w-1.5 h-1.5 rounded-full bg-white"></div>
							</div>
							<span class="text-xs" :class="tempRows === 3 ? 'text-white font-medium' : 'text-gray-400'">
								{{ isRu ? 'до 3 рядов' : 'up to 3 rows' }}
							</span>
						</div>
					</div>

					<div
						class="rounded-2xl p-4 bg-[#1a1c22] border transition-all cursor-pointer flex flex-col justify-between h-32"
						:class="tempRows === 4 ? 'border-[#3b82f6]' : 'border-[#26282f] hover:border-[#333642]'"
						@click="tempRows = 4"
					>
						<div class="flex flex-col gap-0.5">
							<div v-for="r in 4" :key="r" class="flex gap-2">
								<div class="flex-1 h-2.5 rounded bg-[#22252e] border border-[#2d303b] flex items-center justify-between px-1.5">
									<div class="w-4 h-1 rounded bg-gray-500/50"></div>
									<div class="w-1 h-1 rounded bg-gray-500/50"></div>
								</div>
								<div class="flex-1 h-2.5 rounded bg-[#22252e] border border-[#2d303b] flex items-center justify-between px-1.5">
									<div class="w-4 h-1 rounded bg-gray-500/50"></div>
									<div class="w-1 h-1 rounded bg-gray-500/50"></div>
								</div>
							</div>
						</div>

						<div class="flex items-center gap-2 pt-2">
							<div
								class="w-3.5 h-3.5 rounded-full border flex items-center justify-center"
								:class="tempRows === 4 ? 'border-[#3b82f6] bg-[#3b82f6]' : 'border-gray-600 bg-transparent'"
							>
								<div v-if="tempRows === 4" class="w-1.5 h-1.5 rounded-full bg-white"></div>
							</div>
							<span class="text-xs" :class="tempRows === 4 ? 'text-white font-medium' : 'text-gray-400'">
								{{ isRu ? 'до 4 рядов' : 'up to 4 rows' }}
							</span>
						</div>
					</div>
				</div>

				<div class="flex flex-col gap-3.5 pt-2">
					<h4 class="m-0 text-sm font-bold text-white">
						{{ isRu ? 'Фиксируемая активность' : 'Tracked activity' }}
					</h4>

					<div class="flex items-center gap-3">
						<button
							class="w-10 h-5 rounded-full transition-colors relative cursor-pointer border-0"
							:class="tempMultiplayer ? 'bg-[#3b82f6]' : 'bg-[#2b2e38]'"
							@click="tempMultiplayer = !tempMultiplayer"
						>
							<div
								class="w-4 h-4 rounded-full bg-white transition-transform duration-200 absolute top-0.5"
								:class="tempMultiplayer ? 'left-5.5' : 'left-0.5'"
							></div>
						</button>
						<div class="flex items-center gap-2 text-xs font-semibold text-gray-200">
							<svg class="w-4 h-4 text-gray-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<rect x="2" y="3" width="20" height="14" rx="2" ry="2"></rect>
								<line x1="8" y1="21" x2="16" y2="21"></line>
								<line x1="12" y1="17" x2="12" y2="21"></line>
							</svg>
							<span>{{ isRu ? 'В сетевых играх' : 'In multiplayer' }}</span>
						</div>
					</div>

					<div class="flex items-center gap-3">
						<button
							class="w-10 h-5 rounded-full transition-colors relative cursor-pointer border-0"
							:class="tempSingleplayer ? 'bg-[#3b82f6]' : 'bg-[#2b2e38]'"
							@click="tempSingleplayer = !tempSingleplayer"
						>
							<div
								class="w-4 h-4 rounded-full bg-white transition-transform duration-200 absolute top-0.5"
								:class="tempSingleplayer ? 'left-5.5' : 'left-0.5'"
							></div>
						</button>
						<div class="flex items-center gap-2 text-xs font-semibold text-gray-200">
							<svg class="w-4 h-4 text-gray-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<circle cx="12" cy="12" r="10"></circle>
								<line x1="2" y1="12" x2="22" y2="12"></line>
								<path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path>
							</svg>
							<span>{{ isRu ? 'В одиночных мирах' : 'In singleplayer' }}</span>
						</div>
					</div>
				</div>

				<div class="flex items-center justify-start gap-3 pt-2">
					<button
						class="px-6 py-2.5 rounded-xl bg-[#3b82f6] hover:bg-[#2563eb] text-sm font-semibold text-white border-0 cursor-pointer shadow-md"
						@click="saveGroupSettings"
					>
						{{ isRu ? 'Сохранить' : 'Save' }}
					</button>
					<button
						class="px-5 py-2.5 rounded-xl bg-transparent hover:bg-white/5 text-sm font-semibold text-gray-400 hover:text-white border border-transparent cursor-pointer"
						@click="closeGroupSettings"
					>
						{{ isRu ? 'Отмена' : 'Cancel' }}
					</button>
				</div>
			</div>
		</div>

		<div
			v-if="showCreateModal"
			class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4"
		>
			<div class="w-full max-w-sm rounded-3xl bg-[#141518] border border-[#2a2d36] p-6 shadow-2xl flex flex-col gap-4">
				<h3 class="m-0 text-base font-bold text-white">{{ isRu ? 'Создать новую группу' : 'Create new group' }}</h3>
				<input
					v-model="newGroupTitle"
					type="text"
					placeholder="Название группы..."
					class="w-full px-3.5 py-2.5 rounded-xl bg-[#1e2026] border border-[#2b2e38] text-sm text-white focus:outline-none focus:border-[#3b82f6]"
					@keydown.enter="handleCreateGroup"
				/>
				<div class="flex items-center justify-end gap-2.5">
					<button
						class="px-4 py-2 rounded-xl text-xs font-semibold text-gray-400 hover:text-white bg-transparent border-0 cursor-pointer"
						@click="showCreateModal = false"
					>
						{{ isRu ? 'Отмена' : 'Cancel' }}
					</button>
					<button
						class="px-5 py-2 rounded-xl bg-[#3b82f6] hover:bg-[#2563eb] text-xs font-semibold text-white border-0 cursor-pointer"
						@click="handleCreateGroup"
					>
						{{ isRu ? 'Создать' : 'Create' }}
					</button>
				</div>
			</div>
		</div>
	</div>
</template>
