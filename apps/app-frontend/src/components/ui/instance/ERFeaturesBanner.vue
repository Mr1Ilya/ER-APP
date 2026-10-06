<template>
	<div
		v-if="compatibleMod && (!isInstalled || showInstalledStatus)"
		class="er-features-banner flex items-center justify-between p-3.5 px-4 rounded-xl bg-[var(--er-card-bg)] border border-[var(--er-card-border)] transition-all gap-4 mb-3"
		:class="{ 'border-emerald-500/30 bg-emerald-950/10': isInstalled }"
	>
		<div class="flex items-center gap-3 min-w-0">
			<div
				class="w-9 h-9 rounded-lg flex items-center justify-center shrink-0 font-bold text-sm tracking-wider"
				:class="
					isInstalled
						? 'bg-emerald-500/15 border border-emerald-500/30 text-emerald-400'
						: 'bg-[var(--color-brand)]/20 border border-[var(--color-brand)]/40 text-[var(--color-brand)]'
				"
			>
				ER
			</div>
			<div class="flex flex-col min-w-0">
				<div class="flex items-center gap-2 flex-wrap">
					<span class="text-sm font-semibold text-[var(--er-text)]">
						Система скинов End-Rage
					</span>
					<span
						class="text-[11px] px-2 py-0.5 rounded-full font-medium"
						:class="
							isInstalled
								? 'bg-emerald-500/15 text-emerald-300'
								: 'bg-[var(--color-brand)]/15 text-[var(--color-brand)]'
						"
					>
						{{ displayVersion }}
					</span>
				</div>
				<span class="text-xs text-[var(--er-text-secondary)] truncate">
					<template v-if="isInstalled">
						Мод {{ compatibleMod.title }} активен в этой сборке. Кастомные скины и плащи отображаются в игре.
					</template>
					<template v-else>
						Доступен мод {{ compatibleMod.title }} ({{ compatibleMod.compatibilityDesc }}). Включает поддержку HD-скинов и плащей.
					</template>
				</span>
			</div>
		</div>

		<div class="flex items-center gap-2 shrink-0">
			<button
				v-if="!isInstalled"
				:disabled="installing"
				class="flex items-center gap-2 px-3.5 py-1.5 rounded-lg text-xs font-semibold text-white bg-[var(--color-brand)] hover:brightness-110 active:scale-95 transition-all cursor-pointer disabled:opacity-50"
				@click="handleInstall(false)"
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
					class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium text-emerald-400 bg-emerald-500/10 border border-emerald-500/20"
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

				<button
					:disabled="installing"
					class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold text-[var(--er-text)] bg-surface-3 hover:bg-surface-4 border border-[var(--er-card-border)] active:scale-95 transition-all cursor-pointer disabled:opacity-50"
					title="Переустановить или обновить файл мода до актуальной версии"
					@click="handleInstall(true)"
				>
					<span
						v-if="installing"
						class="animate-spin inline-block w-3.5 h-3.5 border-2 border-current border-t-transparent rounded-full"
					></span>
					<svg
						v-else
						class="w-3.5 h-3.5"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
						stroke-linejoin="round"
					>
						<path d="M21.5 2v6h-6M21.34 15.57a10 10 0 1 1-.57-8.38l5.67-5.19" />
					</svg>
					<span>{{ installing ? 'Обновление...' : 'Обновить' }}</span>
				</button>
			</template>
		</div>
	</div>
</template>

<script setup lang="ts">
import { injectNotificationManager } from '@erteam/ui'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'

import {
	fetchRemoteManifest,
	getCompatibleERFeaturesMod,
	getInstalledERFeaturesMod,
	installERFeaturesMod,
	type ERFeaturesModInfo,
	type InstalledERFeaturesMod,
} from '@/helpers/erfeatures'
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
const manifestLoaded = ref(0)
const installedInfo = ref<InstalledERFeaturesMod>({ installed: false })

const compatibleMod = computed<ERFeaturesModInfo | null>(() => {
	if (!props.instance) return null
	const _ = manifestLoaded.value
	return getCompatibleERFeaturesMod(props.instance.loader, props.instance.game_version)
})

const displayVersion = computed(() => {
	if (isInstalled.value) {
		return installedInfo.value.installedVersion || compatibleMod.value?.version || '1.2.0'
	}
	return compatibleMod.value?.version || '1.2.0'
})

async function checkInstalled() {
	if (!props.instance?.id) return
	await fetchRemoteManifest()
	manifestLoaded.value++
	const info = await getInstalledERFeaturesMod(props.instance.id, compatibleMod.value)
	installedInfo.value = info
	isInstalled.value = info.installed
	if (info.installed && info.needsUpdate && compatibleMod.value && !installing.value) {
		await handleInstall(true)
	}
}

watch(
	() => props.instance?.id,
	() => {
		checkInstalled()
	},
	{ immediate: true },
)

onMounted(() => {
	checkInstalled()
	window.addEventListener('focus', checkInstalled)
})

onUnmounted(() => {
	window.removeEventListener('focus', checkInstalled)
})

async function handleInstall(isUpdate = false) {
	if (!compatibleMod.value || installing.value) return

	installing.value = true
	try {
		await installERFeaturesMod(props.instance.id, compatibleMod.value, installedInfo.value.installedPath)
		const info = await getInstalledERFeaturesMod(props.instance.id, compatibleMod.value)
		installedInfo.value = info
		isInstalled.value = true
		notificationManager.addNotification({
			title: isUpdate ? 'Мод обновлён' : 'Мод установлен',
			text: isUpdate
				? `Мод ${compatibleMod.value.title} успешно обновлён в сборке "${props.instance.name}"!`
				: `Мод ${compatibleMod.value.title} успешно установлен в сборку "${props.instance.name}"!`,
			type: 'success',
		})
		emit('installed')
	} catch (error: any) {
		console.error('Failed to install ERFeatures mod:', error)
		notificationManager.handleError(error)
	} finally {
		installing.value = false
	}
}
</script>
