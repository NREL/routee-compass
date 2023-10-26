# Static ONNX Runtime

The ONNX Runtime is linked statically into the routee-compass-powertrain binary. This
means that the routee-compass-powertrain binary is self-contained and does not require
any external dependencies.

These static libraries are built from the ONNX Runtime source code and can be
downloaded from [here for each version](https://github.com/supertone-inc/onnxruntime-build/releases).

These files are large and so we store them with git lfs. To download them, you
will need to install git lfs and then run `git lfs pull`. If you just want a
single platform, you can do:

```bash
git lfs pull --include="rust/routee-compass-powertrain/onnx-runtime/v1.15.1/osx-x86_64/libonnxruntime.a"
```

When building the code, the `ort` library needs to know which library to use.
You can set this like:

```bash
export ORT_LIB_LOCATION=rust/routee-compass-powertrain/onnx-runtime/v1.15.1/osx-x86_64/
```

Also see the `.cargo/config.toml` file for a way to set this once at the repo level
