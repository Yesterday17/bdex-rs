# bdex-rs

## Usage

```text
bdex 

USAGE:
    bdex [FLAGS] <hash> <path> --threads <threads> --retry-times <retry-times>

ARGS:
    <hash>    
    <path>    [default: .]

FLAGS:
    -h, --help          Prints help information
    -k, --keep-files    
    -S, --skip-hash     
    -V, --version       Prints version information

OPTIONS:
    -R, --retry-times <retry-times>    [default: 10]
    -t, --threads <threads>            [default: 8]
```

## Workflow

1. Download blocks to `./{hash}/`
2. Merge blocks into `{filename}`
3. Remove `./{hash}/` directory

## Cross compile for macOS

```bash
./scripts/macos_prepare.sh
./scripts/macos_build.sh
```