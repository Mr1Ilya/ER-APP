import { posthog } from 'posthog-js'

interface InstanceProperties {
	loader: string
	game_version: string
}

interface ProjectProperties extends InstanceProperties {
	id: string
	project_type: string
}

type AnalyticsEventMap = {
	Launched: { version: string; dev: boolean; onboarded: boolean }
	PageView: { path: string; fromPath: string; failed: unknown }
	InstanceCreate: { source: string }
	InstanceCreateStart: { source: string }
	InstanceStart: InstanceProperties & { source: string }
	InstanceStop: Partial<InstanceProperties> & { source?: string }
	InstanceDuplicate: InstanceProperties
	InstanceRepair: InstanceProperties
	InstanceSetIcon: Record<string, never>
	InstanceRemoveIcon: Record<string, never>
	InstanceUpdateAll: InstanceProperties & { count: number; selected: boolean }
	InstanceProjectUpdate: InstanceProperties & { id: string; name: string; project_type: string }
	InstanceProjectDisable: InstanceProperties & {
		id: string
		name: string
		project_type: string
		disabled: boolean
	}
	InstanceProjectRemove: InstanceProperties & { id: string; name: string; project_type: string }
	ProjectInstall: ProjectProperties & { version_id: string; title: string; source: string }
	ProjectInstallStart: { source: string }
	PackInstall: { id: string; version_id: string; title: string; source: string }
	PackInstallStart: Record<string, never>
	AccountLogIn: { source?: string }
	AccountLogOut: Record<string, never>
	JavaTest: { path: string; success: boolean }
	JavaManualSelect: { version: string }
	JavaAutoDetect: { path: string; version: string }
}

export type AnalyticsEvent = keyof AnalyticsEventMap

import { getVersion } from '@tauri-apps/api/app'
import { platform as getOsPlatform, version as getOsVersion } from '@tauri-apps/plugin-os'

import { get_default_user, users } from '@/helpers/auth'
import { get as getSettings } from '@/helpers/settings'
import { getLatestLauncherLog } from '@/helpers/utils'

let telemetryEnabled = true

// Initialize cached telemetry setting
getSettings()
	.then((s) => {
		if (s && typeof s.telemetry === 'boolean') {
			telemetryEnabled = s.telemetry
		}
	})
	.catch(() => {})

export const initAnalytics = () => {}
export const debugAnalytics = () => {}
export const optOutAnalytics = () => {
	telemetryEnabled = false
}
export const optInAnalytics = () => {
	telemetryEnabled = true
}

export const trackEvent = <E extends AnalyticsEvent>(
	eventName: E,
	...args: any[]
) => {}

// Debounce map: prevents flooding the exact same error within 30 seconds
const recentErrors = new Map<string, number>()

export interface LauncherCrashReport {
	error: unknown
	context?: string
	extra?: Record<string, any>
	stack?: string
}

/**
 * Reports launcher errors and crashes to backend / Discord telemetry.
 * Only sends when telemetry toggle is enabled by user.
 */
export async function reportLauncherError(
	report: LauncherCrashReport | unknown,
	contextParam?: string
) {
	try {
		if (!telemetryEnabled) return

		let errObj: any = report
		let context = contextParam || ''

		if (report && typeof report === 'object') {
			if ('error' in report) {
				errObj = (report as any).error
				if ((report as any).context) {
					context = (report as any).context
				}
			}
		}

		let errorMessage = ''
		let stackTrace = ''

		if (errObj instanceof Error) {
			errorMessage = errObj.message || errObj.name || String(errObj)
			stackTrace = errObj.stack || ''
		} else if (typeof errObj === 'string') {
			errorMessage = errObj
		} else if (errObj && typeof errObj === 'object') {
			errorMessage = errObj.message || errObj.error || JSON.stringify(errObj)
			stackTrace = errObj.stack || ''
		} else {
			errorMessage = String(errObj || 'Unknown error')
		}

		if (!errorMessage || errorMessage === 'null' || errorMessage === 'undefined') {
			return
		}

		// Filter out benign cancellations or aborts
		const lowerMsg = errorMessage.toLowerCase()
		if (
			lowerMsg.includes('canceled') ||
			lowerMsg.includes('cancelled') ||
			lowerMsg.includes('abort')
		) {
			return
		}

		// Deduplication: maximum 1 report per 30s per unique error
		const now = Date.now()
		const dedupKey = `${context}:${errorMessage.substring(0, 80)}`
		const lastSent = recentErrors.get(dedupKey)
		if (lastSent && now - lastSent < 30_000) {
			return
		}
		recentErrors.set(dedupKey, now)

		// Prune older entries
		for (const [key, timestamp] of recentErrors.entries()) {
			if (now - timestamp > 60_000) {
				recentErrors.delete(key)
			}
		}

		// Gather launcher environment metadata
		const version = await getVersion().catch(() => '1.0.10')

		let os = 'Windows'
		try {
			const p = getOsPlatform()
			const v = getOsVersion()
			os = `${p} ${v}`.trim()
		} catch {}

		let username = 'Анонимный'
		try {
			const defaultId = await get_default_user().catch(() => null)
			const userList = await users().catch(() => [])
			const active = userList.find((u: any) => u.profile?.id === defaultId) || userList[0]
			if (active?.profile?.name) {
				username = active.profile.name
			}
		} catch {}

		// Attempt to fetch launcher session log file from Rust
		let logContent = ''
		try {
			const log = await getLatestLauncherLog()
			if (typeof log === 'string' && log.trim()) {
				logContent = log
			}
		} catch {}

		if (!logContent && stackTrace) {
			logContent = `Context: ${context}\nRoute: ${window.location.hash || window.location.pathname}\n\nCall Stack:\n${stackTrace}`
		}

		const payload = {
			version,
			os,
			username,
			context: context || 'Launcher UI',
			error_message: errorMessage,
			stack_trace: stackTrace || undefined,
			log_content: logContent || undefined,
		}

		const body = JSON.stringify(payload)

		// Send to backend (primary: api.end-rage.ru, fallback: localhost:4004 for local dev)
		try {
			await fetch('https://api.end-rage.ru/api/telemetry/crash', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body,
			})
		} catch {
			await fetch('http://127.0.0.1:4004/api/telemetry/crash', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body,
			}).catch(() => {})
		}
	} catch (reportingError) {
		console.warn('[Telemetry] Failed to report crash:', reportingError)
	}
}
