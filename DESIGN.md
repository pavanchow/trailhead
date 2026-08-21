# Design

## Tokenization (`src/tokenizer.rs`)

Input text is lowercased, then split on any byte that is not alphanumeric. Empty pieces are dropped. An optional stopword filter removes a small fixed list of common English function words (`the`, `is`, `and`, and so on). Stopwords are filtered when indexing documents and when parsing a query, so both sides of the match use the same vocabulary.

Tokenization is intentionally simple: no stemming, no unicode normalization beyond `to_lowercase`, no n-grams. The goal is a tokenizer whose behavior you can predict by reading it once.

## The inverted index (`src/index.rs`)

The index is two things:

- `docs: Vec<DocInfo>`, one entry per indexed document, holding its path and total token count.
- `postings: HashMap<String, Vec<Posting>>`, for each term, the list of `(doc_id, term_freq)` pairs for every document that contains it.

Building the index means, for each document: tokenize it, count occurrences of each term, and append a `Posting` to that term's list. Doc IDs are just indices into `docs`, assigned in the order files are read (directory listing is sorted first, so indexing is deterministic).

This is a classic inverted index: instead of scanning every document for a query term, you look the term up once and get back exactly the documents that contain it.

## TF-IDF math

For term `t` and document `d`:

- `tf(t, d) = term_freq(t, d) / token_count(d)`, the raw count normalized by document length, so longer documents do not win purely by being longer.
- `idf(t) = ln(N / df(t))`, where `N` is the total number of documents and `df(t)` is the number of documents containing `t`. Terms in every document approach `idf = 0`. Terms in one document out of many get a large `idf`.
- `tfidf(t, d) = tf(t, d) * idf(t)`

A query is itself tokenized (with stopwords dropped) into a list of terms. A document's score for the query is the sum of `tfidf(t, d)` over every query term `t`. Documents with a total score of zero (no matching terms) are excluded from results. The rest are sorted descending by score and truncated to the requested top N.

## Persistence

The `Index` struct derives `serde::Serialize`/`Deserialize` and is written with `serde_json::to_string_pretty`. It is a plain JSON file, `trailhead.index.json` by default, readable with any text editor. Loading is the inverse: read the file, `serde_json::from_str` back into an `Index`.

This is not a space-efficient format. It trades index size for the ability to open the file and see exactly what got indexed, which matters more for a project meant to be read end to end than one meant to scale to millions of documents.

## Query evaluation

1. Tokenize the query string the same way documents are tokenized (lowercase, split, drop stopwords).
2. For each query term, look up its postings list. Terms with no postings contribute nothing.
3. For each posting, compute that term's TF-IDF for that document and add it to a running per-document score.
4. Filter out documents with a score of zero, sort the rest by score descending, and return the top N as `(path, score)` pairs.

This is a straightforward disjunctive (OR) evaluation over query terms with additive TF-IDF scoring, not a boolean AND/OR query language. A document containing just one of several query terms can still appear in results, just with a lower score than a document containing all of them.
