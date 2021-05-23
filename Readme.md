# bdex-rs

## Usage

```text
bdex 

USAGE:
    bdex [FLAGS] <hash> <path> --retry-times <retry-times>

ARGS:
    <hash>    
    <path>    [default: .]

FLAGS:
    -h, --help         Prints help information
    -S, --skip-hash    
    -V, --version      Prints version information

OPTIONS:
    -R, --retry-times <retry-times>    [default: 10]
```

## Workflow

1. Download blocks to `./{hash}/`
2. Merge blocks into `{filename}`
3. **[NOT IMPLEMENTED]** Remove `./{hash}/` directory
