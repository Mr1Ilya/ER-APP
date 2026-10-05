import { invoke } from '@tauri-apps/api/core'
import { get_projects } from './instance'

export interface LanModInfo {
	filename: string
	downloadUrl: string
	title: string
	version: string
}

export async function isLanModInstalled(instanceId: string): Promise<boolean> {
	try {
		const projects = await get_projects(instanceId)
		if (!projects) return false

		for (const key of Object.keys(projects)) {
			const lower = key.toLowerCase()
			if (lower.includes('e4mc') || lower.includes('lan-server-properties') || lower.includes('lanserverproperties') || lower.includes('world-host') || lower.includes('essential')) {
				return true
			}
			const item = projects[key]
			if (item?.name) {
				const nameLower = item.name.toLowerCase()
				if (nameLower.includes('e4mc') || nameLower.includes('lan server properties') || nameLower.includes('world host') || nameLower.includes('essential')) {
					return true
				}
			}
		}
		return false
	} catch {
		return false
	}
}

export async function getCompatibleLanMod(
	loader?: string | null,
	gameVersion?: string | null,
): Promise<LanModInfo | null> {
	if (!loader || !gameVersion) return null

	const normLoader = loader.trim().toLowerCase()
	const v = gameVersion.trim()

	try {
		const url = `https://api.modrinth.com/v2/project/e4mc/version?loaders=${encodeURIComponent(JSON.stringify([normLoader]))}&game_versions=${encodeURIComponent(JSON.stringify([v]))}`
		const res = await fetch(url)
		if (res.ok) {
			const versions = await res.json()
			if (Array.isArray(versions) && versions.length > 0) {
				const best = versions[0]
				const primaryFile = best.files?.find((f: { primary: boolean }) => f.primary) || best.files?.[0]
				if (primaryFile?.url && primaryFile?.filename) {
					return {
						filename: primaryFile.filename,
						downloadUrl: primaryFile.url,
						title: 'e4mc (LAN / No Session Error)',
						version: best.version_number || 'Latest',
					}
				}
			}
		}
	} catch {
	}

	try {
		const lspUrl = `https://api.modrinth.com/v2/project/lan-server-properties/version?loaders=${encodeURIComponent(JSON.stringify([normLoader]))}&game_versions=${encodeURIComponent(JSON.stringify([v]))}`
		const res = await fetch(lspUrl)
		if (res.ok) {
			const versions = await res.json()
			if (Array.isArray(versions) && versions.length > 0) {
				const best = versions[0]
				const primaryFile = best.files?.find((f: { primary: boolean }) => f.primary) || best.files?.[0]
				if (primaryFile?.url && primaryFile?.filename) {
					return {
						filename: primaryFile.filename,
						downloadUrl: primaryFile.url,
						title: 'LAN Server Properties',
						version: best.version_number || 'Latest',
					}
				}
			}
		}
	} catch {
	}

	return null
}

export async function installLanMod(
	instanceId: string,
	modInfo: LanModInfo,
): Promise<string> {
	return await invoke('plugin:utils|install_erfeatures_mod', {
		instanceId,
		downloadUrl: modInfo.downloadUrl,
		fileName: modInfo.filename,
	})
}
