<p align="center">
  <img src="assets/logo.png" alt="Spoons" width="128">
</p>

# Spoons

**One GraphQL API over the free and open catalogues of music, podcasts, and audiobooks.**

The open media archives are excellent and completely fragmented. MusicBrainz has the canonical music metadata. Audius and Jamendo have freely licensed audio you can actually stream. PodcastIndex is an open index of the entire podcast ecosystem. LibriVox has thousands of public-domain audiobooks. Five projects, five REST shapes, five auth schemes, five vocabularies for the same handful of ideas.

Spoons puts them behind one schema. Ask for an artist, an episode, or an audiobook — or search all three domains at once — and get back normalized types with stream links already resolved. Nothing downstream needs to know which archive answered.

All five sources are free to use. Three need no credentials at all.

## Sources

| Source | Domain | Credentials |
|---|---|---|
| [MusicBrainz](https://musicbrainz.org/) | Music metadata | None |
| [Audius](https://audius.org/) | Streaming music | None |
| [LibriVox](https://librivox.org/) | Public-domain audiobooks | None |
| [Jamendo](https://devportal.jamendo.com/) | Freely licensed music | Free client ID |
| [PodcastIndex](https://podcastindex.org/) | Open podcast index | Free key + secret |

Each source can be disabled independently in `config.yaml`. Run it with only the no-auth sources and it still works — you just get fewer results.

## What you can ask it

- **Cross-domain search** — `search` queries music, podcasts, and audiobooks in parallel and merges the results. Filter with the `ContentDomain` enum.
- **Cross-domain random** — `random` for discovery without a query. Random artist, random track, random episode, random audiobook.
- **Per-source or fan-out** — target one archive, or let a query hit all of them.
- **Streaming links** — resolved per source behind a normalized type.
- **Trending and categories** — for podcasts, via PodcastIndex.

See [API.md](API.md) for the full schema, query examples, and field reference.

## Quick start

Requires Rust 1.70+ and PostgreSQL 14+.

```bash
git clone https://github.com/pocketcereal/spoons-api
cd spoons-api
createdb spoons
diesel migration run

cp .env.example .env      # optional: add PodcastIndex / Jamendo keys
task dev
```

The API comes up at `http://localhost:4000/graphql`. With `SPOONS_AUTH_DISABLED=true` there's a GraphiQL playground at `/graphiql` for poking at the schema.

## Architecture

### Source provider pattern

Each domain has a provider trait — `MusicProvider`, `PodcastProvider`, `AudiobookProvider`. Sources implement their domain trait and register in the composition root (`server.rs`). Resolvers dispatch through `fan_out_search` and never learn about specific sources.

```
MusicProvider
├── MusicBrainzProvider  (cached)
├── AudiusProvider
└── JamendoProvider

PodcastProvider
└── PodcastIndexProvider (cached)

AudiobookProvider
└── LibriVoxProvider     (cached)
```

Adding an archive means implementing the trait and adding one line to `server.rs`. That's the whole extension story — if a source you want isn't here, it's a small contribution.

### Caching

MusicBrainz, PodcastIndex, and LibriVox responses are cached in PostgreSQL with configurable TTLs. Audius and Jamendo are not DB-cached. Cache writes are fire-and-forget, so a cache failure never fails a request. An in-memory LRU sits in front for hot queries.

Caching is a courtesy to the upstream archives as much as a latency win — these are free services run by nonprofits and volunteers. Don't hammer them.

## Development

```bash
task check          # lint + unit tests
task dev            # run dev server
task test:auth      # smoke tests against a running server
```

## Deploying

`manifests/` holds plain Kubernetes YAML and `terraform/` provisions a single-VM deployment on GCP. Both describe the setup this project happens to run on rather than a recommendation — take the pieces you want. Every credential in them is a placeholder; fill them from your own environment.

## Contributing

New sources are the most useful contribution. The bar is that a source must be free to query and openly licensed — this is an aggregator of open catalogues, and adding something that needs a paid key or restricts redistribution defeats the point.

## License

MIT. See [LICENSE](LICENSE).
