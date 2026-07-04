<template>
	<div
		v-if="accounts.length === 0"
		class="flex flex-col gap-3 mt-2"
	>
		<!-- Tabs -->
		<div class="flex pb-2">
			<button
				class="flex-1 pb-1.5 text-xs font-semibold text-center border-b-2 border-solid transition-colors cursor-pointer bg-transparent border-0 outline-none"
				:class="addMethod === 'microsoft' ? 'border-brand text-contrast font-bold' : 'border-transparent text-secondary hover:text-primary'"
				@click="addMethod = 'microsoft'"
			>
				Microsoft (Лицензия)
			</button>
			<button
				class="flex-1 pb-1.5 text-xs font-semibold text-center border-b-2 border-solid transition-colors cursor-pointer bg-transparent border-0 outline-none"
				:class="addMethod === 'offline' ? 'border-brand text-contrast font-bold' : 'border-transparent text-secondary hover:text-primary'"
				@click="addMethod = 'offline'"
			>
				Пиратский вход
			</button>
		</div>

		<!-- Tab Content: Microsoft -->
		<div v-if="addMethod === 'microsoft'" class="flex flex-col gap-2 pt-1">
			<span class="text-xs text-secondary mb-1">{{ formatMessage(messages.notSignedIn) }}</span>
			<ButtonStyled color="brand" class="w-full">
				<button color="primary" :disabled="loginDisabled" @click="login()">
					<LogInIcon v-if="!loginDisabled" />
					<SpinnerIcon v-else class="animate-spin" />
					{{ formatMessage(messages.signInToMinecraft) }}
				</button>
			</ButtonStyled>
		</div>

		<!-- Tab Content: Offline -->
		<div v-else class="flex flex-col gap-2 pt-1">
			<span class="text-xs text-secondary mb-1">Никнейм для игры в офлайн/пиратском режиме:</span>
			<input
				v-model="offlineUsername"
				type="text"
				placeholder="Никнейм"
				class="w-full bg-bg border border-solid border-surface-5 rounded px-3 py-1.5 text-sm text-contrast focus:outline-none focus:border-brand"
				@keyup.enter="loginOfflineMode"
			/>
			<div class="flex items-center justify-between mt-1.5 px-1 w-full">
				<Checkbox v-model="useElyby" label="Скины с Ely.by" label-class="text-xs text-secondary" />
				<a href="https://ely.by" class="text-[11px] text-brand hover:underline cursor-pointer" @click.prevent="openUrl('https://ely.by')">Регистрация</a>
			</div>
			<ButtonStyled color="brand" class="w-full mt-1">
				<button
					:disabled="!offlineUsername || loginDisabled"
					class="w-full py-1.5 text-sm font-semibold flex items-center justify-center gap-2"
					@click="loginOfflineMode"
				>
					<LogInIcon v-if="!loginDisabled" />
					<SpinnerIcon v-else class="animate-spin" />
					Войти
				</button>
			</ButtonStyled>
		</div>
	</div>
	<Accordion
		v-else
		class="w-full mt-2 bg-button-bg border border-solid border-surface-5 rounded-xl overflow-clip"
		button-class="button-base w-full bg-transparent px-3 py-2 border-0 cursor-pointer"
		:open-by-default="false"
	>
		<template #title>
			<div class="flex gap-2 w-full min-w-0">
				<Avatar
					size="36px"
					:src="
						selectedAccount
							? avatarUrl
							: 'https://launcher-files.modrinth.com/assets/steve_head.png'
					"
				/>
				<div class="flex flex-col items-start w-full min-w-0">
					<span class="truncate w-full text-left">{{
						selectedAccount ? selectedAccount.profile.name : formatMessage(messages.selectAccount)
					}}</span>
					<span class="text-secondary text-xs">
						{{ selectedAccount && selectedAccount.access_token?.startsWith('offline') ? 'Пиратский аккаунт' : formatMessage(messages.minecraftAccount) }}
					</span>
				</div>
			</div>
		</template>
		<div class="bg-button-bg pt-1 pb-2 border border-solid border-surface-5">
			<!-- If not showing add form, show accounts list and "Add account" button -->
			<template v-if="!showAddForm">
				<div v-for="account in accounts" :key="account.profile.id" class="flex gap-1 items-center">
					<button
						class="flex items-center flex-shrink flex-grow overflow-clip gap-2 p-2 border-0 bg-transparent cursor-pointer button-base min-w-0"
						@click="setAccount(account)"
					>
						<RadioButtonCheckedIcon
							v-if="selectedAccount && selectedAccount.profile.id === account.profile.id"
							class="w-5 h-5 text-brand shrink-0"
						/>
						<RadioButtonIcon v-else class="w-5 h-5 text-secondary shrink-0" />
						<Avatar :src="getAccountAvatarUrl(account)" size="24px" />
						<div class="flex flex-col items-start min-w-0">
							<p
								class="m-0 truncate min-w-0 text-sm"
								:class="
									selectedAccount && selectedAccount.profile.id === account.profile.id
										? 'text-contrast font-semibold'
										: 'text-primary'
								"
							>
								{{ account.profile.name }}
							</p>
							<span class="text-[10px] text-secondary">
								{{ account.access_token?.startsWith('offline') ? 'Пиратский' : 'Microsoft' }}
							</span>
						</div>
					</button>
					<ButtonStyled circular color="red" color-fill="none" hover-color-fill="background">
						<button
							v-tooltip="formatMessage(messages.removeAccount)"
							class="mr-2"
							@click="logout(account.profile.id)"
						>
							<TrashIcon />
						</button>
					</ButtonStyled>
				</div>
				<div class="px-2 pt-2">
					<ButtonStyled class="w-full">
						<button @click="showAddForm = true">
							<PlusIcon />
							Добавить аккаунт
						</button>
					</ButtonStyled>
				</div>
			</template>

			<!-- If showing add form inside Accordion -->
			<div v-else class="px-2 pt-1">
				<!-- Tabs -->
				<div class="flex pb-2 mb-2">
					<button
						class="flex-1 pb-1 text-xs font-semibold text-center border-b-2 border-solid transition-colors cursor-pointer bg-transparent border-0 outline-none"
						:class="addMethod === 'microsoft' ? 'border-brand text-contrast' : 'border-transparent text-secondary hover:text-primary'"
						@click="addMethod = 'microsoft'"
					>
						Microsoft
					</button>
					<button
						class="flex-1 pb-1 text-xs font-semibold text-center border-b-2 border-solid transition-colors cursor-pointer bg-transparent border-0 outline-none"
						:class="addMethod === 'offline' ? 'border-brand text-contrast' : 'border-transparent text-secondary hover:text-primary'"
						@click="addMethod = 'offline'"
					>
						Пиратский
					</button>
				</div>

				<!-- Tab Content: Microsoft -->
				<div v-if="addMethod === 'microsoft'" class="flex flex-col gap-2 pt-1">
					<ButtonStyled color="brand" class="w-full">
						<button :disabled="loginDisabled" @click="login()">
							<LogInIcon v-if="!loginDisabled" />
							<SpinnerIcon v-else class="animate-spin" />
							{{ formatMessage(messages.signInToMinecraft) }}
						</button>
					</ButtonStyled>
				</div>

				<!-- Tab Content: Offline -->
				<div v-else class="flex flex-col gap-2 pt-1">
					<input
						v-model="offlineUsername"
						type="text"
						placeholder="Никнейм"
						class="w-full bg-bg border border-solid border-surface-5 rounded px-2 py-1 text-sm text-contrast focus:outline-none focus:border-brand"
						@keyup.enter="loginOfflineMode"
					/>
					<div class="flex items-center justify-between mt-1.5 px-1 w-full">
						<Checkbox v-model="useElyby" label="Скины с Ely.by" label-class="text-xs text-secondary" />
						<a href="https://ely.by" class="text-[11px] text-brand hover:underline cursor-pointer" @click.prevent="openUrl('https://ely.by')">Регистрация</a>
					</div>
					<ButtonStyled color="brand" class="w-full mt-1">
						<button
							:disabled="!offlineUsername || loginDisabled"
							class="w-full py-1 text-sm font-semibold flex items-center justify-center gap-2"
							@click="loginOfflineMode"
						>
							<LogInIcon v-if="!loginDisabled" />
							<SpinnerIcon v-else class="animate-spin" />
							Войти
						</button>
					</ButtonStyled>
				</div>

				<div class="text-center mt-3">
					<button
						class="text-xs text-secondary hover:text-contrast bg-transparent border-0 cursor-pointer underline"
						@click="showAddForm = false"
					>
						Назад к списку
					</button>
				</div>
			</div>
		</div>
	</Accordion>
</template>

<script setup lang="ts">
import { openUrl } from '@tauri-apps/plugin-opener'
import {
	LogInIcon,
	PlusIcon,
	RadioButtonCheckedIcon,
	RadioButtonIcon,
	SpinnerIcon,
	TrashIcon,
} from '@erteam/assets'
import {
	Accordion,
	Avatar,
	ButtonStyled,
	Checkbox,
	defineMessages,
	injectNotificationManager,
	useVIntl,
} from '@erteam/ui'
import type { Ref } from 'vue'
import { computed, onUnmounted, ref } from 'vue'

import { trackEvent } from '@/helpers/analytics'
import {
	get_default_user,
	login as login_flow,
	login_offline,
	remove_user,
	set_default_user,
	users,
} from '@/helpers/auth'
import { process_listener } from '@/helpers/events'
import { getPlayerHeadUrl } from '@/helpers/rendering/batch-skin-renderer.ts'
import type { Skin } from '@/helpers/skins'
import { get_available_skins } from '@/helpers/skins'
import { handleSevereError } from '@/store/error.js'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()

const emit = defineEmits<{
	change: []
}>()

type MinecraftCredential = {
	profile: {
		id: string
		name: string
	}
}

const accounts: Ref<MinecraftCredential[]> = ref([])
const loginDisabled = ref(false)
const offlineUsername = ref('')
const useElyby = ref(true)
const defaultUser = ref<string | undefined>()
const equippedSkin = ref<Skin | null>(null)
const headUrlCache = ref(new Map<string, string>())
const addMethod = ref('microsoft')
const showAddForm = ref(false)

async function refreshValues() {
	defaultUser.value = await get_default_user().catch(handleError)
	const userList = await users().catch(handleError)
	accounts.value = Array.isArray(userList) ? [...userList] : []
	accounts.value.sort((a, b) => (a.profile?.name ?? '').localeCompare(b.profile?.name ?? ''))

	try {
		const skins = await get_available_skins()
		equippedSkin.value = skins.find((skin) => skin.is_equipped) ?? null

		if (equippedSkin.value) {
			try {
				const headUrl = await getPlayerHeadUrl(equippedSkin.value)
				headUrlCache.value = new Map(headUrlCache.value).set(
					equippedSkin.value.texture_key,
					headUrl,
				)
			} catch (error) {
				console.warn('Failed to get head render for equipped skin:', error)
			}
		}
	} catch {
		equippedSkin.value = null
	}
}

async function setEquippedSkin(skin: Skin) {
	equippedSkin.value = skin

	try {
		const headUrl = await getPlayerHeadUrl(skin)
		headUrlCache.value = new Map(headUrlCache.value).set(skin.texture_key, headUrl)
	} catch (error) {
		console.warn('Failed to get head render for equipped skin:', error)
	}
}

function setLoginDisabled(value: boolean) {
	loginDisabled.value = value
}

defineExpose({
	refreshValues,
	setEquippedSkin,
	setLoginDisabled,
	loginDisabled,
})

await refreshValues()

const selectedAccount = computed(() =>
	accounts.value.find((account) => account.profile.id === defaultUser.value),
)

const avatarUrl = computed(() => {
	if (equippedSkin.value?.texture_key) {
		const cachedUrl = headUrlCache.value.get(equippedSkin.value.texture_key)
		if (cachedUrl) {
			return cachedUrl
		}
		return `https://mc-heads.net/avatar/${equippedSkin.value.texture_key}/128`
	}
	if (selectedAccount.value?.profile?.id) {
		return `https://mc-heads.net/avatar/${selectedAccount.value.profile.id}/128`
	}
	return 'https://launcher-files.modrinth.com/assets/steve_head.png'
})

function getAccountAvatarUrl(account: MinecraftCredential) {
	if (
		account.profile.id === selectedAccount.value?.profile?.id &&
		equippedSkin.value?.texture_key
	) {
		const cachedUrl = headUrlCache.value.get(equippedSkin.value.texture_key)
		if (cachedUrl) {
			return cachedUrl
		}
	}
	return `https://mc-heads.net/avatar/${account.profile.id}/128`
}

async function setAccount(account: MinecraftCredential) {
	defaultUser.value = account.profile.id
	await set_default_user(account.profile.id).catch(handleError)
	await refreshValues()
	emit('change')
}

async function login() {
	loginDisabled.value = true
	const loggedIn = await login_flow().catch(handleSevereError)

	if (loggedIn) {
		await setAccount(loggedIn)
		showAddForm.value = false
	}

	trackEvent('AccountLogIn')
	loginDisabled.value = false
}

async function loginOfflineMode() {
	if (!offlineUsername.value) return
	loginDisabled.value = true
	const loggedIn = await login_offline(offlineUsername.value, useElyby.value).catch(handleSevereError)

	if (loggedIn) {
		await setAccount(loggedIn)
		offlineUsername.value = ''
		showAddForm.value = false
	}

	trackEvent('AccountLogIn')
	loginDisabled.value = false
}

async function logout(id: string) {
	await remove_user(id).catch(handleError)
	await refreshValues()
	if (!selectedAccount.value && accounts.value.length > 0) {
		await setAccount(accounts.value[0])
	} else {
		emit('change')
	}
	trackEvent('AccountLogOut')
}

const unlisten = await process_listener(async (e) => {
	if (e.event === 'launched') {
		await refreshValues()
	}
})

onUnmounted(() => {
	unlisten()
})

const messages = defineMessages({
	notSignedIn: {
		id: 'minecraft-account.not-signed-in',
		defaultMessage: 'Not signed in',
	},
	addAccount: {
		id: 'minecraft-account.add-account',
		defaultMessage: 'Add account',
	},
	removeAccount: {
		id: 'minecraft-account.remove-account',
		defaultMessage: 'Remove account',
	},
	selectAccount: {
		id: 'minecraft-account.select-account',
		defaultMessage: 'Select account',
	},
	minecraftAccount: {
		id: 'minecraft-account.label',
		defaultMessage: 'Minecraft account',
	},
	signInToMinecraft: {
		id: 'minecraft-account.sign-in',
		defaultMessage: 'Sign in to Minecraft',
	},
})
</script>
