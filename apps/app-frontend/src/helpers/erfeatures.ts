import { invoke } from '@tauri-apps/api/core'
import { get, get_projects, remove_project } from './instance'

export interface ERFeaturesModInfo {
	filename: string
	downloadUrl: string
	title: string
	version: string
	compatibilityDesc: string
}

export interface InstalledERFeaturesMod {
	installed: boolean
	installedPath?: string
	installedFilename?: string
	installedVersion?: string
	needsUpdate?: boolean
}

let cachedManifest: any = null
let manifestFetchPromise: Promise<any> | null = null

export async function fetchRemoteManifest(): Promise<any> {
	if (cachedManifest) return cachedManifest
	if (manifestFetchPromise) return await manifestFetchPromise

	manifestFetchPromise = (async () => {
		try {
			const res = await fetch('https://releases.end-rage.ru/mods/manifest.json', {
				headers: { 'Cache-Control': 'no-cache' },
			})
			if (res.ok) {
				const json = await res.json()
				cachedManifest = json
				return json
			}
		} catch {
		}
		return null
	})()

	return await manifestFetchPromise
}

export function extractVersionFromFilename(filename: string): string | null {
	const match = filename.match(/(\d+\.\d+\.\d+(?:-[a-zA-Z0-9.]+)?)/)
	return match ? match[1] : null
}

function parseVersionNumbers(v: string): number[] {
	if (!v) return [0]
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

export function getCompatibleERFeaturesMod(
	loader?: string | null,
	gameVersion?: string | null,
): ERFeaturesModInfo | null {
	if (!loader || !gameVersion) return null

	const normLoader = loader.trim().toLowerCase()
	const v = gameVersion.trim()
	const remoteMods = cachedManifest?.mods

	if (normLoader === 'fabric' || normLoader === 'quilt') {
		if (compareMinecraftVersions(v, '1.20.5') >= 0) {
			const m = remoteMods?.fabric_modern
			return {
				filename: m?.filename ?? 'ERFeatures-1.2.0-fabric-1.21.jar',
				downloadUrl: m?.url ?? 'https://releases.end-rage.ru/mods/ERFeatures-1.2.0-fabric-1.21.jar',
				title: m?.title ?? 'ERFeatures (Fabric/Quilt)',
				version: m?.version ?? '1.2.0',
				compatibilityDesc: m?.compatibilityDesc ?? 'Для Fabric и Quilt 1.20.5 – 1.21.4+',
			}
		}
		if (compareMinecraftVersions(v, '1.14') >= 0 && compareMinecraftVersions(v, '1.20.5') < 0) {
			const m = remoteMods?.fabric_legacy
			return {
				filename: m?.filename ?? 'ERFeatures-1.2.0-fabric-legacy-1.14-1.20.4.jar',
				downloadUrl: m?.url ?? 'https://releases.end-rage.ru/mods/ERFeatures-1.2.0-fabric-legacy-1.14-1.20.4.jar',
				title: m?.title ?? 'ERFeatures (Fabric Legacy)',
				version: m?.version ?? '1.2.0',
				compatibilityDesc: m?.compatibilityDesc ?? 'Для Fabric и Quilt 1.14 – 1.20.4',
			}
		}
	}

	if (normLoader === 'neoforge') {
		if (compareMinecraftVersions(v, '1.20.2') >= 0) {
			const m = remoteMods?.neoforge
			return {
				filename: m?.filename ?? 'ERFeatures-1.2.0-neoforge-1.20.2-plus.jar',
				downloadUrl: m?.url ?? 'https://releases.end-rage.ru/mods/ERFeatures-1.2.0-neoforge-1.20.2-plus.jar',
				title: m?.title ?? 'ERFeatures (NeoForge)',
				version: m?.version ?? '1.2.0',
				compatibilityDesc: m?.compatibilityDesc ?? 'Для NeoForge 1.20.2 – 1.21.4+',
			}
		}
	}

	if (normLoader === 'forge') {
		if (compareMinecraftVersions(v, '1.8') >= 0 && compareMinecraftVersions(v, '1.16.5') <= 0) {
			const m = remoteMods?.forge_legacy
			return {
				filename: m?.filename ?? 'ERFeatures-1.2.0-forge-1.8-1.16.5.jar',
				downloadUrl: m?.url ?? 'https://releases.end-rage.ru/mods/ERFeatures-1.2.0-forge-1.8-1.16.5.jar',
				title: m?.title ?? 'ERFeatures (Forge 1.16.5)',
				version: m?.version ?? '1.2.0',
				compatibilityDesc: m?.compatibilityDesc ?? 'Для Forge 1.8 – 1.16.5',
			}
		}
		if (compareMinecraftVersions(v, '1.20.6') >= 0) {
			const m = remoteMods?.forge_modern
			return {
				filename: m?.filename ?? 'ERFeatures-1.2.0-forge-1.20.6-plus.jar',
				downloadUrl: m?.url ?? 'https://releases.end-rage.ru/mods/ERFeatures-1.2.0-forge-1.20.6-plus.jar',
				title: m?.title ?? 'ERFeatures (Forge Modern)',
				version: m?.version ?? '1.2.0',
				compatibilityDesc: m?.compatibilityDesc ?? 'Для Forge 1.20.6 – 1.21.4+',
			}
		}
	}

	return null
}

export async function getInstalledERFeaturesMod(
	instanceId: string,
	compatibleMod?: ERFeaturesModInfo | null,
): Promise<InstalledERFeaturesMod> {
	try {
		const projects = await get_projects(instanceId, 'bypass')
		if (!projects) return { installed: false }

		for (const [pathKey, item] of Object.entries(projects)) {
			const lowerKey = pathKey.toLowerCase()
			const lowerName = item?.name ? item.name.toLowerCase() : ''
			if (lowerKey.includes('erfeatures') || lowerKey.includes('elfeatures') || lowerName.includes('erfeatures') || lowerName.includes('elfeatures')) {
				const filename = pathKey.split('/').pop()?.split('\\').pop() || item?.name || ''
				const installedVersion = extractVersionFromFilename(filename) || '1.2.0'
				const needsUpdate = compatibleMod ? filename.toLowerCase() !== compatibleMod.filename.toLowerCase() : false
				return {
					installed: true,
					installedPath: pathKey,
					installedFilename: filename,
					installedVersion,
					needsUpdate,
				}
			}
		}
		return { installed: false }
	} catch {
		return { installed: false }
	}
}

export async function isERFeaturesModInstalled(instanceId: string): Promise<boolean> {
	const info = await getInstalledERFeaturesMod(instanceId)
	return info.installed
}

export async function installERFeaturesMod(
	instanceId: string,
	modInfo: ERFeaturesModInfo,
	oldPath?: string | null,
): Promise<string> {
	if (oldPath) {
		try {
			await remove_project(instanceId, oldPath)
		} catch {
		}
	}
	return await invoke('plugin:utils|install_erfeatures_mod', {
		instanceId,
		downloadUrl: modInfo.downloadUrl,
		fileName: modInfo.filename,
	})
}

export async function autoSyncERFeaturesForInstance(instanceId: string): Promise<boolean> {
	try {
		await fetchRemoteManifest()
		const inst = await get(instanceId)
		if (!inst) return false
		const modInfo = getCompatibleERFeaturesMod(inst.loader, inst.game_version)
		if (!modInfo) return false
		const installedInfo = await getInstalledERFeaturesMod(instanceId, modInfo)
		if (installedInfo.installed && installedInfo.needsUpdate) {
			await installERFeaturesMod(instanceId, modInfo, installedInfo.installedPath)
			return true
		}
		return false
	} catch {
		return false
	}
}
