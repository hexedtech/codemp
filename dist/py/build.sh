ROOT_DIR="$(pwd)"
WHEEL_DIR="$ROOT_DIR/wheels"

PYO3_PYTHON="$(pyenv which python)"
TARGET_EXT="$($PYO3_PYTHON -c 'import sysconfig; print(sysconfig.get_config_var("EXT_SUFFIX"))')"

maturin build -i "$PYO3_PYTHON" --out "$WHEEL_DIR"

echo "\nAUTOINSTALL - REMOVE AFTER TESTING\n"

CODEMPSUBLIME_DIR="../../../codemp-sublime/"
CODEMPTEST_DIR="../../../codemp-python-test/"

wheels=($WHEEL_DIR/*.whl)
for wheel in $wheels; do
	echo "moving $wheel to $CODEMPSUBLIME_DIR"
	cp $wheel "$CODEMPSUBLIME_DIR/bindings/"
	cp $wheel "$CODEMPTEST_DIR"
done

pushd "$CODEMPTEST_DIR"
source .venv/bin/activate
pip install $wheel --force-reinstall
deactivate
popd

pushd "$CODEMPSUBLIME_DIR"
source .venv/bin/activate
pip install $wheel --force-reinstall
deactivate
popd
