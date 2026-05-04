use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};

const SKILL_CONTENT: &str = include_str!("../skills/exodata.md");
const MARKER: &str = "installed-by: exodata";
const SKILL_DIR_NAME: &str = "exodata";
const SKILL_FILE_NAME: &str = "SKILL.md";

fn agents_skills_dir(base: &Path) -> PathBuf {
    base.join(".agents").join("skills").join(SKILL_DIR_NAME)
}

fn skill_file_path(base: &Path) -> PathBuf {
    agents_skills_dir(base).join(SKILL_FILE_NAME)
}

fn is_our_install(path: &Path) -> bool {
    fs::read_to_string(path).is_ok_and(|content| content.contains(MARKER))
}

pub fn install_local() -> Result<()> {
    let base = std::env::current_dir().context("cannot get current directory")?;
    let message = install_to(&base)?;
    println!("{message}");
    Ok(())
}

pub fn install_global() -> Result<()> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow!("cannot determine home directory"))?;
    let message = install_to(&home)?;
    println!("{message}");
    Ok(())
}

fn install_to(base: &Path) -> Result<String> {
    let dir = agents_skills_dir(base);
    let target = skill_file_path(base);

    fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create {}", dir.display()))?;

    if target.exists() && !is_our_install(&target) {
        return Ok(format!(
            "Skipped: {} exists but was not installed by exodata (no '{}' marker). Remove it manually to install.",
            target.display(),
            MARKER
        ));
    }

    let updating = target.exists();
    fs::write(&target, SKILL_CONTENT)
        .with_context(|| format!("failed to write {}", target.display()))?;

    let action = if updating { "Updated" } else { "Installed" };
    Ok(format!("{action} skill: {}", target.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_base(name: &str) -> PathBuf {
        std::env::temp_dir()
            .join(format!("exodata-skill-{name}-{}", std::process::id()))
    }

    #[test]
    fn marker_is_in_embedded_content() {
        assert!(
            SKILL_CONTENT.contains(MARKER),
            "embedded skill content must contain the marker '{MARKER}'"
        );
    }

    #[test]
    fn agents_skills_dir_path() {
        let base = Path::new("/tmp/project");
        assert_eq!(
            agents_skills_dir(base),
            PathBuf::from("/tmp/project/.agents/skills/exodata")
        );
    }

    #[test]
    fn skill_file_path_resolves() {
        let base = Path::new("/tmp/project");
        assert_eq!(
            skill_file_path(base),
            PathBuf::from("/tmp/project/.agents/skills/exodata/SKILL.md")
        );
    }

    #[test]
    fn is_our_install_detects_marker() {
        let dir = temp_base("marker");
        let file = dir.join("SKILL.md");
        fs::create_dir_all(&dir).unwrap();
        fs::write(&file, "---\ninstalled-by: exodata\n---\n").unwrap();

        assert!(is_our_install(&file));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn is_our_install_rejects_missing_marker() {
        let dir = temp_base("nomarker");
        let file = dir.join("SKILL.md");
        fs::create_dir_all(&dir).unwrap();
        fs::write(&file, "---\nname: exodata\n---\n").unwrap();

        assert!(!is_our_install(&file));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn install_to_creates_dirs_and_writes_file() {
        let dir = temp_base("install");

        let result = install_to(&dir).unwrap();

        assert!(result.starts_with("Installed skill:"));
        assert!(skill_file_path(&dir).exists());
        assert_eq!(
            fs::read_to_string(skill_file_path(&dir)).unwrap(),
            SKILL_CONTENT
        );

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn install_to_updates_existing_our_install() {
        let dir = temp_base("update");
        let target = skill_file_path(&dir);
        fs::create_dir_all(agents_skills_dir(&dir)).unwrap();
        fs::write(&target, format!("---\n{MARKER}\n---\nold content")).unwrap();

        let result = install_to(&dir).unwrap();

        assert!(result.starts_with("Updated skill:"));
        assert_eq!(fs::read_to_string(&target).unwrap(), SKILL_CONTENT);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn install_to_skips_foreign_file() {
        let dir = temp_base("skip");
        let target = skill_file_path(&dir);
        fs::create_dir_all(agents_skills_dir(&dir)).unwrap();
        fs::write(&target, "manual content").unwrap();

        let result = install_to(&dir).unwrap();

        assert!(result.starts_with("Skipped:"));
        assert_eq!(fs::read_to_string(&target).unwrap(), "manual content");

        fs::remove_dir_all(&dir).unwrap();
    }
}
