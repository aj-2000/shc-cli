REPO_URL="https://github.com/aj-2000/shc-cli.git"
REPO_DIR=$(mktemp -d)

if ! command -v rustc &> /dev/null; then
    echo "Rust is not installed. Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source $HOME/.cargo/env
fi

echo "Cloning the shc repository..."
git clone --depth 1 "$REPO_URL" "$REPO_DIR"

echo "Installing shc..."
cd "$REPO_DIR"
cargo install --path .

cd ..

echo "Cleaning up..."
rm -rf "$REPO_DIR"

echo "shc installed successfully!"
