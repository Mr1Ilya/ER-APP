<template>
	<div
		v-if="compatibleMod && (!isInstalled || showInstalledStatus)"
		class="lan-features-banner flex items-center justify-between p-3.5 px-4 rounded-xl bg-[var(--er-card-bg)] border border-[var(--er-card-border)] transition-all gap-4 mb-3"
		:class="{ 'border-sky-500/30 bg-sky-950/10': isInstalled }"
	>
		<div class="flex items-center gap-3 min-w-0">
			<div
				class="w-9 h-9 rounded-lg flex items-center justify-center shrink-0 font-bold text-sm tracking-wider"
				:class="
					isInstalled
						? 'bg-sky-500/15 border border-sky-500/30 text-sky-400'
						: 'bg-indigo-500/20 border border-indigo-500/40 text-indigo-400'
				"
			>
				LAN
			</div>
			<div class="flex flex-col min-w-0">
				<div class="flex items-center gap-2 flex-wrap">
					<span class="text-sm font-semibold text-[var(--er-text)]">
						Совместная игра (LAN / Без ошибки сессии)
					</span>
					<span
						class="text-[11px] px-2 py-0.5 rounded-full font-medium"
						:class="
							isInstalled
								? 'bg-sky-500/15 text-sky-300'
								: 'bg-indigo-500/15 text-indigo-300'
						"
					>
						{{ compatibleMod.version }}
					</span>
				</div>
				<span class="text-xs text-[var(--er-text-secondary)] truncate">
					<template v-if="isInstalled">
						Мод {{ compatibleMod.title }} активен в этой сборке. Друзья могут подключаться без ошибки «Недействительная сессия».
					</template>
					<template v-else>
						Мод {{ compatibleMod.title }} позволяет играть с друзьями по сети и Radmin VPN без ошибки «Недействительная сессия».
					</template>
				</span>
			</div>
		</div>

		<div class="flex items-center gap-2 shrink-0">
			<button
				v-if="!isInstalled"
				:disabled="installing"
				class="flex items-center gap-2 px-3.5 py-1.5 rounded-lg text-xs font-semibold text-white bg-indigo-600 hover:bg-indigo-500 active:scale-95 transition-all cursor-pointer disabled:opacity-50"
				@click="handleInstall"
			>
				<span
					v-if="installing"
					class="animate-spin inline-block w-3.5 h-3.5 border-2 border-white border-t-transparent rounded-full"
				></span>
				<svg
					v-else
					class="w-3.5 h-3.5"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2.5"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
					<polyline points="7 10 12 15 17 10"></polyline>
					<line x1="12" y1="15" x2="12" y2="3"></line>
				</svg>
				<span>{{ installing ? 'Установка...' : 'Установить мод' }}</span>
			</button>

			<template v-else>
				<div
					class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium text-sky-400 bg-sky-500/10 border border-sky-500/20"
				>
					<svg
						class="w-3.5 h-3.5"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2.5"
						stroke-linecap="round"
						stroke-linejoin="round"
					>
						<polyline points="20 6 9 17 4 12"></polyline>
					</svg>
					<span>Установлен</span>
				</div>
			</template>
		</div>
	</div>
</template>

<script setup lang="ts">
import { injectNotificationManager } from '@erteam/ui'
import { onMounted, ref, watch } from 'vue'

import {
	getCompatibleLanMod,
	installLanMod,
	isLanModInstalled,
	type LanModInfo,
} from '@/helpers/lanmod'
import type { GameInstance } from '@/helpers/types'

const props = withDefaults(
	defineProps<{
		instance: GameInstance
		showInstalledStatus?: boolean
	}>(),
	{
		showInstalledStatus: true,
	},
)

const emit = defineEmits<{
	(e: 'installed'): void
}>()

const notificationManager = injectNotificationManager()
const isInstalled = ref(false)
const installing = ref(false)
const compatibleMod = ref<LanModInfo | null>(null)

async function resolveMod() {
	if (!props.instance) {
		compatibleMod.value = null
		return
	}
	compatibleMod.value = await getCompatibleLanMod(props.instance.loader, props.instance.game_version)
}

async function checkInstalled() {
	if (!props.instance?.id) return
	isInstalled.value = await isLanModInstalled(props.instance.id)
}

watch(
	() => [props.instance?.id, props.instance?.loader, props.instance?.game_version],
	async () => {
		await resolveMod()
		await checkInstalled()
	},
	{ immediate: true },
)

onMounted(async () => {
	await resolveMod()
	await checkInstalled()
})

async function handleInstall() {
	if (!compatibleMod.value || installing.value) return

	installing.value = true
	try {
		await installLanMod(props.instance.id, compatibleMod.value)
		isInstalled.value = true
		notificationManager.addNotification({
			title: 'Мод установлен',
			text: `Мод ${compatibleMod.value.title} успешно установлен в сборку "${props.instance.name}"! Теперь можно играть с друзьями.`,
			type: 'success',
		})
		emit('installed')
	} catch (error: any) {
		notificationManager.handleError(error)
	} finally {
		installing.value = false
	}
}
</script>
