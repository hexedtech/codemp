ROOT_DIR="$(pwd)"
WHEEL_DIR="$ROOT_DIR/wheels"

PYO3_PYTHON="$(pyenv which python)"
TARGET_EXT="$($PYO3_PYTHON -c 'import sysconfig; print(sysconfig.get_config_var("EXT_SUFFIX"))')"

MATURIN_PYPI_TOKEN="$(cat ~/.local/secrets/pypi_codemp_token)" maturin publish -i "$PYO3_PYTHON" \
--out "$WHEEL_DIR" --non-interactive --repository "pypi"
