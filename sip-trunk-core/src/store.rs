use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::error::{CoreError, Result};
use crate::models::{Call, CallStatus};

pub type CallStore = Arc<Mutex<HashMap<Uuid, Call>>>;

pub fn new_call_store() -> CallStore {
    Arc::new(Mutex::new(HashMap::new()))
}

pub fn insert_call(store: &CallStore, call: Call) -> Result<()> {
    store.lock().unwrap().insert(call.id, call);
    Ok(())
}

pub fn get_call(store: &CallStore, id: &Uuid) -> Result<Call> {
    store
        .lock()
        .unwrap()
        .get(id)
        .cloned()
        .ok_or_else(|| CoreError::CallNotFound(id.to_string()))
}

pub fn get_call_by_control_id(store: &CallStore, call_control_id: &str) -> Result<Call> {
    store
        .lock()
        .unwrap()
        .values()
        .find(|c| c.call_control_id == call_control_id)
        .cloned()
        .ok_or_else(|| CoreError::CallNotFound(call_control_id.to_string()))
}

pub fn list_calls(store: &CallStore) -> Vec<Call> {
    store.lock().unwrap().values().cloned().collect()
}

pub fn update_call_status(store: &CallStore, id: &Uuid, status: CallStatus) -> Result<()> {
    let mut guard = store.lock().unwrap();
    let call = guard
        .get_mut(id)
        .ok_or_else(|| CoreError::CallNotFound(id.to_string()))?;
    call.status = status;
    Ok(())
}

pub fn remove_call(store: &CallStore, id: &Uuid) -> Result<()> {
    store
        .lock()
        .unwrap()
        .remove(id)
        .map(|_| ())
        .ok_or_else(|| CoreError::CallNotFound(id.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CallDirection, CallStatus};
    use chrono::Utc;

    fn make_call() -> Call {
        Call {
            id: Uuid::new_v4(),
            call_control_id: "ctrl_123".to_string(),
            direction: CallDirection::Inbound,
            from: "+15551234567".to_string(),
            to: "+15559876543".to_string(),
            status: CallStatus::Initiated,
            started_at: Utc::now(),
            ended_at: None,
        }
    }

    #[test]
    fn test_insert_and_get() {
        let store = new_call_store();
        let call = make_call();
        let id = call.id;
        insert_call(&store, call).unwrap();
        let retrieved = get_call(&store, &id).unwrap();
        assert_eq!(retrieved.call_control_id, "ctrl_123");
    }

    #[test]
    fn test_get_missing_returns_error() {
        let store = new_call_store();
        let result = get_call(&store, &Uuid::new_v4());
        assert!(matches!(result, Err(CoreError::CallNotFound(_))));
    }

    #[test]
    fn test_get_by_control_id() {
        let store = new_call_store();
        let call = make_call();
        insert_call(&store, call).unwrap();
        let retrieved = get_call_by_control_id(&store, "ctrl_123").unwrap();
        assert_eq!(retrieved.call_control_id, "ctrl_123");
    }

    #[test]
    fn test_list_calls() {
        let store = new_call_store();
        assert_eq!(list_calls(&store).len(), 0);
        insert_call(&store, make_call()).unwrap();
        insert_call(&store, make_call()).unwrap();
        assert_eq!(list_calls(&store).len(), 2);
    }

    #[test]
    fn test_update_status() {
        let store = new_call_store();
        let call = make_call();
        let id = call.id;
        insert_call(&store, call).unwrap();
        update_call_status(&store, &id, CallStatus::Answered).unwrap();
        let retrieved = get_call(&store, &id).unwrap();
        assert_eq!(retrieved.status, CallStatus::Answered);
    }

    #[test]
    fn test_remove_call() {
        let store = new_call_store();
        let call = make_call();
        let id = call.id;
        insert_call(&store, call).unwrap();
        remove_call(&store, &id).unwrap();
        assert!(get_call(&store, &id).is_err());
    }
}
