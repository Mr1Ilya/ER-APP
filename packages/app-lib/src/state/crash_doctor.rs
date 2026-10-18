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
    #[serde(alias = "category")]
    pub problem_type: String,
    pub title: String,
    #[serde(alias = "description")]
    pub explanation: String,
    #[serde(alias = "action_type")]
    pub action: CrashActionType,
    #[serde(alias = "action_payload")]
    pub target: Option<String>,
    #[serde(alias = "action_label")]
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
static RE_NON_FABRIC: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)non-fabric\s+mod:\s*[\r\n]+\s*-\s*([^\r\n\t]+)").unwrap()
});
static RE_FABRIC_SOLUTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)A potential solution has been determined:\s*[\r\n]+\s*-\s*Replace mod '([^']+)'").unwrap()
});

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

    if let Some(caps) = RE_NON_FABRIC.captures(log_text) {
        if let Some(m) = caps.get(1) {
            let mod_name = m.as_str().trim();
            let clean_name = mod_name.trim_end_matches(".jar");
            return Some(CrashDiagnosis {
                problem_type: "non_fabric_mod".to_string(),
                title: "Несовместимый мод (не для Fabric)".to_string(),
                explanation: format!(
                    "В сборку добавлен файл «{mod_name}», созданный для Forge или NeoForge. Fabric не может работать с этим модом."
                ),
                action: CrashActionType::DisableMod,
                target: Some(mod_name.to_string()),
                button_label: format!("Отключить {clean_name}"),
            });
        }
    }

    if let Some(caps) = RE_FABRIC_SOLUTION.captures(log_text) {
        if let Some(m) = caps.get(1) {
            let mod_name = m.as_str().trim();
            return Some(CrashDiagnosis {
                problem_type: "incompatible_mod".to_string(),
                title: "Конфликт модов сборки".to_string(),
                explanation: format!(
                    "Мод «{mod_name}» несовместим с текущей версией игры или другими модами сборки."
                ),
                action: CrashActionType::DisableMod,
                target: Some(mod_name.to_string()),
                button_label: format!("Отключить {mod_name}"),
            });
        }
    }

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
        || lower.contains("unable to call jdk findmainmethod")
    {
        return Some(CrashDiagnosis {
            problem_type: "java_version_mismatch".to_string(),
            title: "Несовместимая версия Java".to_string(),
            explanation: "Сборка или один из модов не совместимы с текущей версией Java. Рекомендуется переключить версию Java в настройках сборки.".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_non_fabric_mod_detection() {
        let sample = "[20:43:38] [main] [FabricLoader/WARN]: Found 1 non-fabric mod:\n\t- create-1.20.1-6.0.8.jar\n# Process exited with status: exit code: 1";
        let diag = analyze_crash_log(sample).expect("Must detect non-fabric mod");
        assert_eq!(diag.problem_type, "non_fabric_mod");
        assert_eq!(diag.action, CrashActionType::DisableMod);
        assert_eq!(diag.target.as_deref(), Some("create-1.20.1-6.0.8.jar"));
        assert_eq!(diag.button_label, "Отключить create-1.20.1-6.0.8");
    }

    #[test]
    fn test_out_of_memory_detection() {
        let sample = "java.lang.OutOfMemoryError: Java heap space";
        let diag = analyze_crash_log(sample).expect("Must detect OOM");
        assert_eq!(diag.problem_type, "out_of_memory");
        assert_eq!(diag.action, CrashActionType::IncreaseRam);
    }
}
