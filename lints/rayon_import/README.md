# template

### What it does

add rayon import to crate root

### Why is this bad?

no rayon no par_iter traits

### Example

```rust
fn main() {}
```

Use instead:

```rust
use rayon::prelude::*;

fn main () {}
```
