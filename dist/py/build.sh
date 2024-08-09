ROOT_DIR="$(pwd)"
WHEEL_DIR="$ROOT_DIR/wheels"

PYO3_PYTHON="$(pyenv which python)"
TARGET_EXT="$($PYO3_PYTHON -c 'import sysconfig; print(sysconfig.get_config_var("EXT_SUFFIX"))')"

maturin build -i "$PYO3_PYTHON" --out "$WHEEL_DIR"

CODEMPSUBLIME_DIR="../../../codemp-sublime/bindings/"

wheels=($WHEEL_DIR/*.whl)
for wheel in $wheels; do
	cp $wheel "$CODEMPSUBLIME_DIR"
done
