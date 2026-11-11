mod common;
use common::at;
use devlapse::naming::{day_folder_name, file_name};
use devlapse::storage::Storage;

#[test]
fn day_folder_is_iso_date() {
    assert_eq!(day_folder_name(at(9, 5, 7)), "2026-11-11");
}

#[test]
fn file_name_has_full_timestamp_and_one_based_screen() {
    assert_eq!(file_name(at(9, 5, 7), 0, "jpg"), "2026-11-11_09-05-07_screen1.jpg");
    assert_eq!(file_name(at(23, 59, 59), 1, "png"), "2026-11-11_23-59-59_screen2.png");
}

#[test]
fn file_names_sort_chronologically_for_ffmpeg_globbing() {
    let mut names = vec![file_name(at(10, 0, 0), 0, "jpg"), file_name(at(9, 59, 59), 0, "jpg"),
                         file_name(at(9, 0, 1), 0, "jpg")];
    let expected = vec![names[2].clone(), names[1].clone(), names[0].clone()];
    names.sort();
    assert_eq!(names, expected);
}

#[test]
fn save_creates_day_folder_and_writes_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("not/yet/existing");
    let storage = Storage::new(root.clone());
    let path = storage.save(at(14, 30, 0), 0, "jpg", b"pixels").unwrap();
    assert_eq!(path, root.join("2026-11-11").join("2026-11-11_14-30-00_screen1.jpg"));
    assert_eq!(std::fs::read(&path).unwrap(), b"pixels");
}

#[test]
fn save_never_overwrites_an_existing_capture() {
    let dir = tempfile::tempdir().unwrap();
    let storage = Storage::new(dir.path().to_path_buf());
    let a = storage.save(at(14, 30, 0), 0, "jpg", b"first").unwrap();
    let b = storage.save(at(14, 30, 0), 0, "jpg", b"second").unwrap();
    assert_ne!(a, b);
    assert_eq!(std::fs::read(&a).unwrap(), b"first");
    assert_eq!(std::fs::read(&b).unwrap(), b"second");
    assert!(b.file_name().unwrap().to_str().unwrap().ends_with("_screen1-1.jpg"));
}

#[test]
fn save_leaves_no_temp_files_behind() {
    let dir = tempfile::tempdir().unwrap();
    let storage = Storage::new(dir.path().to_path_buf());
    storage.save(at(8, 0, 0), 0, "png", b"x").unwrap();
    let leftovers: Vec<_> = std::fs::read_dir(dir.path().join("2026-11-11")).unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .filter(|n| !n.ends_with(".png"))
        .collect();
    assert!(leftovers.is_empty(), "unexpected files: {leftovers:?}");
}

#[test]
fn save_reports_error_when_root_is_unwritable() {
    let dir = tempfile::tempdir().unwrap();
    let blocker = dir.path().join("a_file");
    std::fs::write(&blocker, b"").unwrap();
    let storage = Storage::new(blocker.join("sub")); // parent is a file → cannot create dir
    assert!(storage.save(at(8, 0, 0), 0, "png", b"x").is_err());
}
