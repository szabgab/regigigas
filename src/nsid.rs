use std::{
  fmt::{Debug, Display},
  str::FromStr,
  sync::RwLock,
};

use lasso::{Rodeo, Spur};
use once_cell::sync::Lazy;

use crate::{InvalidNamespace, InvalidPath, NSIDParseError};

/// Light-weight friendly-printable handle to an entry in a registry.
///
/// whats a minecraft
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct NamespacedID {
  namespace: Spur,
  path: Spur,
}

static NSID_INTERNER: Lazy<RwLock<Rodeo>> =
  Lazy::new(|| RwLock::new(Rodeo::new()));

impl NamespacedID {
  pub fn is_valid_namespace_char(chr: char) -> bool {
    ('a'..='z').contains(&chr)
      || ('0'..='9').contains(&chr)
      || "-_".contains(chr)
  }

  pub fn is_valid_path_char(chr: char) -> bool {
    ('a'..='z').contains(&chr)
      || ('0'..='9').contains(&chr)
      || "./-_".contains(chr)
  }

  pub fn check_namespace<S: AsRef<str>>(s: S) -> Result<(), InvalidNamespace> {
    let s = s.as_ref();
    if s.is_empty() {
      Err(InvalidNamespace::Empty)?;
    }
    for (idx, c) in s.char_indices() {
      if !NamespacedID::is_valid_namespace_char(c) {
        Err(InvalidNamespace::BadChar(idx, c))?;
      }
    }
    Ok(())
  }

  pub fn check_path<S: AsRef<str>>(s: S) -> Result<(), InvalidPath> {
    let s = s.as_ref();
    if s.is_empty() {
      Err(InvalidPath::Empty)?;
    }
    for (idx, c) in s.char_indices() {
      if !NamespacedID::is_valid_path_char(c) {
        Err(InvalidPath::BadChar(idx, c))?;
      }
    }
    Ok(())
  }

  /// Make a new NSID.
  ///
  /// This is just a wrapper around FromStr.
  pub fn new<S: AsRef<str>>(nsid: S) -> Result<Self, NSIDParseError> {
    nsid.as_ref().parse()
  }

  pub fn new_from_parts<S1, S2>(
    namespace: S1,
    path: S2,
  ) -> Result<Self, NSIDParseError>
  where
    S1: AsRef<str>,
    S2: AsRef<str>,
  {
    let namespace = namespace.as_ref();
    let path = path.as_ref();

    NamespacedID::check_namespace(namespace)?;
    NamespacedID::check_path(namespace)?;

    let mut interner = NSID_INTERNER.try_write()?;
    let ns = interner.get_or_intern(namespace);
    let p = interner.get_or_intern(path);
    Ok(Self {
      namespace: ns,
      path: p,
    })
  }

  /// Get this NSID's namespace
  pub fn namespace(&self) -> String {
    let interner = NSID_INTERNER.try_read().unwrap();
    interner.resolve(&self.namespace).to_owned()
  }

  /// Get this NSID's path
  pub fn path(&self) -> String {
    let interner = NSID_INTERNER.try_read().unwrap();
    interner.resolve(&self.path).to_owned()
  }

  /// Decompose this into a namespace and path
  pub fn dissolve(&self) -> (String, String) {
    (self.namespace(), self.path())
  }
}

impl Display for NamespacedID {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let interner = NSID_INTERNER.try_read().unwrap();
    let n = interner.resolve(&self.namespace);
    let p = interner.resolve(&self.path);
    write!(f, "{}:{}", n, p)
  }
}

impl Debug for NamespacedID {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    <NamespacedID as Display>::fmt(&self, f)
  }
}

impl FromStr for NamespacedID {
  type Err = NSIDParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let (namespace, maybe_path) = match s.split_once(':') {
      None => return Err(NSIDParseError::NoSeparator),
      Some((ns, path)) => {
        NamespacedID::check_namespace(ns)?;
        (String::from(ns), path)
      }
    };
    let ns_char_len = namespace.chars().count();
    NamespacedID::check_path(maybe_path).map_err(|e| {
      if let InvalidPath::BadChar(idx, c) = e {
        // add the namespace, the colon, and the char
        InvalidPath::BadChar(ns_char_len + 1 + idx, c)
      } else {
        e
      }
    })?;

    let mut interner = NSID_INTERNER.try_write()?;
    let ns = interner.get_or_intern(namespace);
    let p = interner.get_or_intern(maybe_path);
    Ok(Self {
      namespace: ns,
      path: p,
    })
  }
}

/// Convenience function to create and unwrap an NSID
pub fn nsid(s: impl AsRef<str>) -> NamespacedID {
  NamespacedID::new(s).unwrap()
}
