use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Default template categories to create when initializing templates directory
const DEFAULT_CATEGORIES: &[&str] = &["pwn", "crypto", "web", "rev"];

/// Ensures the templates directory exists with default category subdirectories.
/// Creates ~/ctf/templates/ with pwn/, crypto/, web/, rev/ subdirectories if they don't exist.
/// These are example categories - users can add custom categories as needed.
///
/// # Arguments
/// * `base_path` - The base CTF directory path (e.g., ~/ctf/)
///
/// # Returns
/// * `Ok(())` if templates directory was created or already exists
/// * `Err` if directory creation failed
pub fn ensure_templates_directory(base_path: &Path) -> Result<()> {
    let templates_path = base_path.join("templates");

    // Create templates directory if it doesn't exist
    if !templates_path.exists() {
        fs::create_dir_all(&templates_path)
            .context("Failed to create templates directory")?;

        // Create default category subdirectories
        for category in DEFAULT_CATEGORIES {
            let category_path = templates_path.join(category);
            fs::create_dir_all(&category_path)
                .with_context(|| format!("Failed to create template category: {}", category))?;
        }
    }

    Ok(())
}

/// Gets the path to a template directory for a specific category.
///
/// # Arguments
/// * `base_path` - The base CTF directory path (e.g., ~/ctf/)
/// * `category` - The challenge category (e.g., "pwn", "web", "crypto", etc.)
///
/// # Returns
/// * Path to the template directory: `{base_path}/templates/{category}/`
///
/// Note: This function does not validate if the path exists - it only constructs the path.
pub fn get_template_path(base_path: &Path, category: &str) -> PathBuf {
    base_path.join("templates").join(category)
}

/// Copies template files from the template directory to the challenge directory.
/// If the template directory doesn't exist or is empty, does nothing (empty challenge dir is created).
///
/// # Arguments
/// * `template_path` - Path to the template category directory
/// * `destination` - Path to the challenge directory where files should be copied
///
/// # Returns
/// * `Ok(())` if template was copied successfully or if no template exists
/// * `Err` if copying failed
pub fn copy_template_if_exists(template_path: &Path, destination: &Path) -> Result<()> {
    // Check if template directory exists and has contents
    if !template_path.exists() {
        return Ok(()); // No template, just create empty directory
    }

    // Check if directory has any files
    let entries = fs::read_dir(template_path)
        .context("Failed to read template directory")?;

    let has_contents = entries.count() > 0;

    if !has_contents {
        return Ok(()); // Empty template directory, nothing to copy
    }

    // Copy all contents from template to destination
    let copy_options = fs_extra::dir::CopyOptions {
        overwrite: false,
        skip_exist: true,
        buffer_size: 64000,
        copy_inside: true,
        content_only: true,
        depth: 0,
    };

    fs_extra::dir::copy(template_path, destination, &copy_options)
        .context("Failed to copy template files")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_ensure_templates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Ensure templates directory is created
        ensure_templates_directory(base_path).unwrap();

        // Check that templates directory exists
        let templates_path = base_path.join("templates");
        assert!(templates_path.exists());

        // Check that default category directories exist
        for category in DEFAULT_CATEGORIES {
            let category_path = templates_path.join(category);
            assert!(category_path.exists());
        }
    }

    #[test]
    fn test_get_template_path() {
        let base_path = Path::new("/home/user/ctf");
        let category = "pwn";

        let template_path = get_template_path(base_path, category);

        assert_eq!(template_path, Path::new("/home/user/ctf/templates/pwn"));
    }

    #[test]
    fn test_copy_template_if_exists_no_template() {
        let temp_dir = TempDir::new().unwrap();
        let template_path = temp_dir.path().join("nonexistent");
        let destination = temp_dir.path().join("dest");

        fs::create_dir_all(&destination).unwrap();

        // Should succeed even if template doesn't exist
        let result = copy_template_if_exists(&template_path, &destination);
        assert!(result.is_ok());
    }

    #[test]
    fn test_copy_template_if_exists_with_files() {
        let temp_dir = TempDir::new().unwrap();
        let template_path = temp_dir.path().join("template");
        let destination = temp_dir.path().join("dest");

        // Create template with a file
        fs::create_dir_all(&template_path).unwrap();
        fs::write(template_path.join("test.txt"), "test content").unwrap();

        // Create destination
        fs::create_dir_all(&destination).unwrap();

        // Copy template
        copy_template_if_exists(&template_path, &destination).unwrap();

        // Check that file was copied
        let copied_file = destination.join("test.txt");
        assert!(copied_file.exists());
        let content = fs::read_to_string(copied_file).unwrap();
        assert_eq!(content, "test content");
    }
}
