# Data Management Specification

Simple approach for fetching and managing exoplanet data from NASA Exoplanet Archive.

## Overview

**Strategy:**
- Use `curl` to download VOTable files from NASA
- Add timestamps to filenames for ordering
- Keep multiple versions, process only the newest
- Simple shell script for automation

## Data Source

NASA Exoplanet Archive TAP Service:
- **Stellar Hosts**: `https://exoplanetarchive.ipac.caltech.edu/TAP/sync?query=select+*+from+stellarhosts&format=votable`
- **Exoplanets**: `https://exoplanetarchive.ipac.caltech.edu/TAP/sync?query=select+*+from+pscomppars&format=votable`

## File Naming Convention

```
data/raw/stellarhosts_YYYYMMDD_HHMMSS.xml
data/raw/exoplanets_YYYYMMDD_HHMMSS.xml
data/parquet/stellarhosts.parquet  # Latest converted
data/parquet/exoplanets.parquet    # Latest converted
```

Timestamp format: `20250128_143022` (Jan 28, 2025 at 14:30:22)

## Download Script

```bash
#!/bin/bash
# scripts/fetch-data.sh

set -e

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RAW_DIR="data/raw"
mkdir -p $RAW_DIR

echo "Fetching exoplanet data at $TIMESTAMP..."

# Download stellar hosts
echo "Downloading stellar hosts..."
curl -o "$RAW_DIR/stellarhosts_$TIMESTAMP.xml" \
  "https://exoplanetarchive.ipac.caltech.edu/TAP/sync?query=select+*+from+stellarhosts&format=votable"

# Download exoplanets
echo "Downloading exoplanets..."
curl -o "$RAW_DIR/exoplanets_$TIMESTAMP.xml" \
  "https://exoplanetarchive.ipac.caltech.edu/TAP/sync?query=select+*+from+pscomppars&format=votable"

echo "Download complete!"
echo "Files saved with timestamp: $TIMESTAMP"

# List all versions
echo -e "\nAvailable versions:"
ls -lh $RAW_DIR/stellarhosts_*.xml | tail -5
ls -lh $RAW_DIR/exoplanets_*.xml | tail -5
```

## Conversion Script

Uses exo-cli to convert the newest files:

```bash
#!/bin/bash
# scripts/convert-latest.sh

set -e

RAW_DIR="data/raw"
PARQUET_DIR="data/parquet"
mkdir -p $PARQUET_DIR

# Find newest stellarhosts file
LATEST_HOSTS=$(ls -t $RAW_DIR/stellarhosts_*.xml | head -1)
echo "Converting: $LATEST_HOSTS"
cargo run --package exo-cli -- convert "$LATEST_HOSTS" "$PARQUET_DIR/stellarhosts.parquet"

# Find newest exoplanets file
LATEST_PLANETS=$(ls -t $RAW_DIR/exoplanets_*.xml | head -1)
echo "Converting: $LATEST_PLANETS"
cargo run --package exo-cli -- convert "$LATEST_PLANETS" "$PARQUET_DIR/exoplanets.parquet"

echo "Conversion complete!"
echo "Parquet files ready in $PARQUET_DIR/"
```

## Full Update Process

Combined script for complete update:

```bash
#!/bin/bash
# scripts/update-data.sh

set -e

echo "=== Exoplanet Data Update ==="
echo ""

# Step 1: Fetch new data
./scripts/fetch-data.sh

echo ""
echo "=== Converting to Parquet ==="
echo ""

# Step 2: Convert latest
./scripts/convert-latest.sh

echo ""
echo "=== Update Complete ==="
```

## Cleanup Old Files

Keep only last N versions to save space:

```bash
#!/bin/bash
# scripts/cleanup-old.sh

KEEP_LAST=5
RAW_DIR="data/raw"

echo "Keeping last $KEEP_LAST versions..."

# Cleanup stellarhosts
ls -t $RAW_DIR/stellarhosts_*.xml | tail -n +$((KEEP_LAST + 1)) | xargs rm -f

# Cleanup exoplanets
ls -t $RAW_DIR/exoplanets_*.xml | tail -n +$((KEEP_LAST + 1)) | xargs rm -f

echo "Cleanup complete!"
```

## Directory Structure

```
data/
├── raw/                          # Downloaded VOTable files
│   ├── stellarhosts_20250128_143022.xml
│   ├── stellarhosts_20250127_120000.xml
│   ├── exoplanets_20250128_143022.xml
│   └── exoplanets_20250127_120000.xml
└── parquet/                      # Converted Parquet files
    ├── stellarhosts.parquet      # Always the latest
    └── exoplanets.parquet        # Always the latest
```

## Automated Updates

Add to crontab for weekly updates:

```bash
# Update data every Monday at 2 AM
0 2 * * 1 cd /path/to/exoplanets-catalog && ./scripts/update-data.sh

# Cleanup old files every month
0 3 1 * * cd /path/to/exoplanets-catalog && ./scripts/cleanup-old.sh
```

## Manual Usage

```bash
# Fetch new data
./scripts/fetch-data.sh

# Convert latest to parquet
./scripts/convert-latest.sh

# Or do both at once
./scripts/update-data.sh

# Cleanup old versions
./scripts/cleanup-old.sh
```

## Error Handling

Add to scripts for robustness:

```bash
# Check if curl succeeded
if [ ! -f "$RAW_DIR/stellarhosts_$TIMESTAMP.xml" ]; then
    echo "Error: Download failed!"
    exit 1
fi

# Check file size (should be > 1MB)
SIZE=$(stat -f%z "$RAW_DIR/stellarhosts_$TIMESTAMP.xml" 2>/dev/null || stat -c%s "$RAW_DIR/stellarhosts_$TIMESTAMP.xml")
if [ $SIZE -lt 1000000 ]; then
    echo "Error: Downloaded file too small, may be corrupted"
    exit 1
fi
```

## Initial Setup

```bash
# Create directories
mkdir -p data/raw data/parquet

# Make scripts executable
chmod +x scripts/*.sh

# Fetch initial data
./scripts/update-data.sh
```

## Future Enhancements

- Verify data integrity (checksums)
- Compare versions to detect changes
- Notification on data updates
- Integration with deployment pipeline
