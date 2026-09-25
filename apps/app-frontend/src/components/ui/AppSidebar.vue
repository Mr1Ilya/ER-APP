<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import { get_default_user, users } from '@/helpers/auth'

const props = defineProps<{
	collapsed?: boolean
}>()

const emit = defineEmits<{
	(e: 'toggle-collapse'): void
	(e: 'open-settings'): void
	(e: 'open-accounts'): void
}>()

const route = useRoute()
const router = useRouter()

const accountList = ref<any[]>([])
const currentUserId = ref<string | null>(null)

const activeAccount = computed(() => {
	if (!accountList.value.length) return null
	if (currentUserId.value) {
		return accountList.value.find((acc) => acc.profile?.id === currentUserId.value) || accountList.value[0]
	}
	return accountList.value[0]
})

const avatarUrl = computed(() => {
	const acc = activeAccount.value
	if (!acc?.profile) {
		return 'https://launcher-files.modrinth.com/assets/steve_head.png'
	}
	if (acc.access_token?.startsWith('endrage')) {
		return `https://skins.end-rage.ru/head/${encodeURIComponent(acc.profile.name)}?size=64`
	}
	if (acc.access_token?.startsWith('offline')) {
		return 'https://launcher-files.modrinth.com/assets/steve_head.png'
	}
	const cleanId = (acc.profile.id || '').replace(/-/g, '')
	if (cleanId) {
		return `https://mc-heads.net/avatar/${cleanId}/64`
	}
	if (acc.profile.name) {
		return `https://mc-heads.net/avatar/${encodeURIComponent(acc.profile.name)}/64`
	}
	return 'https://launcher-files.modrinth.com/assets/steve_head.png'
})

function onAvatarError(e: Event) {
	const img = e.target as HTMLImageElement
	if (activeAccount.value?.access_token?.startsWith('endrage')) {
		if (!img.src.includes('127.0.0.1:4003')) {
			img.src = `http://127.0.0.1:4003/head/${encodeURIComponent(activeAccount.value?.profile?.name || '')}?size=64`
			return
		}
	}
	const name = activeAccount.value?.profile?.name
	if (name && !img.src.includes('minotar.net')) {
		img.src = `https://minotar.net/helm/${encodeURIComponent(name)}/64`
	} else if (!img.src.includes('steve_head')) {
		img.src = 'https://launcher-files.modrinth.com/assets/steve_head.png'
	}
}

import i18n from '@/i18n.config'

const isRu = computed(() => (i18n.global.locale.value || '').startsWith('ru'))

const accountTypeLabel = computed(() => {
	if (!activeAccount.value) return isRu.value ? 'Нажмите для входа' : 'Click to log in'
	if (activeAccount.value.access_token?.startsWith('endrage')) {
		return 'EndRage Auth'
	}
	if (activeAccount.value.access_token?.startsWith('offline')) {
		return isRu.value ? 'Офлайн (Ник)' : 'Offline account'
	}
	return isRu.value ? 'Лицензия Microsoft' : 'Microsoft account'
})

async function loadUserData() {
	try {
		currentUserId.value = await get_default_user().catch(() => null)
		const list = await users().catch(() => [])
		accountList.value = list || []
	} catch (e) {
		console.error('Failed to load user data in sidebar', e)
	}
}

onMounted(() => {
	loadUserData()
	window.addEventListener('endrage-account-changed', loadUserData)
})

onUnmounted(() => {
	window.removeEventListener('endrage-account-changed', loadUserData)
})

const navItems = computed(() => [
	{
		id: 'play',
		label: isRu.value ? 'Играть' : 'Play',
		to: '/',
		exact: true,
		icon: 'play',
	},
	{
		id: 'library',
		label: isRu.value ? 'Мои сборки' : 'Library',
		to: '/library',
		matchPrefix: '/instance',
		icon: 'cube',
	},
	{
		id: 'content',
		label: isRu.value ? 'Контент' : 'Browse',
		to: '/browse/modpack',
		matchPrefix: ['/browse', '/project'],
		icon: 'content',
	},
	{
		id: 'skins',
		label: isRu.value ? 'Скины' : 'Skins',
		to: '/skins',
		icon: 'skins',
	},
])

function isItemActive(item: { to: string; exact?: boolean; matchPrefix?: string | string[] }) {
	if (item.exact) {
		return route.path === item.to
	}
	if (item.id === 'content') {
		return route.path.startsWith('/browse') || route.path.startsWith('/project')
	}
	if (item.matchPrefix) {
		const prefixes = Array.isArray(item.matchPrefix) ? item.matchPrefix : [item.matchPrefix]
		return prefixes.some((p) => route.path === p || route.path.startsWith(p + '/'))
	}
	return route.path.startsWith(item.to)
}

function navigate(to: string) {
	if (route.path === to) return
	router.push(to).catch(() => {})
}

</script>

<template>
	<aside
		class="sidebar-container select-none flex flex-col justify-between h-full bg-[var(--er-sidebar-bg)] border-r border-[var(--er-border)] transition-all duration-200 z-20"
		:class="collapsed ? 'w-[72px] px-2 py-3' : 'w-[220px] px-3 py-3'"
	>
		<!-- Top section -->
		<div class="flex flex-col gap-1.5">
			<!-- Collapse Toggle Button -->
			<div class="mb-2 px-1 flex items-center" :class="collapsed ? 'justify-center' : 'justify-start'">
				<button
					class="flex items-center gap-2 text-xs font-medium text-[var(--er-text-secondary)] hover:text-[var(--er-text)] transition-colors bg-transparent border-0 cursor-pointer p-1.5 rounded-lg hover:bg-white/5"
					:title="collapsed ? (isRu ? 'Развернуть меню' : 'Expand sidebar') : (isRu ? 'Свернуть меню' : 'Collapse sidebar')"
					@click="emit('toggle-collapse')"
				>
					<svg
						class="w-4 h-4 transition-transform duration-200"
						:class="{ 'rotate-180': collapsed }"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2.2"
						stroke-linecap="round"
						stroke-linejoin="round"
					>
						<polyline points="15 18 9 12 15 6"></polyline>
					</svg>
					<span v-if="!collapsed">{{ isRu ? 'Свернуть' : 'Collapse' }}</span>
				</button>
			</div>

			<!-- Nav links -->
			<nav class="flex flex-col gap-1">
				<button
					v-for="item in navItems"
					:key="item.id"
					class="nav-link-item group relative flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm font-medium transition-all border cursor-pointer text-left w-full"
					:class="[
						isItemActive(item)
							? 'bg-white/10 text-white border-white/20 font-semibold shadow-sm'
							: 'bg-transparent text-[var(--er-text-secondary)] hover:text-[var(--er-text)] hover:bg-white/5 border-transparent',
						collapsed ? 'justify-center px-2' : ''
					]"
					:title="collapsed ? item.label : undefined"
					@click="navigate(item.to)"
				>
					<!-- Icon: Play -->
					<svg
						v-if="item.icon === 'play'"
						class="w-4 h-4 shrink-0"
						viewBox="0 0 24 24"
						fill="currentColor"
					>
						<polygon points="6,4 20,12 6,20"></polygon>
					</svg>

					<!-- Icon: Cube (Мои сборки) -->
					<svg
						v-else-if="item.icon === 'cube'"
						class="w-4 h-4 shrink-0"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
						stroke-linejoin="round"
					>
						<path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"></path>
						<polyline points="3.27 6.96 12 12.01 20.73 6.96"></polyline>
						<line x1="12" y1="22.08" x2="12" y2="12"></line>
					</svg>

					<!-- Icon: Content (Контент) -->
					<svg
						v-else-if="item.icon === 'content'"
						class="w-4 h-4 shrink-0"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
						stroke-linejoin="round"
					>
						<rect x="3" y="3" width="7" height="7" rx="1.5"></rect>
						<rect x="14" y="3" width="7" height="7" rx="1.5"></rect>
						<rect x="14" y="14" width="7" height="7" rx="1.5"></rect>
						<rect x="3" y="14" width="7" height="7" rx="1.5"></rect>
					</svg>

					<!-- Icon: Skins (Скины) -->
					<svg
						v-else-if="item.icon === 'skins'"
						class="w-4 h-4 shrink-0"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
						stroke-linejoin="round"
					>
						<path d="M20.38 3.46 16 2a4 4 0 0 1-8 0L3.62 3.46a2 2 0 0 0-1.34 2.23l.58 3.5a2 2 0 0 0 2 1.67h1.49v9.14a2 2 0 0 0 2 2h6.5a2 2 0 0 0 2-2v-9.14h1.49a2 2 0 0 0 2-1.67l.58-3.5a2 2 0 0 0-1.34-2.23z"></path>
					</svg>

					<!-- Text label -->
					<span v-if="!collapsed" class="truncate flex-1 tracking-wide">{{ item.label }}</span>
				</button>
			</nav>
		</div>

		<!-- Bottom section -->
		<div class="flex flex-col gap-2 pt-2 border-t border-[var(--er-border)]">
			<!-- Settings button -->
			<button
				class="group flex items-center gap-3 px-3 py-2 rounded-xl text-sm font-medium text-[var(--er-text-secondary)] hover:text-[var(--er-text)] hover:bg-white/5 transition-all bg-transparent border-0 cursor-pointer w-full"
				:class="collapsed ? 'justify-center px-2' : ''"
				:title="collapsed ? (isRu ? 'Настройки' : 'Settings') : undefined"
				@click="emit('open-settings')"
			>
				<svg
					class="w-4 h-4 shrink-0 text-[var(--er-text-secondary)] group-hover:text-[var(--er-text)] transition-colors"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<circle cx="12" cy="12" r="3"></circle>
					<path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
				</svg>
				<span v-if="!collapsed" class="truncate tracking-wide">{{ isRu ? 'Настройки' : 'Settings' }}</span>
			</button>

			<!-- Account Card -->
			<button
				class="account-card group flex items-center gap-2.5 p-2 rounded-xl bg-[var(--er-card-bg)] hover:bg-[var(--er-card-hover)] border border-[var(--er-card-border)] hover:border-white/20 transition-all cursor-pointer text-left w-full"
				:class="collapsed ? 'justify-center p-1.5' : ''"
				:title="collapsed ? (activeAccount?.profile?.name || (isRu ? 'Аккаунты' : 'Accounts')) : undefined"
				@click="emit('open-accounts')"
			>
				<img
					:src="avatarUrl"
					alt="Avatar"
					class="w-8 h-8 rounded-lg bg-[var(--er-subtle-bg)] object-cover shrink-0 border border-white/10"
					@error="onAvatarError"
				/>
				<div v-if="!collapsed" class="flex flex-col min-w-0 flex-1">
					<span class="text-xs font-semibold text-[var(--er-text)] truncate leading-tight">
						{{ activeAccount?.profile?.name || (isRu ? 'Войти в игру' : 'Log in') }}
					</span>
					<span class="text-[10px] text-[var(--er-text-secondary)] truncate leading-tight mt-0.5">
						{{ accountTypeLabel }}
					</span>
				</div>
				<svg
					v-if="!collapsed"
					class="w-3.5 h-3.5 text-[var(--er-text-secondary)] group-hover:text-[var(--er-text)] transition-colors shrink-0"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2.5"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<polyline points="9 18 15 12 9 6"></polyline>
				</svg>
			</button>
		</div>
	</aside>
</template>

<style scoped>
.sidebar-container {
	font-family: inherit;
}
.nav-link-item {
	letter-spacing: 0.01em;
}
</style>
