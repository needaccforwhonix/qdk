// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! What discovery found of each library: where it was loaded from, the
//! versions it reports, and every fault found in it.
//!
//! A caller reads the report to tell the user what is installed, or what to
//! fix. An [`Availability`](crate::Availability) carries it when every
//! library is usable, and
//! [`AvailabilityError::Unusable`](crate::AvailabilityError::Unusable) when
//! one is not. It is a list with one entry per library, so binding another
//! library adds an entry and changes no type.
//!
//! Each library at fault lists every fault discovery could find in it, so
//! that the user fixes their installation in one pass: the variable, path,
//! function or version at fault. Only a fault that leaves nothing more to
//! check about its library (an override that is not a file, a library not
//! found or refused by the system loader) stops the checks of that library.

use std::path::PathBuf;
use thiserror::Error;

/// What [`discover`](crate::discover) found: one [`LibraryReport`] per
/// library, in the order they are opened (today the CUDA Runtime, then
/// cuTensorNet).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AvailabilityReport {
    /// One report per library.
    pub libraries: Vec<LibraryReport>,
}

impl AvailabilityReport {
    /// The report of the library named `name` (for instance
    /// `"cuTensorNet"`), if discovery checks it.
    #[must_use]
    pub fn library(&self, name: &str) -> Option<&LibraryReport> {
        self.libraries
            .iter()
            .find(|library| library.library == name)
    }
}

/// What [`discover`](crate::discover) found of one library.
///
/// A library is usable when it has no fault. A library with a fault is
/// reported with what was found of it all the same: its path once it was
/// opened, and the versions read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LibraryReport {
    /// The library's name, for instance `cuTensorNet`.
    pub library: &'static str,
    /// The path it was loaded from; `None` when it was not loaded.
    pub path: Option<PathBuf>,
    /// The versions it reports, in the order they are read.
    pub versions: Vec<Version>,
    /// Every fault found, in the order found; empty when it is usable.
    pub faults: Vec<LibraryFault>,
}

impl LibraryReport {
    /// The version of `component`, if it was read.
    #[must_use]
    pub fn version(&self, component: &str) -> Option<i128> {
        self.versions
            .iter()
            .find(|version| version.component == component)
            .map(|version| version.value)
    }
}

/// One version a library reports, as the library encodes it.
///
/// Each library's module says which versions it reads: today the CUDA
/// Runtime reports `CUDA Runtime` and `CUDA driver`, and cuTensorNet reports
/// `cuTensorNet` and `cuTensorNet's CUDA Runtime`, the CUDA Runtime it was
/// built against (README, "Versions").
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Version {
    /// What the version is of.
    pub component: &'static str,
    /// The version, as reported: `12090` is CUDA 12.9 and `21300` is
    /// cuTensorNet 2.13.0.
    pub value: i128,
}

/// One thing wrong with one library, in its [`LibraryReport`].
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum LibraryFault {
    /// The library's override variable is not an absolute path to a file
    /// (a symlink to a file is followed).
    #[error("invalid {variable} override {path:?}: {reason}")]
    InvalidOverride {
        variable: &'static str,
        path: PathBuf,
        reason: &'static str,
    },

    /// No file of the library was found at any of its candidate paths.
    #[error("not found; attempted {attempted:?}")]
    NotFound { attempted: Vec<PathBuf> },

    /// A file of the library was found but the system loader refused it:
    /// for instance, it is not a library for this host, or a library it
    /// needs is missing.
    #[error("failed to load {path:?}: {message}")]
    LoadFailed { path: PathBuf, message: String },

    /// The loaded library lacks a required function.
    #[error("missing required symbol {symbol}")]
    MissingRequiredSymbol { symbol: &'static str },

    /// A version is not the audited one. `component` names the version as
    /// in [`Version`].
    #[error("{component} version {found} is unsupported; supported: {supported}")]
    UnsupportedVersion {
        component: &'static str,
        found: i128,
        supported: i128,
    },

    /// A call that reads a version failed, with the library's status and
    /// message (today only the CUDA Runtime's calls can fail).
    #[error("{component} version probe failed with status {status}: {message}")]
    VersionProbeFailed {
        component: &'static str,
        status: u32,
        message: String,
    },
}
