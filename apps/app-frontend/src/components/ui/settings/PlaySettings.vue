<script setup lang="ts">
import { Toggle } from '@erteam/ui'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { list } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'
import {
	addCustomHomeGroup,
	deleteHomeGroup,
	homeGroups,
	moveHomeGroup,
	toggleHomeGroupVisibility,
	type HomeGroupConfig,
} from '@/store/launcherPreferences'
import i18n from '@/i18n.config'

const isRu = computed(() => (i18n.global.locale.value || '').startsWith('ru'))

const allInstances = ref<GameInstance[]>([])

const editingGroup = ref<HomeGroupConfig | null>(null)
const tempRows = ref<1 | 2 | 3 | 4>(2)
const tempMultiplayer = ref(true)
const tempSingleplayer = ref(true)
const tempFilterType = ref<'all' | 'manual' | 'version' | 'loader'>('all')
const tempSelectedIds = ref<string[]>([])
const tempTargetLoader = ref<string>('fabric')
const tempTargetVersion = ref<string>('')

const showCreateModal = ref(false)
const newGroupTitle = ref('')
const newGroupFilterType = ref<'all' | 'manual' | 'version' | 'loader'>('all')
const newGroupSelectedIds = ref<string[]>([])
const newGroupTargetLoader = ref<string>('fabric')
const newGroupTargetVersion = ref<string>('')

const draggedIndex = ref<number | null>(null)

function onGlobalKeydown(e: KeyboardEvent) {
	if (e.key === 'Escape') {
		if (editingGroup.value) closeGroupSettings()
		if (showCreateModal.value) showCreateModal.value = false
	}
}

onMounted(async () => {
	window.addEventListener('keydown', onGlobalKeydown)
	try {
		allInstances.value = (await list()) || []
	} catch {
		allInstances.value = []
	}
})

onUnmounted(() => {
	window.removeEventListener('keydown', onGlobalKeydown)
})

function openGroupSettings(group: HomeGroupConfig) {
	editingGroup.value = group
	tempRows.value = group.rows || 2
	tempMultiplayer.value = group.activityMultiplayer ?? true
	tempSingleplayer.value = group.activitySingleplayer ?? true
	tempFilterType.value = group.filterType || 'all'
	tempSelectedIds.value = [...(group.selectedInstanceIds || [])]
	tempTargetLoader.value = group.targetLoader || 'fabric'
	tempTargetVersion.value = group.targetVersion || ''
}

function closeGroupSettings() {
	editingGroup.value = null
}

function saveGroupSettings() {
	if (!editingGroup.value) return
	editingGroup.value.rows = tempRows.value
	editingGroup.value.activityMultiplayer = tempMultiplayer.value
	editingGroup.value.activitySingleplayer = tempSingleplayer.value
	editingGroup.value.filterType = tempFilterType.value
	editingGroup.value.selectedInstanceIds = [...tempSelectedIds.value]
	editingGroup.value.targetLoader = tempTargetLoader.value
	editingGroup.value.targetVersion = tempTargetVersion.value
	if (editingGroup.value.filterType === 'manual') {
		editingGroup.value.badge = tempSelectedIds.value.length
	}
	editingGroup.value = null
}

function handleCreateGroup() {
	if (!newGroupTitle.value.trim()) return
	addCustomHomeGroup({
		title: newGroupTitle.value.trim(),
		filterType: newGroupFilterType.value,
		selectedInstanceIds: [...newGroupSelectedIds.value],
		targetLoader: newGroupTargetLoader.value,
		targetVersion: newGroupTargetVersion.value,
	})
	newGroupTitle.value = ''
	newGroupFilterType.value = 'all'
	newGroupSelectedIds.value = []
	showCreateModal.value = false
}

function toggleManualInstanceSelection(id: string, listRef: string[]) {
	const idx = listRef.indexOf(id)
	if (idx === -1) {
		listRef.push(id)
	} else {
		listRef.splice(idx, 1)
	}
}

function moveUp(index: number) {
	if (index <= 1) return
	moveHomeGroup(index, index - 1)
}

function moveDown(index: number) {
	if (index === 0 || index >= homeGroups.value.length - 1) return
	moveHomeGroup(index, index + 1)
}

function handleDeleteGroup(id: string) {
	deleteHomeGroup(id)
}

function handleDragStart(e: DragEvent, index: number) {
	if (index === 0) return
	draggedIndex.value = index
	if (e.dataTransfer) {
		e.dataTransfer.effectAllowed = 'move'
	}
}

function handleDragOver(e: DragEvent) {
	e.preventDefault()
	if (e.dataTransfer) {
		e.dataTransfer.dropEffect = 'move'
	}
}

function handleDrop(targetIndex: number) {
	if (draggedIndex.value === null || targetIndex === 0) return
	if (draggedIndex.value !== targetIndex) {
		moveHomeGroup(draggedIndex.value, targetIndex)
	}
	draggedIndex.value = null
}
</script>

<template>
	<div class="flex flex-col gap-6 max-w-2xl select-none">
		<div class="flex items-start gap-3">
			<div class="text-[var(--er-text)] mt-1">
				<svg class="w-6 h-6 text-gray-200" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
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
					class="group/row px-4 py-3 rounded-2xl bg-[var(--er-card-bg)] border border-[var(--er-card-border)] flex items-center justify-between transition-all hover:border-[var(--er-border)]"
					:class="{
						'opacity-50': !group.visible,
						'cursor-grab active:cursor-grabbing': !group.locked
					}"
					:draggable="!group.locked"
					@dragstart="handleDragStart($event, index)"
					@dragover="handleDragOver"
					@drop="handleDrop(index)"
				>
					<div class="flex items-center gap-3">
						<div v-if="!group.locked" class="text-gray-500 hover:text-gray-300 p-0.5">
							<svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
								<circle cx="9" cy="5" r="1.5"></circle>
								<circle cx="15" cy="5" r="1.5"></circle>
								<circle cx="9" cy="12" r="1.5"></circle>
								<circle cx="15" cy="12" r="1.5"></circle>
								<circle cx="9" cy="19" r="1.5"></circle>
								<circle cx="15" cy="19" r="1.5"></circle>
							</svg>
						</div>
						<div v-else class="text-gray-600 p-0.5">
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
							<svg v-else-if="group.icon === 'zap'" class="w-4 h-4 text-amber-400 fill-current" viewBox="0 0 24 24">
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
							<svg v-else class="w-4 h-4 text-purple-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
							</svg>
						</div>

						<span class="text-sm font-semibold text-contrast">{{ group.title }}</span>

						<div
							v-if="group.badge !== undefined"
							class="px-2 py-0.5 rounded-full bg-[var(--er-subtle-bg)] text-xs font-semibold text-gray-400"
						>
							{{ group.badge }}
						</div>
					</div>

					<div class="flex items-center gap-1.5 opacity-80 group-hover/row:opacity-100 transition-opacity">
						<button
							v-if="!group.locked"
							class="p-1 rounded text-secondary hover:text-contrast hover:bg-surface-3 bg-transparent border-0 cursor-pointer disabled:opacity-20"
							:disabled="index <= 1"
							:title="isRu ? 'Вверх' : 'Up'"
							@click.stop="moveUp(index)"
						>
							<svg class="w-3.5 h-3.5 text-secondary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
								<polyline points="18 15 12 9 6 15"></polyline>
							</svg>
						</button>
						<button
							v-if="!group.locked"
							class="p-1 rounded text-secondary hover:text-contrast hover:bg-surface-3 bg-transparent border-0 cursor-pointer disabled:opacity-20"
							:disabled="index >= homeGroups.length - 1"
							:title="isRu ? 'Вниз' : 'Down'"
							@click.stop="moveDown(index)"
						>
							<svg class="w-3.5 h-3.5 text-secondary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
								<polyline points="6 9 12 15 18 9"></polyline>
							</svg>
						</button>

						<button
							v-if="!group.locked"
							class="p-1.5 rounded-lg text-gray-400 hover:text-white hover:bg-white/10 transition-colors bg-transparent border-0 cursor-pointer"
							:title="group.visible ? (isRu ? 'Скрыть группу' : 'Hide group') : (isRu ? 'Показать группу' : 'Show group')"
							@click.stop="toggleHomeGroupVisibility(group.id)"
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
							:title="isRu ? 'Настройка отображения' : 'Display settings'"
							@click.stop="openGroupSettings(group)"
						>
							<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<circle cx="12" cy="12" r="3"></circle>
								<path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
							</svg>
						</button>

						<button
							v-if="!group.locked"
							class="p-1.5 rounded-lg text-red-400 hover:text-red-300 hover:bg-red-500/10 transition-colors bg-transparent border-0 cursor-pointer"
							:title="isRu ? 'Удалить группу' : 'Delete group'"
							@click.stop="handleDeleteGroup(group.id)"
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
				class="w-full mt-3 py-3 rounded-2xl bg-[var(--er-card-bg)] hover:bg-[var(--er-card-hover)] text-gray-300 hover:text-white border border-[var(--er-card-border)] hover:border-[var(--er-border)] text-sm font-semibold transition-all cursor-pointer flex items-center justify-center gap-2"
				@click="showCreateModal = true"
			>
				<span>{{ isRu ? 'Создать группу +' : 'Create group +' }}</span>
			</button>
		</div>

		<Teleport to="body">
			<Transition name="fade">
				<div
					v-if="editingGroup"
					class="fixed inset-0 z-[200] flex items-center justify-center bg-black/75 backdrop-blur-md p-4 overflow-y-auto select-none"
					@click.self="closeGroupSettings"
				>
					<div
						class="w-full max-w-2xl bg-bg-raised border border-white/10 rounded-3xl p-6 shadow-2xl flex flex-col gap-5 text-contrast my-auto"
						@click.stop
					>
						<div class="flex items-center justify-between pb-3 border-b border-divider">
							<div class="flex items-center gap-3">
								<div class="w-10 h-10 rounded-2xl bg-brand/15 text-brand flex items-center justify-center border border-brand/20">
									<svg v-if="editingGroup.icon === 'zap'" class="w-5 h-5 fill-current" viewBox="0 0 24 24">
										<polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon>
									</svg>
									<svg v-else-if="editingGroup.icon === 'creeper'" class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
										<rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
										<path d="M8 8h2v3H8zM14 8h2v3h-2zM10 13h4v4h-4zM8 17h2v2H8zM14 17h2v2h-2z"></path>
									</svg>
									<svg v-else-if="editingGroup.icon === 'pin'" class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
										<line x1="12" y1="17" x2="12" y2="22"></line>
										<path d="M5 17h14v-1.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V6h1a2 2 0 0 0 0-4H8a2 2 0 0 0 0 4h1v4.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24Z"></path>
									</svg>
									<svg v-else class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
										<path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
									</svg>
								</div>
								<div>
									<h3 class="text-lg font-bold text-contrast m-0">
										{{ editingGroup.title }}
									</h3>
									<p class="text-xs text-secondary m-0">
										{{ isRu ? 'Настройка отображения группы на главной' : 'Group display settings on home page' }}
									</p>
								</div>
							</div>

							<button
								class="w-8 h-8 rounded-xl bg-transparent hover:bg-surface-3 text-secondary hover:text-contrast flex items-center justify-center transition-colors border-0 cursor-pointer"
								:title="isRu ? 'Закрыть (Esc)' : 'Close (Esc)'"
								@click="closeGroupSettings"
							>
								<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<line x1="18" y1="6" x2="6" y2="18"></line>
									<line x1="6" y1="6" x2="18" y2="18"></line>
								</svg>
							</button>
						</div>

						<div class="flex flex-col gap-4">
							<div class="flex flex-col gap-2">
								<label class="text-xs font-semibold text-secondary">
									{{ isRu ? 'Количество рядов для отображения' : 'Number of rows to display' }}
								</label>
								<div class="grid grid-cols-4 gap-2.5">
									<button
										v-for="r in [1, 2, 3, 4] as const"
										:key="r"
										type="button"
										class="py-2.5 px-3 rounded-2xl text-center flex flex-col items-center justify-center gap-1 transition-all cursor-pointer border-2 border-solid"
										:class="tempRows === r ? 'bg-[rgba(172,81,251,0.18)] border-purple-500 text-white font-bold' : 'bg-surface-2 border-white/10 hover:bg-surface-3 text-secondary hover:text-contrast'"
										@click="tempRows = r"
									>
										<span class="text-xs font-bold">{{ r }} {{ isRu ? (r === 1 ? 'ряд' : (r < 5 ? 'ряда' : 'рядов')) : (r === 1 ? 'row' : 'rows') }}</span>
										<span class="text-[10px]" :class="tempRows === r ? 'text-purple-300' : 'text-secondary'">
											{{ r === 1 ? (isRu ? 'в одну строку' : 'single line') : (isRu ? `сетка до ${r}` : `grid up to ${r}`) }}
										</span>
									</button>
								</div>
							</div>

							<div class="p-4 rounded-2xl bg-surface-2 border border-divider flex flex-col gap-3">
								<h4 class="m-0 text-xs font-bold text-contrast">
									{{ isRu ? 'Фиксируемая активность' : 'Tracked activity' }}
								</h4>

								<div class="flex items-center justify-between">
									<div class="flex items-center gap-2 text-xs font-semibold text-contrast">
										<Toggle id="group-multiplayer-toggle" v-model="tempMultiplayer" />
										<span>{{ isRu ? 'В сетевых играх' : 'In multiplayer' }}</span>
									</div>
								</div>

								<div class="flex items-center justify-between">
									<div class="flex items-center gap-2 text-xs font-semibold text-contrast">
										<Toggle id="group-singleplayer-toggle" v-model="tempSingleplayer" />
										<span>{{ isRu ? 'В одиночных мирах' : 'In singleplayer' }}</span>
									</div>
								</div>
							</div>

							<div v-if="editingGroup.id.startsWith('custom_') || editingGroup.id === 'pinned'" class="p-4 rounded-2xl bg-surface-2 border border-divider flex flex-col gap-3">
								<h4 class="m-0 text-xs font-bold text-contrast">
									{{ isRu ? 'Содержимое группы' : 'Group content' }}
								</h4>

								<div class="grid grid-cols-3 gap-2">
									<button
										type="button"
										class="py-2 px-3 rounded-xl text-xs font-semibold border-2 border-solid cursor-pointer transition-all"
										:class="tempFilterType === 'all' ? 'bg-[rgba(172,81,251,0.18)] text-white border-purple-500 font-bold' : 'bg-surface-1 text-secondary border-white/10 hover:bg-surface-3'"
										@click="tempFilterType = 'all'"
									>
										{{ isRu ? 'Все сборки' : 'All instances' }}
									</button>
									<button
										type="button"
										class="py-2 px-3 rounded-xl text-xs font-semibold border-2 border-solid cursor-pointer transition-all"
										:class="tempFilterType === 'manual' ? 'bg-[rgba(172,81,251,0.18)] text-white border-purple-500 font-bold' : 'bg-surface-1 text-secondary border-white/10 hover:bg-surface-3'"
										@click="tempFilterType = 'manual'"
									>
										{{ isRu ? 'Вручную' : 'Manually' }}
									</button>
									<button
										type="button"
										class="py-2 px-3 rounded-xl text-xs font-semibold border-2 border-solid cursor-pointer transition-all"
										:class="tempFilterType === 'loader' ? 'bg-[rgba(172,81,251,0.18)] text-white border-purple-500 font-bold' : 'bg-surface-1 text-secondary border-white/10 hover:bg-surface-3'"
										@click="tempFilterType = 'loader'"
									>
										{{ isRu ? 'Загрузчик' : 'Loader' }}
									</button>
								</div>

								<div v-if="tempFilterType === 'manual'" class="flex flex-col gap-1.5 max-h-44 overflow-y-auto pr-1">
									<div
										v-for="inst in allInstances"
										:key="inst.id"
										class="flex items-center justify-between p-2.5 rounded-xl bg-surface-1 border border-divider hover:bg-surface-3 transition-colors cursor-pointer"
										@click="toggleManualInstanceSelection(inst.id, tempSelectedIds)"
									>
										<div class="flex items-center gap-2">
											<span class="text-xs font-medium text-contrast">{{ inst.name }}</span>
											<span class="text-[10px] text-secondary capitalize">({{ inst.loader }} {{ inst.game_version }})</span>
										</div>
										<div
											class="w-4 h-4 rounded border flex items-center justify-center transition-colors"
											:class="tempSelectedIds.includes(inst.id) ? 'bg-brand border-brand text-white' : 'border-divider bg-transparent'"
										>
											<svg v-if="tempSelectedIds.includes(inst.id)" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
												<polyline points="20 6 9 17 4 12"></polyline>
											</svg>
										</div>
									</div>
								</div>

								<div v-if="tempFilterType === 'loader'" class="flex gap-2">
									<button
										v-for="l in ['fabric', 'forge', 'neoforge', 'vanilla']"
										:key="l"
										type="button"
										class="flex-1 py-1.5 rounded-xl text-xs font-semibold capitalize border-2 border-solid cursor-pointer transition-all"
										:class="tempTargetLoader === l ? 'bg-[rgba(172,81,251,0.18)] text-white border-purple-500 font-bold' : 'bg-surface-1 text-secondary border-white/10 hover:bg-surface-3'"
										@click="tempTargetLoader = l"
									>
										{{ l }}
									</button>
								</div>
							</div>
						</div>

						<div class="flex items-center justify-end gap-3 pt-2 border-t border-divider">
							<button
								type="button"
								class="px-4 py-2.5 rounded-xl bg-transparent hover:bg-surface-3 text-secondary hover:text-contrast text-xs font-semibold border-0 cursor-pointer transition-colors"
								@click="closeGroupSettings"
							>
								{{ isRu ? 'Отмена' : 'Cancel' }}
							</button>
							<button
								type="button"
								class="px-6 py-2.5 rounded-xl bg-brand hover:brightness-110 active:scale-95 text-white font-bold text-xs uppercase tracking-wider border-0 cursor-pointer shadow-lg transition-all"
								@click="saveGroupSettings"
							>
								{{ isRu ? 'Сохранить' : 'Save' }}
							</button>
						</div>
					</div>
				</div>
			</Transition>
		</Teleport>

		<Teleport to="body">
			<Transition name="fade">
				<div
					v-if="showCreateModal"
					class="fixed inset-0 z-[200] flex items-center justify-center bg-black/75 backdrop-blur-md p-4 overflow-y-auto select-none"
					@click.self="showCreateModal = false"
				>
					<div
						class="w-full max-w-md bg-bg-raised border border-white/10 rounded-3xl p-6 shadow-2xl flex flex-col gap-5 text-contrast my-auto"
						@click.stop
					>
						<div class="flex items-center justify-between pb-3 border-b border-divider">
							<div class="flex items-center gap-3">
								<div class="w-9 h-9 rounded-2xl bg-brand/15 text-brand flex items-center justify-center border border-brand/20">
									<svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
										<line x1="12" y1="5" x2="12" y2="19"></line>
										<line x1="5" y1="12" x2="19" y2="12"></line>
									</svg>
								</div>
								<div>
									<h3 class="text-base font-bold text-contrast m-0">
										{{ isRu ? 'Создать новую группу' : 'Create new group' }}
									</h3>
									<p class="text-xs text-secondary m-0">
										{{ isRu ? 'Группа появится на вашей главной странице' : 'The group will appear on your home page' }}
									</p>
								</div>
							</div>

							<button
								class="w-8 h-8 rounded-xl bg-transparent hover:bg-surface-3 text-secondary hover:text-contrast flex items-center justify-center transition-colors border-0 cursor-pointer"
								:title="isRu ? 'Закрыть (Esc)' : 'Close (Esc)'"
								@click="showCreateModal = false"
							>
								<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<line x1="18" y1="6" x2="6" y2="18"></line>
									<line x1="6" y1="6" x2="18" y2="18"></line>
								</svg>
							</button>
						</div>

						<div class="flex flex-col gap-4">
							<div class="flex flex-col gap-1.5">
								<label class="text-xs font-semibold text-secondary">
									{{ isRu ? 'Название группы' : 'Group title' }}
								</label>
								<input
									v-model="newGroupTitle"
									type="text"
									:placeholder="isRu ? 'Например: Избранные сборки' : 'e.g. Favorite Instances'"
									class="w-full bg-surface-1 border border-divider focus:border-brand rounded-xl px-3.5 py-2.5 text-sm text-contrast placeholder:text-secondary/60 outline-none transition-colors"
								/>
							</div>

							<div class="p-4 rounded-2xl bg-surface-2 border border-divider flex flex-col gap-3">
								<label class="text-xs font-semibold text-contrast">{{ isRu ? 'Что будет в группе' : 'Group content' }}</label>
								<div class="grid grid-cols-3 gap-2">
									<button
										type="button"
										class="py-2 px-2 rounded-xl text-xs font-semibold border-2 border-solid cursor-pointer transition-all"
										:class="newGroupFilterType === 'all' ? 'bg-[rgba(172,81,251,0.18)] text-white border-purple-500 font-bold' : 'bg-surface-1 text-secondary border-white/10 hover:bg-surface-3'"
										@click="newGroupFilterType = 'all'"
									>
										{{ isRu ? 'Все сборки' : 'All instances' }}
									</button>
									<button
										type="button"
										class="py-2 px-2 rounded-xl text-xs font-semibold border-2 border-solid cursor-pointer transition-all"
										:class="newGroupFilterType === 'manual' ? 'bg-[rgba(172,81,251,0.18)] text-white border-purple-500 font-bold' : 'bg-surface-1 text-secondary border-white/10 hover:bg-surface-3'"
										@click="newGroupFilterType = 'manual'"
									>
										{{ isRu ? 'Вручную' : 'Manually' }}
									</button>
									<button
										type="button"
										class="py-2 px-2 rounded-xl text-xs font-semibold border-2 border-solid cursor-pointer transition-all"
										:class="newGroupFilterType === 'loader' ? 'bg-[rgba(172,81,251,0.18)] text-white border-purple-500 font-bold' : 'bg-surface-1 text-secondary border-white/10 hover:bg-surface-3'"
										@click="newGroupFilterType = 'loader'"
									>
										{{ isRu ? 'Загрузчик' : 'Loader' }}
									</button>
								</div>

								<div v-if="newGroupFilterType === 'manual'" class="flex flex-col gap-1.5 max-h-40 overflow-y-auto pr-1 mt-1">
									<div
										v-for="inst in allInstances"
										:key="inst.id"
										class="flex items-center justify-between p-2 rounded-xl bg-surface-1 border border-divider hover:bg-surface-3 transition-colors cursor-pointer"
										@click="toggleManualInstanceSelection(inst.id, newGroupSelectedIds)"
									>
										<div class="flex items-center gap-2">
											<span class="text-xs font-medium text-contrast">{{ inst.name }}</span>
											<span class="text-[10px] text-secondary capitalize">({{ inst.loader }} {{ inst.game_version }})</span>
										</div>
										<div
											class="w-4 h-4 rounded border flex items-center justify-center transition-colors"
											:class="newGroupSelectedIds.includes(inst.id) ? 'bg-brand border-brand text-white' : 'border-divider bg-transparent'"
										>
											<svg v-if="newGroupSelectedIds.includes(inst.id)" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
												<polyline points="20 6 9 17 4 12"></polyline>
											</svg>
										</div>
									</div>
								</div>

								<div v-if="newGroupFilterType === 'loader'" class="flex gap-1.5 mt-1">
									<button
										v-for="l in ['fabric', 'forge', 'neoforge', 'vanilla']"
										:key="l"
										type="button"
										class="flex-1 py-1.5 rounded-lg text-xs font-semibold capitalize border cursor-pointer transition-all"
										:class="newGroupTargetLoader === l ? 'bg-purple-500/15 text-white border-2 border-purple-500 font-bold' : 'bg-surface-1 text-secondary border border-white/10 hover:bg-surface-3'"
										@click="newGroupTargetLoader = l"
									>
										{{ l }}
									</button>
								</div>
							</div>
						</div>

						<div class="flex items-center justify-end gap-3 pt-2 border-t border-divider">
							<button
								type="button"
								class="px-4 py-2.5 rounded-xl bg-transparent hover:bg-surface-3 text-secondary hover:text-contrast text-xs font-semibold border-0 cursor-pointer transition-colors"
								@click="showCreateModal = false"
							>
								{{ isRu ? 'Отмена' : 'Cancel' }}
							</button>
							<button
								type="button"
								class="px-6 py-2.5 rounded-xl bg-brand hover:brightness-110 active:scale-95 text-white font-bold text-xs uppercase tracking-wider border-0 cursor-pointer shadow-lg transition-all disabled:opacity-50 disabled:cursor-not-allowed"
								:disabled="!newGroupTitle.trim()"
								@click="handleCreateGroup"
							>
								{{ isRu ? 'Создать' : 'Create' }}
							</button>
						</div>
					</div>
				</div>
			</Transition>
		</Teleport>
	</div>
</template>
