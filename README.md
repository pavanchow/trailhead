<img src="docs/logo.svg" alt="Trailhead logo" width="96">

# Trailhead: a full-text search engine in Rust

Trailhead is a full-text search engine written from scratch in Rust, with an inverted index and TF-IDF ranking you can read end to end. It indexes a folder of `.txt` and `.md` files and lets you query it from the command line, with no external search service and no black box. The whole pipeline, from tokenizing text to ranking results, lives in a few hundred lines, so it doubles as a readable reference for how full-text search and TF-IDF ranking work.

**[Live demo](https://pavanchow.github.io/trailhead/)** · MIT licensed · written in Rust

Built from scratch by [Pavan Nallamothu](https://pavanchow.github.io/) ([LinkedIn](https://www.linkedin.com/in/pavanchow/), [GitHub](https://github.com/pavanchow)).

## What it is

- A tokenizer that lowercases text and splits it into words, with an optional stopword filter.
- An inverted index: a map from each term to the list of documents it appears in, along with how many times it appears in each one.
- A TF-IDF scorer that ranks documents for a query by combining term frequency and inverse document frequency.
- A CLI, built with `clap`, to build and query the index.

## How the ranking works

Each term in each document gets a TF-IDF score:

- **Term frequency (TF)**: how often the term shows up in a document, normalized by that document's length. A term that appears 3 times in a 30-word file scores higher on TF than the same term appearing 3 times in a 3,000-word file.
- **Inverse document frequency (IDF)**: `ln(N / df)`, where `N` is the total number of documents and `df` is how many of them contain the term. Common terms that show up everywhere get a low IDF. Rare terms get a high one.
- **Score**: TF times IDF, summed across every term in the query. A document that matches more query terms, or matches them more often relative to its length, ranks higher.

## Usage

Build the index over a directory:

```
trailhead index ./my-notes
```

This writes `trailhead.index.json` in the current directory.

Search it:

```
trailhead search "inverted index ranking" --top 5
```

Each result line shows the document path and its TF-IDF score, highest first.

## Persistence

The index is serialized to JSON with `serde_json`. It is not the most compact format, but it stays readable and makes debugging the index trivial.

## Tests

`cargo test` covers the tokenizer and the index: exact-match queries, TF-IDF ranking a document with more occurrences of a rarer term above one with fewer, absent terms returning no results, multi-term queries combining scores, and a save/load round trip.

## License

MIT licensed. By Pavan Nallamothu.
