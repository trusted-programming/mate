pub(crate) const TRAIT_PATHS: &[&[&str]] = &[
    &["rayon", "iter", "IntoParallelIterator"],
    &["rayon", "iter", "ParallelIterator"],
    &["rayon", "iter", "IndexedParallelIterator"],
];

pub(crate) const TRAIT_REF_PATHS: &[&[&str]] = &[
    &["rayon", "iter", "IntoParallelRefMutIterator"],
    &["rayon", "iter", "IntoParallelRefIterator"],
];
