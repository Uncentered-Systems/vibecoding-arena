# Hyperware Process-Lib Documentation Tool

This tool extracts and processes documentation from the `hyperware_process_lib` crate, making it searchable through a vector database. It consists of two main components:
1. A Rust script that extracts documentation from Rust source files
2. An experimental Python script that processes the documentation and enables semantic search

## Prerequisites

- **Rust**: Installed via `rustup` (includes `cargo`)
- **curl**: For downloading the tarball
- **tar**: For extracting the tarball (usually pre-installed on Unix-like systems)
- **rust-script**: To run Rust scripts without a full Cargo project

### Hardware Requirements

For the embedding model (`all-MiniLM-L6-v2`):
- **RAM**: Minimum 4GB (8GB recommended)
- **Disk Space**: ~100MB for the model
- **CPU**: Any modern CPU (2+ cores recommended)
- **GPU**: Not required, but can speed up embedding generation if available

## Setup Instructions

### 1. Install Required Tools

#### Install Rust (if not already installed)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Follow the on-screen instructions
```

#### Install rust-script
```bash
cargo install rust-script
```

#### Set up Python environment
```bash
# Create and activate a virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install required Python packages
pip install langchain langchain-text-splitters langchain-community
pip install sentence-transformers  # For HuggingFaceEmbeddings
pip install chromadb  # For vector storage
```

#### Install PyTorch (with GPU support if available)

**For Windows/Linux with NVIDIA GPU:**
```bash
# For CUDA 11.8 (adjust version as needed for your GPU)
pip install torch torchvision --index-url https://download.pytorch.org/whl/cu118
```

**For Mac (both Intel and Apple Silicon):**
```bash
pip install torch torchvision
```

### 2. Fetch the Hyperware Process Library
Make it executable and run it:
```bash
chmod +x fetch_hyperware.sh
./fetch_hyperware.sh
```

### 3. Set Up the Documentation Processing Scripts

#### Create the Rust AST Parser (ast-parser.rs)

Create a file named `ast-parser.rs` in the `utilities/process-lib/` directory with the content from the provided Rust script. This script extracts documentation from Rust source files and generates a markdown file.

## Running the Documentation Extractor

Run the Rust script to extract documentation from the source code:

```bash
# Navigate to the project root
cd utilities/process-lib

# Run the Rust script to extract documentation from the source code
# This will create api_documentation.md
rust-script ast-parser.rs utilities/rust-script/hyperware_process_lib
```

## How It Works

### Rust Component (ast-parser.rs)

The Rust script:
1. Walks through all `.rs` files in the specified directory
2. Parses each file using the Rust AST (Abstract Syntax Tree)
3. Extracts documentation comments, function signatures, struct definitions, etc.
4. Generates a markdown file (`api_documentation.md`) with the extracted documentation

Dependencies (automatically handled by rust-script):
- `syn`: For parsing Rust code
- `walkdir`: For traversing directories
- `quote`: For handling Rust tokens
- `serde_json`: For JSON serialization

Based on estimates from: https://platform.openai.com/tokenizer and other similar sources, the resulting document
is ~16000 tokens in length.

## Experimental: Semantic Search with Python

> **Note**: This part of the tool is experimental and requires additional setup.

The Python component allows for semantic search through the documentation using vector embeddings and similarity search.

### Additional Prerequisites for Python Component

- **Python**: Version 3.8 or higher
- **RAM**: Minimum 4GB (8GB recommended)
- **Disk Space**: ~100MB for the embedding model
- **CPU**: Any modern CPU (2+ cores recommended)
- **GPU**: Not required, but can speed up embedding generation if available

### Python Setup

#### Set up Python environment
```bash
# Create and activate a virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install required Python packages
pip install langchain langchain-text-splitters langchain-community
pip install sentence-transformers  # For HuggingFaceEmbeddings
pip install chromadb  # For vector storage
```

#### Install PyTorch (with GPU support if available)

**For Windows/Linux with NVIDIA GPU:**
```bash
# For CUDA 11.8 (adjust version as needed for your GPU)
pip install torch torchvision --index-url https://download.pytorch.org/whl/cu118
```

**For Mac (both Intel and Apple Silicon):**
```bash
pip install torch torchvision
```

#### Create the Python Processing Script (process-docs.py)

Create a file named `process-docs.py` in the `utilities/process-lib/` directory with the content from the provided Python script. This script processes the markdown documentation and enables semantic search.

### Using the Python Search Tool

Use the Python script to search through the documentation:

```bash
# Search with a specific query
python process-docs.py "How do I add something to the homepage?"

# Or run without arguments to use the default query
python process-docs.py
```

### GPU Acceleration for Python Component

The embedding model can run on a GPU to significantly speed up processing. The script automatically detects and uses available GPU hardware:

### NVIDIA GPUs (Windows/Linux)
- Requires CUDA-compatible NVIDIA GPU
- PyTorch with CUDA support must be installed (see setup instructions above)
- The script will automatically use CUDA if available

### Apple Silicon Macs (M1/M2/M3)
- Uses Metal Performance Shaders (MPS) for GPU acceleration
- Standard PyTorch installation supports MPS on macOS 12.3+
- The script will automatically use MPS if available

### Intel Macs
- Will run on CPU only (no GPU acceleration available)

### Python Component Details (process-docs.py)

The Python script:
1. Splits the markdown documentation into manageable chunks
2. Creates vector embeddings using the HuggingFace `all-MiniLM-L6-v2` model
3. Stores the embeddings in a Chroma vector database
4. Performs similarity search based on user queries
5. Returns the most relevant documentation section

Dependencies:
- `langchain`: For document processing and retrieval
- `sentence-transformers`: For creating embeddings
- `chromadb`: For vector storage and efficient similarity search of documentation chunks
- `torch torchvision`: For GPU-accelerated tensor operations and deep learning model support

### Troubleshooting Python Component

- If you encounter memory issues with the Python script, try reducing the `chunk_size` parameter
- Make sure the `api_documentation.md` file exists in the correct location
- Ensure all Python dependencies are properly installed

#### GPU-Specific Issues

- **CUDA errors**: Make sure your NVIDIA drivers are up to date
- **MPS errors on Mac**: MPS support is still evolving in PyTorch. If you encounter issues, modify the script to force CPU usage by setting `device = "cpu"`
- **Out of memory errors**: Reduce batch size or chunk size if your GPU runs out of memory

### Complete Workflow with Python (Experimental)

To run the entire process in one go:

```bash
# Generate documentation and immediately search it
rust-script ast-parser.rs | python process-docs.py "Your query here"
```

This will extract the documentation using the Rust script and then immediately process and search it using the Python script.


### Note on token count:
Based on estimates from [OpenAI's tokenizer](https://platform.openai.com/tokenizer), the token count for Hyperware Book is 121,221 tokens, and 
