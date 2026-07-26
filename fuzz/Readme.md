# Fuzzing

Fuzz targets for the `shapefile` reader, using [`cargo-fuzz`].

```
cargo install cargo-fuzz
cargo +nightly fuzz run reader_new
ASAN_OPTIONS=allocator_may_return_null=1 cargo +nightly fuzz run read_all
ASAN_OPTIONS=allocator_may_return_null=1 cargo +nightly fuzz run read_with_shx
```

`ASAN_OPTIONS=allocator_may_return_null=1` is required for the targets that
read shapes: the reader uses `Vec::try_reserve` to reject oversized allocations
gracefully, but by default AddressSanitizer aborts on a large allocation
instead of returning null (which would subvert `try_reserve`). With the option
set, `try_reserve` sees the failure and returns an error as intended.

## Targets

- `reader_new` — parses the `.shp` header only (`ShapeReader::new`).
- `read_all` — parses the header then all shapes (`ShapeReader::new` + `read`).
- `read_with_shx` — parses the `.shx` index and reads shapes using the index
  offsets and seeks (`ShapeReader::with_shx`). The input is a `(shp, shx)` byte
  tuple.
- `iter_shapes` — drives `iter_shapes_as` in a `for` loop instead of collecting.
  `read` / `read_as` are `iter_shapes_as().collect()`, which stops at the first
  error, so the other targets cannot observe an iterator that keeps yielding
  errors instead of stopping. Run it with a timeout so libFuzzer reports a hang:
  `cargo +nightly fuzz run iter_shapes -- -timeout=10`.

## Seeding the corpus (optional, speeds convergence)


```
mkdir -p corpus/read_all corpus/reader_new
cp ../tests/data/*.shp corpus/read_all/
cp ../tests/data/*.shp corpus/reader_new/
```

`read_with_shx` takes a structured `(shp, shx)` tuple input, so the raw
`.shp`/`.shx` files are not drop-in seeds for it.

## Known limitation: out-of-memory findings

`read_all` / `read_with_shx` can still report **out-of-memory** on inputs that
declare a huge point/part count. The reader guards these allocations with
`try_reserve` (so it never *aborts* the process), but it does not yet impose a
maximum allocation size, so a small crafted file can still request a
multi-gigabyte buffer. Under the fuzzer this trips libFuzzer's RSS limit / ASan
shadow memory.

These OOMs are the known unbounded-allocation limitation
```
ASAN_OPTIONS=allocator_may_return_null=1 cargo +nightly fuzz run read_all -- -rss_limit_mb=0
```

[`cargo-fuzz`]: https://github.com/rust-fuzz/cargo-fuzz
