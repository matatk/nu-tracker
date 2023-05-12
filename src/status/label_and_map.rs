use std::{collections::HashMap, fmt, str::FromStr};

// TODO: Switch to lifetimes rather than Strings
macro_rules! make_status_label_and_flag_map {
    ($(($variant:ident, $long:expr, $short:expr)),* $(,)?) => {
        #[derive(Debug, PartialEq, Clone)]
        pub enum StatusLabel {
            $(
                $variant,
            )+
        }

        #[derive(Debug, PartialEq)]
        pub struct StatusLabelError;

        impl FromStr for StatusLabel {
            type Err = StatusLabelError;

            /// Create a StatusLabel from a text string
            fn from_str(label: &str) -> Result<StatusLabel, StatusLabelError> {
                match label {
                    $(
                        $long => Ok(StatusLabel::$variant),
                    )*
                    _ => Err(StatusLabelError),
                }
            }
        }

        impl fmt::Display for StatusLabel {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(
                        StatusLabel::$variant => write!(f, $short)?,
                    )*
                }

                Ok(())
            }
        }

        #[derive(Default)]
        pub struct FlagLabelMap(InternalMap);
        type InternalMap = HashMap<String, String>;

        impl FlagLabelMap {
            pub fn new() -> Self {
                let mut map: InternalMap = HashMap::new();
                $(
                    map.insert($short.to_string(), $long.to_string());
                )*
                let map = map;

                Self(map)
            }

            pub fn get(&self, key: &str) -> Option<&String> {
                self.0.get(key)
            }
        }

        impl fmt::Display for FlagLabelMap {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let mut strings: Vec<String> = vec![];
                $(
                    strings.push(format!("{}: {}", $short, $long));
                )*
                write!(f, "{}", strings.join("\n"))?;
                Ok(())
            }
        }
    }
}

make_status_label_and_flag_map!(
	(Pending, "pending", "P"),
	(Close, "close?", "C"),
	(Tracker, "tracker", "T"), // Prefixed, e.g. with "a11y-" in issue in source group's repo.
	(NeedsResolution, "needs-resolution", "N"), // Prefixed, e.g. with "a11y-" in issue in source group's repo.
	(Recycle, "recycle", "R"),
	(AdviceRequested, "advice-requested", "A"), // Optional - source group is asking for advice
	(NeedsAttention, "needs-attention", "X"),   // Optional - HR group realises this is an urgent issue
);

// TODO: Test/separate FlagLabelMap?
#[cfg(test)]
mod tests {
	use std::assert_eq;

	use super::*;

	// TODO: Check that only the below tested variants exist

	#[test]
	fn valid_source_is_not_status() {
		let result = StatusLabel::from_str("s:html");
		assert_eq!(result, Err(StatusLabelError))
	}

	#[test]
	fn pending() {
		let result = StatusLabel::from_str("pending").unwrap();
		assert_eq!(result, StatusLabel::Pending)
	}

	#[test]
	fn close() {
		let result = StatusLabel::from_str("close?").unwrap();
		assert_eq!(result, StatusLabel::Close)
	}

	#[test]
	fn tracker() {
		let result = StatusLabel::from_str("tracker").unwrap();
		assert_eq!(result, StatusLabel::Tracker)
	}

	#[test]
	fn needs_resolution() {
		let result = StatusLabel::from_str("needs-resolution").unwrap();
		assert_eq!(result, StatusLabel::NeedsResolution)
	}

	#[test]
	fn recycle() {
		let result = StatusLabel::from_str("recycle").unwrap();
		assert_eq!(result, StatusLabel::Recycle)
	}

	#[test]
	fn advice_requested() {
		let result = StatusLabel::from_str("advice-requested").unwrap();
		assert_eq!(result, StatusLabel::AdviceRequested)
	}

	#[test]
	fn needs_attention() {
		let result = StatusLabel::from_str("needs-attention").unwrap();
		assert_eq!(result, StatusLabel::NeedsAttention)
	}
}
