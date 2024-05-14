/// The origin of the issue
#[derive(Clone)]
pub enum OriginQuery {
	/// Us
	OurGroup,
	/// A different group
	OtherGroup,
	/// I don't mind
	Whatevs,
}

impl OriginQuery {
	/// Create a new origin query
	// FIXME: param names
	#[must_use]
	pub fn new(ours: bool, others: bool) -> Self {
		match (ours, others) {
			(true, false) => Self::OurGroup,
			(false, true) => Self::OtherGroup,
			(false, false) => Self::Whatevs,
			(true, true) => {
				unreachable!("Clap should stop both 'ours' and 'others' from being set at once");
			}
		}
	}
}
