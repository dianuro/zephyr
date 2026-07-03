use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// UI 颜色配置（亮色或暗色）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub app: AppColors,
    pub text: TextColors,
    pub border: BorderColors,
    pub accent: AccentColors,
    pub scrollbar: ScrollbarColors,
    pub search: SearchColors,
    pub syntax: SyntaxColors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppColors {
    pub background: String,
    pub sidebar_bg: String,
    pub toolbar_bg: String,
    pub content_bg: String,
    pub hover_bg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextColors {
    pub primary: String,
    pub secondary: String,
    pub muted: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorderColors {
    pub default: String,
    pub muted: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccentColors {
    pub default: String,
    pub hover: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrollbarColors {
    pub thumb: String,
    pub thumb_hover: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchColors {
    pub highlight_bg: String,
    pub active_bg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxColors {
    pub theme: String,
}

/// 获取配置目录路径
pub fn config_dir() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
    base.join("zephyr").join("theme").join("default")
}

/// 获取亮色配置路径
pub fn light_path() -> PathBuf {
    config_dir().join("light.toml")
}

/// 获取暗色配置路径
pub fn dark_path() -> PathBuf {
    config_dir().join("dark.toml")
}

/// 默认亮色配置
pub fn default_light() -> ThemeConfig {
    ThemeConfig {
        app: AppColors {
            background: "#ffffff".into(),
            sidebar_bg: "#f6f8fa".into(),
            toolbar_bg: "#ffffff".into(),
            content_bg: "#ffffff".into(),
            hover_bg: "#eaeef2".into(),
        },
        text: TextColors {
            primary: "#1F2328".into(),
            secondary: "#656d76".into(),
            muted: "#6e7781".into(),
        },
        border: BorderColors {
            default: "#d0d7de".into(),
            muted: "#d8dee4".into(),
        },
        accent: AccentColors {
            default: "#0969da".into(),
            hover: "#0550ae".into(),
        },
        scrollbar: ScrollbarColors {
            thumb: "#d0d7de".into(),
            thumb_hover: "#6e7781".into(),
        },
        search: SearchColors {
            highlight_bg: "#fff8c5".into(),
            active_bg: "#f5e6a3".into(),
        },
        syntax: SyntaxColors {
            theme: "InspiredGitHub".into(),
        },
    }
}

/// 默认暗色配置
pub fn default_dark() -> ThemeConfig {
    ThemeConfig {
        app: AppColors {
            background: "#0d1117".into(),
            sidebar_bg: "#161b22".into(),
            toolbar_bg: "#161b22".into(),
            content_bg: "#0d1117".into(),
            hover_bg: "#1c2128".into(),
        },
        text: TextColors {
            primary: "#e6edf3".into(),
            secondary: "#848d97".into(),
            muted: "#6e7681".into(),
        },
        border: BorderColors {
            default: "#30363d".into(),
            muted: "#21262d".into(),
        },
        accent: AccentColors {
            default: "#2f81f7".into(),
            hover: "#58a6ff".into(),
        },
        scrollbar: ScrollbarColors {
            thumb: "#30363d".into(),
            thumb_hover: "#6e7681".into(),
        },
        search: SearchColors {
            highlight_bg: "rgba(187,128,9,0.15)".into(),
            active_bg: "rgba(187,128,9,0.4)".into(),
        },
        syntax: SyntaxColors {
            theme: "base16-ocean.dark".into(),
        },
    }
}

/// 加载或创建配置文件
fn load_or_create(path: &PathBuf, default: ThemeConfig) -> ThemeConfig {
    if path.exists() {
        match fs::read_to_string(path) {
            Ok(content) => {
                match toml::from_str::<ThemeConfig>(&content) {
                    Ok(config) => config,
                    Err(e) => {
                        eprintln!("配置文件 {} 解析失败: {}，使用默认值", path.display(), e);
                        default
                    }
                }
            }
            Err(e) => {
                eprintln!("配置文件 {} 读取失败: {}，使用默认值", path.display(), e);
                default
            }
        }
    } else {
        // 创建目录
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        // 写入默认配置
        let toml_str = toml::to_string_pretty(&default).unwrap_or_default();
        let comment = format!(
            "# Zephyr {} 主题配置\n\
             # 修改后重启应用生效\n\n",
            if path.to_string_lossy().contains("light") { "亮色" } else { "暗色" }
        );
        match fs::write(path, comment + &toml_str) {
            Ok(_) => eprintln!("已生成默认配置文件: {}", path.display()),
            Err(e) => eprintln!("无法创建配置文件 {}: {}", path.display(), e),
        }
        default
    }
}

/// 加载两个主题的配置（启动时调用）
pub fn load_configs() -> (ThemeConfig, ThemeConfig) {
    let light = load_or_create(&light_path(), default_light());
    let dark = load_or_create(&dark_path(), default_dark());
    (light, dark)
}

/// 根据主题模式获取对应的配置
pub fn get_config(is_dark: bool) -> ThemeConfig {
    if is_dark {
        load_or_create(&dark_path(), default_dark())
    } else {
        load_or_create(&light_path(), default_light())
    }
}
