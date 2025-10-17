use ruma::{
	CanonicalJsonValue, OwnedUserId,
	api::{
		IncomingRequest,
		client::uiaa::{AuthData, AuthFlow, AuthType, Jwt, UiaaInfo},
	},
};
use tuwunel_core::{Err, Error, Result, err, utils};
use tuwunel_service::{Services, uiaa::SESSION_ID_LENGTH};

use crate::{Ruma, client::jwt};

pub(crate) async fn auth_uiaa<T>(services: &Services, body: &Ruma<T>) -> Result<OwnedUserId>
where
	T: IncomingRequest + Send + Sync,
{
	let flows = [
		AuthFlow::new([AuthType::Password].into()),
		AuthFlow::new([AuthType::Jwt].into()),
	];

	let mut uiaainfo = UiaaInfo {
		flows: flows.into(),
		..Default::default()
	};

	match body
		.json_body
		.as_ref()
		.and_then(CanonicalJsonValue::as_object)
		.and_then(|body| body.get("auth"))
		.cloned()
		.map(CanonicalJsonValue::into)
		.map(serde_json::from_value)
		.transpose()? {
		Some(AuthData::Jwt(Jwt { ref token, .. })) => {
			let sender_user = jwt::validate_user(services, token)?;
			if !services.users.exists(&sender_user).await {
				return Err!(Request(NotFound("User {sender_user} is not registered.")));
			}
			Ok(sender_user)
		},
		Some(ref auth) => {
			let sender_user = body
				.sender_user
				.as_deref()
				.ok_or_else(|| err!(Request(MissingToken("Missing access token."))))?;
			let (worked, uiaainfo) = services
				.uiaa
				.try_auth(sender_user, body.sender_device(), auth, &uiaainfo)
				.await?;
			if !worked {
				return Err(Error::Uiaa(uiaainfo));
			}
			Ok(sender_user.to_owned())
		},
		_ => {
			// 如果已经通过 access token 认证（即 sender_user 存在），且没有 auth 字段，则直接通过
			if let Some(sender_user) = body.sender_user.as_ref() {
				return Ok(sender_user.to_owned());
			}
			match body.json_body {
				Some(ref json) => {
					let sender_user = body
						.sender_user
						.as_ref()
						.ok_or_else(|| err!(Request(MissingToken("Missing access token."))))?;
					uiaainfo.session = Some(utils::random_string(SESSION_ID_LENGTH));
					services
						.uiaa
						.create(sender_user, body.sender_device(), &uiaainfo, json);
					Err(Error::Uiaa(uiaainfo))
				},
				_ => Err!(Request(NotJson("JSON body is not valid"))),
			}
		}
	}
}
