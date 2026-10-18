# План реализации EndRage Crash Doctor

## Задача:
Внедрить систему автоматической диагностики и автопочинки вылетов Minecraft (Крашлитика) на базе гибридного пайплайна (локальный парсер в Rust + fallback на серверный ИИ).

---

### Шаг 1: Ядро Rust (`packages/app-lib`)
- [ ] Создать модуль `packages/app-lib/src/state/crash_doctor.rs`:
  - Функция очистки персональных данных `scrub_log`: скрытие токенов сессий, ников, почтовых адресов и путей Windows (`C:\Users\...`).
  - Детектор сигнатур `analyze_crash_log`: распознавание OutOfMemory, Missing Fabric API / зависимости, несовместимости видеодрайвера (OpenGL/GLFW), ошибки версий Java, конфликтов миксинов.
  - Структуры данных `CrashDiagnosis`, `CrashAction`, `GameCrashPayload`.
- [ ] Зарегистрировать модуль в `packages/app-lib/src/state/mod.rs`.
- [ ] Расширить события в `packages/app-lib/src/event/mod.rs` и `packages/app-lib/src/event/emit.rs`:
  - Добавить событие `game_crashed` с полезной нагрузкой `GameCrashPayload`.
- [ ] В `packages/app-lib/src/state/process.rs` (`sequential_process_manager`):
  - При `!mc_exit_status.success()` или появлении свежего файла в `crash-reports/`: собрать последние строки из `LOG_BUFFERS`, прогнать через `crash_doctor` и отправить событие в UI.

---

### Шаг 2: Серверный микросервис (`launcher-backend`)
- [ ] Добавить эндпоинт `POST /api/crash-doctor` в `services/apiService.ts`:
  - Проверка и очистка входного лога (до 6000 символов).
  - Кэширование ответов по SHA-256 хешу стектрейса для мгновенного ответа и экономии токенов.
  - Интеграция с API нейросети (Gemini Flash) со строгим JSON-промптом.
  - Ограничение частоты запросов (Rate Limiting).
- [ ] Протестировать локально и развернуть на боевом сервере `api.end-rage.ru` через PM2.

---

### Шаг 3: Интерфейс и модальное окно (`apps/app-frontend`)
- [ ] Создать компонент `apps/app-frontend/src/components/ui/modal/CrashDoctorModal.vue`:
  - Строгий стиль EndRage: плоские панели, `border-2 border-solid`, фиолетовые акценты, строго без glow (`NO GLOW`).
  - Плашка диагноза: мгновенный показ локального вердикта или индикатор анализа ИИ.
  - Действие в 1 клик:
    - `increase_ram`: авто-обновление настроек сборки через `instance_edit`.
    - `disable_mod`: отключение конфликтующего jar файла.
    - `install_dep`: вызов встроенного установщика мода.
  - Раскрывающийся лог вылета с моноширинным шрифтом.
  - Кнопка «Скопировать для поддержки» с готовым форматированием для Telegram.
- [ ] Подключить модалку в `apps/app-frontend/src/App.vue` и подписаться на событие `game_crashed`.

---

### Шаг 4: Сборка и проверка
- [ ] Проверить сборку фронтенда:
  ```powershell
  pnpm --filter @erteam/app-frontend build
  ```
- [ ] Проверить форматирование Cargo:
  ```powershell
  & "C:\Users\ggeiz\AppData\Local\Programs\Python\Python313\Scripts\tombi.exe" fmt apps/app/Cargo.toml
  ```
- [ ] Протестировать перехват на тестовом запуске с эмулированным сбоем.
