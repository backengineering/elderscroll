// Copyright (C) Back Engineering Labs, Inc. - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential

/// This is how you can return a string error easily. The nice thing about
/// this that if you are only returning a Str, it will use the ENUM type
/// which won't be a problem for .ok_or which does eager evaluation. But if you
/// pass additional parameters, it will call format! and then you want to use
/// .ok_or_else which does lazy evaluation.
///
/// Use this for unrecoverable errors.
#[macro_export]
macro_rules! string_err {
    ($fmt:literal) => {
        $crate::error::KsError::Str(concat!(file!(), ":", line!(), " - ", $fmt))
    };
    ($fmt:literal, $($args:tt)*) => {
        $crate::error::KsError::String(format!(concat!(file!(), ":", line!(), " - ", $fmt), $($args)*))
    };
}

pub mod directory;
pub mod msf;
pub mod overlays;
pub mod pagelist;
pub mod view;
