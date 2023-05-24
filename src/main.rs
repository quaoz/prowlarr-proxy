use std::collections::HashMap;
use std::io::Cursor;
use std::thread::sleep;
use std::time::Duration;
use reqwest;
use reqwest::Url;
use select::document::Document;
use select::predicate::Class;
use serde::{Serialize, Deserialize};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use quick_xml::{Reader, Writer};
use quick_xml::events::{BytesEnd, BytesStart, Event};
use warp::Filter;
use warp::http::Response;

#[derive(Default)]
enum ContentType {
	#[default]
	Any,
	JournalArticle,
	BookAny,
	BookUnknown,
	BookFiction,
	BookNonFiction,
	ComicBook,
	Magazine,
	StandardsDocument,
}

impl ContentType {
	fn as_str(&self) -> &'static str {
		match self {
			ContentType::Any => "",
			ContentType::JournalArticle => "journal_article",
			ContentType::BookAny => "book_any",
			ContentType::BookUnknown => "book_unknown",
			ContentType::BookFiction => "book_fiction",
			ContentType::BookNonFiction => "book_nonfiction",
			ContentType::ComicBook => "book_comic",
			ContentType::Magazine => "magazine",
			ContentType::StandardsDocument => "standards_document",
		}
	}
}

#[derive(Default)]
enum SortType {
	#[default]
	MostRelevant,
	Newest,
	Oldest,
	Largest,
	Smallest,
}

impl SortType {
	fn as_str(&self) -> &'static str {
		match self {
			SortType::MostRelevant => "",
			SortType::Newest => "newest",
			SortType::Oldest => "oldest",
			SortType::Largest => "largest",
			SortType::Smallest => "smallest",
		}
	}
}

#[derive(Default)]
enum FileType {
	#[default]
	ANY,
	PDF,
	EPUB,
	CBR,
	FB2,
	MOBI,
	CBZ,
	DJVU,
	AZW3,
	FB2ZIP,
	TXT,
	RAR,
	ZIP,
	DOC,
	LIT,
	RTF,
	HTM,
	HTML,
	LRF,
	MHT,
	DOCX,
}

impl FileType {
	fn as_str(&self) -> &'static str {
		match self {
			FileType::ANY => "",
			FileType::PDF => "pdf",
			FileType::EPUB => "epub",
			FileType::CBR => "cbr",
			FileType::FB2 => "fb2",
			FileType::MOBI => "mobi",
			FileType::CBZ => "cbz",
			FileType::DJVU => "djvu",
			FileType::AZW3 => "azw3",
			FileType::FB2ZIP => "fb2.zip",
			FileType::TXT => "txt",
			FileType::RAR => "rar",
			FileType::ZIP => "zip",
			FileType::DOC => "doc",
			FileType::LIT => "lit",
			FileType::RTF => "rtf",
			FileType::HTM => "htm",
			FileType::HTML => "html",
			FileType::LRF => "lrf",
			FileType::MHT => "mht",
			FileType::DOCX => "docx",
		}
	}
}

#[tokio::main]
async fn main() {
	let caps = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<caps>
   <server version=\"1.0\" title=\"Torznab proxy\" strapline=\"\"
         email=\"\" url=\"http://127.0.0.1:3030/\"
         image=\"\" />

   <limits max=\"100\" default=\"50\" />

   <registration available=\"no\" open=\"no\" />

   <searching>
      <search available=\"yes\" supportedParams=\"q\" />
      <tv-search available=\"yes\" supportedParams=\"q,rid,tvdbid,season,ep\" />
      <movie-search available=\"yes\" supportedParams=\"q,imdbid\" />
      <audio-search available=\"yes\" supportedParams=\"q\" />
      <book-search available=\"yes\" supportedParams=\"q\" />
   </searching>

   <categories>
      <category id=\"0\" name=\"Other\">
         <subcat id=\"10\" name=\"Misc\" />
         <subcat id=\"20\" name=\"Hashed\" />
      </category>
      <category id=\"1000\" name=\"Console\">
         <subcat id=\"1010\" name=\"NDS\" />
         <subcat id=\"1020\" name=\"PSP\" />
         <subcat id=\"1030\" name=\"Wii\" />
         <subcat id=\"1040\" name=\"XBox\" />
         <subcat id=\"1050\" name=\"XBox 360\" />
         <subcat id=\"1060\" name=\"Wiiware\" />
         <subcat id=\"1070\" name=\"XBox 360 DLC\" />
         <subcat id=\"1070\" name=\"PS3\" />
         <subcat id=\"1090\" name=\"Other\" />
         <subcat id=\"1110\" name=\"3DS\" />
         <subcat id=\"1120\" name=\"PS Vita\" />
         <subcat id=\"1130\" name=\"WiiU\" />
         <subcat id=\"1140\" name=\"XBox One\" />
         <subcat id=\"1180\" name=\"PS4\" />
      </category>
      <category id=\"2000\" name=\"Movies\">
         <subcat id=\"2010\" name=\"Foreign\" />
         <subcat id=\"2020\" name=\"Other\" />
         <subcat id=\"2030\" name=\"SD\" />
         <subcat id=\"2040\" name=\"HD\" />
         <subcat id=\"2045\" name=\"UHD\" />
         <subcat id=\"2050\" name=\"BluRay\" />
         <subcat id=\"2060\" name=\"3D\" />
         <subcat id=\"2070\" name=\"DVD\" />
         <subcat id=\"2080\" name=\"WEB-DL\" />
      </category>
      <category id=\"3000\" name=\"Audio\">
         <subcat id=\"3010\" name=\"MP3\" />
         <subcat id=\"3020\" name=\"Video\" />
         <subcat id=\"3030\" name=\"Audiobook\" />
         <subcat id=\"3040\" name=\"Lossless\" />
         <subcat id=\"3050\" name=\"Other\" />
         <subcat id=\"3060\" name=\"Foreign\" />
      </category>
      <category id=\"4000\" name=\"PC\">
         <subcat id=\"4010\" name=\"0day\" />
         <subcat id=\"4020\" name=\"ISO\" />
         <subcat id=\"4030\" name=\"Mac\" />
         <subcat id=\"4040\" name=\"Mobile-Other\" />
         <subcat id=\"4050\" name=\"Games\" />
         <subcat id=\"4060\" name=\"Mobile-iOS\" />
         <subcat id=\"4070\" name=\"Mobile-Android\" />
      </category>
      <category id=\"5000\" name=\"TV\">
         <subcat id=\"5010\" name=\"WEB-DL\" />
         <subcat id=\"5020\" name=\"Foreign\" />
         <subcat id=\"5030\" name=\"SD\" />
         <subcat id=\"5040\" name=\"HD\" />
         <subcat id=\"5045\" name=\"UHD\" />
         <subcat id=\"5050\" name=\"Other\" />
         <subcat id=\"5060\" name=\"Sport\" />
         <subcat id=\"5070\" name=\"Anime\" />
         <subcat id=\"5080\" name=\"Documentary\" />
      </category>
      <category id=\"6000\" name=\"XXX\">
         <subcat id=\"6010\" name=\"DVD\" />
         <subcat id=\"6020\" name=\"WMV\" />
         <subcat id=\"6030\" name=\"XviD\" />
         <subcat id=\"6040\" name=\"x264\" />
         <subcat id=\"6045\" name=\"UHD\" />
         <subcat id=\"6050\" name=\"Pack\" />
         <subcat id=\"6060\" name=\"ImageSet\" />
         <subcat id=\"6070\" name=\"Other\" />
         <subcat id=\"6080\" name=\"SD\" />
         <subcat id=\"6090\" name=\"WEB-DL\" />
      </category>
      <category id=\"7000\" name=\"Books\">
         <subcat id=\"7010\" name=\"Mags\" />
         <subcat id=\"7020\" name=\"EBook\" />
         <subcat id=\"7030\" name=\"Comics\" />
         <subcat id=\"7040\" name=\"Technical\" />
         <subcat id=\"7050\" name=\"Other\" />
         <subcat id=\"7060\" name=\"Foreign\" />
      </category>
      <category id=\"8000\" name=\"Other\">
         <subcat id=\"8010\" name=\"Misc\" />
         <subcat id=\"8010\" name=\"Hashed\" />
      </category>
   </categories>
</caps>";

	let route = warp::get()
			.and(warp::path("api"))
			.and(warp::query::<HashMap<String, String>>())
			.map(|params: HashMap<String, String>| {
				let function = params.get("t");
				let mut response = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<error code=\"200\" description=\"Missing parameter (t)\" />";

				println!("{:?}", params);
				if function.is_some() {
					if function.unwrap() == "caps" {
						response = caps;
					} else if function.unwrap() == "search" {

					} else if function.unwrap() == "tvsearch" {

					} else if function.unwrap() == "movie" {

					} else if function.unwrap() == "music" {

					} else if function.unwrap() == "book" {

					}
				}
				Response::builder().header("Content-Type", "text/xml").body(String::from(response))
			});

	warp::serve(route).run(([127, 0, 0, 1], 3030)).await;

	search_book(
		String::from(""),
		ContentType::default(),
		FileType::default(),
		SortType::default(),
		String::from("dune frank herbert"),
	)
			.await;
}

async fn search_book(
	lang: String,
	content: ContentType,
	filetype: FileType,
	sort: SortType,
	query: String,
) {
	let base_url = "annas-archive.org";
	let content_string = content.as_str();
	let filetype_string = filetype.as_str();
	let sort_string = sort.as_str();

	let search_url = Url::parse(&*format!(
		"https://{}/search?lang={}&content={}&ext={}&sort={}&q={}",
		base_url, lang, content_string, filetype_string, sort_string, query
	))
			.unwrap();

	let response = reqwest::get(search_url)
			.await
			.unwrap()
			.text()
			.await
			.unwrap();
	let document = Document::from(response.as_str());

	let mut book_urls: Vec<Url> = vec![];

	for book in document.find(Class("h-[125]")) {
		let link = book.find(Class("custom-a")).next();

		if link.is_some() {
			let book_url = Url::parse(&*format!(
				"https://{}/{}?tech_details=y",
				base_url,
				link.unwrap().attr("href").unwrap()
			))
					.unwrap();

			book_urls.push(book_url)
		} else {
			// break when reaching the partial matches
			break;
		}
	}

	let mut book_results: Vec<Book> = Vec::new();
	let mut futures = FuturesUnordered::new();

	for book_url in book_urls {
		let future = get_book(book_url);
		futures.push(future);
	}

	while let Some(book) = futures.next().await {
		book_results.push(book);
	}

	for book in book_results {
		println!("{}", book.md5)
	}
}

async fn get_book(book_url: Url) -> Book {
	let response = reqwest::get(book_url)
			.await
			.unwrap()
			.text()
			.await
			.unwrap();
	let document = Document::from(response.as_str());

	let info = document.find(Class("text-xs")).next().unwrap().text().replace(" ", " ");
	let book: Book = serde_json::from_str(&*info).unwrap();

	book
}

#[derive(Serialize, Deserialize)]
struct Book {
	md5: String,
	lgrsnf_book: Option<LibGenRSBook>,
	lgrsfic_book: Option<LibGenRSBook>,
	lgli_file: Option<LibGenLIBook>,
	zlib_book: Option<ZLibBook>,
	ipfs_infos: Option<Vec<IPFSInfo>>,
	file_unified_data: FileUnifiedData,
	search_only_fields: SearchOnlyFields,
	additional: Additional,
}

// libgen.rs book
#[derive(Serialize, Deserialize)]
struct LibGenRSBook {
	id: i32,
	md5: String,
}

// libgen.rs non-fiction
#[derive(Serialize, Deserialize)]
struct LibGenLIBook {
	f_id: i32,
	md5: String,
	libgen_topic: String,
}

// zlib book
#[derive(Serialize, Deserialize)]
struct ZLibBook {
	zlibrary_id: i32,
	md5: Option<String>,
	md5_reported: String,
	filesize: Option<i32>,
	filesize_reported: i32,
	in_libgen: i32,
	pilimi_torrent: Option<String>,
}

// IPFS info
#[derive(Serialize, Deserialize)]
struct IPFSInfo {
	ipfs_cid: String,
	filename: String,
	from: String,
}

// Unified file data
#[derive(Serialize, Deserialize)]
struct FileUnifiedData {
	original_filename_best: String,
	original_filename_additional: Vec<String>,
	original_filename_best_name_only: String,
	cover_url_best: String,
	cover_url_additional: Vec<String>,
	extension_best: String,
	extension_additional: Vec<String>,
	filesize_best: i32,
	filesize_additional: Vec<String>,
	title_best: String,
	title_additional: Vec<String>,
	author_best: String,
	author_additional: Vec<String>,
	publisher_best: String,
	publisher_additional: Vec<String>,
	edition_varia_best: String,
	edition_varia_additional: Vec<String>,
	year_best: String,
	year_additional: Vec<String>,
	comments_best: String,
	comments_additional: Vec<String>,
	stripped_description_best: String,
	stripped_description_additional: Vec<String>,
	language_codes: Vec<String>,
	most_likely_language_code: String,
	sanitized_isbns: Vec<String>,
	asin_multiple: Vec<String>,
	googlebookid_multiple: Vec<String>,
	openlibraryid_multiple: Vec<String>,
	doi_multiple: Vec<String>,
	problems: Vec<String>,
	content_type: String,
}

// Search only fields
#[derive(Serialize, Deserialize)]
struct SearchOnlyFields {
	search_text: String,
	score_base: f32,
}

// Additional
#[derive(Serialize, Deserialize)]
struct Additional {
	most_likely_language_name:String,
	top_box: TopBox,
	isbns_rich: Vec<Vec<String>>,
	download_urls: Vec<Vec<String>>,
}

// Top box
#[derive(Serialize, Deserialize)]
struct TopBox {
	meta_information: Vec<String>,
	cover_url: String,
	top_row: String,
	title: String,
	publisher_and_edition: String,
	author: String,
	description: String,
}
