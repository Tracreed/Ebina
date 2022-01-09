use chrono::Timelike;
//use graphql_client::{GraphQLQuery, Response};
use cynic::{QueryBuilder, http::ReqwestExt};
use queries::queries::MediaType;
use std::error::Error;

const GQL_URL: &str = "https://graphql.anilist.co/";

pub mod queries;

pub async fn search<S>(title: S) -> Result<queries::queries::MediaSearch, Box<dyn Error>> where
S: Into<String> {
	use queries::queries::{MediaSearch, MediaSearchArguments};
	let arguments = MediaSearchArguments {
		title: Some(title.into())
	};
	let operation = MediaSearch::build(arguments);
	/*for (key, val) in &operation.variables {
		println!("key: {} val: {:?}", key, val.name);
	}*/
	let client = reqwest::Client::new();
	let response = client.post(GQL_URL)
	.run_graphql(operation)
	.await?;
	Ok(response.data.unwrap())
}

pub async fn search_specific<S>(title: S, media_type: Option<MediaType>) -> Result<queries::queries::MediaSpecific, Box<dyn Error>> where
S: Into<String> {
	use queries::queries::{MediaSpecific, MediaSpecificArguments};
	let arguments = MediaSpecificArguments {
		title: Some(title.into()),
    	r#type: media_type,
	};
	let operation = MediaSpecific::build(arguments);
	/*for (key, val) in &operation.variables {
		println!("key: {} val: {:?}", key, val.name);
	}*/
	let client = reqwest::Client::new();
	let response = client.post(GQL_URL)
	.run_graphql(operation)
	.await?;
	Ok(response.data.unwrap())
}

pub async fn get_schedule(date_utc: chrono::DateTime<chrono::Utc>) -> Result<queries::queries::Schedule, Box<dyn Error>> {
	use queries::queries::{Schedule, ScheduleArguments};
	let off = chrono::FixedOffset::east(9 * 3600);
	let date = date_utc.with_timezone(&off);
	let start_time = date.timestamp() - (((date.hour() as i64 * 60) * 60) + (date.minute () as i64 * 60));
	let end_time = date.timestamp() + ((24 - date.hour() as i64) * 60) * 60;
	let arguments = ScheduleArguments {
		airing_at_greater: Some(start_time as i32),
		airing_at_lesser: Some(end_time as i32),
	};
	let operation = Schedule::build(arguments);
	let client = reqwest::Client::new();
	let response = client.post(GQL_URL)
		.run_graphql(operation)
		.await?;
	Ok(response.data.unwrap())
}

#[cfg(test)]
mod tests {
	#[tokio::test]
	async fn search_manga() {
		use super::*;
		let manga_res = search("To love Ru").await.unwrap();
		let manga = manga_res.page.unwrap().media.unwrap();
		assert_eq!(manga[0].as_ref().unwrap().title.as_ref().unwrap().romaji.as_ref().unwrap(), "To LOVE-Ru");
		println!("{:?}", manga[0].as_ref().unwrap());
	}
}
