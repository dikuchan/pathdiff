// Copyright 2012-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Adapted from rustc's path_relative_from
// https://github.com/rust-lang/rust/blob/e1d0de82cc40b666b88d4a6d2c9dcbc81d7ed27f/src/librustc_back/rpath.rs#L116-L158

use std::{error, fmt, path::*};

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    NotAbsolute,
    ContainsRelative,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::NotAbsolute => "paths are not absolute",
                Self::ContainsRelative => "base path contains reference",
            }
        )
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

/// Construct a relative path from a provided base directory path to the provided path.
///
/// ```rust
/// use pathdiff::diff_paths;
/// use std::path::*;
///
/// let baz = "/foo/bar/baz";
/// let bar = "/foo/bar";
/// let quux = "/foo/bar/quux";
/// assert_eq!(diff_paths(bar, baz), Ok("../".into()));
/// assert_eq!(diff_paths(baz, bar), Ok("baz".into()));
/// assert_eq!(diff_paths(quux, baz), Ok("../quux".into()));
/// assert_eq!(diff_paths(baz, quux), Ok("../baz".into()));
/// assert_eq!(diff_paths(bar, quux), Ok("../".into()));
///
/// assert_eq!(diff_paths(&baz, &bar.to_string()), Ok("baz".into()));
/// assert_eq!(diff_paths(Path::new(baz), Path::new(bar).to_path_buf()), Ok("baz".into()));
/// ```
pub fn diff_paths<P, B>(path: P, base: B) -> Result<PathBuf, Error>
where
    P: AsRef<Path>,
    B: AsRef<Path>,
{
    let path = path.as_ref();
    let base = base.as_ref();

    if !path.is_absolute() || !base.is_absolute() {
        return Err(Error::NotAbsolute);
    } else {
        let mut ita = path.components();
        let mut itb = base.components();
        let mut comps: Vec<Component> = vec![];
        loop {
            match (ita.next(), itb.next()) {
                (None, None) => break,
                (Some(a), None) => {
                    comps.push(a);
                    comps.extend(ita.by_ref());
                    break;
                }
                (None, _) => comps.push(Component::ParentDir),
                (Some(a), Some(b)) if comps.is_empty() && a == b => (),
                (Some(a), Some(b)) if b == Component::CurDir => comps.push(a),
                (Some(_), Some(b)) if b == Component::ParentDir => {
                    return Err(Error::ContainsRelative)
                }
                (Some(a), Some(_)) => {
                    comps.push(Component::ParentDir);
                    for _ in itb {
                        comps.push(Component::ParentDir);
                    }
                    comps.push(a);
                    comps.extend(ita.by_ref());
                    break;
                }
            }
        }
        Ok(comps.iter().map(|c| c.as_os_str()).collect())
    }
}
