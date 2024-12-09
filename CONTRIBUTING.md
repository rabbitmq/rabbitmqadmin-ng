# Contributing

## Running Tests

While tests support the standard `cargo test` option, another option
for running tests is []().

### Run All Tests

``` bash
NEXTEST_RETRIES=3 cargo nextest run --all-features
```

### Run a Specific Test

``` bash
NEXTEST_RETRIES=3 cargo nextest run -E "test(test_list_all_vhost_limits)"
```