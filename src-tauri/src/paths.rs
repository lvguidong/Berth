use std::path::PathBuf;

pub fn key(path: &str) -> String {
    normalize(path, cfg!(windows))
}
pub fn normalize(path: &str, windows: bool) -> String {
    if windows {
        let p = path.replace('/', "\\");
        let p = if let Some(s) = p.strip_prefix("\\\\?\\UNC\\") {
            format!("\\\\{s}")
        } else {
            p.strip_prefix("\\\\?\\").unwrap_or(&p).to_owned()
        };
        p.trim_end_matches('\\').to_lowercase()
    } else {
        if path == "/" {
            "/".into()
        } else {
            path.trim_end_matches('/').into()
        }
    }
}
pub fn expand(input: &str) -> Result<PathBuf, String> {
    let mut s = input.trim().trim_matches('"').to_string();
    if s == "~" || s.starts_with("~/") || s.starts_with("~\\") {
        let home = dirs::home_dir().ok_or("无法确定用户目录")?;
        s = format!("{}{}", home.display(), &s[1..]);
    }
    #[cfg(windows)]
    {
        let mut output = String::new();
        let mut rest = s.as_str();
        while let Some(start) = rest.find('%') {
            output.push_str(&rest[..start]);
            if let Some(end) = rest[start + 1..].find('%') {
                let name = &rest[start + 1..start + 1 + end];
                output.push_str(&std::env::var(name).unwrap_or_else(|_| format!("%{name}%")));
                rest = &rest[start + end + 2..];
            } else {
                output.push_str(&rest[start..]);
                rest = "";
                break;
            }
        }
        output.push_str(rest);
        s = output;
    }
    let p = PathBuf::from(s);
    if !p.is_absolute() {
        return Err("请输入完整的文件夹路径".into());
    }
    Ok(p)
}
pub fn resolve(input: &str) -> Result<String, String> {
    let p = expand(input)?;
    if !p.is_dir() {
        return Err("路径不可用：文件夹不存在、已离线或无访问权限".into());
    }
    let canonical = std::fs::canonicalize(&p).map_err(|e| e.to_string())?;
    let s = canonical.to_string_lossy().into_owned();
    #[cfg(windows)]
    {
        if let Some(p) = s.strip_prefix("\\\\?\\UNC\\") {
            return Ok(format!("\\\\{p}"));
        }
        Ok(s.strip_prefix("\\\\?\\").unwrap_or(&s).into())
    }
    #[cfg(not(windows))]
    {
        Ok(s)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn windows_equivalent() {
        assert_eq!(normalize("C:/Work/", true), normalize(r"\\?\c:\work", true));
        assert_eq!(
            normalize(r"\\?\UNC\Server\Share\", true),
            normalize(r"\\server\share", true)
        );
    }
    #[test]
    fn mac_case_is_preserved() {
        assert_ne!(
            normalize("/Volumes/A", false),
            normalize("/Volumes/a", false)
        );
        assert_eq!(normalize("/", false), "/");
    }
    #[test]
    fn rejects_relative_and_files() {
        assert!(expand("relative").is_err());
        let f = tempfile::NamedTempFile::new().unwrap();
        assert!(resolve(f.path().to_str().unwrap()).is_err());
    }
    #[test]
    fn resolves_existing_directory() {
        let d = tempfile::tempdir().unwrap();
        assert!(resolve(d.path().to_str().unwrap()).is_ok());
    }
    #[test]
    fn literal_percent_and_unicode_are_valid() {
        let d = tempfile::Builder::new()
            .prefix("Berth 设计 # % ")
            .tempdir()
            .unwrap();
        assert!(resolve(d.path().to_str().unwrap()).is_ok());
    }
}
