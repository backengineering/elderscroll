// Copyright (C) Back Engineering Labs, Inc. - All Rights Reserved

/// Computes the minimum size of the structure, which is the max of all elements
/// offset + size in the struct.
#[macro_export]
macro_rules! struct_overlay_min_size {
    () => { 0 };
    ([$off:literal] $name:ident: $ty:ty,) => ($off + std::mem::size_of::<$ty>());
    ([$off:literal] $name:ident: $ty:ty, $($next:tt)+) => {{
        let l = $crate::struct_overlay_min_size!($($next)*);
        let r = ($off + std::mem::size_of::<$ty>());
        if l < r {
            r
        } else {
            l
        }
    }};
}
#[macro_export]
macro_rules! struct_overlay_set_gen {
    () => {};
    ([$off:literal] $name:ident: [$ty:ty;$count:literal], $($next:tt)*) => {
        concat_idents::concat_idents!(set_name_at = set_, $name, _at, {
            #[allow(dead_code)]
            #[inline(always)]
            pub fn set_name_at(&mut self, pos: usize, val: $ty) -> bool {
                let offset = $off + pos * std::mem::size_of::<$ty>();
                if offset <= Self::MINIMUM_SIZE - std::mem::size_of::<$ty>() {
                    unsafe { ((self.ptr.as_mut_ptr() as usize + offset) as *mut $ty).write_unaligned(val) };
                    true
                } else {
                    false
                }
            }
        });
        concat_idents::concat_idents!(set_name_full = set_, $name, {
            #[allow(dead_code)]
            #[inline(always)]
            pub fn set_name_full(&mut self, val: [$ty; $count]) {
                unsafe { ((self.ptr.as_mut_ptr() as usize + $off) as *mut [$ty; $count]).write_unaligned(val) }
            }
        });
        $crate::struct_overlay_set_gen!($($next)*);
    };
    ([$off:literal] $name:ident: $ty:ty, $($next:tt)*) => {
        concat_idents::concat_idents!(set_name = set_, $name, {
            #[allow(dead_code)]
            #[inline(always)]
            pub fn set_name(&mut self, val: $ty) {
                unsafe { ((self.ptr.as_mut_ptr() as usize + $off) as *mut $ty).write_unaligned(val) }
            }
        });
        $crate::struct_overlay_set_gen!($($next)*);
    };
}
#[macro_export]
macro_rules! struct_overlay_get_gen {
    () => {};
    ([$off:literal] $name:ident: [$ty:ty; $count:literal], $($next:tt)*) => {
        concat_idents::concat_idents!(get_name_at = get_, $name, _at, {
            #[allow(dead_code)]
            #[inline(always)]
            pub fn get_name_at(&self, pos: usize) -> Option<$ty> {
                let offset = $off + pos * std::mem::size_of::<$ty>();
                if offset <= Self::MINIMUM_SIZE - std::mem::size_of::<$ty>() {
                    Some(unsafe { ((self.ptr.as_ptr() as usize + offset) as *mut $ty).read_unaligned() })
                } else {
                    None
                }
            }
        });
        concat_idents::concat_idents!(get_name_full = get_, $name{
            #[allow(dead_code)]
            #[inline(always)]
            pub fn get_name_full(&self) -> [$ty; $count] {
                unsafe { ((self.ptr.as_ptr() as usize + $off) as *const [$ty; $count]).read_unaligned() }
            }
        });
        $crate::struct_overlay_get_gen!($($next)*);
    };
    ([$off:literal] $name:ident: $ty:ty, $($next:tt)*) => {
        concat_idents::concat_idents!(get_name = get_, $name, {
            #[allow(dead_code)]
            #[inline(always)]
            pub fn get_name(&self) -> $ty {
                unsafe { ((self.ptr.as_ptr() as usize + $off) as *const $ty).read_unaligned() }
            }
        });
        $crate::struct_overlay_get_gen!($($next)*);
    };
}
#[macro_export]
macro_rules! struct_overlay_debug_gen {
    ($struct_name:ident, $([$off:literal] $name:ident: $ty:ty,)*) => {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct(stringify!($struct_name))
                $(.field(stringify!($name), &unsafe { ((self.ptr.as_ptr() as usize + $off) as *const $ty).read_unaligned() }))*
                .finish()
        }
    };
}
#[macro_export]
macro_rules! struct_overlay {
    ($vis:vis $struct_name:ident { $($next:tt)* }) => {
        $vis struct $struct_name<'a> {
            ptr: &'a [u8]
        }
        impl<'a> $struct_name<'a> {
            const MINIMUM_SIZE: usize = $crate::struct_overlay_min_size!($($next)*);
            #[allow(dead_code)]
            #[inline]
            pub fn new(ptr: &'a [u8]) -> Option<Self> {
                if ptr.len() >= Self::MINIMUM_SIZE {
                    Some(Self {
                        ptr: &ptr[0..Self::MINIMUM_SIZE],
                    })
                } else {
                    None
                }
            }
            #[allow(dead_code)]
            pub const fn size() -> usize {
                Self::MINIMUM_SIZE
            }
            $crate::struct_overlay_get_gen!($($next)*);
        }
        impl<'a> core::fmt::Debug for $struct_name<'a> {
            $crate::struct_overlay_debug_gen!($struct_name, $($next)*);
        }
    }
}
#[macro_export]
macro_rules! struct_overlay_mut {
    ($vis:vis $struct_name:ident { $($next:tt)* }) => {
        $vis struct $struct_name<'a> {
            ptr: &'a mut [u8]
        }
        impl<'a> $struct_name<'a> {
            const MINIMUM_SIZE: usize = $crate::struct_overlay_min_size!($($next)*);
            #[allow(dead_code)]
            #[inline]
            pub fn new(ptr: &'a mut[u8]) -> Option<Self> {
                if ptr.len() >= Self::MINIMUM_SIZE {
                    Some(Self {
                        ptr: &mut ptr[0..Self::MINIMUM_SIZE],
                    })
                } else {
                    None
                }
            }
            #[allow(dead_code)]
            pub fn zero(&mut self) {
                for byte in self.ptr.iter_mut() {
                    *byte = 0;
                }
            }
            #[allow(dead_code)]
            pub const fn size() -> usize {
                Self::MINIMUM_SIZE
            }
            $crate::struct_overlay_set_gen!($($next)*);
            $crate::struct_overlay_get_gen!($($next)*);
        }
        impl<'a> core::fmt::Debug for $struct_name<'a> {
            $crate::struct_overlay_debug_gen!($struct_name, $($next)*);
        }
    }
}

#[macro_export]
macro_rules! struct_overlay_both {
    (($vis:vis $struct_name:ident,$vis_mut:vis $struct_name_mut:ident) { $($next:tt)* }) => {
        $crate::struct_overlay!($vis $struct_name { $($next)* });
        $crate::struct_overlay_mut!($vis_mut $struct_name_mut { $($next)* });
    }
}

#[macro_export]
macro_rules! offset_struct {
    ($vis:vis $struct_name:ident { $($next:tt)* }) => {
        $vis struct $struct_name {
            ptr: [u8; Self::MINIMUM_SIZE],
        }
        impl $struct_name {
            const MINIMUM_SIZE: usize = $crate::struct_overlay_min_size!($($next)*);
            #[allow(dead_code)]
            #[inline]
            pub fn new() -> Self {
                Self::default()
            }
            #[allow(dead_code)]
            pub fn zero(&mut self) {
                for byte in self.ptr.iter_mut() {
                    *byte = 0;
                }
            }
            #[allow(dead_code)]
            pub const fn size() -> usize {
                Self::MINIMUM_SIZE
            }
            #[allow(dead_code)]
            #[inline(always)]
            pub fn slice(&self) -> &[u8] {
                &self.ptr
            }
            #[allow(dead_code)]
            #[inline(always)]
            pub fn slice_mut(&mut self) -> &mut [u8] {
                &mut self.ptr
            }
            $crate::struct_overlay_set_gen!($($next)*);
            $crate::struct_overlay_get_gen!($($next)*);
        }
        impl std::default::Default for $struct_name {
            fn default() -> Self {
                Self {
                    ptr: [0; Self::MINIMUM_SIZE],
                }
            }
        }
        impl core::fmt::Debug for $struct_name {
            $crate::struct_overlay_debug_gen!($struct_name, $($next)*);
        }
    }
}
