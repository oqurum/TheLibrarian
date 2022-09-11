use std::{collections::HashMap, ops::{Deref, DerefMut}, fmt::{self, Debug}, borrow::Cow};

use async_trait::async_trait;
use common::{Source, ThumbnailStore, PersonId, BookId, Agent, Either};
use common_local::{SearchFor, MetadataItemCached, api::MetadataBookItem, util::{serialize_naivedate_opt, deserialize_naivedate_opt}};
use chrono::{Utc, NaiveDate};
use serde::{Serialize, Deserialize};
use tokio_postgres::Client;

use crate::{Result, model::{NewPersonModel, PersonAltModel, BookModel, PersonModel}};

use self::{
    google_books::GoogleBooksMetadata,
    openlibrary::OpenLibraryMetadata
};

pub mod google_books;
pub mod openlibrary;


#[async_trait]
pub trait Metadata {
    fn prefix_text<V: AsRef<str>>(&self, value: V) -> String {
        format!("{}:{}", self.get_agent(), value.as_ref())
    }

    fn get_agent(&self) -> Agent;

    // Metadata
    async fn get_metadata_by_source_id(&mut self, value: &str, upgrade_editions: bool, db: &Client) -> Result<Option<MetadataReturned>>;

    // Person

    #[allow(unused_variables)]
    async fn get_person_by_source_id(&mut self, value: &str, db: &Client) -> Result<Option<AuthorMetadata>> {
        Ok(None)
    }


    // Both

    #[allow(unused_variables)]
    async fn search(&mut self, search: &str, search_for: SearchFor, db: &Client) -> Result<Vec<SearchItem>> {
        Ok(Vec::new())
    }
}

/// Doesn't check local
pub async fn get_metadata_by_source(source: &Source, upgrade_editions: bool, db: &Client) -> Result<Option<MetadataReturned>> {
    match &source.agent {
        v if v == &OpenLibraryMetadata.get_agent() => OpenLibraryMetadata.get_metadata_by_source_id(&source.value, upgrade_editions, db).await,
        v if v == &GoogleBooksMetadata.get_agent() => GoogleBooksMetadata.get_metadata_by_source_id(&source.value, upgrade_editions, db).await,

        _ => Ok(None)
    }
}



/// Searches all agents except for local.
pub async fn search_all_agents(search: &str, search_for: SearchFor, db: &Client) -> Result<SearchResults> {
    let mut map = HashMap::new();

    // Checks to see if we can use get_metadata_by_source (source:id)
    if let Ok(source) = Source::try_from(search) {
        // Check if it's a Metadata Source.
        if let Some(val) = get_metadata_by_source(&source, false, db).await? {
            map.insert(
                source.agent.into_owned(),
                vec![SearchItem::Book(val.meta)],
            );

            return Ok(SearchResults(map));
        }
    }

    // Search all sources
    let prefixes = [OpenLibraryMetadata.get_agent(), GoogleBooksMetadata.get_agent()];
    let asdf = futures::future::join_all(
        [OpenLibraryMetadata.search(search, search_for, db), GoogleBooksMetadata.search(search, search_for, db)]
    ).await;

    for (val, prefix) in asdf.into_iter().zip(prefixes) {
        match val {
            Ok(val) => {
                map.insert(
                    prefix.to_string(),
                    val,
                );
            }

            Err(e) => eprintln!("{:?}", e),
        }
    }

    Ok(SearchResults(map))
}

/// Searches all agents except for local.
pub async fn get_person_by_source(source: &Source, db: &Client) -> Result<Option<AuthorMetadata>> {
    match &source.agent {
        v if v == &OpenLibraryMetadata.get_agent() => OpenLibraryMetadata.get_person_by_source_id(&source.value, db).await,
        v if v == &GoogleBooksMetadata.get_agent() => GoogleBooksMetadata.get_person_by_source_id(&source.value, db).await,

        _ => Ok(None)
    }
}



pub struct SearchResults(pub HashMap<String, Vec<SearchItem>>);

impl SearchResults {
    pub fn sort_items_by_similarity(self, match_with: &str) -> Vec<(f64, SearchItem)> {
        let mut items = Vec::new();

        for item in self.0.into_values().flatten() {
            let score = match &item {
                SearchItem::Book(v) => v.title.as_deref().map(|v| strsim::jaro_winkler(match_with, v)).unwrap_or_default(),
                SearchItem::Author(v) => strsim::jaro_winkler(match_with, &v.name),
            };

            items.push((score, item));
        }

        items.sort_unstable_by(|(a, _), (b, _)| b.partial_cmp(a).unwrap());

        items
    }

    pub fn into_search_items(self) -> Vec<SearchItem> {
        self.0.into_values().flatten().collect()
    }
}

impl Deref for SearchResults {
    type Target = HashMap<String, Vec<SearchItem>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SearchResults {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchItem {
    Author(AuthorMetadata),
    Book(BookMetadata)
}

impl SearchItem {
    pub fn into_author(self) -> Option<AuthorMetadata> {
        match self {
            SearchItem::Author(v) => Some(v),
            _ => None,
        }
    }

    pub fn into_book(self) -> Option<BookMetadata> {
        match self {
            SearchItem::Book(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_author(&self) -> Option<&AuthorMetadata> {
        match self {
            SearchItem::Author(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_book(&self) -> Option<&BookMetadata> {
        match self {
            SearchItem::Book(v) => Some(v),
            _ => None,
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorMetadata {
    pub source: Source,

    pub cover_image_url: Option<FoundImageLocation>,

    pub name: String,
    pub other_names: Option<Vec<String>>,
    pub description: Option<String>,

    pub birth_date: Option<String>,
    pub death_date: Option<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataReturned {
    pub authors: Option<Vec<Either<AuthorMetadata, String>>>,
    pub publisher: Option<String>,

    pub meta: BookMetadata
}

impl MetadataReturned {
    /// Returns ((Id, Main Author), Person IDs)
    pub async fn add_or_ignore_authors_into_database(&mut self, client: &Client) -> Result<(Option<PersonModel>, Vec<PersonId>)> {
        let mut main_author = None;
        let mut person_ids = Vec::new();

        if let Some(authors_with_alts) = self.authors.take() {
            for author_or_name in authors_with_alts {
                // Return AuthorMetadata by either Fetching DB person, found person or, search for person by name
                let author_info = match author_or_name {
                    Either::Left(value) => value,

                    Either::Right(author_name) => {
                        // Check if we already have a person by that name anywhere in the two database tables.
                        if let Some(person) = PersonModel::find_one_by_name(author_name.trim(), client).await? {
                            person_ids.push(person.id);

                            if main_author.is_none() {
                                main_author = Some(person);
                            }

                            continue;
                        }

                        let search = search_all_agents(&author_name, SearchFor::Person, client).await
                            .map(|v| v.into_search_items())
                            .unwrap_or_default(); // I want to ignore errors and just continue.

                        if let Some(meta) = search.first().and_then(|v| v.as_author()).cloned() {
                            meta
                        } else {
                            continue;
                        }
                    }
                };

                // Check if we already have a person by that name anywhere in the two database tables.
                // TODO: Better way. There can be multiple people with the same name.
                if let Some(person) = PersonModel::find_one_by_name(author_info.name.trim(), client).await? {
                    person_ids.push(person.id);

                    if main_author.is_none() {
                        main_author = Some(person);
                    }

                    continue;
                }

                // Check by source.
                if let Some(person) = PersonModel::get_by_source(&author_info.source.to_string(), client).await? {
                    person_ids.push(person.id);

                    if main_author.is_none() {
                        main_author = Some(person);
                    }

                    continue;
                }


                let mut thumb_url = ThumbnailStore::None;

                // Download thumb url and store it.
                if let Some(mut url) = author_info.cover_image_url {
                    match url.download(client).await {
                        Ok(_) => {
                            if let FoundImageLocation::Local(value) = url {
                                thumb_url = value;
                            }
                        }

                        Err(e) => {
                            eprintln!("add_or_ignore_authors_into_database Error: {}", e);
                        }
                    }
                }

                let new_person = NewPersonModel {
                    source: author_info.source,
                    name: author_info.name,
                    description: author_info.description,
                    birth_date: None,
                    thumb_url,
                    // TODO: death_date: author_info.death_date,
                    updated_at: Utc::now(),
                    created_at: Utc::now(),
                };

                let person = new_person.insert(client).await?;

                if let Some(alts) = author_info.other_names {
                    for name in alts {
                        // Ignore errors. Errors should just be UNIQUE constraint failed
                        if let Err(e) = (PersonAltModel { person_id: person.id, name, }).insert(client).await {
                            eprintln!("[OL]: Add Alt Name Error: {e}");
                        }
                    }
                }

                person_ids.push(person.id);

                if main_author.is_none() {
                    main_author = Some(person);
                }
            }
        }

        if let Some(person) = main_author.as_ref() {
            self.meta.cached.author_id = Some(person.id);
        }

        Ok((main_author, person_ids))
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookMetadata {
    pub source: Source,
    pub title: Option<String>,
    pub description: Option<String>,
    pub rating: f64,

    pub thumb_locations: Vec<FoundImageLocation>,

    // TODO: Make table for all tags. Include publisher in it. Remove country.
    pub cached: MetadataItemCached,

    pub isbn_10: Option<String>,
    pub isbn_13: Option<String>,

    #[serde(serialize_with = "serialize_naivedate_opt", deserialize_with = "deserialize_naivedate_opt")]
    pub available_at: Option<NaiveDate>,
    pub language: Option<u16>
}

impl From<BookMetadata> for BookModel {
    fn from(val: BookMetadata) -> Self {
        BookModel {
            id: BookId::none(),
            title: val.title.clone(),
            clean_title: val.title,
            description: val.description,
            rating: val.rating,
            thumb_path: val.thumb_locations.iter()
                .find_map(|v| v.as_local_value().cloned())
                .unwrap_or(ThumbnailStore::None),
            cached: val.cached,
            isbn_10: val.isbn_10,
            isbn_13: val.isbn_13,
            is_public: false,
            edition_count: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            available_at: val.available_at,
            language: val.language,
        }
    }
}

impl From<BookMetadata> for MetadataBookItem {
    fn from(val: BookMetadata) -> Self {
        MetadataBookItem {
            source: val.source,
            title: val.title,
            description: val.description,
            rating: val.rating,
            thumbnails: val.thumb_locations.into_iter().map(|v| v.as_api_path().into_owned()).collect(),
            cached: val.cached,
            isbn_10: val.isbn_10,
            isbn_13: val.isbn_13,
            available_at: val.available_at,
            language: val.language,
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FoundImageLocation {
    Url(String),
    Local(ThumbnailStore),
}

impl FoundImageLocation {
    pub fn as_api_path(&self) -> Cow<'_, str> {
        match self {
            Self::Url(v) => Cow::Borrowed(v.as_str()),
            Self::Local(v) => Cow::Owned(format!("/api/v1/image/{}", v.as_value().unwrap())),
        }
    }

    pub fn into_url_value(self) -> Option<String> {
        match self {
            Self::Url(v) => Some(v),
            _ => None
        }
    }

    pub fn as_url_value(&self) -> Option<&str> {
        match self {
            Self::Url(v) => Some(v.as_str()),
            _ => None
        }
    }

    pub fn as_local_value(&self) -> Option<&ThumbnailStore> {
        match self {
            Self::Local(v) => Some(v),
            _ => None
        }
    }

    pub fn is_local(&self) -> bool {
        matches!(self, Self::Local(_))
    }

    pub fn is_url(&self) -> bool {
        matches!(self, Self::Url(_))
    }

    pub async fn download(&mut self, db: &Client) -> Result<()> {
        if let FoundImageLocation::Url(ref url) = self {
            let resp = reqwest::get(url)
                .await?
                .bytes()
                .await?;

            if resp.len() > 1300 {
                let model = crate::store_image(resp.to_vec(), db).await?;

                *self = Self::Local(model.path);
            }
        }

        Ok(())
    }
}

impl fmt::Display for FoundImageLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.as_api_path(), f)
    }
}