# wcapp — multi-file word-count CLI

Counts lines, words, and bytes for files given as argv (like a tiny `wc`).

## Layout
- `wcapp/counts.py` — counting helpers
- `wcapp/cli.py` — argument parsing and printing
- `wcapp/__main__.py` — `python -m wcapp`

## Run
```bash
python3 -m wcapp path/to/file
# prints: <lines> <words> <bytes> <path>
```

## Tests
```bash
cd source && PYTHONPATH=. python3 -m unittest discover -s tests -q
```
