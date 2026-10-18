# EndRage Crash Doctor — Спецификация и дизайн системы

## 1. Обзор системы
**EndRage Crash Doctor (Крашлитика)** — встроенная в лаунчер система автоматической диагностики и устранения сбоев Minecraft. Система перехватывает аварийное завершение процесса игры, очищает лог от персональных данных, определяет первопричину сбоя через гибридный пайплайн (локальные сигнатуры + серверный ИИ) и предлагает игроку исправление в 1 клик.

---

## 2. Архитектура и поток данных

```
+-------------------------------------------------------------------+
|                           Minecraft                               |
|   (Аварийный выход: mc_exit_status != 0 или crash-report)         |
+---------------------------------+---------------------------------+
                                  |
                                  v
+---------------------------------+---------------------------------+
|               Rust Core (Tauri / packages/app-lib)                |
|  1. sequential_process_manager перехватывает ненулевой код        |
|  2. Чтение хвоста лога (LOG_BUFFERS) и crash-reports/*.txt        |
|  3. Scrubbing: удаление токенов, ников, e-mail, путей Windows     |
|  4. Локальный детектор частых ошибок (OOM, Driver, Mixin, Java)   |
|  5. Эмиссия события: game_crashed                                 |
+---------------------------------+---------------------------------+
                                  |
                                  v
+---------------------------------+---------------------------------+
|             Frontend UI (Vue 3 / apps/app-frontend)               |
|  1. Разворачивание окна лаунчера из фонового режима               |
|  2. Отображение CrashDoctorModal в строгом стиле EndRage          |
|  3. Если причина определена локально — мгновенный вердикт         |
|  4. Если причина неизвестна — запрос к api.end-rage.ru            |
|  5. Кнопка автопочинки в 1 клик (изменить RAM, отключить мод)     |
+---------------------------------+---------------------------------+
                                  | (только при неизвестном краше)
                                  v
+---------------------------------+---------------------------------+
|           Server Backend (launcher-backend: 4004)                 |
|  1. Эндпоинт POST /api/crash-doctor                               |
|  2. Проверка кэша по SHA-256 хешу стектрейса                      |
|  3. Запрос к LLM (Gemini Flash / DeepSeek)                        |
|  4. Возврат структурированного JSON вердикта                      |
+-------------------------------------------------------------------+
```

---

## 3. Детализация компонентов

### 3.1. Ядро Rust (`packages/app-lib/src/state/process.rs`)
1. **Детекция аварии:**
   - Проверка `!mc_exit_status.success()`.
   - Проверка наличия свежего файла в директории `<instance_path>/crash-reports/` с временем создания не старше 15 секунд.
2. **Очистка данных (Scrubbing):**
   - `--accessToken\s+\S+` -> `--accessToken ***`
   - `--uuid\s+\S+` -> `--uuid ***`
   - `[a-zA-Z]:[\\/]+[Uu]sers[\\/]+[^\\/\r\n]+` -> `~`
   - `/(?:home|Users)/[^/\r\n]+` -> `~`
   - `[\w.+-]+@[\w-]+\.[\w.]+` -> `***@***`
3. **Локальные сигнатуры (Regex & Keywords):**
   - **`OutOfMemory`:** `java.lang.OutOfMemoryError` или `There is insufficient memory for the Java Runtime Environment to continue`.
     *Действие:* `increase_ram`, рекомендованное значение: 4096 МБ или 6144 МБ.
   - **`MissingFabricApi`:** `requires fabric` / `requires fabric-api` / `EntrypointException: ... fabric`.
     *Действие:* `install_mod`, `target: fabric-api`.
   - **`DriverOpenGl`:** `ig4icd64.dll` / `nvoglv64.dll` / `atig6pxx.dll` / `GLFW error 65542: WGL`.
     *Действие:* `driver_warning`, инструкция по выбору дискретной графики.
   - **`JavaVersionMismatch`:** `has been compiled by a more recent version of the Java Runtime` / `UnsupportedClassVersionError`.
     *Действие:* `switch_java`, переход на Java 17/21.
   - **`MixinConflict`:** `org.spongepowered.asm.mixin.transformer.throwables.MixinTransformerError`.
     *Действие:* извлечение имени проблемного мода и `disable_mod`.

4. **Событие Tauri:**
   - Имя события: `game_crashed`.
   - Структура:
     ```rust
     pub struct GameCrashPayload {
         pub instance_id: String,
         pub instance_name: String,
         pub exit_code: Option<i32>,
         pub local_diagnosis: Option<CrashDiagnosis>,
         pub scrubbed_log: String,
     }
     ```

---

### 3.2. Интерфейс `CrashDoctorModal.vue`
- Построен в строгом стиле EndRage:
  - Никакого свечения (`NO GLOW`, запрет `shadow-[0_0_...]`).
  - Обязательный `border-solid` у всех рамок (`border-2 border-solid`).
  - Акцентная кнопка починки: `border-2 border-solid border-purple-500 bg-[rgba(172,81,251,0.18)] text-white hover:bg-[rgba(172,81,251,0.28)]`.
  - Второстепенные кнопки: `border-2 border-solid border-white/10 hover:border-white/20`.
- **Поддерживаемые действия в 1 клик:**
  - `increase_ram`: авто-применение настроек ОЗУ сборки через IPC `instance_edit`.
  - `disable_mod`: отключение конфликтующего jar (добавление расширения `.disabled` или вызов `toggle_mod`).
  - `install_dep`: вызов встроенной установки мода/библиотеки.
  - `copy_log`: копирование очищенного стектрейса в буфер с шаблоном для саппорта.

---

### 3.3. Серверный API (`launcher-backend/src/services/apiService.ts`)
- Эндпоинт: `POST /api/crash-doctor`
- Входные данные:
  ```json
  {
    "gameVersion": "1.20.1",
    "loader": "fabric",
    "logTail": "string (максимум 6000 символов)"
  }
  ```
- Логика работы:
  1. Вычисление SHA-256 хэша от очищенного стектрейса.
  2. Проверка наличия ответа в кэше.
  3. При промахе кэша — обращение к модели Gemini Flash через официальный API.
  4. Формат ответа клиенту:
     ```json
     {
       "title": "Краткая суть ошибки",
       "explanation": "Объяснение на понятном русском языке (2-3 предложения)",
       "action": "disable_mod" | "increase_ram" | "update_java" | "install_dep" | "generic",
       "target": "имя_мода_или_параметра",
       "buttonLabel": "Текст кнопки исправления"
     }
     ```
  5. Сохранение ответа в кэш.

---

## 4. Безопасность и отказоустойчивость
1. **Полная изоляция личных данных:** Токены Microsoft / EndRage, реальные пути к файлам Windows пользователя (`C:\Users\<имя>`) и email адреса никогда не покидают компьютер игрока.
2. **Отказоустойчивость при отсутствии сети:** Если сервер недоступен, локальный детектор отрабатывает автономно, а при неизвестной ошибке интерфейс предлагает открыть локальный файл лога и скопировать его в поддержку.
3. **Защита от спама:** Ограничение частоты запросов на сервере (Rate limit) до 10 запросов в час на IP-адрес.
