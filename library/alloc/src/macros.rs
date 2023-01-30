/// Creates a [`Vec`] containing the arguments.
///
/// `vec!` allows `Vec`s to be defined with the same syntax as array expressions. There are two
/// forms of this macro:
///
/// - Create a [`Vec`] containing a given list of elements:
///
/// ```
/// let v = vec![1, 2, 3];
/// assert_eq!(v[0], 1);
/// assert_eq!(v[1], 2);
/// assert_eq!(v[2], 3);
/// ```
///
/// - Create a [`Vec`] from a given element and size:
///
/// ```
/// let v = vec![1; 3];
/// assert_eq!(v, [1, 1, 1]);
/// ```
///
/// Note that unlike array expressions this syntax supports all elements which implement [`Clone`]
/// and the number of elements doesn't have to be a constant.
///
/// This will use `clone` to duplicate an expression, so one should be careful using this with types
/// having a nonstandard `Clone` implementation. For example, `vec![Rc::new(1); 5]` will create a
/// vector of five references to the same boxed integer value, not five references pointing to
/// independently boxed integers.
///
/// Also, note that `vec![expr; 0]` is allowed, and produces an empty vector. This will still
/// evaluate `expr`, however, and immediately drop the resulting value, so be mindful of side
/// effects.
///
/// [`Vec`]: crate::vec::Vec
#[cfg(all(not(no_global_oom_handling), not(test)))]
#[macro_export]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_diagnostic_item = "vec_macro"]
#[allow_internal_unstable(rustc_attrs, liballoc_internals)]
macro_rules! vec {
    () => (
        $crate::__rust_force_expr!($crate::vec::Vec::new())
    );
    ($elem:expr; $n:expr) => (
        $crate::__rust_force_expr!($crate::vec::from_elem($elem, $n))
    );
    ($($x:expr),+ $(,)?) => (
        $crate::__rust_force_expr!(<[_]>::into_vec(
            #[rustc_box]
            $crate::boxed::Box::new([$($x),+])
        ))
    );
}

// HACK(japaric): with cfg(test) the inherent `[T]::into_vec` method, which is required for this
// macro definition, is not available. Instead use the `slice::into_vec`  function which is only
// available with cfg(test) NB see the slice::hack module in slice.rs for more information
#[cfg(all(not(no_global_oom_handling), test))]
#[allow(unused_macro_rules)]
macro_rules! vec {
    () => (
        $crate::vec::Vec::new()
    );
    ($elem:expr; $n:expr) => (
        $crate::vec::from_elem($elem, $n)
    );
    ($($x:expr),*) => (
        $crate::slice::into_vec($crate::boxed::Box::new([$($x),*]))
    );
    ($($x:expr,)*) => (vec![$($x),*])
}

/// Creates a `String` using interpolation of runtime expressions.
///
/// The first argument `format!` receives is a format string. This must be a string literal. The
/// power of the formatting string is in the `{}`s contained.
///
/// Additional parameters passed to `format!` replace the `{}`s within the formatting string in the
/// order given unless named or positional parameters are used; see [`std::fmt`] for more
/// information.
///
/// A common use for `format!` is concatenation and interpolation of strings. The same convention is
/// used with [`print!`] and [`write!`] macros, depending on the intended destination of the string.
///
/// To convert a single value to a string, use the [`to_string`] method. This will use the
/// [`Display`] formatting trait.
///
/// [`std::fmt`]: ../std/fmt/index.html
/// [`print!`]: ../std/macro.print.html
/// [`write!`]: core::write
/// [`to_string`]: crate::string::ToString
/// [`Display`]: core::fmt::Display
///
/// # Panics
///
/// `format!` panics if a formatting trait implementation returns an error. This indicates an
/// incorrect implementation since `fmt::Write for String` never returns an error itself.
///
/// # Examples
///
/// ```
/// format!("test");
/// format!("hello {}", "world!");
/// format!("x = {}, y = {y}", 10, y = 30);
/// let (x, y) = (1, 2);
/// format!("{x} + {y} = 3");
/// ```
#[macro_export]
#[stable(feature = "rust1", since = "1.0.0")]
#[cfg_attr(not(test), rustc_diagnostic_item = "format_macro")]
macro_rules! format {
    ($($arg:tt)*) => {{
        let res = $crate::fmt::format($crate::__export::format_args!($($arg)*));
        res
    }}
}

/// Force AST node to an expression to improve diagnostics in pattern position.
#[doc(hidden)]
#[macro_export]
#[unstable(feature = "liballoc_internals", issue = "none", reason = "implementation detail")]
macro_rules! __rust_force_expr {
    ($e:expr) => {
        $e
    };
}

// ----- CoAlloc constant-like macros:

// @FIXME Docs
#[unstable(feature = "global_co_alloc_meta", issue = "none")]
#[macro_export]
macro_rules! CO_ALLOC_PREF_NUM_META_SLOTS_ONE {
    () => {
        (1 as $crate::co_alloc::CoAllocMetaNumSlotsPref)
    };
}

// @FIXME Docs
#[unstable(feature = "global_co_alloc_meta", issue = "none")]
#[macro_export]
macro_rules! CO_ALLOC_PREF_NUM_META_SLOTS_ZERO {
    () => {
        (0 as $crate::co_alloc::CoAllocMetaNumSlotsPref)
    };
}

// @FIXME Docs
#[unstable(feature = "global_co_alloc_meta", issue = "none")]
#[macro_export]
macro_rules! CO_ALLOC_PREF_NUM_META_SLOTS_DEFAULT {
    () => {
        (0 as $crate::co_alloc::CoAllocMetaNumSlotsPref)
    };
}

/// "Yes" as a type's preference for coallocation using metadata (in either user space, or `alloc`
/// or `std` space).
///
/// It may be overriden by the allocator. For example, if the allocator doesn't support
/// coallocation, then this value makes no difference.
///
/// This constant and its type WILL CHANGE (once ``#![feature(generic_const_exprs)]` and
/// `#![feature(adt_const_params)]` are stable) to a dedicated struct/enum. Hence DO NOT hard
/// code/replace/mix this any other values/parameters.
#[unstable(feature = "global_co_alloc_meta", issue = "none")]
#[macro_export]
macro_rules! CO_ALLOC_PREF_META_YES {
    () => {
        $crate::co_alloc_pref!($crate::CO_ALLOC_PREF_NUM_META_SLOTS_ONE!())
    };
}

/// "No" as a type's preference for coallocation using metadata (in either user space, or `alloc` or
/// `std` space).
///
/// Any allocator is required to respect this. Even if the allocator does support coallocation, it
/// will not coallocate types that use this value.
///
/// This constant and its type WILL CHANGE (once ``#![feature(generic_const_exprs)]` and
/// `#![feature(adt_const_params)]` are stable) to a dedicated struct/enum. Hence DO NOT hard
/// code/replace/mix this any other values/parameters.
#[unstable(feature = "global_co_alloc_meta", issue = "none")]
#[macro_export]
macro_rules! CO_ALLOC_PREF_META_NO {
    () => {
        $crate::co_alloc_pref!($crate::CO_ALLOC_PREF_NUM_META_SLOTS_ZERO!())
    };
}

/// "Default" as a type's preference for coallocation using metadata (in either user space, or
/// `alloc` or `std` space).
/// 
/// This value and its type WILL CHANGE (once ``#![feature(generic_const_exprs)]` and
/// `#![feature(adt_const_params)]` are stable) to a dedicated struct/enum. Hence DO NOT hard
/// code/replace/mix this any other values/parameters.
/// 
/// (@FIXME) This WILL BE BECOME OBSOLETE and it WILL BE REPLACED with a `const` (and/or some kind
/// of compile time preference) once a related ICE is fixed (@FIXME add the ICE link here). Then
/// consider moving such a `const` to a submodule, for example `::alloc::co_alloc`.
#[unstable(feature = "global_co_alloc_default", issue = "none")]
#[macro_export]
macro_rules! CO_ALLOC_PREF_META_DEFAULT {
    () => {
        $crate::co_alloc_pref!($crate::CO_ALLOC_PREF_NUM_META_SLOTS_DEFAULT!())
    };
}

/// Default [::alloc::CoAllocPref] value/config, based on `CO_ALLOC_PREF_META_DEFAULT`.
#[unstable(feature = "global_co_alloc_meta", issue = "none")]
#[macro_export]
macro_rules! CO_ALLOC_PREF_DEFAULT {
    () => { $crate::CO_ALLOC_PREF_META_DEFAULT!() };
}

// @FIXME Move to library/alloc - if possible:
#[unstable(feature = "global_co_alloc", issue = "none")]
//pub const SHORT_TERM_VEC_CO_ALLOC_PREF: bool = true;
#[macro_export]
macro_rules! SHORT_TERM_VEC_CO_ALLOC_PREF {
    () => { $crate::CO_ALLOC_PREF_META_NO!() };
}

// ------ CoAlloc preference/config conversion macros:

/// Create a `CoAllocPref` value based on the given parameter(s). For now, only one parameter is
/// supported, and it's required: `meta_pref`.
/// 
/// @param `meta_pref` is one of: `CO_ALLOC_PREF_META_YES, CO_ALLOC_PREF_META_NO`, or
/// `CO_ALLOC_PREF_META_DEFAULT`.
/// 
/// @return `CoAllocPref` value
#[unstable(feature = "global_co_alloc_meta", issue = "none")]
#[macro_export]
macro_rules! co_alloc_pref {
    // ($meta_pref + (0 as CoAllocMetaNumSlotsPref)) ensures that $meta_pref is of type
    // `CoAllocMetaNumSlotsPref`. Otherwise the casting of the result to `CoAllocPref` would not
    // report the incorrect type of $meta_pref (if $meta_pref were some other integer, casting would
    // compile, and we would not be notified).
    ($meta_pref:expr) => {
        (($meta_pref + (0 as $crate::co_alloc::CoAllocMetaNumSlotsPref)) as $crate::co_alloc::CoAllocPref)
    };
}

/// Return 0 or 1, indicating whether to use coallocation metadata (or not) with the given allocator
/// type `alloc` and cooperation preference `co_alloc_pref`.
///
/// NOT for public use. Param `co_alloc_pref` - can override the allocator's default preference for
/// cooperation, or can make the type not cooperative, regardless of whether allocator `A` is
/// cooperative.
/// 
/// @param `alloc` Allocator (implementation) type. @param `co_alloc_pref` The heap-based type's
/// preference for coallocation, as an [::alloc::CoAllocPref] value.
/// 
/// The type of second parameter `co_alloc_pref` WILL CHANGE. DO NOT hardcode/cast/mix that type.
/// Instead, use [::alloc::CoAllocPref].
/// 
// FIXME replace the macro with an (updated version of the below) `const` function). Only once
// generic_const_exprs is stable (that is, when consumer crates don't need to declare
// generic_const_exprs feature anymore). Then consider moving the function to a submodule, for
// example ::alloc::co_alloc.
#[unstable(feature = "global_co_alloc", issue = "none")]
#[macro_export]
macro_rules! meta_num_slots {
    // Generating, for example, (0 as usize), here, triggers an ICE.

    // This "validates" types of both params - to prevent mix ups.
    // @FIXME remove this comment line: Removing/commenting out the part: <$alloc as ::core::alloc::Allocator>::CO_ALLOC_META_NUM_SLOTS +
    // does NOT fix the ICE (unless there are multiple ICE's).
    ($alloc:ty, $co_alloc_pref:expr) => {
        /*(
            ((<$alloc as ::core::alloc::Allocator>::CO_ALLOC_META_NUM_SLOTS + (0 as ::core::alloc::CoAllocatorMetaNumSlots))0
            as usize)
        * ($co_alloc_pref + (0 as $crate::co_alloc::CoAllocPref))
         as usize)*/0usize
    };
}
// -\---> replace with something like:
/*
#[unstable(feature = "global_co_alloc", issue = "none")]
pub const fn meta_num_slots<A: Allocator>(
    CO_ALLOC_PREF: bool,
) -> usize {
    if A::CO_ALLOC_META_NUM_SLOTS && CO_ALLOC_PREF { 1 } else { 0 }
}
*/

/// Like `meta_num_slots`, but for the default coallocation preference (`DEFAULT_CO_ALLOC_PREF`).
///
/// Return 0 or 1, indicating whether to use coallocation metadata (or not) with the given allocator
/// type `alloc` and the default coallocation preference (`DEFAULT_CO_ALLOC_PREF()!`).
///
// FIXME replace the macro with a `const` function. Only once generic_const_exprs is stable (that
// is, when consumer crates don't need to declare generic_const_exprs feature anymore). Then
// consider moving the function to a submodule, for example ::alloc::co_alloc.
#[unstable(feature = "global_co_alloc", issue = "none")]
#[macro_export]
macro_rules! meta_num_slots_default {
    // Can't generate if ... {1} else {0}
    // because it's "overly complex generic constant".
    ($alloc:ty) => {
        $crate::meta_num_slots!( $alloc, $crate::CO_ALLOC_PREF_DEFAULT!() )
    };
}

/// Like `meta_num_slots`, but for the default coallocation preference (`DEFAULT_CO_ALLOC_PREF`).
///
/// Return 0 or 1, indicating whether to use coallocation metadata (or not) with the global allocator
/// type `alloc` and the given coallocation preference `co_alloc_`.
///
// FIXME replace the macro with a `const` function. Only once generic_const_exprs is stable (that
// is, when consumer crates don't need to declare `generic_const_exprs` feature anymore). Then
// consider moving the function to a submodule, for example ::alloc::co_alloc. See above.
#[unstable(feature = "global_co_alloc", issue = "none")]
#[macro_export]
macro_rules! meta_num_slots_global {
    ($co_alloc_pref:expr) => {
        $crate::meta_num_slots!( $crate::alloc::Global, $co_alloc_pref)
    };
}

/// Like `meta_num_slots`, but for `Global allocator and default coallocation preference
/// (`CO_ALLOC_PREF_DEFAULT`).
///
// @FIXME once generic_const_exprs is stable, replace this with a `const` function. Then consider
// moving the function to a submodule, for example alloc::co_alloc. See above.
#[unstable(feature = "global_co_alloc", issue = "none")]
#[macro_export]
macro_rules! meta_num_slots_default_global {
    () => {
        $crate::meta_num_slots!( $crate::alloc::Global, $crate::CO_ALLOC_PREF_DEFAULT!())
    };
}
