# Publishing hanzo-net to PyPI

## Prerequisites

1. Create a PyPI account at https://pypi.org/
2. Generate an API token from your PyPI account settings
3. Install twine: `uv pip install twine`

## Publishing Steps

### Option 1: Using environment variables

```bash
export TWINE_USERNAME=__token__
export TWINE_PASSWORD=your-pypi-api-token
twine upload dist/*
```

### Option 2: Using .pypirc file

Create `~/.pypirc`:
```ini
[pypi]
username = __token__
password = your-pypi-api-token
```

Then run:
```bash
twine upload dist/*
```

### Option 3: Test on TestPyPI first

```bash
twine upload --repository testpypi dist/*
```

## Current Build Status

✅ Package built successfully:
- hanzo_net-0.0.1-py3-none-any.whl (903KB)
- hanzo_net-0.0.1.tar.gz (878KB)

✅ Dependencies resolved and installed
✅ Module imports successfully

⚠️ Some tests have import errors that need fixing (exo → net)

## Next Steps

1. Fix remaining test import issues
2. Add API token to environment
3. Run `twine upload dist/*` to publish to PyPI