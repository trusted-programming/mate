// pub(crate) const ALLOWABLE_METHODS: &[&str] = &[
//     "all",
//     "any",
//     "chain",
//     "cloned",
//     "collect",
//     "copied",
//     "count",
//     "enumerate",
//     "filter",
//     "filter_map",
//     "flat_map",
//     "flatten",
//     // "fold", // TODO: need to be combined with reduce
//     "for_each",
//     "inspect",
//     "intersperse",
//     "map",
//     "max",
//     "max_by",
//     "max_by_key",
//     "min",
//     "min_by",
//     "min_by_key",
//     // "partition", // TODO: works differently verify if we can handle
//     "product", // might be an issue for floating points as it is not associative
//     "reduce",  // might be an issue for floating points as it is not associative
//     "sum",
//     // "try_fold", // TODO: need to be combined with reduce
//     "try_for_each",
//     "try_reduce",
//     "unzip",
//     "rev",
//     "skip",
//     "take",
//     "zip",
//     "cmp",
//     "eq",
//     "ge",
//     "gt",
//     "le",
//     "lt",
//     "ne",
//     "partial_cmp",
//     "step_by",
// ];

pub(crate) const TRAIT_PATHS: &[&[&str]] = &[
    &["rayon", "iter", "IntoParallelIterator"],
    &["rayon", "iter", "ParallelIterator"],
    &["rayon", "iter", "IndexedParallelIterator"],
];

pub(crate) const TRAIT_REF_PATHS: &[&[&str]] = &[
    &["rayon", "iter", "IntoParallelRefMutIterator"],
    &["rayon", "iter", "IntoParallelRefIterator"],
];
