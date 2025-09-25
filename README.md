# staging

`staging` is a crate that streamlines validation of multiple fields before returning errors.
Rust has good ergonomics for early return in error cases using the `?` operator,
but early return is sometimes undesirable:

-   Callers will have to make multiple calls to find all the errors in their request
-   Early return can disclose validation ordering, which may be a security issue

## Usage

Derive `Staging` on a struct `Example` gives you:

1. A new struct `ExampleStaging` where all the fields are now `Result<_, Error>`
2. A `TryFrom<ExampleStaging>` impl for the deriving struct
