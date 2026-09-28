# pipeline — small CSV-like data pipeline

1. Read CSV-like lines: `key,value` (no header required; skip blanks and `#` comments)
2. Filter: keep rows where value is a positive integer (value > 0)
3. Aggregate: sum values by key
4. Write output: `key,sum` lines sorted by key ascending

## Layout
- `pipeline/io.py` — read/write
- `pipeline/transform.py` — filter + aggregate
- `pipeline/cli.py` — CLI

## Run
```bash
python3 -m pipeline sample/input.csv sample/out.csv
```

## Tests
```bash
cd source && PYTHONPATH=. python3 -m unittest discover -s tests -q
