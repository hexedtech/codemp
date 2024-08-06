ROOT_DIR="$(pwd)"
WHEEL_DIR="$ROOT_DIR/wheels"

PYO3_PYTHON="$(pyenv which python)"
TARGET_EXT="$($PYO3_PYTHON -c 'import sysconfig; print(sysconfig.get_config_var("EXT_SUFFIX"))')"

maturin build -i "$PYO3_PYTHON" --out "$WHEEL_DIR"
