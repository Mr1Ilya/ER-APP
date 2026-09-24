<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { get_default_user, login, login_offline, remove_user, set_default_user, users } from '@/helpers/auth'

const isVisible = ref(false)
const activeTab = ref<'list' | 'add'>('list')
const addType = ref<'offline' | 'microsoft'>('offline')

const offlineNickname = ref('')
const isActionRunning = ref(false)
const errorMessage = ref<string | null>(null)

interface Account {
	profile: {
		id: string
		name: string
	}
	access_token?: string
}

const accountList = ref<Account[]>([])
const currentDefaultId = ref<string | null>(null)

async function loadAccounts() {
	try {
		currentDefaultId.value = (await get_default_user().catch(() => null)) ?? null
		const rawUsers = await users().catch(() => [])
		accountList.value = Array.isArray(rawUsers) ? rawUsers : []

		if (accountList.value.length === 0) {
			activeTab.value = 'add'
		}
	} catch (e) {
		console.error('Failed to load accounts in AccountsModal', e)
	}
}

function show() {
	errorMessage.value = null
	isVisible.value = true
	void loadAccounts()
}

function hide() {
	isVisible.value = false
}

defineExpose({
	show,
	hide,
})

function notifyAccountChanged() {
	window.dispatchEvent(new CustomEvent('endrage-account-changed'))
}

async function handleSelectAccount(account: Account) {
	if (isActionRunning.value) return
	try {
		isActionRunning.value = true
		currentDefaultId.value = account.profile.id
		await set_default_user(account.profile.id)
		notifyAccountChanged()
		await loadAccounts()
	} catch (e: any) {
		errorMessage.value = e?.message || 'Не удалось выбрать аккаунт'
	} finally {
		isActionRunning.value = false
	}
}

async function handleRemoveAccount(account: Account) {
	if (isActionRunning.value) return
	try {
		isActionRunning.value = true
		await remove_user(account.profile.id)
		notifyAccountChanged()
		await loadAccounts()
		if (accountList.value.length === 0) {
			activeTab.value = 'add'
		}
	} catch (e: any) {
		errorMessage.value = e?.message || 'Не удалось удалить аккаунт'
	} finally {
		isActionRunning.value = false
	}
}

async function handleAddOffline() {
	const nickname = offlineNickname.value.trim()
	if (!nickname) {
		errorMessage.value = 'Пожалуйста, введите никнейм'
		return
	}
	if (nickname.length < 3 || nickname.length > 16) {
		errorMessage.value = 'Никнейм должен содержать от 3 до 16 символов'
		return
	}

	try {
		isActionRunning.value = true
		errorMessage.value = null
		const account = await login_offline(nickname, false)
		if (account?.profile?.id) {
			await set_default_user(account.profile.id)
		}
		offlineNickname.value = ''
		notifyAccountChanged()
		await loadAccounts()
		activeTab.value = 'list'
	} catch (e: any) {
		errorMessage.value = e?.message || 'Ошибка добавления офлайн аккаунта'
	} finally {
		isActionRunning.value = false
	}
}

async function handleAddMicrosoft() {
	try {
		isActionRunning.value = true
		errorMessage.value = null
		const account = await login()
		if (account?.profile?.id) {
			await set_default_user(account.profile.id)
		}
		notifyAccountChanged()
		await loadAccounts()
		activeTab.value = 'list'
	} catch (e: any) {
		errorMessage.value = e?.message || 'Вход отменён или произошла ошибка'
	} finally {
		isActionRunning.value = false
	}
}

function getAvatar(account: Account) {
	if (account.profile?.id) {
		return `https://crafatar.com/avatars/${account.profile.id}?size=64&overlay=true`
	}
	return 'https://launcher-files.modrinth.com/assets/steve_head.png'
}

function isOffline(account: Account) {
	return Boolean(account.access_token?.startsWith('offline'))
}
</script>

<template>
	<Teleport to="body">
		<Transition name="fade">
			<div
				v-if="isVisible"
				class="fixed inset-0 z-[150] flex items-center justify-center bg-black/75 backdrop-blur-sm p-4 select-none"
				@click.self="hide"
			>
				<div class="w-full max-w-lg bg-[#141518] border border-[#262830] rounded-3xl p-6 shadow-2xl flex flex-col gap-5 text-white">
					<!-- Header -->
					<div class="flex items-center justify-between pb-3 border-b border-[#22242c]">
						<div class="flex items-center gap-3">
							<div class="w-9 h-9 rounded-2xl bg-[#22c55e]/15 text-[#22c55e] flex items-center justify-center border border-[#22c55e]/20">
								<svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
									<path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2"></path>
									<circle cx="12" cy="7" r="4"></circle>
								</svg>
							</div>
							<div>
								<h3 class="text-lg font-bold text-white m-0">Управление аккаунтами</h3>
								<p class="text-xs text-[#8e929b] m-0">Выберите активный профиль или добавьте новый</p>
							</div>
						</div>
						<button
							class="w-8 h-8 rounded-xl bg-transparent hover:bg-white/10 text-gray-400 hover:text-white flex items-center justify-center transition-colors border-0 cursor-pointer"
							@click="hide"
						>
							<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
								<line x1="18" y1="6" x2="6" y2="18"></line>
								<line x1="6" y1="6" x2="18" y2="18"></line>
							</svg>
						</button>
					</div>

					<!-- Error alert -->
					<div v-if="errorMessage" class="p-3 bg-red-500/15 border border-red-500/30 rounded-xl text-xs text-red-300 flex items-center gap-2">
						<svg class="w-4 h-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							<circle cx="12" cy="12" r="10"></circle>
							<line x1="12" y1="8" x2="12" y2="12"></line>
							<line x1="12" y1="16" x2="12.01" y2="16"></line>
						</svg>
						<span>{{ errorMessage }}</span>
					</div>

					<!-- Top Mode Tabs -->
					<div class="flex items-center gap-2 bg-[#1b1c21] p-1.5 rounded-2xl border border-[#262830]">
						<button
							class="flex-1 py-2 px-3 rounded-xl text-xs font-bold transition-all border-0 cursor-pointer flex items-center justify-center gap-2"
							:class="activeTab === 'list' ? 'bg-[#252730] text-white shadow-sm' : 'bg-transparent text-[#8e929b] hover:text-white'"
							@click="activeTab = 'list'"
						>
							<span>Мои аккаунты</span>
							<span class="px-2 py-0.5 rounded-full text-[10px] font-black bg-white/10 text-gray-300">
								{{ accountList.length }}
							</span>
						</button>
						<button
							class="flex-1 py-2 px-3 rounded-xl text-xs font-bold transition-all border-0 cursor-pointer flex items-center justify-center gap-1.5"
							:class="activeTab === 'add' ? 'bg-[#252730] text-white shadow-sm' : 'bg-transparent text-[#8e929b] hover:text-white'"
							@click="activeTab = 'add'"
						>
							<svg class="w-3.5 h-3.5 text-[#22c55e]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
								<line x1="12" y1="5" x2="12" y2="19"></line>
								<line x1="5" y1="12" x2="19" y2="12"></line>
							</svg>
							<span>Добавить аккаунт</span>
						</button>
					</div>

					<!-- TAB 1: ACCOUNTS LIST -->
					<div v-if="activeTab === 'list'" class="flex flex-col gap-2.5 max-h-[340px] overflow-y-auto pr-1">
						<div v-if="accountList.length === 0" class="flex flex-col items-center justify-center py-8 text-center gap-2">
							<div class="w-12 h-12 rounded-2xl bg-white/5 flex items-center justify-center text-gray-500 mb-1">
								<svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
									<circle cx="12" cy="7" r="4"></circle>
									<path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path>
								</svg>
							</div>
							<p class="text-sm font-semibold text-gray-300 m-0">Нет добавленных аккаунтов</p>
							<p class="text-xs text-[#8e929b] m-0 max-w-xs">Добавьте пиратский никнейм или войдите через лицензию Microsoft</p>
							<button
								class="mt-3 px-4 py-2 rounded-xl bg-[#22c55e] text-black font-bold text-xs border-0 cursor-pointer hover:bg-[#1ea850] transition-colors"
								@click="activeTab = 'add'"
							>
								Добавить аккаунт
							</button>
						</div>

						<div
							v-for="account in accountList"
							:key="account.profile.id"
							class="flex items-center justify-between p-3 rounded-2xl border transition-all cursor-pointer"
							:class="account.profile.id === currentDefaultId ? 'bg-[#1e2027] border-[#22c55e]/40 shadow-sm' : 'bg-[#18191f] border-[#252731] hover:bg-[#1d1f26]'"
							@click="handleSelectAccount(account)"
						>
							<div class="flex items-center gap-3 min-w-0">
								<img
									:src="getAvatar(account)"
									class="w-10 h-10 rounded-xl bg-black/40 border border-white/10 shrink-0"
									alt="avatar"
									@error="($event.target as HTMLImageElement).src = 'https://launcher-files.modrinth.com/assets/steve_head.png'"
								/>
								<div class="flex flex-col min-w-0">
									<div class="flex items-center gap-2">
										<span class="text-sm font-bold text-white truncate">{{ account.profile.name }}</span>
										<span
											v-if="account.profile.id === currentDefaultId"
											class="text-[10px] font-black uppercase tracking-wider bg-[#22c55e]/15 text-[#22c55e] px-2 py-0.5 rounded-full border border-[#22c55e]/30 shrink-0"
										>
											Активен
										</span>
									</div>
									<span class="text-xs text-[#8e929b] flex items-center gap-1.5 mt-0.5">
										<span
											class="w-1.5 h-1.5 rounded-full"
											:class="isOffline(account) ? 'bg-blue-400' : 'bg-[#22c55e]'"
										></span>
										{{ isOffline(account) ? 'Офлайн (Пиратка)' : 'Лицензия Microsoft' }}
									</span>
								</div>
							</div>

							<div class="flex items-center gap-2">
								<button
									v-if="account.profile.id !== currentDefaultId"
									class="px-3 py-1.5 rounded-xl bg-[#282a33] hover:bg-[#323540] text-xs font-semibold text-gray-200 transition-colors border-0 cursor-pointer"
									@click.stop="handleSelectAccount(account)"
								>
									Выбрать
								</button>
								<button
									class="w-8 h-8 rounded-xl bg-transparent hover:bg-red-500/15 text-gray-500 hover:text-red-400 flex items-center justify-center transition-colors border-0 cursor-pointer"
									title="Удалить аккаунт"
									@click.stop="handleRemoveAccount(account)"
								>
									<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
										<path d="M3 6h18"></path>
										<path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"></path>
										<path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
									</svg>
								</button>
							</div>
						</div>
					</div>

					<!-- TAB 2: ADD ACCOUNT -->
					<div v-if="activeTab === 'add'" class="flex flex-col gap-4">
						<!-- Sub-type selector -->
						<div class="grid grid-cols-2 gap-2">
							<button
								class="p-3.5 rounded-2xl border text-left flex flex-col gap-1 transition-all cursor-pointer"
								:class="addType === 'offline' ? 'bg-[#1e2027] border-[#22c55e]/50' : 'bg-[#18191f] border-[#252731] hover:bg-[#1d1f26]'"
								@click="addType = 'offline'"
							>
								<div class="flex items-center justify-between w-full">
									<span class="text-xs font-black uppercase tracking-wider text-blue-400">Офлайн / Пиратка</span>
									<span v-if="addType === 'offline'" class="w-2 h-2 rounded-full bg-[#22c55e]"></span>
								</div>
								<span class="text-sm font-bold text-white">Вход по нику</span>
								<span class="text-[11px] text-[#8e929b]">Без пароля, мгновенная игра</span>
							</button>

							<button
								class="p-3.5 rounded-2xl border text-left flex flex-col gap-1 transition-all cursor-pointer"
								:class="addType === 'microsoft' ? 'bg-[#1e2027] border-[#22c55e]/50' : 'bg-[#18191f] border-[#252731] hover:bg-[#1d1f26]'"
								@click="addType = 'microsoft'"
							>
								<div class="flex items-center justify-between w-full">
									<span class="text-xs font-black uppercase tracking-wider text-[#22c55e]">Лицензия</span>
									<span v-if="addType === 'microsoft'" class="w-2 h-2 rounded-full bg-[#22c55e]"></span>
								</div>
								<span class="text-sm font-bold text-white">Microsoft</span>
								<span class="text-[11px] text-[#8e929b]">Официальный аккаунт</span>
							</button>
						</div>

						<!-- Offline form -->
						<div v-if="addType === 'offline'" class="flex flex-col gap-3 p-4 rounded-2xl bg-[#18191f] border border-[#252731]">
							<label class="text-xs font-semibold text-gray-300">Игровой никнейм</label>
							<div class="relative flex items-center">
								<input
									v-model="offlineNickname"
									type="text"
									placeholder="Например: EndRagePlayer"
									maxlength="16"
									class="w-full bg-[#111215] border border-[#292c36] focus:border-[#22c55e] rounded-xl px-3.5 py-2.5 text-sm text-white placeholder-gray-500 outline-none transition-colors"
									@keyup.enter="handleAddOffline"
								/>
							</div>
							<p class="text-[11px] text-[#8e929b] m-0">Никнейм должен состоять от 3 до 16 символов.</p>
							<button
								:disabled="!offlineNickname.trim() || isActionRunning"
								class="w-full py-2.5 px-4 mt-1 rounded-xl bg-[#22c55e] hover:bg-[#1ea850] disabled:bg-gray-700 disabled:text-gray-400 text-black font-bold text-xs uppercase tracking-wider transition-all border-0 cursor-pointer disabled:cursor-not-allowed flex items-center justify-center gap-2"
								@click="handleAddOffline"
							>
								<span v-if="!isActionRunning">Добавить офлайн аккаунт</span>
								<span v-else class="animate-spin w-4 h-4 border-2 border-black border-t-transparent rounded-full"></span>
							</button>
						</div>

						<!-- Microsoft form -->
						<div v-else class="flex flex-col gap-3 p-4 rounded-2xl bg-[#18191f] border border-[#252731] text-center items-center">
							<div class="w-12 h-12 rounded-2xl bg-[#22c55e]/15 text-[#22c55e] flex items-center justify-center border border-[#22c55e]/20 mt-1">
								<svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
									<rect x="3" y="3" width="8" height="8" rx="1.5"></rect>
									<rect x="13" y="3" width="8" height="8" rx="1.5"></rect>
									<rect x="3" y="13" width="8" height="8" rx="1.5"></rect>
									<rect x="13" y="13" width="8" height="8" rx="1.5"></rect>
								</svg>
							</div>
							<div class="flex flex-col gap-1 max-w-sm">
								<span class="text-sm font-bold text-white">Вход через Microsoft</span>
								<span class="text-xs text-[#8e929b]">Откроется официальное окно входа Microsoft в браузере для авторизации вашего лицензионного аккаунта.</span>
							</div>
							<button
								:disabled="isActionRunning"
								class="w-full py-2.5 px-4 mt-2 rounded-xl bg-[#22c55e] hover:bg-[#1ea850] disabled:bg-gray-700 text-black font-bold text-xs uppercase tracking-wider transition-all border-0 cursor-pointer disabled:cursor-not-allowed flex items-center justify-center gap-2"
								@click="handleAddMicrosoft"
							>
								<span v-if="!isActionRunning">Войти через Microsoft</span>
								<span v-else class="animate-spin w-4 h-4 border-2 border-black border-t-transparent rounded-full"></span>
							</button>
						</div>
					</div>
				</div>
			</div>
		</Transition>
	</Teleport>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
	transition: opacity 0.2s ease, transform 0.2s ease;
}
.fade-enter-from,
.fade-leave-to {
	opacity: 0;
	transform: scale(0.96);
}
</style>
