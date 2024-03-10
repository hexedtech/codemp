use std::collections::HashMap;
use napi_derive::napi;

pub type OpTuple = (String, u32, String, u32); // buf_path, start, text, end

#[napi]
pub struct OpCache {
	store: HashMap<OpTuple, i32>
}

#[napi]
impl OpCache {
	#[napi(constructor)]
	pub fn new() -> Self {
		OpCache {
			store: HashMap::new()
		}
	}

	#[napi]
	pub fn to_string(&self) -> String {
		self.store.iter()
			.map(|(k, v)| format!("{}x Op(@{} {}:{} '{}')", k.0, v, k.1, k.3, k.2))
			.collect::<Vec<String>>()
			.join(", ")
	}

	#[napi]
	pub fn put(&mut self, buf: String, start: u32, text: String, end: u32) -> i32 {
		let op = (buf, start, text, end);
		match self.store.get_mut(&op) {
			Some(val) => {
				if *val < 0 { *val = 0 }
				*val += 1;
				*val
			},
			None => {
				self.store.insert(op, 1);
				return 1;
			}
		}
	}

	#[napi]
	pub fn get(&mut self, buf: String, start: u32, text: String, end: u32) -> bool {
		let op = (buf, start, text, end);
		match self.store.get_mut(&op) {
			Some(val) => {
				*val -= 1;
				*val >= 0
			}
			None => {
				tracing::warn!("never seen this op: {:?}", op);
				self.store.insert(op, -1);
				false
			},
		}
	}
}
//a
//consume a
//a 




#[cfg(test)]
mod test {
	#[test]
	fn opcache_put_increments_internal_counter() {
		let mut op = super::OpCache::new();
		assert_eq!(op.put("default".into(), 0, "hello world".into(), 0), 1); // 1: did not already contain it
		assert_eq!(op.put("default".into(), 0, "hello world".into(), 0), 2); // 2: already contained it
	}
	#[test]
	fn op_cache_get_checks_count() {
		let mut op = super::OpCache::new();
		assert_eq!(op.get("default".into(), 0, "hello world".into(), 0), false);
		assert_eq!(op.put("default".into(), 0, "hello world".into(), 0), 1);
		assert_eq!(op.get("default".into(), 0, "hello world".into(), 0), true);
		assert_eq!(op.get("default".into(), 0, "hello world".into(), 0), false);
	}
	#[test]
	fn op_cache_get_works_for_multiple_puts() {
		let mut op = super::OpCache::new();
		assert_eq!(op.get("default".into(), 0, "hello world".into(), 0), false);
		assert_eq!(op.put("default".into(), 0, "hello world".into(), 0), 1);
		assert_eq!(op.put("default".into(), 0, "hello world".into(), 0), 2);
		assert_eq!(op.get("default".into(), 0, "hello world".into(), 0), true);
		assert_eq!(op.get("default".into(), 0, "hello world".into(), 0), true);
		assert_eq!(op.get("default".into(), 0, "hello world".into(), 0), false);
	}

	#[test]
	fn op_cache_different_keys(){
		let mut op = super::OpCache::new();
		assert_eq!(op.get("default".into(), 0, "hello world".into(), 0), false);
		assert_eq!(op.put("default".into(), 0, "hello world".into(), 0), 1);
		assert_eq!(op.get("workspace".into(), 0, "hi".into(), 0), false);
		assert_eq!(op.put("workspace".into(), 0, "hi".into(), 0), 1);
		assert_eq!(op.get("workspace".into(), 0, "hi".into(), 0), true);
		assert_eq!(op.get("default".into(), 0, "hello world".into(), 0), true);
	}
}