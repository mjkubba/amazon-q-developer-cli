#[cfg(test)]
#[cfg(windows)]
mod tests {
    use std::path::Path;
    use std::fs;
    use tokio::runtime::Runtime;
    use crate::Fs;

    #[test]
    fn test_symlink_file() {
        let rt = Runtime::new().unwrap();
        
        // Create a temporary directory for testing
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file_path = temp_dir.path().join("test_file.txt");
        let symlink_path = temp_dir.path().join("test_symlink.txt");
        
        // Create a test file
        fs::write(&test_file_path, "test content").unwrap();
        
        // Create a file symlink
        let fs_provider = Fs::new();
        rt.block_on(async {
            fs_provider.symlink(&test_file_path, &symlink_path).await.unwrap();
            
            // Verify that the symlink exists and points to the correct file
            assert!(symlink_path.exists());
            let content = fs::read_to_string(&symlink_path).unwrap();
            assert_eq!(content, "test content");
        });
    }

    #[test]
    fn test_symlink_dir() {
        let rt = Runtime::new().unwrap();
        
        // Create a temporary directory for testing
        let temp_dir = tempfile::tempdir().unwrap();
        let test_dir_path = temp_dir.path().join("test_dir");
        let symlink_path = temp_dir.path().join("test_symlink_dir");
        
        // Create a test directory with a file inside
        fs::create_dir(&test_dir_path).unwrap();
        let test_file_path = test_dir_path.join("test_file.txt");
        fs::write(&test_file_path, "test content").unwrap();
        
        // Create a directory symlink
        let fs_provider = Fs::new();
        rt.block_on(async {
            fs_provider.symlink(&test_dir_path, &symlink_path).await.unwrap();
            
            // Verify that the symlink exists and points to the correct directory
            assert!(symlink_path.exists());
            assert!(symlink_path.is_dir());
            
            // Verify that we can access files through the symlink
            let symlinked_file = symlink_path.join("test_file.txt");
            assert!(symlinked_file.exists());
            let content = fs::read_to_string(&symlinked_file).unwrap();
            assert_eq!(content, "test content");
        });
    }

    #[test]
    fn test_symlink_sync() {
        // Create a temporary directory for testing
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file_path = temp_dir.path().join("test_file_sync.txt");
        let symlink_path = temp_dir.path().join("test_symlink_sync.txt");
        
        // Create a test file
        fs::write(&test_file_path, "test content sync").unwrap();
        
        // Create a file symlink using the sync method
        let fs_provider = Fs::new();
        fs_provider.symlink_sync(&test_file_path, &symlink_path).unwrap();
        
        // Verify that the symlink exists and points to the correct file
        assert!(symlink_path.exists());
        let content = fs::read_to_string(&symlink_path).unwrap();
        assert_eq!(content, "test content sync");
    }
}
