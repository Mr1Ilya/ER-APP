import { ref, watch } from 'vue'

export interface HomeGroupConfig {
	id: string
	title: string
	icon: string
	badge?: number
	locked?: boolean
	visible: boolean
	rows: 1 | 2 | 3 | 4
	activityMultiplayer: boolean
	activitySingleplayer: boolean
}

export interface ConnectionItem {
	id: string
	title: string
	subtitle: string
	locked: boolean
	pingMs: number
	active: boolean
	type: 'direct' | 'system' | 'custom'
	proxyUrl?: string
}

export interface ConnectionSettingsState {
	autoSelect: boolean
	items: ConnectionItem[]
}

const DEFAULT_HOME_GROUPS: HomeGroupConfig[] = [
	{
		id: 'news',
		title: 'Слайдер новостей',
		icon: 'news',
		locked: true,
		visible: true,
		rows: 1,
		activityMultiplayer: true,
		activitySingleplayer: true,
	},
	{
		id: 'recent',
		title: 'Недавние запуски',
		icon: 'zap',
		badge: 2,
		locked: false,
		visible: true,
		rows: 2,
		activityMultiplayer: true,
		activitySingleplayer: true,
	},
	{
		id: 'releases',
		title: 'Последние релизы',
		icon: 'creeper',
		badge: 2,
		locked: false,
		visible: true,
		rows: 2,
		activityMultiplayer: true,
		activitySingleplayer: true,
	},
	{
		id: 'pinned',
		title: 'Закрепленные',
		icon: 'pin',
		badge: 0,
		locked: false,
		visible: true,
		rows: 1,
		activityMultiplayer: true,
		activitySingleplayer: true,
	},
]

const DEFAULT_CONNECTIONS: ConnectionSettingsState = {
	autoSelect: true,
	items: [
		{
			id: 'direct',
			title: 'Прямое подключение',
			subtitle: 'Не использовать прокси для соединений',
			locked: true,
			pingMs: 190,
			active: true,
			type: 'direct',
		},
		{
			id: 'system',
			title: 'Системный прокси',
			subtitle: 'Системный прокси отключен',
			locked: true,
			pingMs: 190,
			active: false,
			type: 'system',
		},
	],
}

function loadFromStorage<T>(key: string, fallback: T): T {
	if (typeof window === 'undefined') return fallback
	try {
		const raw = localStorage.getItem(key)
		if (!raw) return fallback
		return JSON.parse(raw) as T
	} catch {
		return fallback
	}
}

function saveToStorage<T>(key: string, value: T): void {
	if (typeof window === 'undefined') return
	try {
		localStorage.setItem(key, JSON.stringify(value))
	} catch {}
}

export const streamerMode = ref<boolean>(loadFromStorage('er_streamer_mode', false))

watch(
	streamerMode,
	(val) => {
		saveToStorage('er_streamer_mode', val)
		if (typeof window !== 'undefined') {
			window.dispatchEvent(new CustomEvent('er-streamer-mode-changed', { detail: val }))
		}
	},
	{ immediate: true },
)

export const homeGroups = ref<HomeGroupConfig[]>(loadFromStorage('er_home_groups', DEFAULT_HOME_GROUPS))

watch(
	homeGroups,
	(val) => {
		saveToStorage('er_home_groups', val)
		if (typeof window !== 'undefined') {
			window.dispatchEvent(new CustomEvent('er-home-groups-changed', { detail: val }))
		}
	},
	{ deep: true },
)

export const connectionSettings = ref<ConnectionSettingsState>(
	loadFromStorage('er_connection_settings', DEFAULT_CONNECTIONS),
)

watch(
	connectionSettings,
	(val) => {
		saveToStorage('er_connection_settings', val)
	},
	{ deep: true },
)

export function updateHomeGroup(id: string, updates: Partial<HomeGroupConfig>): void {
	const index = homeGroups.value.findIndex((g) => g.id === id)
	if (index !== -1) {
		homeGroups.value[index] = { ...homeGroups.value[index], ...updates }
	}
}

export function toggleHomeGroupVisibility(id: string): void {
	const group = homeGroups.value.find((g) => g.id === id)
	if (group && !group.locked) {
		group.visible = !group.visible
	}
}

export function moveHomeGroup(fromIndex: number, toIndex: number): void {
	if (fromIndex <= 0 || toIndex <= 0) return
	if (toIndex >= homeGroups.value.length) return
	const item = homeGroups.value.splice(fromIndex, 1)[0]
	homeGroups.value.splice(toIndex, 0, item)
}

export function addCustomHomeGroup(title: string): void {
	const newId = 'custom_' + Date.now()
	homeGroups.value.push({
		id: newId,
		title: title.trim() || 'Новая группа',
		icon: 'folder',
		badge: 0,
		locked: false,
		visible: true,
		rows: 1,
		activityMultiplayer: true,
		activitySingleplayer: true,
	})
}
