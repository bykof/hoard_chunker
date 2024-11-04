# hoard_chunker

hoard_chunker is designed to efficiently split large files into smaller, manageable
chunks and reassemble them when needed. This functionality is particularly useful for handling massive datasets,
facilitating easier processing, storage, or transfer for backups.

## Features:

- Backup: Backup files from `input_dir` into `output_dir` and chunk them with `FastCDC`.
- Restore: Restore chunks from `input_dir` into `output_dir` to backed up files.

### Setup

Clone the Repository:

```sh
git clone https://github.com/bykof/hoard_chunker.git
cd hoard_chunker
```

Build the Project:

```sh
cargo build --release
```

### Backup

```sh
hoard_chunker backup --input-path <INPUT_PATH> --output-path <OUTPUT_PATH>

--input-path <INPUT_PATH> (path files that need to be backed up)
--output-path <OUTPUT_PATH> (where to put the chunks)
``` 

### Restore

```sh
hoard_chunker restore --input-path <INPUT_PATH> --output-path <OUTPUT_PATH>

--input-path <INPUT_PATH> (path to chunks and metadata.json)
--output-path <OUTPUT_PATH> (where to restore)
```

## Contributing

Contributions are welcome! Feel free to submit a pull request or open an issue if you find a bug or have suggestions for
improvements.

## License

This project is licensed under the MIT License. See the LICENSE file for details.

## Contact

For any issues, questions, or feature requests, feel free to open an issue in the GitHub repository.

