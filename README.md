# openapi-generator-cli

[![crates.io](https://img.shields.io/crates/v/openapi-generator-cli.svg)](https://crates.io/crates/openapi-generator-cli)

**Rust CLI wrapper for [OpenAPI Generator](https://openapi-generator.tech/).**

This crate downloads the latest [OpenAPI Generator](https://openapi-generator.tech/) CLI JAR at build time
and wraps it in a convenient `cargo` command — no manual Java installation required.

---

## Quick start

```bash
# Install from crates.io
cargo install openapi-generator-cli

# Generate a Rust client from an OpenAPI spec
openapi-generator-cli generate \
  -i specs/petstore.yaml \
  -g rust \
  -o petstore-client \
  -p packageName=my_crate
```

The first run downloads the OpenAPI Generator JAR (and a JRE) at build time,
then forwards all arguments to the generator CLI.

## Installation

### From crates.io (recommended)

```bash
cargo install openapi-generator-cli
```

Then run it anywhere:

```bash
openapi-generator-cli generate -i spec.yaml -g rust -o ./client
```

### Without the bundled JRE

```bash
cargo install openapi-generator-cli --no-default-features
```

This skips downloading a JRE — Java must be on your `PATH`.

### Run directly from the repo

```bash
cargo run -- generate -i <spec> -g <generator> -o <output_dir> [options…]
```

---

## Feature flags

| Feature        | Default | Description |
|----------------|---------|-------------|
| `bundled-jre`  | ✅ On   | Downloads a JRE alongside the JAR so Java does **not** need to be pre-installed. Disable with `--no-default-features` if you already have Java on `PATH`. |

---

## Usage

The binary is a thin wrapper around `java -jar openapi-generator-cli.jar …`.
All arguments are forwarded directly, so the full [OpenAPI Generator CLI reference](https://openapi-generator.tech/docs/usage)
applies.

### Examples

```bash
# List available generators
openapi-generator-cli list

# Generate a Rust client
openapi-generator-cli generate \
  -i specs/petstore.yaml \
  -g rust \
  -o petstore-client \
  -p packageName=petstore

# Generate a TypeScript client
openapi-generator-cli generate \
  -i specs/petstore.yaml \
  -g typescript \
  -o ts-client

# Generate a Python client
openapi-generator-cli generate \
  -i specs/petstore.yaml \
  -g python \
  -o python-client

# Generate a Rust client with additional options
openapi-generator-cli generate \
  -i specs/petstore.yaml \
  -g rust \
  -o rust-client \
  -p packageName=my_api,packageVersion=0.2.0 \
  --additional-properties library=reqwest,supportAsync=true,useChrono=true
```

### Generating a Rust crate (full workflow)

1. **Generate** the client code:
   ```bash
   openapi-generator-cli generate \
     -i specs/petstore.yaml \
     -g rust \
     -o my-api/client \
     -p packageName=my_api
   ```

2. **Add it to your `Cargo.toml`**:
   ```toml
   [dependencies]
   my-api = { path = "./my-api/client" }
   tokio = { version = "1", features = ["full"] }
   ```

3. **Use it in your code**:
   ```rust
   use my_api::apis::configuration::Configuration;
   use my_api::apis::pet_api;

   #[tokio::main]
   async fn main() {
       let cfg = Configuration::new();
       match pet_api::get_pet_by_id(&cfg, 1).await {
           Ok(pet) => println!("Got pet: {}", pet.name),
           Err(e) => eprintln!("Error: {e}"),
       }
   }
   ```

---

## Rust generator config options

The OpenAPI Generator Rust client supports many options passed via
`-p` or `--additional-properties`. Key ones:

| Option | Default | Description |
|--------|---------|-------------|
| `packageName` | `openapi` | Rust package name (lowercase convention) |
| `packageVersion` | `1.0.0` | Rust package version |
| `library` | `reqwest` | HTTP client: `reqwest`, `reqwest-trait`, `hyper`, `hyper0x` |
| `supportAsync` | `true` | Generate async functions (reqwest only) |
| `useChrono` | `true` | Use `chrono` for date/time types |
| `hideGenerationTimestamp` | `true` | Omit timestamp in generated files |

See the full list at [openapi-generator.tech/docs/generators/rust](https://openapi-generator.tech/docs/generators/rust/#config-options).

---

## Documentation

- [OpenAPI Generator CLI usage](https://openapi-generator.tech/docs/usage)
- [Rust generator reference](https://openapi-generator.tech/docs/generators/rust)
- [All generators](https://openapi-generator.tech/docs/generators)
- [Configuration options](https://openapi-generator.tech/docs/configuration)

---

## How it works

1. `build.rs` fetches the latest OpenAPI Generator CLI version from Maven Central
   and downloads the JAR into `OUT_DIR`.
2. If the `bundled-jre` feature is enabled (default), it also downloads a JRE
   from Adoptium so Java doesn't need to be pre-installed.
3. The binary reads `$OPENAPI_GENERATOR_JAR` and `$JRE_HOME` (set at compile
   time) and runs `java -jar openapi-generator-cli.jar <your args>`.
4. The JAR/JRE are cached across builds and only re-downloaded when the
   generator version bumps.
