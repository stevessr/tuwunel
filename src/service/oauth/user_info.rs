use serde::{
	Deserialize, Serialize,
	de::{self, Deserializer},
};

/// Selection of userinfo response claims.
#[derive(Clone, Debug, Default, Serialize)]
pub struct UserInfo {
	/// Unique identifier number or login username. Usually a number on most
	/// services. We consider a concatenation of the `iss` and `sub` to be a
	/// universally unique identifier for some user/identity; we index that in
	/// `oauthidpsub_oauthid`.
	///
	/// Considered for user mxid only if none of the better fields are defined.
	/// `login` alias intended for github.
	pub sub: String,

	/// The login username we first consider when defined.
	pub preferred_username: Option<String>,

	/// The login username considered if none preferred.
	pub nickname: Option<String>,

	/// Full name.
	pub name: Option<String>,

	/// First name.
	pub given_name: Option<String>,

	/// Last name.
	pub family_name: Option<String>,

	/// Email address (`email` scope).
	pub email: Option<String>,

	/// URL to pfp (github/gitlab)
	pub avatar_url: Option<String>,

	/// URL to pfp (google)
	pub picture: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawUserInfo {
	sub: Option<String>,
	#[serde(rename = "login")]
	login: Option<String>,
	preferred_username: Option<String>,
	nickname: Option<String>,
	name: Option<String>,
	given_name: Option<String>,
	family_name: Option<String>,
	email: Option<String>,
	avatar_url: Option<String>,
	picture: Option<String>,
}

impl<'de> Deserialize<'de> for UserInfo {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let raw = RawUserInfo::deserialize(deserializer)?;
		let sub = raw
			.sub
			.or(raw.login)
			.ok_or_else(|| de::Error::missing_field("sub"))?;

		Ok(UserInfo {
			sub,
			preferred_username: raw.preferred_username,
			nickname: raw.nickname,
			name: raw.name,
			given_name: raw.given_name,
			family_name: raw.family_name,
			email: raw.email,
			avatar_url: raw.avatar_url,
			picture: raw.picture,
		})
	}
}
