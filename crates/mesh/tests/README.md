# Integration Tests

These really could be unit-style tests, but due to some circular deps
between `stl-loader` and `mesh`, we need to put these tests in an integration
test to successfully compile.