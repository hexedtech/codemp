use operational_transform::OperationSeq;
use similar::{TextDiff, ChangeTag};

pub trait OperationFactory {
	fn content(&self) -> String;

	fn replace(&self, txt: &str) -> OperationSeq {
		let mut out = OperationSeq::default();
		let content = self.content();
		if content == txt {
			return out; // TODO this won't work, should we return a noop instead?
		}

		let diff = TextDiff::from_chars(content.as_str(), txt);

		for change in diff.iter_all_changes() {
			match change.tag() {
				ChangeTag::Equal => out.retain(1),
				ChangeTag::Delete => out.delete(1),
				ChangeTag::Insert => out.insert(change.value()),
			}
		}

		out
	}

	fn insert(&self, txt: &str, pos: u64) -> OperationSeq {
		let mut out = OperationSeq::default();
		let total = self.content().len() as u64;
		out.retain(pos);
		out.insert(txt);
		out.retain(total - pos);
		out
	}

	fn delete(&self, pos: u64, count: u64) -> OperationSeq {
		let mut out = OperationSeq::default();
		let len = self.content().len() as u64;
		out.retain(pos - count);
		out.delete(count);
		out.retain(len - pos);
		out
	}

	fn cancel(&self, pos: u64, count: u64) -> OperationSeq {
		let mut out = OperationSeq::default();
		let len = self.content().len() as u64;
		out.retain(pos);
		out.delete(count);
		out.retain(len - (pos+count));
		out
	}
}
