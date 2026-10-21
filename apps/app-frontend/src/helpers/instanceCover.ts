import { convertFileSrc } from '@tauri-apps/api/core'
import defaultMinecraftBlock from '@/assets/minecraft_block.png'

const wallpaperModules = import.meta.glob<{ default: string }>(
	'@/assets/wallpapers/*.{png,jpg,jpeg,webp}',
	{ eager: true },
)
export const wallpapers = Object.values(wallpaperModules).map((m) => m.default)

export function getInstanceCover(instance?: { name?: string; icon_path?: string | null } | null): string {
	if (!instance) return defaultMinecraftBlock
	if (instance.icon_path) {
		return convertFileSrc(instance.icon_path)
	}
	let hash = 0
	const name = instance.name || ''
	for (let i = 0; i < name.length; i++) {
		hash = (hash << 5) - hash + name.charCodeAt(i)
		hash |= 0
	}
	if (wallpapers.length > 0) {
		const idx = Math.abs(hash) % wallpapers.length
		return wallpapers[idx] || defaultMinecraftBlock
	}
	return defaultMinecraftBlock
}
