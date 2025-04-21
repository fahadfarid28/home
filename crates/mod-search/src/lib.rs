include!(".dylo/spec.rs");
include!(".dylo/support.rs");

use config::WebConfig;
#[cfg(feature = "impl")]
use noteyre::BsForResults;

use conflux::RevisionView;
use conflux::{Completion, InputPath, LoadedPage, SearchResults, Viewer};
#[cfg(feature = "impl")]
use conflux::{CompletionKind, Html, SearchResult};

#[cfg(feature = "impl")]
use tantivy::{
    collector::{Count, TopDocs},
    schema::{
        IndexRecordOption, Schema, TextFieldIndexing, TextOptions, Value, INDEXED, STORED, TEXT,
    },
    SnippetGenerator, TantivyDocument,
};

pub type Result<T, E = noteyre::BS> = std::result::Result<T, E>;

#[cfg(feature = "impl")]
#[derive(Default)]
struct ModImpl;

#[dylo::export]
impl Mod for ModImpl {
    fn indexer(&self) -> Box<dyn Indexer> {
        let mut schema_builder = Schema::builder();

        let text_options = TextOptions::default().set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer("en_stem")
                .set_index_option(IndexRecordOption::WithFreqsAndPositions),
        );

        schema_builder.add_text_field("path", TEXT | STORED);
        schema_builder.add_bool_field("draft", INDEXED);
        schema_builder.add_bool_field("dual_feature", INDEXED);
        schema_builder.add_text_field("title", text_options.clone());
        schema_builder.add_text_field("body", text_options);
        let schema = schema_builder.build();

        let index = tantivy::Index::create_in_ram(schema.clone());
        let index_writer = index.writer(100_000_000).unwrap();
        let isi = indicium::simple::SearchIndexBuilder::default()
            .max_string_len(Some(0))
            .build();

        Box::new(IndexerImpl {
            isi,
            index,
            index_writer,
            schema,
        })
    }
}

#[cfg(feature = "impl")]
struct IndexerImpl {
    isi: indicium::simple::SearchIndex<InputPath>,
    schema: Schema,
    index: tantivy::Index,
    index_writer: tantivy::IndexWriter<TantivyDocument>,
}

// TODO: fallible ops

#[dylo::export]
impl Indexer for IndexerImpl {
    fn insert(&mut self, key: InputPath, page: &LoadedPage) {
        self.isi.insert(
            &key,
            &IndexableCompat(vec![page.title.clone(), page.plain_text.clone()]),
        );

        let mut doc = TantivyDocument::default();
        let path = self.schema.get_field("path").unwrap();
        let draft = self.schema.get_field("draft").unwrap();
        let dual_feature = self.schema.get_field("dual_feature").unwrap();
        let title = self.schema.get_field("title").unwrap();
        let body = self.schema.get_field("body").unwrap();

        doc.add_text(path, key.as_str());
        doc.add_bool(draft, page.draft);
        doc.add_bool(dual_feature, page.video_info.dual_feature);
        doc.add_text(title, &page.title);
        doc.add_text(body, &page.plain_text);

        self.index_writer.add_document(doc).unwrap();
    }

    fn commit(self: Box<Self>) -> Box<dyn Index> {
        let isi = self.isi;
        let index = self.index;
        let mut index_writer = self.index_writer;
        let schema = self.schema;

        index_writer.commit().unwrap();
        let index_reader = index
            .reader_builder()
            .reload_policy(tantivy::ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .unwrap();

        Box::new(IndexImpl {
            isi,
            schema,
            index,
            index_reader,
        })
    }
}

#[cfg(feature = "impl")]
struct IndexImpl {
    isi: indicium::simple::SearchIndex<InputPath>,
    schema: Schema,
    index: tantivy::Index,
    index_reader: tantivy::IndexReader,
}

#[cfg(feature = "impl")]
impl IndexImpl {
    fn search_inner(
        &self,
        rv: &dyn RevisionView,
        viewer: &Viewer,
        query: &str,
        per_page: usize,
        page_number: usize,
    ) -> Result<SearchResults> {
        let query: String = if viewer.is_admin {
            query.to_string()
        } else {
            // TODO: also exclude futures, but this requires looking into the syntax of
            // tantivy's query language re: dates (maybe just unix timestamps?)
            format!("{query} AND draft:false")
        };

        let searcher = self.index_reader.searcher();
        let mut query_parser = tantivy::query::QueryParser::for_index(
            &self.index,
            vec![
                self.schema.get_field("title").unwrap(),
                self.schema.get_field("body").unwrap(),
                self.schema.get_field("draft").unwrap(),
            ],
        );

        let title = self.schema.get_field("title").unwrap();
        let body = self.schema.get_field("body").unwrap();
        query_parser.set_field_boost(title, 3.0);

        tracing::debug!("query = {query}");

        let (query, _errs) = query_parser.parse_query_lenient(&query);

        let page = page_number.saturating_sub(1);
        let offset = page * per_page;

        let num_results = searcher.search(&query, &Count).bs()?;
        let mut results = SearchResults {
            results: Default::default(),
            terms: Default::default(),
            num_results,
            // This is correct because:
            // 1. `page` is zero-indexed (page_number - 1)
            // 2. `(page + 1) * per_page` gives the number of results up to and including the current page
            // 3. If num_results is greater than this, it means there are more results on the next page
            has_more: num_results > (page + 1) * per_page,
        };

        query.query_terms(&mut |term, _positions_required| {
            if let Some(s) = term.value().as_str() {
                tracing::debug!("found term: {s}");
                results.terms.push(s.to_string());
            }
        });

        tracing::debug!(
            "page = {page}, offset = {offset}, per_page = {per_page}, num_results = {num_results}"
        );

        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(per_page).and_offset(offset))
            .bs()?;

        tracing::debug!("num top docs = {}", top_docs.len());

        let mut title_snippet_generator =
            SnippetGenerator::create(&searcher, &*query, title).bs()?;
        title_snippet_generator.set_max_num_chars(150);

        let mut body_snippet_generator = SnippetGenerator::create(&searcher, &*query, body).bs()?;
        body_snippet_generator.set_max_num_chars(350);

        let path = self.schema.get_field("path").unwrap();
        let rev = rv.rev().bs()?;

        for (_score, doc_address) in top_docs {
            let doc: TantivyDocument = searcher.doc(doc_address).bs()?;

            let doc_path = doc.get_first(path).unwrap();
            let doc_path = InputPath::new(doc_path.as_str().unwrap().to_owned());
            let page = rev.pages.get(&doc_path).unwrap().clone();

            let title_snippet = title_snippet_generator.snippet(&page.title).to_html();
            let body_snippet = body_snippet_generator.snippet(&page.plain_text).to_html();

            tracing::debug!("title snippet = {title_snippet}");
            tracing::debug!("body snippet = {body_snippet}");

            results.results.push(SearchResult {
                body_snippet: Html::new(body_snippet),
                title_snippet: Html::new(title_snippet),
                page,
            });
        }

        Ok(results)
    }

    fn autocomplete_inner(
        &self,
        rv: &dyn RevisionView,
        viewer: &Viewer,
        query_str: &str,
        web: WebConfig,
    ) -> eyre::Result<Vec<Completion>> {
        let mut results: Vec<Completion> = Default::default();

        // use tantivy look for articles with titles that strongly match
        {
            let title = self.schema.get_field("title").unwrap();
            let searcher = self.index_reader.searcher();
            let query_parser = tantivy::query::QueryParser::for_index(
                &self.index,
                vec![self.schema.get_field("title").unwrap()],
            );
            let query_str = if viewer.is_admin {
                query_str.to_string()
            } else {
                // TODO: also exclude futures, but this requires looking into the syntax of
                // tantivy's query language re: dates (maybe just unix timestamps?)
                //
                // Also, deduplicate from search_inner
                format!("{query_str} AND draft:false")
            };

            let (query, errs) = query_parser.parse_query_lenient(&query_str);
            for err in errs {
                tracing::warn!("query error: {err}");
            }

            let mut title_snippet_generator = SnippetGenerator::create(&searcher, &*query, title)?;
            title_snippet_generator.set_max_num_chars(150);

            let rev = rv.rev()?;
            let top_docs = searcher.search(&query, &TopDocs::with_limit(3))?;

            for (score, doc_address) in top_docs {
                let doc: TantivyDocument = searcher.doc(doc_address)?;
                let doc_path = doc
                    .get_first(self.schema.get_field("path").unwrap())
                    .unwrap();
                let doc_path = InputPath::new(doc_path.as_str().unwrap().to_owned());
                tracing::debug!("score = {score}, doc_path = {doc_path}");

                if score >= 1.5 {
                    let page = rev.pages.get(&doc_path).unwrap().clone();

                    let title_snippet = title_snippet_generator.snippet(&page.title).to_html();
                    tracing::debug!("title snippet = {title_snippet}");

                    results.push(Completion {
                        kind: CompletionKind::Article,
                        text: page.title.clone(),
                        html: Html::new(title_snippet),
                        url: Some(page.canonical_url(web).into()),
                    });
                }
            }
        }

        // use indicium to find terms
        {
            for term in self.isi.autocomplete(query_str) {
                results.push(Completion {
                    kind: CompletionKind::Term,
                    text: term.clone(),
                    html: Html::new(html_escape::encode_text(&term).into()),
                    url: None,
                });
            }
        }

        Ok(results)
    }
}

#[dylo::export]
impl Index for IndexImpl {
    fn autocomplete(
        &self,
        rv: &dyn RevisionView,
        viewer: &Viewer,
        query: &str,
        web: WebConfig,
    ) -> Vec<Completion> {
        match self.autocomplete_inner(rv, viewer, query, web) {
            Ok(results) => results,
            Err(e) => {
                tracing::warn!("Failed to autocomplete: {e}");
                Default::default()
            }
        }
    }

    fn search(
        &self,
        rv: &dyn RevisionView,
        viewer: &Viewer,
        query: &str,
        per_page: usize,
        page_number: usize,
    ) -> SearchResults {
        match self.search_inner(rv, viewer, query, per_page, page_number) {
            Ok(results) => results,
            Err(e) => {
                tracing::warn!("Failed to search index: {e}");
                Default::default()
            }
        }
    }
}

#[cfg(feature = "impl")]
struct IndexableCompat(Vec<String>);

#[cfg(feature = "impl")]
impl indicium::simple::Indexable for IndexableCompat {
    fn strings(&self) -> Vec<String> {
        self.0.clone()
    }
}
