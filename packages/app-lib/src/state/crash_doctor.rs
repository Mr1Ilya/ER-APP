use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CrashActionType {
    IncreaseRam,
    DisableMod,
    InstallMod,
    SwitchJava,
    DriverWarning,
    Generic,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CrashDiagnosis {
    pub problem_type: String,
    pub title: String,
    pub explanation: String,
    pub action: CrashActionType,
    pub target: Option<String>,
    pub button_label: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameCrashPayload {
    pub instance_id: String,
    pub instance_name: String,
    pub exit_code: Option<i32>,
    pub diagnosis: Option<CrashDiagnosis>,
    pub scrubbed_log: String,
}

static RE_TOKEN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(--accessToken\s+)\S+").unwrap());
static RE_UUID: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(--uuid\s+)\S+").unwrap());
static RE_WIN_USER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)[a-z]:[\\/]+users[\\/]+[^\\/\r\n]+").unwrap()
});
static RE_UNIX_USER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"/(?:home|Users)/[^/\r\n]+").unwrap());
static RE_EMAIL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b[\w.+-]+@[\w-]+\.[\w.]+\b").unwrap());

pub fn scrub_log(raw: &str) -> String {
    let t1 = RE_TOKEN.replace_all(raw, "$1***");
    let t2 = RE_UUID.replace_all(&t1, "$1***");
    let t3 = RE_WIN_USER.replace_all(&t2, "~");
    let t4 = RE_UNIX_USER.replace_all(&t3, "~");
    let cleaned = RE_EMAIL.replace_all(&t4, "***@***").to_string();

    if cleaned.len() > 10_000 {
        cleaned[cleaned.len() - 10_000..].to_string()
    } else {
        cleaned
    }
}

pub fn analyze_crash_log(log_text: &str) -> Option<CrashDiagnosis> {
    let lower = log_text.to_lowercase();

    if lower.contains("outofmemoryerror")
        || lower.contains("insufficient memory for the java runtime")
    {
        return Some(CrashDiagnosis {
            problem_type: "out_of_memory".to_string(),
            title: "Недостаточно оперативной памяти".to_string(),
            explanation: "Игре не хватило выделенной оперативной памяти (OutOfMemoryError). Рекомендуется увеличить объем памяти в настройках сборки.".to_string(),
            action: CrashActionType::IncreaseRam,
            target: Some("4096".to_string()),
            button_label: "Выделить 4 ГБ ОЗУ".to_string(),
        });
    }

    if lower.contains("requires fabric-api")
        || lower.contains("requires fabric")
        || (lower.contains("entrypointexception") && lower.contains("fabric"))
    {
        return Some(CrashDiagnosis {
            problem_type: "missing_fabric_api".to_string(),
            title: "Отсутствует Fabric API".to_string(),
            explanation: "Для работы модов сборки требуется базовый мод Fabric API, но он не установлен.".to_string(),
            action: CrashActionType::InstallMod,
            target: Some("fabric-api".to_string()),
            button_label: "Установить Fabric API".to_string(),
        });
    }

    if lower.contains("ig4icd64.dll")
        || lower.contains("nvoglv64.dll")
        || lower.contains("atig6pxx.dll")
        || lower.contains("glfw error 65542")
        || lower.contains("pixel format not accelerated")
    {
        return Some(CrashDiagnosis {
            problem_type: "graphics_driver".to_string(),
            title: "Сбой графического драйвера OpenGL".to_string(),
            explanation: "Видеокарта или драйвер сообщили о сбое инициализации OpenGL. Рекомендуется обновить драйвер видеокарты или выбрать дискретный GPU в системе.".to_string(),
            action: CrashActionType::DriverWarning,
            target: None,
            button_label: "Как настроить видеокарту".to_string(),
        });
    }

    if lower.contains("unsupportedclassversionerror")
        || lower.contains("has been compiled by a more recent version of the java runtime")
    {
        return Some(CrashDiagnosis {
            problem_type: "java_version_mismatch".to_string(),
            title: "Несовместимая версия Java".to_string(),
            explanation: "Один или несколько модов собраны под более новую версию Java, чем установлена в настройках сборки.".to_string(),
            action: CrashActionType::SwitchJava,
            target: Some("21".to_string()),
            button_label: "Переключить на Java 21".to_string(),
        });
    }

    if lower.contains("mixintransformererror")
        || lower.contains("mixinapplyerror")
    {
        return Some(CrashDiagnosis {
            problem_type: "mixin_conflict".to_string(),
            title: "Конфликт модов (ошибка Mixin)".to_string(),
            explanation: "Один из модов вызвал конфликт при внедрении кода в клиент игры. Проверьте недавние установленные моды.".to_string(),
            action: CrashActionType::Generic,
            target: None,
            button_label: "Открыть лог сбоя".to_string(),
        });
    }

    None
}
