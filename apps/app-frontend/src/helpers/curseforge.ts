import { fetch as tauriFetch } from '@tauri-apps/plugin-http'
import { mkdir, readFile, writeFile } from '@tauri-apps/plugin-fs'
import JSZip from 'jszip'
import { install_create_instance, type InstanceLoader } from './install'
import { get_full_path, list } from './instance'

export const CURSEFORGE_API_KEY = '$2a$10$bL4bIL5pUWqfcO7KQtnMReakwtfHbNKh6v1uTpKlzhwoueEJQnPnm'
export const CURSEFORGE_API_BASE = 'https://api.curseforge.com/v1'

export interface CurseForgeModItem {
	id: number
	name: string
	slug: string
	summary: string
	downloadCount: number
	logo?: {
		id: number
		title: string
		thumbnailUrl: string
		url: string
	}
	authors?: {
		id: number
		name: string
		url: string
	}[]
	latestFilesIndexes?: {
		gameVersion: string
		fileId: number
		filename: string
		modLoader?: number
	}[]
}

export interface CurseForgeFileItem {
	id: number
	modId: number
	displayName: string
	fileName: string
	fileDate: string
	fileLength: number
	downloadUrl: string | null
	gameVersions: string[]
}

export interface CurseForgeManifest {
	minecraft: {
		version: string
		modLoaders: {
			id: string
			primary: boolean
		}[]
	}
	manifestType: string
	manifestVersion: number
	name: string
	version: string
	author: string
	files: {
		projectID: number
		fileID: number
		required: boolean
	}[]
	overrides: string
}

async function cfRequest<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
	const url = endpoint.startsWith('http') ? endpoint : `${CURSEFORGE_API_BASE}${endpoint}`
	const res = await tauriFetch(url, {
		...options,
		headers: {
			'x-api-key': CURSEFORGE_API_KEY,
			'Accept': 'application/json',
			...(options.headers || {}),
		},
	})

	if (!res.ok) {
		throw new Error(`CurseForge API error: ${res.status} ${res.statusText}`)
	}

	return (await res.json()) as T
}

export async function searchCurseForgeModpacks(
	query = '',
	pageSize = 20,
): Promise<CurseForgeModItem[]> {
	try {
		const searchFilter = query ? `&searchFilter=${encodeURIComponent(query)}` : ''
		const data = await cfRequest<{ data: CurseForgeModItem[] }>(
			`/mods/search?gameId=432&classId=4471&pageSize=${pageSize}&sortField=2&sortOrder=desc${searchFilter}`,
		)
		return data.data || []
	} catch (err) {
		console.error('Failed to search CurseForge modpacks:', err)
		return []
	}
}

export async function getCurseForgeModpackFiles(modId: number): Promise<CurseForgeFileItem[]> {
	try {
		const data = await cfRequest<{ data: CurseForgeFileItem[] }>(
			`/mods/${modId}/files?pageSize=25`,
		)
		return data.data || []
	} catch (err) {
		console.error('Failed to get CurseForge modpack files:', err)
		return []
	}
}

export async function getCurseForgeModDetails(modId: number): Promise<CurseForgeModItem | null> {
	try {
		const data = await cfRequest<{ data: CurseForgeModItem }>(`/mods/${modId}`)
		return data.data || null
	} catch (err) {
		console.error('Failed to get CurseForge mod details:', err)
		return null
	}
}

export function getCurseForgeFileDownloadUrl(file: { id: number; fileName: string; downloadUrl?: string | null }): string {
	if (file.downloadUrl) return file.downloadUrl
	const part1 = Math.floor(file.id / 1000)
	const part2 = file.id % 1000
	return `https://edge.forgecdn.net/files/${part1}/${part2}/${encodeURIComponent(file.fileName)}`
}

export async function parseCurseForgeZip(data: ArrayBuffer | Uint8Array): Promise<{
	manifest: CurseForgeManifest
	zip: JSZip
}> {
	const zip = await JSZip.loadAsync(data)
	const manifestFile = zip.file('manifest.json')
	if (!manifestFile) {
		throw new Error('Архив не является сборкой CurseForge (файл manifest.json не найден)')
	}
	const content = await manifestFile.async('string')
	const manifest = JSON.parse(content) as CurseForgeManifest
	return { manifest, zip }
}

export interface InstallCurseForgeModpackOptions {
	filePath?: string
	zipData?: ArrayBuffer | Uint8Array
	zipFile?: File
	downloadUrl?: string
	customName?: string
	onProgress?: (status: string, current?: number, total?: number) => void
}

export async function installCurseForgeModpack(options: InstallCurseForgeModpackOptions): Promise<string> {
	const { onProgress } = options

	let zipBytes: ArrayBuffer | Uint8Array

	if (options.filePath) {
		onProgress?.('Чтение файла сборки...')
		zipBytes = await readFile(options.filePath)
	} else if (options.zipData) {
		zipBytes = options.zipData
	} else if (options.zipFile) {
		onProgress?.('Чтение файла сборки...')
		zipBytes = await options.zipFile.arrayBuffer()
	} else if (options.downloadUrl) {
		onProgress?.('Скачивание архива сборки с CurseForge...')
		const res = await tauriFetch(options.downloadUrl)
		if (!res.ok) {
			throw new Error(`Не удалось скачать сборку: ${res.status} ${res.statusText}`)
		}
		zipBytes = await res.arrayBuffer()
	} else {
		throw new Error('Не указан источник сборки')
	}

	onProgress?.('Анализ манифеста сборки...')
	const { manifest, zip } = await parseCurseForgeZip(zipBytes)

	// Clean name
	const rawName = options.customName?.trim() || manifest.name || 'CurseForge Pack'
	const packName = rawName.replace(/[<>:"/\\|?*]/g, '').trim()
	const gameVersion = manifest.minecraft?.version || '1.20.1'

	let loader: InstanceLoader = 'forge'
	let loaderVersion: string | null = null

	if (manifest.minecraft?.modLoaders && manifest.minecraft.modLoaders.length > 0) {
		const rawLoaderId = manifest.minecraft.modLoaders[0].id.toLowerCase()
		const parts = rawLoaderId.split('-')
		const lType = parts[0]
		if (lType.includes('fabric')) loader = 'fabric'
		else if (lType.includes('neoforge')) loader = 'neoforge'
		else if (lType.includes('quilt')) loader = 'quilt'
		else loader = 'forge'
		loaderVersion = parts.slice(1).join('-') || null
	}

	onProgress?.(`Создание профиля «${packName}» (${loader} ${gameVersion})...`)
	await install_create_instance({
		name: packName,
		gameVersion,
		loader,
		loaderVersion,
		iconPath: null,
	})

	// Wait briefly for instance folder to register
	let instanceId: string | null = null
	let fullPath: string | null = null

	for (let i = 0; i < 20; i++) {
		const instances = await list().catch(() => [])
		const target = instances.find((inst) => inst.name === packName)
		if (target) {
			instanceId = target.id
			fullPath = await get_full_path(instanceId).catch(() => null)
			if (fullPath) break
		}
		await new Promise((r) => setTimeout(r, 250))
	}

	if (!instanceId || !fullPath) {
		throw new Error('Не удалось получить путь к создаваемому профилю')
	}

	// Unpack overrides
	const overridesPrefix = `${manifest.overrides || 'overrides'}/`
	onProgress?.('Распаковка конфигураций и файлов сборки (overrides)...')

	for (const [relativePath, entry] of Object.entries(zip.files)) {
		if (entry.dir) continue
		if (relativePath.startsWith(overridesPrefix)) {
			const subPath = relativePath.slice(overridesPrefix.length)
			if (!subPath) continue

			const normalizedSubPath = subPath.replace(/\//g, '\\')
			const targetFilePath = `${fullPath}\\${normalizedSubPath}`

			const parentDir = targetFilePath.substring(0, targetFilePath.lastIndexOf('\\'))
			try {
				await mkdir(parentDir, { recursive: true })
			} catch (_) {}

			const fileBytes = await entry.async('uint8array')
			try {
				await writeFile(targetFilePath, fileBytes)
			} catch (writeErr) {
				console.warn(`Не удалось записать файл override ${subPath}:`, writeErr)
			}
		}
	}

	// Fetch files info from CurseForge in chunks of 50
	const filesList = manifest.files || []
	if (filesList.length > 0) {
		const modsDir = `${fullPath}\\mods`
		try {
			await mkdir(modsDir, { recursive: true })
		} catch (_) {}

		const fileIds = filesList.map((f) => f.fileID)
		const chunkSize = 50
		const allResolvedFiles: { id: number; fileName: string; downloadUrl: string }[] = []

		onProgress?.(`Получение информации о модах (${fileIds.length} шт.)...`, 0, fileIds.length)

		for (let i = 0; i < fileIds.length; i += chunkSize) {
			const chunk = fileIds.slice(i, i + chunkSize)
			try {
				const res = await cfRequest<{ data: CurseForgeFileItem[] }>('/mods/files', {
					method: 'POST',
					headers: { 'Content-Type': 'application/json' },
					body: JSON.stringify({ fileIds: chunk }),
				})

				for (const item of res.data || []) {
					allResolvedFiles.push({
						id: item.id,
						fileName: item.fileName,
						downloadUrl: getCurseForgeFileDownloadUrl(item),
					})
				}
			} catch (err) {
				console.warn('Ошибка при получении файлов из CurseForge API:', err)
			}
		}

		// Download all mods concurrently (6 at a time) with timeout and retries
		let downloaded = 0
		const totalMods = allResolvedFiles.length

		const downloadModWithTimeout = async (mod: { id: number; fileName: string; downloadUrl: string }) => {
			const targetPath = `${modsDir}\\${mod.fileName}`
			const urlsToTry = [
				mod.downloadUrl,
				`https://mediafilez.forgecdn.net/files/${Math.floor(mod.id / 1000)}/${mod.id % 1000}/${encodeURIComponent(mod.fileName)}`,
			]

			for (const url of urlsToTry) {
				for (let attempt = 0; attempt < 2; attempt++) {
					try {
						// 20s timeout per download
						const controller = new AbortController()
						const timeoutId = setTimeout(() => controller.abort(), 20000)

						let res: Response
						try {
							res = await tauriFetch(url, { signal: controller.signal })
						} catch (_) {
							res = await fetch(url, { signal: controller.signal })
						} finally {
							clearTimeout(timeoutId)
						}

						if (res && res.ok) {
							const modBytes = new Uint8Array(await res.arrayBuffer())
							if (modBytes.length > 0) {
								await writeFile(targetPath, modBytes)
								return
							}
						}
					} catch (e) {
						// retry next
					}
				}
			}
			console.warn(`Не удалось скачать мод после попыток: ${mod.fileName}`)
		}

		const poolSize = 6
		const queue = [...allResolvedFiles]
		const workers = Array.from({ length: poolSize }, async () => {
			while (queue.length > 0) {
				const item = queue.shift()
				if (!item) break
				await downloadModWithTimeout(item).catch(() => {})
				downloaded++
				onProgress?.(
					`Загрузка модов CurseForge: ${downloaded} из ${totalMods}...`,
					downloaded,
					totalMods,
				)
			}
		})

		await Promise.all(workers)
	}

	onProgress?.('Сборка CurseForge успешно установлена!')
	return instanceId
}
