# BG-Wiki Reference Data Scraper

A standalone CLI that pulls AF3, Empyrean, and Relic armor set data from BG-Wiki
and writes it to JSON, for seeding the app's gear set reference tables.

## Usage

```
node tools/scraper/discover.ts <af3|empyrean|relic> [--force]
node tools/scraper/extract.ts <af3|empyrean|relic> [--force]
```

`discover.ts` crawls one BG-Wiki category and writes a manifest to
`out/discovered-<category>.json` for review. `extract.ts` reads that manifest
and writes the final `out/gearsets-<category>.json`.

Add `--force` to bypass the local cache in `.cache/` and re-fetch from BG-Wiki.

## Testing

```
npm run test:scraper
```

## Data source

Structural data (job, slot, tier, item name, item ID) is sourced from
bg-wiki.com, the community wiki for Final Fantasy XI. Requests are throttled
and identified with a real User-Agent as a courtesy to the wiki. No wiki prose,
notes, or images are captured, only the structural fields needed to build the
gear set reference table.
