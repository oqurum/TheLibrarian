use chrono::{DateTime, Utc, TimeZone};
use common::MemberId;
use librarian_common::EditId;

use crate::model::{TableRow, AdvRow};



#[derive(Debug, Clone)]
pub struct EditPersonModel {
	pub edit_id: EditId,
	pub member_id: MemberId,

	pub vote: bool,

	pub created_at: DateTime<Utc>,
}


impl TableRow<'_> for EditPersonModel {
	fn create(row: &mut AdvRow<'_>) -> rusqlite::Result<Self> {
		Ok(Self {
			edit_id: row.next()?,
			member_id: row.next()?,

			vote: row.next()?,

			created_at: Utc.timestamp_millis(row.next()?),
		})
	}
}


// type PerformerEdit struct {
// 	EditID            uuid.UUID           `json:"-"`
// 	Name              *string             `json:"name,omitempty"`
// 	Disambiguation    *string             `json:"disambiguation,omitempty"`
// 	AddedAliases      []string            `json:"added_aliases,omitempty"`
// 	RemovedAliases    []string            `json:"removed_aliases,omitempty"`
// 	Gender            *string             `json:"gender,omitempty"`
// 	AddedUrls         []*URL              `json:"added_urls,omitempty"`
// 	RemovedUrls       []*URL              `json:"removed_urls,omitempty"`
// 	Birthdate         *string             `json:"birthdate,omitempty"`
// 	BirthdateAccuracy *string             `json:"birthdate_accuracy,omitempty"`
// 	Ethnicity         *string             `json:"ethnicity,omitempty"`
// 	Country           *string             `json:"country,omitempty"`
// 	EyeColor          *string             `json:"eye_color,omitempty"`
// 	HairColor         *string             `json:"hair_color,omitempty"`
// 	Height            *int64              `json:"height,omitempty"`
// 	CupSize           *string             `json:"cup_size,omitempty"`
// 	BandSize          *int64              `json:"band_size,omitempty"`
// 	WaistSize         *int64              `json:"waist_size,omitempty"`
// 	HipSize           *int64              `json:"hip_size,omitempty"`
// 	BreastType        *string             `json:"breast_type,omitempty"`
// 	CareerStartYear   *int64              `json:"career_start_year,omitempty"`
// 	CareerEndYear     *int64              `json:"career_end_year,omitempty"`
// 	AddedImages       []uuid.UUID         `json:"added_images,omitempty"`
// 	RemovedImages     []uuid.UUID         `json:"removed_images,omitempty"`
// }