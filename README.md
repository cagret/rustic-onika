# Rust Onika

Rust Onika is a program designed to perform indexing operations on files. It provides functionality to build an index of words contained in multiple files, allowing for efficient searching and retrieval of information.

## Features

- Indexing: Rust Onika reads files from a specified directory and creates an index of the words found in those files. It uses parallel processing to speed up the indexing process.

- Searching: Once the index is built, Rust Onika allows users to search for specific words or phrases within the indexed files. It provides fast and accurate search results.

- File Types: Rust Onika supports various file types, including text files (.txt), code files (.rs, .py, .cpp), and more. It can be easily extended to support additional file types.

## Installation

1. Make sure you have Rust installed on your system. If not, you can install it from the official Rust website: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

2. Clone the Rust Onika repository: `git clone git@github.com:cagret/rustic-onika.git`

3. Change to the project directory: `cd rustic-onika`

4. Build the project using Cargo: `cargo build --release`

5. Run Rust Onika: `target/release/rustic-onika [OPTIONS]`


## Usage

Rust Onika accepts the following command-line options:

- `-d, --directory <DIRECTORY>`: Specifies the directory to index. Rust Onika will recursively search for files within this directory and its subdirectories.

- `-s, --search <QUERY>`: Performs a search for the specified query within the indexed files. The query can be a single word or a phrase.

- `-h, --help`: Displays the help message and usage instructions.

## Examples

1. Index files in the current directory:

2. Index files in a specific directory:

3. Search for a word in the indexed files:

4. Search for a phrase in the indexed files:


## Contributing

Contributions to Rust Onika are welcome! If you encounter any issues or have suggestions for improvements, please open an issue on the GitHub repository: [git@github.com:cagret/rustic-onika.git/issues](https://github.com/cagret/rust-onika/issues)

To contribute code, follow these steps:

1. Fork the repository on GitHub.

2. Create a new branch with a descriptive name:

3. Make your changes and commit them:

4. Push the changes to your forked repository:

5. Open a pull request on the main repository.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more information.

