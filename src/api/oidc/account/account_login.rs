use tuwunel_core::utils::html::escape as html_escape;

/// Render the login selection page (SSO or password).
pub(super) fn account_login_page_html(sso_url: &str) -> String {
	let sso_url_enc = html_escape(sso_url);

	format!(
		r#"<!DOCTYPE html>
<html lang="en">
	<head>
		<meta charset="UTF-8">
		<link rel="stylesheet" href="/_tuwunel/oidc/account.css">
		<title>Account Management — Login</title>
	</head>
	<body>
		<h1>Account Management</h1>
		<p>Sign in to manage your account sessions and profile.</p>

		<div class="actions">
			<a href="{sso_url_enc}">Sign in with Single Sign-On</a>
		</div>

		<hr>

		<h2>Or sign in with password</h2>
		<form method="post" action="/_tuwunel/oidc/account">
			<label for="username">Username (e.g. @user:server.tld)</label>
			<input type="text" id="username" name="username" required autocomplete="username">

			<label for="password">Password</label>
			<input type="password" id="password" name="password" required autocomplete="current-password">

			<div class="submit-row">
				<button type="submit">Sign in</button>
			</div>
		</form>
	</body>
</html>"#,
		sso_url_enc = sso_url_enc,
	)
}

/// Render the password-only login form.
///
/// `action` — if set, included as a hidden form field to carry a target action
/// through to the callback (e.g. `org.matrix.bind_sso`).
/// `message` — optional informational or error message shown above the form.
pub(super) fn password_login_page_html(action: Option<&str>, message: &str) -> String {
	let action_field = action
		.map(|a| format!(r#"<input type="hidden" name="action" value="{}">"#, html_escape(a)))
		.unwrap_or_default();

	let msg_html = if message.is_empty() {
		String::new()
	} else {
		format!(r#"<p class="meta">{}</p>"#, html_escape(message))
	};

	format!(
		r#"<!DOCTYPE html>
<html lang="en">
	<head>
		<meta charset="UTF-8">
		<link rel="stylesheet" href="/_tuwunel/oidc/account.css">
		<title>Account Management — Password Login</title>
	</head>
	<body>
		<h1>Account Management</h1>
		{msg_html}
		<p>Enter your Matrix password to manage your account.</p>

		<form method="post" action="/_tuwunel/oidc/account">
			<label for="username">Username (e.g. @user:server.tld)</label>
			<input type="text" id="username" name="username" required autocomplete="username">

			<label for="password">Password</label>
			<input type="password" id="password" name="password" required autocomplete="current-password">

			{action_field}

			<div class="submit-row">
				<button type="submit">Sign in</button>
			</div>
		</form>

		<div class="nav">
			<a href="/_tuwunel/oidc/account">Back to login options</a>
		</div>
	</body>
</html>"#
	)
}

/// Form data for the password-based login endpoint.
#[derive(Debug, serde::Deserialize)]
pub(crate) struct LoginForm {
	pub username: String,
	pub password: String,
	/// Optional action to redirect to after successful authentication.
	pub action: Option<String>,
}
