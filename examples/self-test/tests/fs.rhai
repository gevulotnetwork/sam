import "assert" as assert;

describe("File System Functions", || {
    let temp_dir = "";

    it("should be possible to get a temp directory", || {
        temp_dir = temp_dir("test-fs");
        assert(file_exists(temp_dir), "Temp directory should exist");
    });

    it("should be able to write and read files", || {
        write_file(temp_dir + "/test.txt", "Hello World");
        let content = read_file(temp_dir + "/test.txt");
        assert::eq(content, "Hello World");
    });

    it("should be able to create and remove directories", || {
        mkdir(temp_dir + "/nested/dir");
        assert(is_dir(temp_dir + "/nested/dir"), "Directory should exist");
        remove(temp_dir + "/nested");
        assert(!file_exists(temp_dir + "/nested"), "Directory should be removed");
    });

    it("should be able to list directory contents", || {
        write_file(temp_dir + "/a.txt", "a");
        write_file(temp_dir + "/b.txt", "b");
        mkdir(temp_dir + "/dir");
        
        let contents = ls(temp_dir);
        assert(contents.contains("a.txt"), "Should contain a.txt");
        assert(contents.contains("b.txt"), "Should contain b.txt"); 
        assert(contents.contains("dir"), "Should contain dir");
    });

    it("should be able to check file existence", || {
        write_file(temp_dir + "/exists.txt", "test");
        assert(file_exists(temp_dir + "/exists.txt"), "File should exist");
        assert(!file_exists(temp_dir + "/nonexistent.txt"), "File should not exist");
    });

    it("should be able to get file stats", || {
        write_file(temp_dir + "/stat.txt", "test content");
        let stats = stat(temp_dir + "/stat.txt");
        assert(stats.is_file, "Should be a file");
        assert(!stats.is_dir, "Should not be a directory");
        assert(stats.size.to_int() == 12, "File size should be 12 bytes");
        assert(stats.modified.to_int() > 0, "Modified time should be set");
    });

    it("should be able to copy files and directories", || {
        write_file(temp_dir + "/src.txt", "source");
        copy(temp_dir + "/src.txt", temp_dir + "/dst.txt");
        assert(read_file(temp_dir + "/dst.txt") == "source", "File content should match");

        mkdir(temp_dir + "/src_dir");
        write_file(temp_dir + "/src_dir/file.txt", "test");
        copy(temp_dir + "/src_dir", temp_dir + "/dst_dir");
        assert(file_exists(temp_dir + "/dst_dir/file.txt"), "Directory should be copied recursively");
    });

    it("should be able to rename/move files and directories", || {
        write_file(temp_dir + "/old.txt", "content");
        rename(temp_dir + "/old.txt", temp_dir + "/new.txt");
        assert(!file_exists(temp_dir + "/old.txt"), "Old file should not exist");
        assert(file_exists(temp_dir + "/new.txt"), "New file should exist");
    });

    it("should be able to check if path is file or directory", || {
        write_file(temp_dir + "/file.txt", "test");
        mkdir(temp_dir + "/dir");

        assert(is_file(temp_dir + "/file.txt"), "Should be a file");
        assert(!is_file(temp_dir + "/dir"), "Should not be a file");
        assert(is_dir(temp_dir + "/dir"), "Should be a directory");
        assert(!is_dir(temp_dir + "/file.txt"), "Should not be a directory");
    });

    it("should be able to get absolute paths", || {
        let abs_path = absolute_path(temp_dir);
        assert(abs_path.starts_with("/"), "Should be an absolute path");
    });
});
