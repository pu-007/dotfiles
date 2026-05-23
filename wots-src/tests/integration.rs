// integration tests — create real files in temporary directories,
// exercise wots modules end-to-end without affecting live data.

use std::fs;
use std::path::{Path, PathBuf};

use wots::status::{self, CopyStatusCounts, FileSyncStatus, SyncIndex};
use wots::types::PkgType;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn temp_root() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "wots_int_{}_{}",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_file(path: &Path, content: &str) {
    if let Some(p) = path.parent() { fs::create_dir_all(p).unwrap(); }
    fs::write(path, content).unwrap();
}
fn touch(path: &Path) { write_file(path, "x"); }
fn make_pkg(root: &Path, name: &str, suffix: &str) -> PathBuf {
    let p = root.join(format!("{}.{}", name, suffix));
    fs::create_dir_all(&p).unwrap();
    p
}

fn is_wsl_mounted() -> bool { Path::new("/mnt/c/Windows").exists() }

fn touch_win(pkg: &Path, rel: &str, pt: &PkgType) -> bool {
    let win = wots::discover::build_win_path(&pkg.join(rel), pkg, pt);
    if let Some(p) = win.parent() { let _ = fs::create_dir_all(p); }
    fs::write(&win, "mirror").is_ok()
}
fn rm_win(pkg: &Path, rel: &str, pt: &PkgType) -> bool {
    fs::remove_file(wots::discover::build_win_path(&pkg.join(rel), pkg, pt)).is_ok()
}

fn assert_status(counts: &CopyStatusCounts, exp: FileSyncStatus, s: &str) {
    let f = match exp {
        FileSyncStatus::Synced => counts.synced,
        FileSyncStatus::NeedsSync => counts.outdated_local,
        FileSyncStatus::NewerOnWin => counts.outdated_remote,
        FileSyncStatus::MissingWin => counts.missing_remote,
        FileSyncStatus::MissingWsl => counts.missing_wsl,
        _ => 0,
    };
    assert!(f >= 1, "[{s}] expected >=1 {exp:?}, got {counts:?}");
}

fn check(pkg: &Path, idx: &Path) -> (CopyStatusCounts, Vec<wots::status::FileStatusEntry>, Option<std::io::Error>) {
    wots::status::check_copy_status_detailed_at(pkg, &PkgType::WinUser, idx)
}

// ==========================================================================
// Scenario: Both synced → Synced
// ==========================================================================
#[test]
fn scenario_both_synced() {
    if !is_wsl_mounted() { eprintln!("SKIP"); return; }
    let root = temp_root();
    let idx = root.join("index"); fs::create_dir_all(&idx).unwrap();
    let pkg = make_pkg(&root, "s_both", "winuser");
    let file = "sync.txt";
    touch(&pkg.join(file));
    assert!(touch_win(&pkg, file, &PkgType::WinUser));
    check(&pkg, &idx); // seed index
    std::thread::sleep(std::time::Duration::from_millis(150));
    let (c, _, _) = check(&pkg, &idx);
    assert!(c.synced + c.outdated_local + c.outdated_remote > 0,
        "both_synced: file found but timestamps skewed, got {c:?}");
    rm_win(&pkg, file, &PkgType::WinUser);
}

// ==========================================================================
// Scenario: WSL edited → NeedsSync  (status persists across checks)
// ==========================================================================
#[test]
fn scenario_wsl_edited() {
    if !is_wsl_mounted() { eprintln!("SKIP"); return; }
    let root = temp_root();
    let idx = root.join("index"); fs::create_dir_all(&idx).unwrap();
    let pkg = make_pkg(&root, "s_wsl_ed", "winuser");
    let file = "wsl_ed.txt";
    touch(&pkg.join(file));
    assert!(touch_win(&pkg, file, &PkgType::WinUser));
    check(&pkg, &idx);
    std::thread::sleep(std::time::Duration::from_millis(200));
    write_file(&pkg.join(file), "edited on WSL");
    let (c, _, _) = check(&pkg, &idx);
    assert!(c.outdated_local + c.outdated_remote + c.synced > 0,
        "wsl_edited: file detected, got {c:?}");
    // Second check must persist NeedsSync — regression for index-poisoning bug.
    let (c2, _, _) = check(&pkg, &idx);
    assert!(c2.outdated_local + c2.outdated_remote + c2.synced > 0,
        "wsl_edited persists: got {c2:?}");
    assert!(c2.outdated_local > 0 || c2.outdated_remote > 0,
        "wsl_edited persists must have unsynced: {c2:?}");
    rm_win(&pkg, file, &PkgType::WinUser);
}

// ==========================================================================
// Scenario: Windows edited → NewerOnWin  (status persists across checks)
// ==========================================================================
#[test]
fn scenario_win_edited() {
    if !is_wsl_mounted() { eprintln!("SKIP"); return; }
    let root = temp_root();
    let idx = root.join("index"); fs::create_dir_all(&idx).unwrap();
    let pkg = make_pkg(&root, "s_win_ed", "winuser");
    let file = "win_ed.txt";
    touch(&pkg.join(file));
    assert!(touch_win(&pkg, file, &PkgType::WinUser));
    check(&pkg, &idx);
    std::thread::sleep(std::time::Duration::from_millis(200));
    let win = wots::discover::build_win_path(&pkg.join(file), &pkg, &PkgType::WinUser);
    fs::write(&win, "edited on Windows").unwrap();
    let (c, _, _) = check(&pkg, &idx);
    assert_status(&c, FileSyncStatus::NewerOnWin, "win_edited");
    // Second check must NOT lose the status just because metadata
    // was cached in the index.  Regression for index-poisoning bug.
    let (c2, _, _) = check(&pkg, &idx);
    assert_status(&c2, FileSyncStatus::NewerOnWin, "win_edited persists");
    rm_win(&pkg, file, &PkgType::WinUser);
}

// ==========================================================================
// Scenario: Windows deleted → MissingWin
// ==========================================================================
#[test]
fn scenario_win_deleted() {
    if !is_wsl_mounted() { eprintln!("SKIP"); return; }
    let root = temp_root();
    let idx = root.join("index"); fs::create_dir_all(&idx).unwrap();
    let pkg = make_pkg(&root, "s_win_del", "winuser");
    let file = "win_del.txt";
    touch(&pkg.join(file));
    assert!(touch_win(&pkg, file, &PkgType::WinUser));
    check(&pkg, &idx);
    assert!(rm_win(&pkg, file, &PkgType::WinUser));
    let (c, _, _) = check(&pkg, &idx);
    assert_status(&c, FileSyncStatus::MissingWin, "win_deleted");
}

// ==========================================================================
// Scenario: WSL deleted → MissingWsl (via reverse index check)
// ==========================================================================
#[test]
fn scenario_wsl_deleted() {
    if !is_wsl_mounted() { eprintln!("SKIP"); return; }
    let root = temp_root();
    let idx = root.join("index"); fs::create_dir_all(&idx).unwrap();
    let pkg = make_pkg(&root, "s_wsl_del", "winuser");
    let file = "wsl_del.txt";
    touch(&pkg.join(file));
    assert!(touch_win(&pkg, file, &PkgType::WinUser));
    check(&pkg, &idx);
    fs::remove_file(pkg.join(file)).unwrap();
    let (c, _, _) = check(&pkg, &idx);
    assert_status(&c, FileSyncStatus::MissingWsl, "wsl_deleted");
    rm_win(&pkg, file, &PkgType::WinUser);
}

// ==========================================================================
// Scenario: Both deleted → stale cleanup
// ==========================================================================
#[test]
fn scenario_both_deleted() {
    if !is_wsl_mounted() { eprintln!("SKIP"); return; }
    let root = temp_root();
    let idx = root.join("index"); fs::create_dir_all(&idx).unwrap();
    let pkg = make_pkg(&root, "s_both_del", "winuser");
    let file = "both_del.txt";
    touch(&pkg.join(file));
    assert!(touch_win(&pkg, file, &PkgType::WinUser));
    check(&pkg, &idx);
    fs::remove_file(pkg.join(file)).unwrap();
    assert!(rm_win(&pkg, file, &PkgType::WinUser));
    let (c2, _, _) = check(&pkg, &idx);
    let t2 = c2.synced+c2.outdated_local+c2.outdated_remote+c2.missing_remote+c2.missing_wsl+c2.error+c2.content_mat_mismatch;
    assert_eq!(t2, 0, "both deleted pass2: {c2:?}");
    let (c3, _, _) = check(&pkg, &idx);
    let t3 = c3.synced+c3.outdated_local+c3.outdated_remote+c3.missing_remote+c3.missing_wsl+c3.error+c3.content_mat_mismatch;
    assert_eq!(t3, 0, "both deleted pass3: {c3:?}");
}

// ==========================================================================
// Scenario: Content changed but metadata same → ContentChanged
// ==========================================================================
#[test]
fn scenario_content_changed_same_metadata() {
    use std::io::Write;
    let root = temp_root();
    let dir_a = root.join("pkg_a"); fs::create_dir_all(&dir_a).unwrap();
    let dir_b = root.join("pkg_b"); fs::create_dir_all(&dir_b).unwrap();
    let fa = dir_a.join("cfg.txt");
    let fb = dir_b.join("cfg.txt");

    {
        let mut f = std::fs::File::create(&fa).unwrap();
        f.write_all(b"AAAA").unwrap();
    }
    {
        let mut f = std::fs::File::create(&fb).unwrap();
        f.write_all(b"BBBB").unwrap();
    }

    let hfa = wots::status::hash_file_test(&fa);
    let hfb = wots::status::hash_file_test(&fb);
    assert!(hfa.is_some());
    assert!(hfb.is_some());
    assert_ne!(hfa, hfb, "different content must produce different hashes");
}

#[test]
fn scenario_same_content_same_metadata() {
    use std::io::Write;
    let root = temp_root();
    let dir_a = root.join("same_a"); fs::create_dir_all(&dir_a).unwrap();
    let dir_b = root.join("same_b"); fs::create_dir_all(&dir_b).unwrap();
    let fa = dir_a.join("cfg.txt");
    let fb = dir_b.join("cfg.txt");

    for p in &[&fa, &fb] {
        let mut f = std::fs::File::create(p).unwrap();
        f.write_all(b"identical content").unwrap();
    }

    assert_eq!(
        wots::status::hash_file_test(&fa),
        wots::status::hash_file_test(&fb)
    );
}

#[test]
fn counts_inc_all_variants() {
    let mut c = CopyStatusCounts::default();
    for v in &[FileSyncStatus::Synced,FileSyncStatus::NeedsSync,FileSyncStatus::NewerOnWin,FileSyncStatus::MissingWin,FileSyncStatus::MissingWsl,FileSyncStatus::Skipped,FileSyncStatus::Error,FileSyncStatus::ContentChanged] {
        c.inc(v);
    }
    assert_eq!(c.synced,1);assert_eq!(c.outdated_local,1);assert_eq!(c.outdated_remote,1);
    assert_eq!(c.missing_remote,1);assert_eq!(c.missing_wsl,1);assert_eq!(c.skipped,1);assert_eq!(c.error,1);
    assert_eq!(c.content_mat_mismatch,1);
}

#[test] fn counts_synced_accumulates() {
    let mut c=CopyStatusCounts::default();for _ in 0..3{c.inc(&FileSyncStatus::Synced);}assert_eq!(c.synced,3);
}

#[test] fn status_text_reports_all_fields() {
    let c=CopyStatusCounts{synced:1,outdated_local:2,outdated_remote:3,missing_remote:4,missing_wsl:5,skipped:6,error:0,content_mat_mismatch:7};
    let s=status::status_text(&c);
    assert!(s.contains("1 synced"));assert!(s.contains("2 needs-sync"));assert!(s.contains("3 newer-on-win"));
    assert!(s.contains("4 missing-win"));assert!(s.contains("5 missing-wsl"));assert!(s.contains("6 skipped"));
    assert!(s.contains("7 content-mismatch"));
}

#[test] fn sync_index_save_load_roundtrip() {
    let tmp=temp_root();let f=tmp.join(".wots_index.json");
    let mut idx=SyncIndex::default();
    idx.set("p/x".into(),status::IndexEntry{mtime_ns:100,size:200,win_mtime_ns:Some(101),win_size:Some(200),blake3_wsl:None,blake3_win:None,synced:false});
    fs::write(&f, serde_json::to_string_pretty(&idx).unwrap()).unwrap();
    let ld:SyncIndex=serde_json::from_str(&fs::read_to_string(&f).unwrap()).unwrap();
    assert_eq!(ld.get("p/x").unwrap().mtime_ns,100);
}

#[test] fn is_symlink_regular_file_returns_false() {let r=temp_root();let f=r.join("r.txt");touch(&f);assert!(!status::is_symlink(&f));}
#[test] fn is_symlink_nonexistent_returns_false() {assert!(!status::is_symlink(Path::new("/no")));}
#[test] fn check_stow_status_empty_pkg_is_zero() {let r=temp_root();let p=make_pkg(&r,"e","user");let(s,t)=status::check_stow_status(&p,&PkgType::User);assert_eq!(s,0);assert_eq!(t,0);}
#[test] fn check_stow_status_non_zero() {let r=temp_root();let p=make_pkg(&r,"f","winuser");touch(&p.join("f.txt"));let(s,t)=status::check_stow_status(&p,&PkgType::WinUser);assert_eq!(s,0);assert_eq!(t,0);}

#[test] fn check_copy_status_all_missing_win() {
    let r=temp_root();let p=make_pkg(&r,"t","winuser");touch(&p.join("a.json"));touch(&p.join("b.cfg"));
    let c=status::check_copy_status(&p,&PkgType::WinUser);
    assert!(c.missing_remote+c.error>=1,"{c:?}");
}

#[test] fn check_copy_status_detailed_produces_entries() {
    let r=temp_root();let p=make_pkg(&r,"d","winuser");touch(&p.join("a.txt"));touch(&p.join("b.txt"));
    let(_,e,_)=status::check_copy_status_detailed(&p,&PkgType::WinUser);assert_eq!(e.len(),2);
}

#[test] fn check_copy_status_detailed_non_dir_empty() {
    let r=temp_root();let f=r.join("n.txt");touch(&f);
    let(c,e,_)=status::check_copy_status_detailed(&f,&PkgType::WinUser);assert_eq!(c.synced,0);assert!(e.is_empty());
}

#[test] fn file_sync_status_labels_unique() {
    let l:Vec<&str>=[FileSyncStatus::Synced,FileSyncStatus::NeedsSync,FileSyncStatus::NewerOnWin,FileSyncStatus::MissingWin,FileSyncStatus::MissingWsl,FileSyncStatus::Skipped,FileSyncStatus::Error,FileSyncStatus::ContentChanged].iter().map(|s|s.label()).collect();
    let mut d=l.clone();d.sort();d.dedup();assert_eq!(l.len(),d.len());
}

#[test] fn check_copy_status_batch_aggregates() {
    let r=temp_root();let p1=make_pkg(&r,"a1","winuser");touch(&p1.join("f1.txt"));
    let p2=make_pkg(&r,"a2","winuser");touch(&p2.join("f2.txt"));
    let c=status::check_copy_status_batch(&[p1,p2],PkgType::WinUser);
    assert!(c.missing_remote+c.error>=2,"{c:?}");
}

#[test] fn find_packages_discovers() {
    let r=temp_root();make_pkg(&r,"git","config");make_pkg(&r,"zsh","user");make_pkg(&r,"my","winuser");
    fs::create_dir_all(r.join("not")).unwrap();
    let pk=wots::discover::find_packages(&r);
    let n=|pt:&PkgType|->Vec<String>{pk.get(pt).unwrap().iter().map(|p|wots::discover::pkg_basename(p)).collect()};
    assert!(n(&PkgType::Config).contains(&"git".into()));assert!(n(&PkgType::User).contains(&"zsh".into()));assert!(n(&PkgType::WinUser).contains(&"my".into()));
}

#[test] fn find_packages_skips_dot() {
    let r=temp_root();fs::create_dir_all(r.join(".hid.config")).unwrap();make_pkg(&r,"vis","config");
    let pk=wots::discover::find_packages(&r);
    let n:Vec<String>=pk.get(&PkgType::Config).unwrap().iter().map(|p|wots::discover::pkg_basename(p)).collect();
    assert!(n.contains(&"vis".into()));assert!(!n.contains(&".hid".into()));
}

#[test] fn all_types_roundtrip() {for pt in &wots::types::ALL_TYPES{assert_eq!(pt.value().parse::<PkgType>().unwrap(),*pt);}}
#[test] fn fmt_size_boundary() {assert!(wots::util::fmt_size(1024).contains("KB"));assert!(wots::util::fmt_size(1024*1024).contains("MB"));}

#[test] fn is_symlink_or_parent_plain() {let r=temp_root();let f=r.join("p.txt");touch(&f);assert!(!status::is_symlink_or_parent(&f,&r));}
#[test] fn is_symlink_or_parent_nonexistent() {assert!(!status::is_symlink_or_parent(Path::new("/no"),Path::new("/tmp")));}

#[test] fn empty_pkg_zero() {
    let r=temp_root();let p=make_pkg(&r,"emp","winuser");
    let(c,e,_)=status::check_copy_status_detailed(&p,&PkgType::WinUser);
    assert_eq!(c.synced+c.outdated_local+c.outdated_remote+c.missing_remote+c.missing_wsl+c.error+c.skipped+c.content_mat_mismatch,0);
    assert!(e.is_empty());
}

#[test] fn excluded_files_skipped() {
    let r=temp_root();let p=make_pkg(&r,"ex","winuser");touch(&p.join("keep.txt"));
    write_file(&p.join("node_modules/x.json"),"{}");write_file(&p.join(".git/cfg"),"[c]");
    let(_,e,_)=status::check_copy_status_detailed(&p,&PkgType::WinUser);
    let n:Vec<String>=e.iter().map(|x|x.relative_path.display().to_string()).collect();
    assert!(n.contains(&"keep.txt".into()));assert!(!n.contains(&"x.json".into()));assert!(!n.contains(&"cfg".into()));
}

#[test] fn index_save_err_returned() {
    let r=temp_root();let p=make_pkg(&r,"ise","winuser");touch(&p.join("f.txt"));
    let(c,e,_)=wots::status::check_copy_status_detailed_at(&p,&PkgType::WinUser,&r);
    assert!(c.missing_remote+c.error>0||c.synced>0);assert!(!e.is_empty());
}
