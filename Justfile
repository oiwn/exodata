download-stellarhosts:
    curl "https://exoplanetarchive.ipac.caltech.edu/TAP/sync?query=select+*+from+stellarhosts" -L --max-time 2000 > data/stellarhosts.vot

download-exoplanets:
    curl "https://exoplanetarchive.ipac.caltech.edu/TAP/sync?query=select+*+from+ps" -L --max-time 2000 > data/exoplanets.vot
