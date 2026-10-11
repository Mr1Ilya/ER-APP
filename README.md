# EndRage APP

<p align="center">
  <img src="apps/app-frontend/src/assets/end_rage_app.svg" width="128" height="128" alt="EndRage APP Logo" />
</p>

<p align="center">
  <a href="https://github.com/Mr1Ilya/ER-APP/releases/latest"><img src="https://img.shields.io/github/v/release/Mr1Ilya/ER-APP?color=8b5cf6&label=Версия" alt="Latest Release" /></a>
  <a href="https://github.com/Mr1Ilya/ER-APP/blob/main/LICENSE"><img src="https://img.shields.io/badge/Лицензия-GPL--3.0-blue" alt="License" /></a>
  <a href="https://tauri.app"><img src="https://img.shields.io/badge/Tauri-2.0-24c8db" alt="Tauri" /></a>
  <a href="https://vuejs.org"><img src="https://img.shields.io/badge/Vue.js-3.5-42b883" alt="Vue 3" /></a>
  <a href="https://t.me/ERTeamAPP"><img src="https://img.shields.io/badge/Telegram-@ERTeamAPP-229ED9" alt="Telegram" /></a>
</p>

**EndRage APP** — современный, быстрый и функциональный Minecraft-лаунчер для игрового сообщества **End-Rage Team**. Построен на базе высокопроизводительного ядра Rust (Tauri) и реактивного интерфейса на Vue 3.

---

## ✨ Ключевые возможности

* 🆔 **Универсальная авторизация**: Поддержка аккаунтов **EndRage ID**, официальных профилей Microsoft и оффлайн-режима.
* 🎭 **Система скинов**: Встроенная интеграция с сервисами скинов EndRage и Ely.by (через `authlib-injector`).
* 🎨 **Редактор главной страницы**: Кастомизация баннера, названия и стиля оформления прямо на главной странице в реальном времени.
* 🖼️ **Адаптивные обложки**: Изображения сборок аккуратно масштабируются на всю карточку без черных полос и искажений.
* 📐 **Настраиваемая сетка каталога**: Выбор удобного количества рядов (2, 3 или 4 ряда) с автоматическим заполнением свободного пространства.
* 📁 **Группировка сборок**: Организация сборок по пользовательским группам, настройка закрепления и удобное управление.
* 🌐 **Выбор маршрута сети**: Переключение режимов подключения (Прямое соединение, Прокси, Cloudflare) с замером пинга в реальном времени.
* 🎥 **Режим стримера**: Скрытие чувствительных данных (email, ID профилей) в интерфейсе при ведении трансляций или записи видео.
* 📰 **Telegram-новости**: Автоматически обновляемая лента новостей прямо в сайдбаре лаунчера из официального канала [@ERTeamAPP](https://t.me/ERTeamAPP).
* 📦 **Каталог Modrinth**: Поиск, быстрая установка и обновление модов, ресурспаков, шейдеров и готовых сборок в один клик.
* ⚡ **Высокая скорость и легковесность**: Ядро на Rust обеспечивает мгновенный запуск и минимальное потребление оперативной памяти.

---

## 📥 Скачать лаунчер

Готовые сборки под все платформы доступны в разделе [GitHub Releases](https://github.com/Mr1Ilya/ER-APP/releases/latest):

* **Windows**: `EndRage.APP_<версия>_x64-setup.exe`
* **macOS**: `EndRage.APP_<версия>_aarch64.dmg` (Apple Silicon)
* **Linux**: `EndRage.APP_<версия>_amd64.AppImage` и `.deb`

---

## 🛠️ Разработка и сборка

### Требования
* **Node.js** >= 22 (рекомендуется 24)
* **pnpm** >= 10 (`corepack enable`)
* **Rust** >= 1.85 (стабильный тулчейн)
* **Java** 17 или 21

### Установка зависимостей
```bash
corepack enable
pnpm install
```

### Запуск в режиме разработки
```bash
pnpm --filter @erteam/app dev
```

### Сборка приложения
* **Сборка фронтенда**:
  ```bash
  pnpm --filter @erteam/app-frontend build
  ```
* **Полная сборка лаунчера (десктопный билд)**:
  ```bash
  pnpm run app:build
  ```
  Исполняемый файл и установщик будут созданы в директории `target/release/`.

---

## ⚠️ Юридическая информация / Disclaimer

EndRage APP является независимым программным обеспечением сообщества End-Rage Team, созданным на базе исходного кода Modrinth App.

Данный проект **НЕ связан, НЕ принадлежит, НЕ одобряется и НЕ спонсируется** компаниями **Microsoft**, **Mojang Studios**, торговой маркой **Minecraft**, а также официальной платформой **Modrinth**. Все права на оригинальные ресурсы и торговые марки принадлежат их законным правообладателям.

---

## 📄 Лицензия

Исходный код проекта распространяется под лицензией **GNU General Public License v3 (GPLv3)**.
