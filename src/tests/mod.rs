use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use super::actor::Actor;
use super::{Result, expand, collapse};

lazy_static! {
    static ref CURRENT_DIR: PathBuf = {
        let path = fs::canonicalize(Path::new(file!()).parent().unwrap()).unwrap();
        println!("setting current dir to {}", path.display());
        env::set_current_dir(&path).unwrap();
        path
    };
}

macro_rules! test {
    ($name:ident $body:block) => {
        #[test]
        fn $name() {
            let _ = &*CURRENT_DIR;
            clear();
            $body
            clear();
        }
    }
}

fn clear() {
    for entry in fs::read_dir(".").unwrap().map(|e| e.unwrap()) {
        let path = entry.path();
        let path_str = path.to_str().unwrap();
        if path_str.contains('_') && path_str.ends_with(".m") {
            fs::remove_file(&path).unwrap();
        }
    }
}

fn with_backup<P1: AsRef<Path>, P2: AsRef<Path>, F: FnOnce()>(orig: P1, bak: P2, f: F) {
    println!("backing up {} to {}", orig.as_ref().display(), bak.as_ref().display());
    fs::copy(orig.as_ref(), bak.as_ref()).unwrap();
    f();
    println!("restoring {} to {}", bak.as_ref().display(), orig.as_ref().display());
    fs::rename(bak.as_ref(), orig.as_ref()).unwrap();
}

fn compare<P1: AsRef<Path>, P2: AsRef<Path>>(a: P1, b: P2, replace_dir: bool) {
    println!("comparing {:?} to {:?}", a.as_ref(), b.as_ref());
    let mut v1 = String::new();
    let mut v2 = String::new();
    File::open(a).unwrap().read_to_string(&mut v1).unwrap();
    File::open(b).unwrap().read_to_string(&mut v2).unwrap();
    if replace_dir {
        v1 = v1.replace(CURRENT_DIR.to_str().unwrap(), "{{dir}}");
    }
    assert_eq!(v1, v2);
}

fn try<T>(res: Result<T>) {
    match res {
        Ok(_) => {},
        Err(e) => panic!("{:?}", e),
    }
}

test!(expanding {
    try(expand(Actor::new(), "script.m", false));
    for part in &["_0", "a_1", "b_2", "if_true_3", "gen"] {
        compare(format!("script_{}.m", part),
                format!("script{}.m", part.replace('_', "")),
                true);
    }
});

test!(collapsing {
    with_backup("script.m", "script_backup.m", || {
        try(expand(Actor::new(), "script.m", false));
        try(collapse(Actor::new(), "script.m"));
        compare("script.m", "script_backup.m", false);
    });
});

