<template>
	<section
		v-if="showControls"
		class="window-controls flex items-center gap-1.5"
		data-tauri-drag-region-exclude
	>
		<!-- Minimize -->
		<button
			type="button"
			class="ctrl-btn"
			title="Свернуть"
			data-tauri-drag-region-exclude
			@click="() => getCurrentWindow().minimize()"
		>
			<svg class="ctrl-icon" viewBox="0 0 16 16" fill="currentColor">
				<path d="M2.5 8.25a.75.75 0 0 1 .75-.75h9.5a.75.75 0 0 1 0 1.5h-9.5a.75.75 0 0 1-.75-.75Z" />
			</svg>
		</button>

		<!-- Maximize / Restore -->
		<button
			type="button"
			class="ctrl-btn"
			:title="isMaximized ? 'Восстановить' : 'Развернуть'"
			data-tauri-drag-region-exclude
			@click="() => getCurrentWindow().toggleMaximize()"
		>
			<!-- Restore icon (double square) -->
			<svg v-if="isMaximized" class="ctrl-icon" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4">
				<rect x="4.5" y="1.5" width="9" height="9" rx="1.5" />
				<path d="M2.5 5.5v7a1.5 1.5 0 0 0 1.5 1.5h7" stroke-linecap="round" />
			</svg>
			<!-- Maximize icon (single square) -->
			<svg v-else class="ctrl-icon" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4">
				<rect x="2.5" y="2.5" width="11" height="11" rx="1.5" />
			</svg>
		</button>

		<!-- Close -->
		<button
			type="button"
			class="ctrl-btn close-btn"
			title="Закрыть"
			data-tauri-drag-region-exclude
			@click="handleClose"
		>
			<svg class="ctrl-icon" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
				<path d="M3.5 3.5l9 9M12.5 3.5l-9 9" />
			</svg>
		</button>
	</section>
</template>

<script setup>
import { getCurrentWindow } from '@tauri-apps/api/window'
import { saveWindowState, StateFlags } from '@tauri-apps/plugin-window-state'
import { computed, onMounted, onUnmounted, ref } from 'vue'

import { get as getSettings } from '@/helpers/settings.ts'
import { getOS } from '@/helpers/utils.js'
import { useTheming } from '@/store/state'

const themeStore = useTheming()

const nativeDecorations = ref(false)
const isMaximized = ref(false)
const os = ref('')

const alwaysShowAppControls = computed(() => themeStore.getFeatureFlag('always_show_app_controls'))

const showControls = computed(
	() =>
		alwaysShowAppControls.value ||
		(!nativeDecorations.value && (os.value === 'Windows' || os.value === 'Linux')),
)

onMounted(async () => {
	os.value = await getOS()

	const settings = await getSettings()
	nativeDecorations.value = settings.native_decorations

	if (os.value !== 'MacOS') {
		await getCurrentWindow().setDecorations(nativeDecorations.value)
	}

	isMaximized.value = await getCurrentWindow().isMaximized()

	const unlisten = await getCurrentWindow().onResized(async () => {
		isMaximized.value = await getCurrentWindow().isMaximized()
	})

	onUnmounted(() => {
		unlisten()
	})
})

const handleClose = async () => {
	await saveWindowState(StateFlags.ALL)
	await getCurrentWindow().close()
}
</script>

<style scoped>
.window-controls {
	user-select: none;
	-webkit-app-region: no-drag;
}

.ctrl-btn {
	display: inline-flex;
	align-items: center;
	justify-content: center;
	width: 2.125rem; /* 34px */
	height: 1.75rem;  /* 28px */
	border-radius: 0.375rem;
	color: var(--er-text-secondary, #9ca3af);
	background: transparent;
	border: none;
	cursor: pointer;
	padding: 0;
	margin: 0;
	transition: background-color 0.15s ease, color 0.15s ease;
	outline: none;
	-webkit-app-region: no-drag;
}

.ctrl-btn:hover {
	color: #ffffff;
	background-color: rgba(255, 255, 255, 0.08);
}

.ctrl-btn:active {
	background-color: rgba(255, 255, 255, 0.14);
}

.ctrl-btn.close-btn:hover {
	color: #ffffff;
	background-color: #e81123;
}

.ctrl-btn.close-btn:active {
	background-color: #bf101d;
}

.ctrl-icon {
	width: 0.95rem;
	height: 0.95rem;
	flex-shrink: 0;
	pointer-events: none;
}
</style>
