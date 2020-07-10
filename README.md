Small utility to dump specified environment variables into IntelliJ workspace file,
so that your project can run with them.

Install with

```
cargo install --path .
```

Use:

```
idea-env-injector -s "USER" -f "~\my-idea-project\.idea\workspace.xml" -c "My run configuration"
```