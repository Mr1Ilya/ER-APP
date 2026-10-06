import { invoke } from '@tauri-apps/api/core'
import { get_projects } from './instance'

export interface ERFeaturesModInfo {
	filename: string
	downloadUrl: string
	title: string
	version: string
	compatibilityDesc: string
}

function parseVersionNumbers(v: string): number[] {
	if (!v) return [0]
	// Handle strings like "1.20.1", "1.16.5", "26.1", etc.
	const parts = v.split(/[-_+.]/)
	const numbers: number[] = []
	for (const p of parts) {
		const parsed = parseInt(p, 10)
		if (!isNaN(parsed)) {
			numbers.push(parsed)
		}
	}
	return numbers.length > 0 ? numbers : [0]
}

export function compareMinecraftVersions(a: string, b: string): number {
	const pa = parseVersionNumbers(a)
	const pb = parseVersionNumbers(b)
	const maxLen = Math.max(pa.length, pb.length)
	for (let i = 0; i < maxLen; i++) {
		const na = pa[i] ?? 0
		const nb = pb[i] ?? 0
		if (na !== nb) {
			return na - nb
		}
	}
	return 0
}

/**
 * Returns matching ERFeatures mod information for a given instance loader and Minecraft version.
 * Returns null if the instance version/loader is not supported.
 */
export function getCompatibleERFeaturesMod(
	loader?: string | null,
	gameVersion?: string | null,
): ERFeaturesModInfo | null {
	if (!loader || !gameVersion) return null

	const normLoader = loader.trim().toLowerCase()
	const v = gameVersion.trim()

	// Fabric & Quilt
	if (normLoader === 'fabric' || normLoader === 'quilt') {
		// Modern: 1.20.5+ and 26.x+
		if (compareMinecraftVersions(v, '1.20.5') >= 0) {
			return {
				filename: 'ERFeatures-1.2.0-fabric-1.21.jar',
				downloadUrl: 'https://releases.end-rage.ru/mods/ERFeatures-1.2.0-fabric-1.21.jar',
				title: 'ERFeatures (Fabric/Quilt)',
				version: '1.2.0',
				compatibilityDesc: 'Для Fabric & Quilt 1.20.5 – 1.21.4+',
			}
		}
		// Legacy: 1.14 <= v < 1.20.5
		if (compareMinecraftVersions(v, '1.14') >= 0 && compareMinecraftVersions(v, '1.20.5') < 0) {
			return {
				filename: 'ERFeatures-1.1.0-fabric-legacy-1.14-1.20.4.jar',
				downloadUrl: 'https://releases.end-rage.ru/mods/ERFeatures-1.1.0-fabric-legacy-1.14-1.20.4.jar',
				title: 'ERFeatures (Fabric Legacy)',
				version: '1.1.0',
				compatibilityDesc: 'Для Fabric & Quilt 1.14 – 1.20.4',
			}
		}
	}

	// NeoForge: 1.20.2+
	if (normLoader === 'neoforge') {
		if (compareMinecraftVersions(v, '1.20.2') >= 0) {
			return {
				filename: 'ERFeatures-1.1.0-neoforge-1.20.2-plus.jar',
				downloadUrl: 'https://releases.end-rage.ru/mods/ERFeatures-1.1.0-neoforge-1.20.2-plus.jar',
				title: 'ERFeatures (NeoForge)',
				version: '1.1.0',
				compatibilityDesc: 'Для NeoForge 1.20.2 – 1.21.4+',
			}
		}
	}

	// Forge
	if (normLoader === 'forge') {
		// Classic / Legacy: 1.8 <= v <= 1.16.5
		if (compareMinecraftVersions(v, '1.8') >= 0 && compareMinecraftVersions(v, '1.16.5') <= 0) {
			return {
				filename: 'ERFeatures-1.1.0-forge-1.8-1.16.5.jar',
				downloadUrl: 'https://releases.end-rage.ru/mods/ERFeatures-1.1.0-forge-1.8-1.16.5.jar',
				title: 'ERFeatures (Forge 1.16.5)',
				version: '1.1.0',
				compatibilityDesc: 'Для Forge 1.8 – 1.16.5',
			}
		}
		// Modern: 1.20.6+
		if (compareMinecraftVersions(v, '1.20.6') >= 0) {
			return {
				filename: 'ERFeatures-1.1.0-forge-1.20.6-plus.jar',
				downloadUrl: 'https://releases.end-rage.ru/mods/ERFeatures-1.1.0-forge-1.20.6-plus.jar',
				title: 'ERFeatures (Forge Modern)',
				version: '1.1.0',
				compatibilityDesc: 'Для Forge 1.20.6 – 1.21.4+',
			}
		}
	}

	// Loader/version not supported (e.g. Vanilla, or Forge 1.18.2)
	return null
}

/**
 * Checks if ERFeatures or ELFeatures is already installed in the specified instance.
 */
export async function isERFeaturesModInstalled(instanceId: string): Promise<boolean> {
	try {
		const projects = await get_projects(instanceId)
		if (!projects) return false

		for (const key of Object.keys(projects)) {
			const lower = key.toLowerCase()
			if (lower.includes('erfeatures') || lower.includes('elfeatures')) {
				return true
			}
			const item = projects[key]
			if (item?.name && (item.name.toLowerCase().includes('erfeatures') || item.name.toLowerCase().includes('elfeatures'))) {
				return true
			}
		}
		return false
	} catch {
		return false
	}
}

/**
 * Downloads and installs the compatible ERFeatures mod into the instance.
 */
export async function installERFeaturesMod(
	instanceId: string,
	modInfo: ERFeaturesModInfo,
): Promise<string> {
	return await invoke('plugin:utils|install_erfeatures_mod', {
		instanceId,
		downloadUrl: modInfo.downloadUrl,
		fileName: modInfo.filename,
	})
}
