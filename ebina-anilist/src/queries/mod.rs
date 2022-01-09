#[cynic::schema_for_derives(
	file = r#"schemas/schema.gql"#,
	module = "schema",
)]
pub mod queries {
	use std::fmt::Display;
	
	use super::schema;
	
	#[derive(cynic::FragmentArguments, Debug, Clone)]
	pub struct MediaSpecificArguments {
		pub title: Option<String>,
		pub r#type: Option<MediaType>,
	}

	#[derive(cynic::FragmentArguments, Debug, Clone)]
    pub struct ScheduleArguments {
        pub airing_at_greater: Option<i32>,
        pub airing_at_lesser: Option<i32>,
    }
	
	#[derive(cynic::FragmentArguments, Debug, Clone)]
	pub struct MediaSearchArguments {
		pub title: Option<String>,
	}
	
	#[derive(cynic::QueryFragment, Debug, Clone)]
	#[cynic(graphql_type = "Query", argument_struct = "MediaSpecificArguments")]
	pub struct MediaSpecific {
		#[arguments(page = 1)]
		pub page: Option<Page>,
	}
	
	#[derive(cynic::QueryFragment, Debug, Clone)]
	#[cynic(graphql_type = "Query", argument_struct = "MediaSearchArguments")]
	pub struct MediaSearch {
		#[arguments(page = 1)]
		pub page: Option<Page2>,
	}
	
	#[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Query", argument_struct = "ScheduleArguments")]
    pub struct Schedule {
        pub page: Option<Page3>,
    }
	
	#[derive(cynic::QueryFragment, Debug, Clone)]
	#[cynic(argument_struct = "MediaSpecificArguments")]
	pub struct Page {
		pub page_info: Option<PageInfo>,
		#[arguments(search = &args.title, r#type = args.r#type)]
		pub media: Option<Vec<Option<Media>>>,
	}
	
	#[derive(cynic::QueryFragment, Debug, Clone)]
	#[cynic(graphql_type = "Page", argument_struct = "MediaSearchArguments")]
	pub struct Page2 {
		pub page_info: Option<PageInfo>,
		#[arguments(search = &args.title)]
		pub media: Option<Vec<Option<Media>>>,
	}
	
	#[derive(cynic::QueryFragment, Debug, Copy, Clone)]
	pub struct PageInfo {
		pub total: Option<i32>,
		pub per_page: Option<i32>,
		pub current_page: Option<i32>,
		pub last_page: Option<i32>,
		pub has_next_page: Option<bool>,
	}
	
	#[derive(cynic::QueryFragment, Debug, Clone)]
    #[cynic(graphql_type = "Page", argument_struct = "ScheduleArguments")]
    pub struct Page3 {
        #[arguments(airing_at_greater = args.airing_at_greater, airing_at_lesser = args.airing_at_lesser)]
        pub airing_schedules: Option<Vec<Option<AiringSchedule>>>,
    }
	
	#[derive(cynic::QueryFragment, Debug, Clone)]
	pub struct Media {
		pub id: i32,
		#[cynic(rename = "type")]
		pub type_: Option<MediaType>,
		pub title: Option<MediaTitle>,
		pub episodes: Option<i32>,
		pub volumes: Option<i32>,
		pub genres: Option<Vec<Option<String>>>,
		pub popularity: Option<i32>,
		pub status: Option<MediaStatus>,
		pub average_score: Option<i32>,
		pub tags: Option<Vec<Option<MediaTag>>>,
		pub cover_image: Option<MediaCoverImage>,
		pub format: Option<MediaFormat>,
		pub season: Option<MediaSeason>,
		pub mean_score: Option<i32>,
		pub is_adult: Option<bool>,
		pub duration: Option<i32>,
		#[arguments(as_html = false)]
		pub description: Option<String>,
		pub chapters: Option<i32>,
		pub rankings: Option<Vec<Option<MediaRank>>>,
		pub season_year: Option<i32>,
		pub site_url: Option<String>,
		pub source: Option<MediaSource>,
		pub start_date: Option<FuzzyDate>,
		pub end_date: Option<FuzzyDate>,
	}
	
	#[derive(cynic::QueryFragment, Debug, Clone)]
	pub struct MediaRank {
		pub rank: i32,
		#[cynic(rename = "type")]
		pub type_: MediaRankType,
	}
	
	#[derive(cynic::QueryFragment, Debug, Copy, Clone)]
	pub struct FuzzyDate {
		pub day: Option<i32>,
		pub month: Option<i32>,
		pub year: Option<i32>,
	}
	
	#[derive(cynic::QueryFragment, Debug, Clone)]
    pub struct AiringSchedule {
        pub episode: i32,
        pub airing_at: i32,
        pub media: Option<Media2>,
    }

    #[derive(cynic::QueryFragment, Debug, Clone)]
    #[cynic(graphql_type = "Media")]
    pub struct Media2 {
        pub id: i32,
        #[cynic(rename = "type")]
        pub type_: Option<MediaType>,
        pub title: Option<MediaTitle>,
        pub episodes: Option<i32>,
        pub season: Option<MediaSeason>,
        pub popularity: Option<i32>,
        pub status: Option<MediaStatus>,
        pub average_score: Option<i32>,
    }
	
	#[derive(cynic::QueryFragment, Debug, Clone)]
	pub struct MediaCoverImage {
		pub extra_large: Option<String>,
		pub large: Option<String>,
		pub medium: Option<String>,
		pub color: Option<String>,
	}
	
	#[derive(cynic::QueryFragment, Debug, Clone)]
	pub struct StaffConnection {
		pub edges: Option<Vec<Option<StaffEdge>>>,
	}
	
	#[derive(cynic::QueryFragment, Debug, Clone)]
	pub struct StaffEdge {
		pub id: Option<i32>,
		pub role: Option<String>,
		pub node: Option<Staff>,
	}
	
	#[derive(cynic::QueryFragment, Debug, Clone)]
	pub struct Staff {
		pub name: Option<StaffName>,
	}
	
	#[derive(cynic::QueryFragment, Debug, Clone)]
	pub struct StaffName {
		pub first: Option<String>,
		pub last: Option<String>,
	}
	
	#[derive(cynic::QueryFragment, Debug, Clone)]
	pub struct MediaTag {
		pub id: i32,
		pub name: String,
		pub rank: Option<i32>,
	}
	
	#[derive(cynic::QueryFragment, Debug, Clone)]
	pub struct MediaTitle {
		pub romaji: Option<String>,
		pub english: Option<String>,
		pub native: Option<String>,
		pub user_preferred: Option<String>,
	}
	
	#[derive(cynic::Enum, Clone, Copy, Debug)]
	pub enum MediaFormat {
		Tv,
		TvShort,
		Movie,
		Special,
		Ova,
		Ona,
		Music,
		Manga,
		Novel,
		OneShot,
	}
	
	impl Display for MediaFormat {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			match self {
				MediaFormat::Tv => f.write_str("TV"),
				MediaFormat::TvShort => f.write_str("TV Short "),
				MediaFormat::Movie => f.write_str("Movie"),
				MediaFormat::Special => f.write_str("Special"),
				MediaFormat::Ova => f.write_str("OVA"),
				MediaFormat::Ona => f.write_str("ONA"),
				MediaFormat::Music => f.write_str("Music"),
				MediaFormat::Manga => f.write_str("Manga"),
				MediaFormat::Novel => f.write_str("Light Novel"),
				MediaFormat::OneShot => f.write_str("One Shot"),
			}
		}
	}
	
	#[derive(cynic::Enum, Clone, Copy, Debug)]
	pub enum MediaRankType {
		Rated,
		Popular,
	}
	
	#[derive(cynic::Enum, Clone, Copy, Debug)]
	pub enum MediaSeason {
		Winter,
		Spring,
		Summer,
		Fall,
	}
	
	#[derive(cynic::Enum, Clone, Copy, Debug)]
	pub enum MediaSource {
		Original,
		Manga,
		LightNovel,
		VisualNovel,
		VideoGame,
		Other,
		Novel,
		Doujinshi,
		Anime,
		WebNovel,
		LiveAction,
		Game,
		Comic,
		MultimediaProject,
		PictureBook,
	}
	
	#[derive(cynic::Enum, Clone, Copy, Debug)]
	pub enum MediaStatus {
		Finished,
		Releasing,
		NotYetReleased,
		Cancelled,
		Hiatus,
	}

	impl Display for MediaStatus {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			match self {
				MediaStatus::Finished => f.write_str("Finished"),
				MediaStatus::Releasing => f.write_str("Releasing"),
				MediaStatus::NotYetReleased => f.write_str("Not Yet Released"),
				MediaStatus::Cancelled => f.write_str("Cancelled"),
				MediaStatus::Hiatus => f.write_str("Hiatus"),
			}
		}
	}
	
	#[derive(cynic::Enum, Clone, Copy, Debug)]
	pub enum MediaType {
		Anime,
		Manga,
	}
	
	impl Display for MediaType {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			match self {
				MediaType::Anime => f.write_str("Anime"),
				MediaType::Manga => f.write_str("Manga"),
			}
		}
	}
	
}

mod schema {
	cynic::use_schema!(r#"schemas/schema.gql"#);
}


#[cfg(test)]
mod tests {
	use super::*;
	
	#[test]
	fn search_manga_query_gql_output() {
		use cynic::QueryBuilder;
		use queries::MediaType;
		let arguments = queries::MediaSpecificArguments {
			title: Some("To love-ru".to_string()),
			r#type: Some(MediaType::Manga)
		};
		let operation = queries::MediaSpecific::build(arguments);
		insta::assert_snapshot!(operation.query);
	}
}