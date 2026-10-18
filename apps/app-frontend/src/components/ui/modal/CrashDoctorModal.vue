<script setup lang="ts">
import {
	CheckIcon,
	ChevronDownIcon,
	CopyIcon,
	FolderOpenIcon,
	PlayIcon,
	RefreshCwIcon,
	SparklesIcon,
	WrenchIcon,
	XIcon,
} from '@erteam/assets'
import { ButtonStyled, NewModal } from '@erteam/ui'
import { openUrl } from '@tauri-apps/plugin-opener'
import { ref } from 'vue'

import { edit, get_projects, run, toggle_disable_project } from '@/helpers/instance'
import { showInstanceInFolder } from '@/helpers/utils'

export interface CrashDiagnosis {
	category: string
	title: string
	description: string
	action_type: 'open_url' | 'allocate_memory' | 'disable_mod' | 'install_fabric_api' | 'manual'
	action_label: string
	action_payload?: string | null
}

export interface GameCrashPayload {
	instance_id: string
	instance_name: string
	exit_code: number | null
	scrubbed_log: string
	diagnosis: CrashDiagnosis | null
}

const modal = ref<InstanceType<typeof NewModal>>()
const payload = ref<GameCrashPayload | null>(null)
const diagnosis = ref<CrashDiagnosis | null>(null)
const analyzing = ref(false)
const actionApplied = ref(false)
const actionApplying = ref(false)
const actionMessage = ref('')
const showLog = ref(false)
const copied = ref(false)

function parseDiagnosis(data: any): CrashDiagnosis {
	let actionType: CrashDiagnosis['action_type'] = 'manual'
	let actionPayload: string | null = null
	const act = String(data.action_type || data.action || '').toLowerCase()

	if (act === 'allocate_memory' || act === 'increase_ram') {
		actionType = 'allocate_memory'
		actionPayload = String(data.action_payload || data.target || '4096')
	} else if (act === 'disable_mod') {
		actionType = 'disable_mod'
		actionPayload = String(data.action_payload || data.target || '')
	} else if (act === 'install_fabric_api' || act === 'install_mod') {
		actionType = 'install_fabric_api'
	} else if (act === 'open_url' || act === 'driver_warning' || act === 'update_gpu_drivers') {
		actionType = 'open_url'
		actionPayload = String(data.action_payload || data.target || 'https://www.nvidia.com/Download/index.aspx')
	}

	return {
		category: data.category || data.problem_type || 'Крашлитика AI',
		title: data.title || 'Обнаружена причина сбоя',
		description: data.description || data.explanation || 'Рекомендуется применить исправление.',
		action_type: actionType,
		action_label: data.action_label || data.button_label || data.buttonLabel || 'Исправить',
		action_payload: actionPayload,
	}
}

async function open(crashPayload: GameCrashPayload) {
	payload.value = crashPayload
	actionApplied.value = false
	actionApplying.value = false
	actionMessage.value = ''
	showLog.value = false
	copied.value = false

	if (crashPayload.diagnosis) {
		diagnosis.value = parseDiagnosis(crashPayload.diagnosis)
		analyzing.value = false
		modal.value?.show()
		return
	}

	diagnosis.value = null
	analyzing.value = true
	modal.value?.show()

	try {
		const response = await fetch('https://api.end-rage.ru/api/crash-doctor', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({
				logTail: crashPayload.scrubbed_log,
				instance_name: crashPayload.instance_name,
				exit_code: crashPayload.exit_code,
			}),
		})

		if (response.ok) {
			const data = await response.json()
			diagnosis.value = parseDiagnosis(data)
		} else {
			diagnosis.value = {
				category: 'Краш игры',
				title: 'Minecraft аварийно завершил работу',
				description:
					'Автоматическая диагностика не смогла выявить точную причину. Проверьте лог или обратитесь в поддержку.',
				action_type: 'manual',
				action_label: '',
				action_payload: null,
			}
		}
	} catch {
		diagnosis.value = {
			category: 'Краш игры',
			title: 'Minecraft аварийно завершил работу',
			description:
				'Автоматическая диагностика не смогла выявить точную причину. Проверьте лог или обратитесь в поддержку.',
			action_type: 'manual',
			action_label: '',
			action_payload: null,
		}
	} finally {
		analyzing.value = false
	}
}

async function applyAction() {
	if (!payload.value || !diagnosis.value || actionApplying.value) return
	actionApplying.value = true
	try {
		const act = diagnosis.value.action_type
		if (act === 'allocate_memory') {
			const ramMb = parseInt(diagnosis.value.action_payload || '4096', 10) || 4096
			await edit(payload.value.instance_id, {
				memory: { maximum: ramMb },
			})
			actionApplied.value = true
			actionMessage.value = `Выделено ${Math.round(ramMb / 1024)} ГБ ОЗУ`
		} else if (act === 'disable_mod') {
			const targetRaw = (diagnosis.value.action_payload || '').trim()
			const targetClean = targetRaw.replace(/\.jar$/i, '').toLowerCase()
			const projects = await get_projects(payload.value.instance_id).catch(() => ({}))
			let candidatePath: string | null = null

			for (const [key, file] of Object.entries(projects)) {
				const kLower = key.toLowerCase()
				const pLower = (file?.path || '').toLowerCase()
				if (
					targetClean &&
					(kLower.includes(targetClean) ||
						pLower.includes(targetClean) ||
						targetRaw.toLowerCase() === kLower ||
						targetRaw.toLowerCase() === pLower)
				) {
					candidatePath = file?.path || key
					break
				}
			}

			if (!candidatePath && targetRaw) {
				candidatePath = targetRaw.startsWith('mods/') ? targetRaw : `mods/${targetRaw}`
			}

			if (candidatePath) {
				await toggle_disable_project(payload.value.instance_id, candidatePath, false)
				actionApplied.value = true
				actionMessage.value = 'Конфликтный мод отключен'
			} else {
				actionMessage.value = 'Мод не найден'
			}
		} else if (act === 'install_fabric_api') {
			await openUrl('https://modrinth.com/mod/fabric-api')
			actionApplied.value = true
			actionMessage.value = 'Открыта страница Fabric API'
		} else if (act === 'open_url') {
			if (diagnosis.value.action_payload) {
				await openUrl(diagnosis.value.action_payload)
			}
			actionApplied.value = true
			actionMessage.value = 'Ссылка открыта'
		}
	} catch {
		actionMessage.value = 'Ошибка применения действия'
	} finally {
		actionApplying.value = false
	}
}

async function relaunch() {
	if (!payload.value) return
	const id = payload.value.instance_id
	modal.value?.hide()
	try {
		await run(id)
	} catch {}
}

async function copySupportReport() {
	if (!payload.value) return
	const lines = [
		'=== EndRage Crash Report ===',
		`Экземпляр: ${payload.value.instance_name} (${payload.value.instance_id})`,
		`Код завершения: ${payload.value.exit_code ?? 'неизвестно'}`,
	]
	if (diagnosis.value) {
		lines.push(`Категория: ${diagnosis.value.category}`)
		lines.push(`Проблема: ${diagnosis.value.title}`)
		lines.push(`Решение: ${diagnosis.value.description}`)
	}
	lines.push(`\n=== Лог краша ===\n${payload.value.scrubbed_log}`)
	await navigator.clipboard.writeText(lines.join('\n'))
	copied.value = true
	setTimeout(() => {
		copied.value = false
	}, 2500)
}

async function openFolder() {
	if (!payload.value) return
	await showInstanceInFolder(payload.value.instance_id)
}

defineExpose({
	open,
	hide: () => modal.value?.hide(),
})
</script>

<template>
	<NewModal ref="modal" :closable="true" max-width="620px">
		<template #title>
			<div class="flex items-center gap-3">
				<div class="p-2 rounded-xl bg-purple-500/20 text-purple-400 border border-solid border-purple-500/30">
					<SparklesIcon class="w-5 h-5" />
				</div>
				<div class="flex flex-col">
					<span class="text-xl font-bold text-white tracking-wide">Крашлитика EndRage</span>
					<span class="text-xs text-secondary">Диагностика и устранение сбоев Minecraft</span>
				</div>
			</div>
		</template>

		<div class="flex flex-col gap-4 text-contrast">
			<div class="flex items-center justify-between px-3.5 py-2.5 rounded-xl bg-surface-2 border border-solid border-white/10 text-xs">
				<div class="flex items-center gap-2 text-secondary">
					<span>Экземпляр:</span>
					<span class="font-semibold text-white px-2 py-0.5 rounded-md bg-white/10">{{ payload?.instance_name }}</span>
				</div>
				<div class="flex items-center gap-1.5 text-secondary">
					<span>Код:</span>
					<span class="font-mono px-2 py-0.5 rounded-md border border-solid border-red-500/30 bg-red-500/10 text-red-400 font-semibold">{{ payload?.exit_code ?? 'CRASH' }}</span>
				</div>
			</div>

			<div v-if="analyzing" class="flex flex-col items-center justify-center p-8 gap-3 text-center rounded-2xl border border-solid border-purple-500/20 bg-surface-2">
				<RefreshCwIcon class="w-8 h-8 text-purple-400 animate-spin" />
				<span class="text-sm font-medium text-white">Анализируем причину падения игры через EndRage AI...</span>
				<span class="text-xs text-secondary">Сверяем журналы краша и базу известных несовместимостей</span>
			</div>

			<div v-else-if="diagnosis" class="p-4 rounded-2xl border border-solid border-purple-500/40 bg-purple-950/20 flex flex-col gap-3">
				<div class="flex items-center justify-between">
					<span class="text-[11px] font-semibold uppercase tracking-wider px-2.5 py-1 rounded-full border border-solid border-purple-500/40 bg-purple-500/20 text-purple-300">
						{{ diagnosis.category }}
					</span>
					<span v-if="diagnosis.action_type !== 'manual'" class="text-xs text-purple-300/80 flex items-center gap-1 font-medium">
						<SparklesIcon class="w-3.5 h-3.5" />
						Решение в 1 клик
					</span>
					<span v-else class="text-xs text-amber-300/80 flex items-center gap-1 font-medium">
						<WrenchIcon class="w-3.5 h-3.5" />
						Требуется ручное действие
					</span>
				</div>

				<div>
					<h3 class="text-base font-bold text-white">{{ diagnosis.title }}</h3>
					<p class="text-xs text-secondary mt-1.5 leading-relaxed">{{ diagnosis.description }}</p>
				</div>

				<div v-if="diagnosis.action_type !== 'manual'" class="pt-3 border-t border-solid border-white/10 flex items-center justify-between gap-3">
					<div v-if="actionApplied" class="flex items-center gap-2 text-green-400 text-xs font-semibold">
						<CheckIcon class="w-4 h-4" />
						<span>{{ actionMessage || 'Исправление применено' }}</span>
					</div>
					<ButtonStyled v-else color="brand">
						<button :disabled="actionApplying" class="flex items-center gap-2 font-semibold" @click="applyAction">
							<RefreshCwIcon v-if="actionApplying" class="w-4 h-4 animate-spin" />
							<WrenchIcon v-else class="w-4 h-4" />
							<span>{{ diagnosis.action_label || 'Исправить в 1 клик' }}</span>
						</button>
					</ButtonStyled>
					<ButtonStyled v-if="actionApplied" color="brand">
						<button class="flex items-center gap-2 font-semibold" @click="relaunch">
							<PlayIcon class="w-4 h-4" />
							<span>Перезапустить игру</span>
						</button>
					</ButtonStyled>
				</div>

				<div v-else class="pt-3 border-t border-solid border-white/10 flex items-center justify-between gap-3">
					<span class="text-xs text-secondary">Откройте папку игры для проверки или удаления файлов</span>
					<ButtonStyled type="outlined">
						<button class="flex items-center gap-1.5 text-xs font-medium" @click="openFolder">
							<FolderOpenIcon class="w-4 h-4" />
							<span>Открыть папку игры</span>
						</button>
					</ButtonStyled>
				</div>
			</div>

			<div class="flex flex-col gap-2">
				<button
					type="button"
					class="flex items-center justify-between px-3 py-2 rounded-xl border border-solid border-white/10 bg-surface-2 text-xs font-medium text-secondary hover:text-white transition-colors"
					@click="showLog = !showLog"
				>
					<div class="flex items-center gap-2">
						<ChevronDownIcon class="w-4 h-4 transition-transform duration-200" :class="{ 'rotate-180': showLog }" />
						<span>Журнал сбоя (scrubbed log)</span>
					</div>
					<span class="text-[11px] opacity-60">{{ showLog ? 'Скрыть' : 'Показать' }}</span>
				</button>
				<div
					v-if="showLog"
					class="p-3 bg-black/60 rounded-xl border border-solid border-white/10 max-h-48 overflow-y-auto text-[11px] font-mono text-gray-300 leading-tight whitespace-pre-wrap select-all"
				>
					{{ payload?.scrubbed_log || 'Лог пуст' }}
				</div>
			</div>
		</div>

		<template #actions>
			<div class="flex items-center justify-between w-full gap-2">
				<div class="flex items-center gap-2">
					<ButtonStyled type="outlined">
						<button class="flex items-center gap-1.5 text-xs" @click="openFolder">
							<FolderOpenIcon class="w-4 h-4" />
							<span>Папка игры</span>
						</button>
					</ButtonStyled>
					<ButtonStyled type="outlined">
						<button class="flex items-center gap-1.5 text-xs" @click="copySupportReport">
							<CheckIcon v-if="copied" class="w-4 h-4 text-green-400" />
							<CopyIcon v-else class="w-4 h-4" />
							<span>{{ copied ? 'Скопировано!' : 'Скопировать для поддержки' }}</span>
						</button>
					</ButtonStyled>
				</div>
				<div class="flex items-center gap-2">
					<ButtonStyled type="outlined">
						<button class="flex items-center gap-1.5 text-xs" @click="modal?.hide()">
							<XIcon class="w-4 h-4" />
							<span>Закрыть</span>
						</button>
					</ButtonStyled>
					<ButtonStyled v-if="!actionApplied" color="brand">
						<button class="flex items-center gap-1.5 text-xs font-semibold" @click="relaunch">
							<PlayIcon class="w-4 h-4" />
							<span>Перезапустить</span>
						</button>
					</ButtonStyled>
				</div>
			</div>
		</template>
	</NewModal>
</template>
