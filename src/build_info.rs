// Lots of other useful stuff here: https://github.com/lukaslueg/built/blob/master/example_project/src/main.rs
// For now, keep it simple

mod built {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub const COMMIT_HASH_STR: &str = match built::GIT_COMMIT_HASH_SHORT {
    Some(hash) => hash,
    None => "UNKNOWN",
};

pub const BUILD_SRC_STR: &str = {
    const GIT_HEAD_REF: &str = match built::GIT_HEAD_REF {
        Some(r) => r,
        None => "UNKNOWN",
    };

    const GIT_DIRTY: &str = match built::GIT_DIRTY {
        Some(r) => {
            if r {
                "dirty"
            } else {
                "clean"
            }
        }
        None => "UNKNOWN",
    };

    const_format::concatcp!(GIT_HEAD_REF, ":(", GIT_DIRTY, ")")
};

pub const BUILD_ENV_STR: &str = {
    const BUILD_LOCATION: &str = match built::CI_PLATFORM {
        None => "local",
        Some(ci) => ci,
    };

    const_format::concatcp!(BUILD_LOCATION, "@", built::BUILT_TIME_UTC)
};

/// Valid SemVer version constructed using declared Cargo version and short commit hash.
pub const FULL_VERSION: &str = const_format::concatcp!(
    built::PKG_VERSION,
    "-",
    COMMIT_HASH_STR,
    "[",
    built::PROFILE,
    "]",
    "-",
    built::RUSTC_VERSION
);

pub const VERBOSE_VERSION: &str =
    const_format::concatcp!(FULL_VERSION, "\n", BUILD_SRC_STR, "\n", BUILD_ENV_STR, "\n");
