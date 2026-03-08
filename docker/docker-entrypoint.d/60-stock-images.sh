#!/usr/bin/env sh

if [ ! -z "$SPIS_MEDIA_FETCH_STOCK" ]; then
    SPIS_MEDIA_STOCK_DIR="$SPIS_MEDIA_DIR/stock"

    EXISTING=$(ls "$SPIS_MEDIA_STOCK_DIR" 2>/dev/null | wc -l)
    if [ "$EXISTING" -lt "$SPIS_MEDIA_FETCH_STOCK" ]; then
        echo "Starting to fetch $SPIS_MEDIA_FETCH_STOCK stock images"

        mkdir -p "$SPIS_MEDIA_STOCK_DIR"

        for i in $(seq $SPIS_MEDIA_FETCH_STOCK); do
            curl -s -L -o "$SPIS_MEDIA_STOCK_DIR/${i}.jpg" "https://picsum.photos/seed/${i}/800/600"
        done

        echo "Done fetching $SPIS_MEDIA_FETCH_STOCK stock images"
    fi
fi
