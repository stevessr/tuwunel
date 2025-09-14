use tuwunel_core::utils::string::common_prefix;

use super::{CommandSystem, Service};

impl Service {
	pub fn complete_command(&self, command_system: &dyn CommandSystem, line: &str) -> String {
		let completion_tree = command_system.get_completion_tree();
		let args = command_system.parse(line);

		let mut cur = &completion_tree;

		let mut ret = Vec::<String>::with_capacity(args.len().saturating_add(1));

		'token: for token in args.iter().skip(1) {
			let mut choice = Vec::new();

			for sub in &cur.nodes {
				if sub.name == *token {
					// token already complete; recurse to subcommand
					ret.push(token.clone());
					cur = sub;
					continue 'token;
				} else if sub.name.starts_with(token) {
					// partial match; add to choices
					choice.push(sub.name.clone());
				}
			}

			if choice.len() == 1 {
				// One choice. Add extra space because it's complete
				let choice = choice.first().unwrap();
				ret.push(choice.clone());
				ret.push(String::new());
			} else if choice.is_empty() {
				// Nothing found, return original string
				ret.push(token.clone());
			} else {
				// Find the common prefix
				ret.push(common_prefix(&choice).into());
			}

			// Return from completion
			return ret.join(" ");
		}

		// Return from no completion. Needs a space though.
		ret.push(String::new());
		ret.join(" ")
	}
}
